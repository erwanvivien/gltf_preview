use wgpu::{Device, SurfaceConfiguration};

use crate::context::shaders::get_shader;
use crate::context::texture::Texture;
use crate::model::Vertex;

use crate::context::render_pipeline::PRIMITIVE_STATE;

pub struct TexturePipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl TexturePipeline {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        #[cfg(feature = "debug_gpu")]
        log::info!("Creating texture pipeline");

        let main_shader = get_shader("texture_shader");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Main Render Pipeline Layout"),
                bind_group_layouts: &[texture_bind_group_layout, camera_bind_group_layout],
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

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        });

        TexturePipeline { pipeline }
    }
}

impl crate::context::render_pipeline::RenderPipeline for TexturePipeline {}
