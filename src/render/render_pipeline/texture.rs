use wgpu::{Device, SurfaceConfiguration};

use crate::render::shaders::get_shader;
use crate::render::texture::Texture;

use crate::render::asset_store::Vertex;
use crate::render::render_pipeline::PRIMITIVE_STATE;
use crate::render::utils::get_or_create_transform_bind_group_layout;

pub struct TexturePipeline {
    pub pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

impl TextureVertex {
    pub fn new(vertex: &Vertex) -> Self {
        Self {
            position: vertex.position,
            normal: vertex.normal,
            tex_coords: vertex
                .tex_coord
                .expect("Vertex must have texture coordinates"),
        }
    }

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
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

        let transform_bind_group_layout = get_or_create_transform_bind_group_layout(device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Main Render Pipeline Layout"),
                bind_group_layouts: &[
                    texture_bind_group_layout,
                    transform_bind_group_layout,
                    camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let vertex_state = wgpu::VertexState {
            module: &main_shader,
            entry_point: "vs_main",
            buffers: &[TextureVertex::desc()],
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
            label: Some("Main Render Pipeline"),
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

        TexturePipeline { pipeline }
    }
}

impl crate::render::render_pipeline::RenderPipeline for TexturePipeline {}
