pub struct Texture(pub gltf::image::Data);

pub struct MeshMaterial {
    pub base_albedo: [f32; 4],
    pub base_metallic: f32,
    pub base_roughness: f32,
    pub texture: Option<Texture>,
}

pub struct MeshPrimitive {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    pub material: MeshMaterial,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    _padding: [f32; 0],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            tex_coords,
            _padding: [0.0; 0],
        }
    }

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3, 3 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct Mesh {
    pub primitives: Vec<MeshPrimitive>,
}

pub struct Node {
    pub index: usize,
    pub meshes: Vec<Mesh>,
    pub transform: glam::Mat4,
    pub children: Vec<usize>,
}

pub struct Scene {
    pub nodes: Vec<Node>,
}
