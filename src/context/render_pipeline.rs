use wgpu::{Device, SurfaceConfiguration};

use crate::context::depth_texture::Texture;
use crate::context::vertex::Vertex;

const PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
    topology: wgpu::PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: wgpu::FrontFace::Ccw,
    cull_mode: Some(wgpu::Face::Back),
    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
    polygon_mode: wgpu::PolygonMode::Fill,
    // Requires Features::DEPTH_CLIP_CONTROL
    unclipped_depth: false,
    // Requires Features::CONSERVATIVE_RASTERIZATION
    conservative: false,
};

pub(super) fn create_main_render_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let main_shader = device.create_shader_module(wgpu::include_wgsl!("main_shader.wgsl"));
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Main Render Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout],
        push_constant_ranges: &[],
    });

    let vertex_state = wgpu::VertexState {
        module: &main_shader,
        entry_point: "vs_main",
        buffers: &[Vertex::desc()],
    };

    let fragment_state = wgpu::FragmentState {
        module: &main_shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })],
    };

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Main Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: vertex_state,
        fragment: Some(fragment_state),
        primitive: PRIMITIVE_STATE,
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: u64::MAX,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
