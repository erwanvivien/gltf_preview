#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    pub(super) position: [f32; 3],
    pub(super) color: [f32; 3],
}

impl Vertex {
    pub const TRIANGLE_2D: &[Vertex] = &[
        Vertex {
            position: [-0.5, -0.5f32, 0f32],
            color: [1f32, 0f32, 0f32],
        },
        Vertex {
            position: [0.5f32, -0.5f32, 0f32],
            color: [0f32, 1f32, 0f32],
        },
        Vertex {
            position: [0.5f32, 0.5f32, 0f32],
            color: [0f32, 0f32, 1f32],
        },
        Vertex {
            position: [-0.5f32, 0.5f32, 0f32],
            color: [1f32, 1f32, 1f32],
        },
    ];

    pub const TRIANGLE_2D_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
