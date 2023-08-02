use wgpu::{Adapter, Instance, Surface, TextureFormat};
use winit::{dpi::PhysicalSize, window::Window};

pub use crate::render::texture::Texture;

use self::render_pipeline::{AlbedoPipeline, TexturePipeline};

mod asset_store;
mod camera;
pub(crate) mod render_pipeline;
mod shaders;
mod texture;
pub mod utils;

pub struct DrawingContext {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    depth_texture: Texture,

    camera: camera::Camera,
    pub input_manager: input_manager::InputManager,

    asset_registry: asset_store::AssetRegistry,
    texture_pipeline: TexturePipeline,
    albedo_pipeline: AlbedoPipeline,

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
    pub async fn new(window: Window) -> DrawingContext {
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

        shaders::build_shaders(&device).await;

        // Shader code in this tutorial assumes an sRGB surface texture.
        let surface_capabilities = surface.get_capabilities(&adapter);
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
            view_formats: vec![surface_format.add_srgb_suffix()],
        };

        surface.configure(&device, &config);

        let mut camera = camera::Camera::new(&window, &device);
        camera.set_camera((-3f32, 0.5f32, -3f32));
        camera.update_projection_matrix(&queue);

        let texture_pipeline = TexturePipeline::new(
            &device,
            &config,
            camera.bind_group_layout(),
            Texture::color_texture_bind_group_layout(&device),
        );
        let albedo_pipeline = AlbedoPipeline::new(&device, &config, camera.bind_group_layout());

        let depth_texture = Texture::create_depth_texture(&device, &config);

        const ASSETS: &[&str] = &["assets/CesiumMilkTruck.glb"];
        let asset_registry = asset_store::AssetRegistry::new(&device, &queue, ASSETS).await;

        let fill_color = wgpu::Color {
            r: 9f64 / 255f64,
            g: 46f64 / 255f64,
            b: 32f64 / 255f64,
            a: 1.0,
        };

        Self {
            config,
            device,
            queue,
            size,
            surface,
            window,
            depth_texture,

            camera,
            input_manager: input_manager::InputManager::new(),

            asset_registry,
            texture_pipeline,
            albedo_pipeline,

            fill_color,
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

    fn capture_mouse(&mut self) {
        use winit::window::CursorGrabMode;

        if self.input_manager.is_focused {
            return;
        }

        self.input_manager.update_focus(true);
        self.input_manager.clear_state();
        self.window.set_cursor_visible(false);
        let _ = self.set_cursor_middle();

        let _res = self
            .window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));

        #[cfg(feature = "debug_input")]
        if let Err(e) = _res {
            log::error!("Failed to capture mouse: {}", e);
        }
    }

    fn uncapture_mouse(&mut self) {
        use winit::window::CursorGrabMode;

        if !self.input_manager.is_focused {
            return;
        }

        self.input_manager.update_focus(false);
        let _ = self.window.set_cursor_grab(CursorGrabMode::None);
        self.window.set_cursor_visible(true);

        self.input_manager.clear_state();
    }

    pub fn process_inputs(&mut self) {
        if self.input_manager.escape_pressed() && self.input_manager.is_focused {
            #[cfg(feature = "debug_input")]
            log::info!("Uncapturing mouse");
            self.uncapture_mouse();
        } else if self.input_manager.left_click_pressed() && !self.input_manager.is_focused {
            #[cfg(feature = "debug_input")]
            log::info!("Capturing mouse");
            self.capture_mouse();
        }

        if !self.input_manager.is_focused {
            return;
        }

        let direction: glam::Vec3 = self.input_manager.get_direction().into();
        self.camera.move_camera(direction * 0.1f32);

        let mouse_delta = self.input_manager.consume_mouse_delta();
        self.camera
            .move_yaw_pitch(mouse_delta.0 as f32, mouse_delta.1 as f32);

        self.camera.update_projection_matrix(&self.queue);
    }

    pub fn set_cursor_middle(&mut self) -> Result<(), winit::error::ExternalError> {
        use winit::dpi::PhysicalPosition;

        if !self.input_manager.is_focused {
            return Ok(());
        }

        let size = self.window.inner_size();
        let new_position =
            PhysicalPosition::new(f64::from(size.width) / 2f64, f64::from(size.height) / 2f64);
        self.input_manager.set_mouse_middle(&new_position);

        self.window.set_cursor_position(new_position)
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.minimized {
            self.maintain();
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.format.add_srgb_suffix()),
            ..Default::default()
        });

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

            render_pass.set_pipeline(&self.texture_pipeline);
            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);

            for opaque in &mut self.asset_registry.opaque_models {
                let meshes = opaque.iter(&self.device);
                for mesh in meshes {
                    let texture = mesh.color_texture.as_ref();
                    let transform = &mesh.instance_transforms_buffer;
                    let vertices = &mesh.vertex_buffer;
                    let indices = &mesh.index_buffer;
                    let vertex_count = mesh.vertex_count;

                    // Vertices
                    render_pass.set_vertex_buffer(0, vertices.slice(..));
                    // Transforms for each instance
                    render_pass.set_vertex_buffer(1, transform.slice(..));

                    if let Some(texture) = texture {
                        render_pass.set_bind_group(1, texture, &[]);
                    }

                    if let Some(index_buffer) = indices {
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..vertex_count, 0, 0..mesh.instance_count);
                    } else {
                        render_pass.draw(0..vertex_count, 0..mesh.instance_count);
                    }
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
}
