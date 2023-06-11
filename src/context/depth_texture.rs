pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
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
    address_mode_u: wgpu::AddressMode::ClampToEdge,
    address_mode_v: wgpu::AddressMode::ClampToEdge,
    address_mode_w: wgpu::AddressMode::ClampToEdge,
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
        let sampler = device.create_sampler(&DEPTH_SAMPLER_DESCRIPTOR);

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn create_texture_from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &crate::model::Texture,
    ) -> Self {
        let image = &texture.0;
        #[cfg(feature = "debug_gpu")]
        #[rustfmt::skip]
        log::info!("Texture {}x{} : {:?}", image.width, image.height, image.format);

        let size = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };

        use gltf::image::Format::{R8G8B8, R8G8B8A8};
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
        let texture_sampler = device.create_sampler(&TEXTURE_SAMPLER_DESCRIPTOR);

        Self {
            texture: color_texture,
            view: texture_view,
            sampler: texture_sampler,
        }
    }
}
