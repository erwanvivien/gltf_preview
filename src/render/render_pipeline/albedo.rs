use wgpu::{Device, SurfaceConfiguration};

use crate::render::asset_store::{PerPrimitive, PrimitiveVertex};
use crate::render::shaders::get_shader;
use crate::render::texture::Texture;

use crate::render::render_pipeline::PRIMITIVE_STATE;

pub struct AlbedoPipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl AlbedoPipeline {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        #[cfg(feature = "debug_gpu")]
        log::info!("Creating albedo pipeline");

        let main_shader = get_shader("albedo_shader");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Albedo Pipeline Layout"),
                bind_group_layouts: &[camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let vertex_state = wgpu::VertexState {
            module: &main_shader,
            entry_point: "vs_main",
            buffers: &[PrimitiveVertex::desc(), PerPrimitive::transform_desc()],
        };

        let fragment_state = wgpu::FragmentState {
            module: &main_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format.add_srgb_suffix(),
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Albedo Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: vertex_state,
            fragment: Some(fragment_state),
            primitive: PRIMITIVE_STATE,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: u64::MAX,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        AlbedoPipeline { pipeline }
    }
}

impl crate::render::render_pipeline::RenderPipeline for AlbedoPipeline {}