use std::{path::Path, time::Instant};

use wgpu::util::DeviceExt;

use crate::{
    render::asset_store::{
        animation::Channel,
        material::Material,
        mesh::{Aabb, Mesh},
        node_layout::NodeLayout,
    },
    render::Texture,
    utils::load_file_buffer,
};

mod animation;
mod material;
mod mesh;
mod mesh_tangent;
mod node_layout;
mod utils;
mod world;

pub use material::TextureInfo;
pub use mesh::PrimitiveVertex;
pub use node_layout::{MeshIndex, NodeIndex};
pub use world::AssetRegistry;

#[derive(Debug, Clone)]
#[cfg(feature = "debug_gltf")]
pub struct ModelMetadata {
    pub name: String,
    pub path: String,
    pub scene_count: usize,
    pub mesh_count: usize,
    pub texture_count: usize,
    pub animation_count: usize,
    pub skin_count: usize,
    pub material_count: usize,
    pub node_count: usize,
    // camera_count: usize,
    // light_count: usize,
}

#[cfg(feature = "debug_gltf")]
impl ModelMetadata {
    pub fn new<P: AsRef<Path>>(path: P, gltf: &gltf::Document) -> Self {
        let name = path.as_ref().file_name().unwrap().to_str().unwrap();
        let path = path.as_ref().to_str().unwrap();

        let scene_count = gltf.scenes().len();
        let mesh_count = gltf.meshes().len();
        let texture_count = gltf.textures().len();
        let animation_count = gltf.animations().len();
        let skin_count = gltf.skins().len();
        let material_count = gltf.materials().len();
        let node_count = gltf.nodes().len();
        // let camera_count = gltf.cameras().len();
        // let light_count = gltf.lights().len();

        Self {
            name: String::from(name),
            path: String::from(path),
            scene_count,
            mesh_count,
            texture_count,
            animation_count,
            skin_count,
            material_count,
            node_count,
            // camera_count,
            // light_count,
        }
    }
}

pub struct Model {
    index: usize,

    #[cfg(feature = "debug_gltf")]
    metadata: ModelMetadata,
    packed_primitives: PackedPrimitives,
    textures: Vec<Texture>,

    cached_model_render: Vec<Option<ModelRender>>,
}

pub struct ModelRender {
    #[cfg(feature = "debug_gltf")]
    pub metadata: ModelMetadata,

    pub instance_transforms_buffer: wgpu::Buffer,
    pub instance_count: u32,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,

    pub color_texture: Option<wgpu::BindGroup>,

    pub vertex_count: u32,
}

impl Model {
    fn update_index(&mut self, device: &wgpu::Device, index: usize, start_time: &Instant) {
        let mesh = &self.packed_primitives.per_primitives[index];

        // TODO: remove clone
        let mut mesh_instance_transforms = mesh.instance_transforms.clone();

        if let Some(_cached) = &self.cached_model_render[index] {
            let mut cached = true;

            let elapsed_time = start_time.elapsed().as_micros() as f32 / 1e6;

            for (index, primitive_animations) in mesh.instance_animations.iter().enumerate() {
                if primitive_animations.is_empty() {
                    continue;
                }

                cached = false;

                let mut node_transform = mesh_instance_transforms[index];

                use animation::PropertyValue;
                for animation_channel in primitive_animations {
                    let interpolation = animation_channel.interpolate(elapsed_time);

                    node_transform = match interpolation {
                        PropertyValue::Rotation(rotation) => {
                            node_transform * glam::Mat4::from_quat(rotation)
                        }
                        PropertyValue::Scale(scale) => {
                            node_transform * glam::Mat4::from_scale(scale)
                        }
                        PropertyValue::Translation(translation) => {
                            node_transform * glam::Mat4::from_translation(translation)
                        }
                        PropertyValue::MorphTargetWeights(_) => unimplemented!(),
                    }
                }

                mesh_instance_transforms[index] = node_transform;
            }

            if cached {
                return;
            }
        };

        // let vertex_range = mesh.vertex_range.0 as u64..mesh.vertex_range.1 as u64;
        // let index_range = mesh.index_range.0 as u64..mesh.index_range.1 as u64;

        // TODO: FIX use global vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&mesh.staging_vertex),
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = mesh.staging_index.as_ref().map(|indices| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                contents: bytemuck::cast_slice(&indices),
                label: Some("Index Buffer"),
                usage: wgpu::BufferUsages::INDEX,
            })
        });

        let instance_transforms_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Transform Buffer"),
                contents: bytemuck::cast_slice(&mesh_instance_transforms),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let color_texture = mesh.material.color_texture.map(|color_texture| {
            let color_texture = &self.textures[color_texture.texture_index];
            color_texture.create_bind_group(device)
        });

        let vertex_count = if mesh.staging_index.is_some() {
            mesh.index_range.1 - mesh.index_range.0
        } else {
            mesh.vertex_range.1 - mesh.vertex_range.0
        };
        let vertex_count = u32::try_from(vertex_count).expect("Not a valid vertex count");

        let model_render = ModelRender {
            #[cfg(feature = "debug_gltf")]
            metadata: self.metadata.clone(),
            instance_transforms_buffer,
            instance_count: mesh.instance_count,
            vertex_buffer,
            index_buffer,
            color_texture,
            vertex_count,
        };

        self.cached_model_render[index] = Some(model_render);
    }

    pub fn iter(&mut self, device: &wgpu::Device, start_time: &Instant) -> Vec<&ModelRender> {
        if self.cached_model_render.len() != self.packed_primitives.per_primitives.len() {
            self.cached_model_render
                .resize_with(self.packed_primitives.per_primitives.len(), || None)
        }

        for i in 0..self.packed_primitives.per_primitives.len() {
            self.update_index(device, i, start_time);
        }

        self.cached_model_render.iter().flatten().collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ModelError {
    InvalidPath,
    InvalidGltf,

    NoScene,
}

type Range = (usize, usize);

#[cfg(feature = "debug_gltf")]
pub struct PerPrimitiveMetadata {
    pub mesh_name: Option<String>,
}

pub struct PerPrimitive {
    #[cfg(feature = "debug_gltf")]
    pub metadata: PerPrimitiveMetadata,

    id: usize,
    index_range: Range,
    vertex_range: Range,

    instance_animations: Vec<Vec<Channel>>,
    instance_transforms: Vec<glam::Mat4>,
    instance_count: u32,

    staging_index: Option<Vec<u32>>,
    staging_vertex: Vec<PrimitiveVertex>,

    material: Material,

    #[cfg(feature = "debug_gltf")]
    instance_node_indices: Vec<NodeIndex>,
}

impl PerPrimitive {
    pub fn transform_desc() -> wgpu::VertexBufferLayout<'static> {
        use wgpu::VertexAttribute;
        const ATTRIBUTES: [VertexAttribute; 4] = wgpu::vertex_attr_array![
            11 => Float32x4, 12 => Float32x4, 13 => Float32x4, 14 => Float32x4
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<glam::Mat4>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

struct PackedPrimitives {
    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,

    per_primitives: Vec<PerPrimitive>,

    aabb: Aabb,
}

static mut MODEL_INDEX: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl Model {
    pub fn from_bytes<P: AsRef<Path>>(
        path: P,
        bytes: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self, ModelError> {
        use ModelError::*;

        let (gltf, buffers, images) = gltf::import_slice(bytes).map_err(|_| InvalidGltf)?;

        if gltf.scenes().len() == 0 {
            return Err(NoScene);
        }

        #[cfg(feature = "debug_gltf")]
        let metadata = ModelMetadata::new(path, &gltf);

        let node_layout = NodeLayout::from_gltf(gltf.nodes(), gltf.animations(), &buffers);
        let meshes = gltf
            .meshes()
            .map(|mesh| Mesh::parse(&node_layout, &mesh, &buffers))
            .collect::<Vec<_>>();

        let mut index_offset = 0;
        let mut vertex_offset = 0;
        let mut per_primitives = Vec::new();
        let mut global_indices = Vec::new();
        let mut global_vertices = Vec::new();
        let mut textures = Vec::with_capacity(images.len());
        let mut aabb = Aabb::ZERO;

        for mesh in meshes {
            aabb = aabb.union(&mesh.aabb);

            for primitive in mesh.primitives.into_iter() {
                let index_count = primitive.indices.as_ref().map(|vec| vec.len()).unwrap_or(0);
                let vertex_count = primitive.vertices.len();

                let index_range = (index_offset, index_offset + index_count);
                let vertex_range = (vertex_offset, vertex_offset + vertex_count);

                index_offset += index_count;
                vertex_offset += vertex_count;

                global_vertices.extend_from_slice(&primitive.vertices);
                if let Some(indices) = &primitive.indices {
                    global_indices.extend_from_slice(&indices);
                }

                let primitive = PerPrimitive {
                    #[cfg(feature = "debug_gltf")]
                    metadata: PerPrimitiveMetadata {
                        mesh_name: mesh.name.clone(),
                    },
                    id: primitive.index,
                    index_range,
                    vertex_range,
                    staging_index: primitive.indices,
                    staging_vertex: primitive.vertices,
                    material: primitive.material.clone(),
                    instance_animations: primitive.instance_animations,
                    instance_transforms: primitive.instance_transforms,
                    instance_count: primitive.instance_count,

                    #[cfg(feature = "debug_gltf")]
                    instance_node_indices: primitive.instance_node_indices,
                };
                per_primitives.push(primitive);
            }
        }

        for image in images {
            let texture = Texture::create_texture_from_image(device, queue, &image);
            textures.push(texture);
        }

        let global_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&global_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let global_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&global_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let packed_primitives = PackedPrimitives {
            index_buffer: global_index_buffer,
            vertex_buffer: global_vertex_buffer,
            per_primitives,
            aabb,
        };

        Ok(Model {
            index: unsafe { MODEL_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed) },

            #[cfg(feature = "debug_gltf")]
            metadata,
            packed_primitives,
            textures,

            cached_model_render: Vec::new(),
        })
    }

    pub async fn from_path<P: AsRef<Path>>(
        path: P,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Model, ModelError> {
        use ModelError::*;

        #[cfg(feature = "debug_gltf")]
        log::info!("⏹ Loading gltf file: {:?}", path.as_ref());

        let file_buffer = load_file_buffer(&path).await.map_err(|_| InvalidPath)?;
        Self::from_bytes(&path, &file_buffer, device, queue)
    }
}
