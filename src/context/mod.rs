use wgpu::{Adapter, Instance, Surface, TextureFormat};
use winit::{dpi::PhysicalSize, window::Window};

pub use crate::context::texture::Texture;
use crate::model::{MeshPrimitive, Scene};

use self::render_pipeline::TexturePipeline;

mod asset_store;
mod camera;
pub(crate) mod render_pipeline;
mod shaders;
mod texture;

pub struct DrawingContext {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    depth_texture: Texture,

    camera: camera::Camera,

    asset_world: asset_store::AssetWorld,
    texture_pipeline: TexturePipeline,

    fill_color: wgpu::Color,
    /// Window has a dimension of 0
    minimized: bool,
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
    pub async fn new(window: Window, scenes: &mut [Scene]) -> Self {
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

        shaders::build_shaders(&device);

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

        let texture_pipeline = TexturePipeline::new(
            &device,
            &config,
            camera.bind_group_layout(),
            &MeshPrimitive::color_bind_group_layout(&device),
        );

        let depth_texture = Texture::create_depth_texture(&device, &config);

        let asset_world = asset_store::AssetWorld::new(scenes, &device, &queue);

        Self {
            config,
            device,
            queue,
            size,
            surface,
            window,
            depth_texture,

            camera,

            asset_world,
            texture_pipeline,

            fill_color: wgpu::Color::BLACK,
            minimized: false,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            self.minimized = true;
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        // No need to destroy old depth texture, it will be dropped
        self.depth_texture = Texture::create_depth_texture(&self.device, &self.config);

        self.minimized = false;
    }

    pub fn reconfigure(&mut self) {
        self.resize(self.size);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.minimized {
            self.maintain();
            return Ok(());
        }

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
                label: Some("Texture Render Pass"),
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

            render_pass.set_pipeline(&self.texture_pipeline.pipeline);
            render_pass.set_bind_group(1, self.camera.bind_group(), &[]);

            use crate::context::asset_store::{Albedo, Indices, TextureBindGroup, Vertices};
            type Query<'a> = (
                &'a Vertices,
                &'a Albedo,
                Option<&'a Indices>,
                &'a TextureBindGroup,
            );

            for (vertices, _albedo, indices, texture) in self
                .asset_world
                .world
                .query::<Query>()
                .iter(&self.asset_world.world)
            {
                render_pass.set_bind_group(0, &texture.bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertices.buffer.slice(..));

                if let Some(index_buffer) = indices {
                    render_pass
                        .set_index_buffer(index_buffer.buffer.slice(..), index_buffer.format);
                    render_pass.draw_indexed(0..vertices.count, 0, 0..1);
                } else {
                    render_pass.draw(0..vertices.count, 0..1);
                }
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }

    pub fn maintain(&self) {
        self.queue.submit(std::iter::empty());
        self.device.poll(wgpu::Maintain::Poll);
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
