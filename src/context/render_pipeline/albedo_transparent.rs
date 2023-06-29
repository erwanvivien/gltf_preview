use wgpu::{Device, SurfaceConfiguration};

use crate::context::shaders::get_shader;
use crate::context::texture::Texture;

use crate::context::render_pipeline::PRIMITIVE_STATE;
use crate::context::utils::get_or_create_transform_bind_group_layout;
use crate::model::Vertex;

pub struct TransparentAlbedoPipeline {
    pub pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransparentAlbedoVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

impl TransparentAlbedoVertex {
    pub fn new(vertex: &Vertex) -> Self {
        Self {
            position: vertex.position,
            normal: vertex.normal,
            color: vertex.color,
        }
    }

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TransparentAlbedoVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl TransparentAlbedoPipeline {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        #[cfg(feature = "debug_gpu")]
        log::info!("Creating albedo pipeline");

        let main_shader = get_shader("albedo_shader");

        let transform_bind_group_layout = get_or_create_transform_bind_group_layout(device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Albedo Pipeline Layout"),
                bind_group_layouts: &[transform_bind_group_layout, camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let vertex_state = wgpu::VertexState {
            module: &main_shader,
            entry_point: "vs_main",
            buffers: &[TransparentAlbedoVertex::desc()],
        };

        let fragment_state = wgpu::FragmentState {
            module: &main_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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

        TransparentAlbedoPipeline { pipeline }
    }
}

impl crate::context::render_pipeline::RenderPipeline for TransparentAlbedoPipeline {}
