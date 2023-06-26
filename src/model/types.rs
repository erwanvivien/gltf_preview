use wgpu::util::DeviceExt;

use crate::context::utils::get_or_create_transform_bind_group_layout;

pub struct Texture(pub gltf::image::Data);

pub struct MeshMaterial {
    pub base_albedo: [f32; 4],
    pub base_metallic: f32,
    pub base_roughness: f32,
    pub texture: Option<Texture>,
}

impl std::fmt::Debug for MeshMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshMaterial").finish()
    }
}

pub struct MeshPrimitive {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    pub material: MeshMaterial,

    pub vertex_buffer: Option<wgpu::Buffer>,
    pub transform_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_count: u32,

    pub texture_bind_group: Option<wgpu::BindGroup>,
    pub transform_bind_group: Option<wgpu::BindGroup>,

    #[cfg(feature = "debug_gltf")]
    pub name: Option<String>,
}

impl std::fmt::Debug for MeshPrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = f.debug_struct("MeshPrimitive");

        #[cfg(feature = "debug_gltf")]
        output.field("name", &self.name);

        output.finish()
    }
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

            texture_bind_group: None,
            transform_bind_group: None,
            transform_buffer: None,

            #[cfg(feature = "debug_gltf")]
            name: None,
        }
    }

    pub fn create_texture_and_vertex_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        transform: glam::Mat4,
    ) {
        use crate::context::Texture;

        if let Some(texture) = &self.material.texture {
            let color_texture = Texture::create_texture_from_image(&device, &queue, &texture.0);

            let color_bind_group_layout = MeshPrimitive::color_bind_group_layout(&device);
            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

            self.texture_bind_group = Some(texture_bind_group);

            use crate::context::render_pipeline::TextureVertex;
            let texture_vertices = self
                .vertices
                .iter()
                .map(TextureVertex::new)
                .collect::<Vec<_>>();

            self.vertex_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&texture_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));
        }

        let transform_bind_group_layout = get_or_create_transform_bind_group_layout(&device);
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[transform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
            label: Some("Transform Bind Group"),
        });

        self.transform_bind_group = Some(transform_bind_group);
        self.transform_buffer = Some(transform_buffer);

        let index_buffer = self.indices.as_ref().map(|indices| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            })
        });

        self.index_buffer = index_buffer;
        self.index_count = self
            .indices
            .as_ref()
            .map(Vec::len)
            .unwrap_or(self.vertices.len()) as u32;
    }

    pub fn get_bind_group_layout(&self) -> Option<&wgpu::BindGroup> {
        self.texture_bind_group.as_ref()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: Option<[f32; 2]>,
    pub color: Option<[f32; 3]>,
}

#[derive(Debug)]
pub struct Mesh {
    pub primitives: Vec<MeshPrimitive>,
}

pub struct Node {
    pub index: usize,
    pub meshes: Vec<Mesh>,
    pub transform: glam::Mat4,
    pub children: Vec<Node>,
    #[cfg(feature = "debug_gltf")]
    pub name: Option<String>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("index", &self.index)
            .field("meshes", &self.meshes)
            .field("children", &self.children)
            .finish()
    }
}

impl IntoIterator for Node {
    type Item = (MeshPrimitive, glam::Mat4);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.meshes
            .into_iter()
            .flat_map(|mesh| {
                mesh.primitives
                    .into_iter()
                    .map(|primitive| (primitive, self.transform))
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[derive(Debug)]
pub struct Scene {
    pub nodes: Vec<Node>,
    #[cfg(feature = "debug_gltf")]
    pub name: Option<String>,
}
