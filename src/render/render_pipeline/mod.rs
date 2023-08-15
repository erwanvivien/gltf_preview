pub const PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
    topology: wgpu::PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: wgpu::FrontFace::Cw,
    cull_mode: Some(wgpu::Face::Back),
    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
    polygon_mode: wgpu::PolygonMode::Fill,
    // Requires Features::DEPTH_CLIP_CONTROL
    unclipped_depth: false,
    // Requires Features::CONSERVATIVE_RASTERIZATION
    conservative: false,
};

mod texture;

trait RenderPipeline {}

pub use texture::TexturePipeline;
