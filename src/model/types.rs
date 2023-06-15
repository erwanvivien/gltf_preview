use wgpu::util::DeviceExt;

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

    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_count: u32,

    pub color_bind_group: Option<wgpu::BindGroup>,
}

const COLOR_BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
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

impl MeshPrimitive {
    #[inline]
    pub fn color_bind_group_layout(device: &wgpu::Device) -> &wgpu::BindGroupLayout {
        static mut COLOR_BIND_GROUP_LAYOUT: Option<wgpu::BindGroupLayout> = None;

        if unsafe { COLOR_BIND_GROUP_LAYOUT.is_none() } {
            let color_bind_group_layout =
                device.create_bind_group_layout(&COLOR_BIND_GROUP_LAYOUT_DESCRIPTOR);

            unsafe {
                COLOR_BIND_GROUP_LAYOUT = Some(color_bind_group_layout);
            }
        }

        unsafe { COLOR_BIND_GROUP_LAYOUT.as_ref().unwrap() }
    }

    pub fn new(vertices: Vec<Vertex>, indices: Option<Vec<u32>>, material: MeshMaterial) -> Self {
        Self {
            vertices,
            indices,
            material,

            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,

            color_bind_group: None,
        }
    }

    pub fn create_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        use crate::context::Texture;

        if let Some(texture) = &self.material.texture {
            let color_texture = Texture::create_texture_from_image(&device, &queue, texture);

            let color_bind_group_layout = MeshPrimitive::color_bind_group_layout(&device);
            let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &color_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&color_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&color_texture.sampler),
                    },
                ],
                label: Some("Color Bind Group"),
            });

            self.color_bind_group = Some(color_bind_group);
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(self.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.indices.as_ref().map(|indices| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            })
        });

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = index_buffer;
        self.index_count = self
            .indices
            .as_ref()
            .map(Vec::len)
            .unwrap_or(self.vertices.len()) as u32;
    }

    pub fn draw_texture<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let color_bind_group = self.color_bind_group.as_ref().unwrap();
        render_pass.set_bind_group(0, color_bind_group, &[]);

        self.draw(render_pass);
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));

        if let Some(index_buffer) = &self.index_buffer {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        } else {
            render_pass.draw(0..self.index_count, 0..1);
        }
    }

    pub fn get_bind_group_layout(&self) -> Option<&wgpu::BindGroup> {
        self.color_bind_group.as_ref()
    }
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
