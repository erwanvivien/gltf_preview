use wgpu::{Adapter, Instance, Surface, TextureFormat};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{context::depth_texture::Texture, model::Scene};

mod camera;
mod depth_texture;
mod render_pipeline;

pub struct DrawingContext {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    depth_texture: Texture,

    camera: camera::Camera,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,

    color_bind_group: wgpu::BindGroup,

    fill_color: wgpu::Color,
}

async fn get_adaptater(instance: &Instance, surface: &Surface) -> Adapter {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await;

    #[cfg(target_arch = "wasm32")]
    return adapter.expect("Failed to find an appropriate adapter");

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(adapter) = adapter {
        return adapter;
    }

    #[cfg(not(target_arch = "wasm32"))]
    instance
        .enumerate_adapters(wgpu::Backends::all())
        .find(|adapter| {
            // Check if this adapter supports our surface
            adapter.is_surface_supported(surface)
        })
        .expect("Failed to find an appropriate fallback adapter")
}

impl DrawingContext {
    pub async fn new(window: Window, scene: Scene) -> Self {
        let device_descriptor = wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            #[cfg(not(feature = "webgl"))]
            limits: wgpu::Limits::default(),
            #[cfg(feature = "webgl")]
            limits: wgpu::Limits::downlevel_webgl2_defaults(),
            label: Some("Global device descriptor"),
        };

        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface =
            unsafe { instance.create_surface(&window) }.expect("Could not create surface");

        let adapter = get_adaptater(&instance, &surface).await;

        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .expect("Failed to create device and queue");

        let surface_capabilities = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(TextureFormat::is_srgb)
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let camera = camera::Camera::new(&window, &device);
        camera.update_projection_matrix(&queue);

        use wgpu::util::DeviceExt;
        let mesh = &scene.nodes[1].meshes[0].primitives[0];
        let texture = &mesh.material.texture;

        let texture = texture.as_ref().unwrap();
        let color_texture = Texture::create_texture_from_image(&device, &queue, &texture);

        let color_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Color Bind Group Layout"),
            });

        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &color_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&color_texture.sampler),
                },
            ],
            label: Some("Color Bind Group"),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(mesh.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = mesh.indices.as_ref().map(|indices| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            })
        });

        let render_pipeline = render_pipeline::create_main_render_pipeline(
            &device,
            &config,
            camera.bind_group_layout(),
            &color_bind_group_layout,
        );

        let index_count = mesh
            .indices
            .as_ref()
            .map(Vec::len)
            .unwrap_or(mesh.vertices.len()) as u32;

        let depth_texture = Texture::create_depth_texture(&device, &config);

        Self {
            config,
            device,
            queue,
            size,
            surface,
            window,
            depth_texture,

            camera,

            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count,

            color_bind_group,

            fill_color: wgpu::Color::BLACK,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            log::error!("Window size is 0, skipping resize");
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        // No need to destroy old depth texture, it will be dropped
        self.depth_texture = Texture::create_depth_texture(&self.device, &self.config);
    }

    pub fn reconfigure(&mut self) {
        self.resize(self.size);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Clear color"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.fill_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1f32),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
            render_pass.set_bind_group(1, &self.color_bind_group, &[]);

            if let Some(index_buffer) = &self.index_buffer {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.index_count, 0, 0..1);
            } else {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..self.index_count, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }
}

impl DrawingContext {
    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> &PhysicalSize<u32> {
        &self.size
    }

    pub fn set_fill_color(&mut self, color: wgpu::Color) {
        self.fill_color = color;
    }
}
