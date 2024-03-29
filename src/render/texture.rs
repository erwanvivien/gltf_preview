pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: &'static wgpu::Sampler,
}

impl Texture {
    const COLOR_TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
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
        };

    #[inline]
    pub fn color_texture_bind_group_layout(device: &wgpu::Device) -> &wgpu::BindGroupLayout {
        static mut COLOR_TEXTURE_BIND_GROUP_LAYOUT: Option<wgpu::BindGroupLayout> = None;

        if unsafe { COLOR_TEXTURE_BIND_GROUP_LAYOUT.is_none() } {
            let color_bind_group_layout =
                device.create_bind_group_layout(&Self::COLOR_TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR);

            unsafe {
                COLOR_TEXTURE_BIND_GROUP_LAYOUT = Some(color_bind_group_layout);
            }
        }

        unsafe { COLOR_TEXTURE_BIND_GROUP_LAYOUT.as_ref().unwrap() }
    }

    pub fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let bind_group_layout = Self::color_texture_bind_group_layout(device);
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(self.sampler),
                },
            ],
            label: Some("Color Texture Bind Group"),
        })
    }
}

const DEPTH_SAMPLER_DESCRIPTOR: wgpu::SamplerDescriptor = wgpu::SamplerDescriptor {
    address_mode_u: wgpu::AddressMode::ClampToEdge,
    address_mode_v: wgpu::AddressMode::ClampToEdge,
    address_mode_w: wgpu::AddressMode::ClampToEdge,
    mag_filter: wgpu::FilterMode::Linear,
    min_filter: wgpu::FilterMode::Linear,
    mipmap_filter: wgpu::FilterMode::Nearest,
    compare: Some(wgpu::CompareFunction::LessEqual), // 5.
    lod_min_clamp: 0.0,
    lod_max_clamp: 100.0,
    anisotropy_clamp: 1,
    border_color: None,
    label: Some("Depth texture sampler"),
};

const TEXTURE_SAMPLER_DESCRIPTOR: wgpu::SamplerDescriptor = wgpu::SamplerDescriptor {
    address_mode_u: wgpu::AddressMode::Repeat,
    address_mode_v: wgpu::AddressMode::Repeat,
    address_mode_w: wgpu::AddressMode::Repeat,
    mag_filter: wgpu::FilterMode::Linear,
    min_filter: wgpu::FilterMode::Linear,
    mipmap_filter: wgpu::FilterMode::Nearest,
    compare: None,
    lod_min_clamp: 0.0,
    lod_max_clamp: 100.0,
    anisotropy_clamp: 1,
    border_color: None,
    label: Some("Depth texture sampler"),
};

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn get_singleton_depth_sampler(device: &wgpu::Device) -> &'static wgpu::Sampler {
        static mut SAMPLER: Option<wgpu::Sampler> = None;

        if unsafe { &SAMPLER }.is_none() {
            unsafe {
                SAMPLER = Some(device.create_sampler(&DEPTH_SAMPLER_DESCRIPTOR));
            }
        }

        unsafe { SAMPLER.as_ref().unwrap() }
    }

    pub fn get_singleton_texture_sampler(device: &wgpu::Device) -> &'static wgpu::Sampler {
        static mut SAMPLER: Option<wgpu::Sampler> = None;

        if unsafe { &SAMPLER }.is_none() {
            unsafe {
                SAMPLER = Some(device.create_sampler(&TEXTURE_SAMPLER_DESCRIPTOR));
            }
        }

        unsafe { SAMPLER.as_ref().unwrap() }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        #[cfg(feature = "debug_gpu")]
        log::info!("Creating depth texture");

        let dimension = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: dimension,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&texture_descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::get_singleton_depth_sampler(device);

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn create_texture_from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &gltf::image::Data,
    ) -> Self {
        use gltf::image::Format::{R8G8B8, R8G8B8A8};

        #[cfg(feature = "debug_gpu")]
        #[rustfmt::skip]
        log::info!("Texture {}x{} : {:?}", image.width, image.height, image.format);

        let size = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };

        assert!(
            matches!(image.format, R8G8B8 | R8G8B8A8),
            "Unsupported texture format"
        );

        let buffer = if image.format == R8G8B8A8 {
            image.pixels.clone()
        } else {
            let mut buffer = Vec::with_capacity(image.width as usize * image.height as usize * 4);
            for pixel in image.pixels.chunks_exact(3) {
                buffer.extend_from_slice(pixel);
                buffer.push(255);
            }
            buffer
        };

        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &color_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: (4 * image.width).into(),
                rows_per_image: image.height.into(),
            },
            size,
        );

        let texture_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = Self::get_singleton_texture_sampler(device);

        Self {
            texture: color_texture,
            view: texture_view,
            sampler: texture_sampler,
        }
    }
}
