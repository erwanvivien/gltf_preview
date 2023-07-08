mod load;
mod types;

pub use load::load_scenes;
pub use types::{Mesh, MeshPrimitive, Scene, Texture, Vertex};

use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;

#[derive(Component)]
pub struct Vertices {
    pub count: u32,
    pub buffer: wgpu::Buffer,
    pub transform_bind_group: wgpu::BindGroup,

    #[cfg(feature = "debug_gltf")]
    pub name: Option<String>,
}

#[derive(Component)]
pub struct Indices {
    pub buffer: wgpu::Buffer,
    pub format: wgpu::IndexFormat,
}

#[derive(Component)]
pub struct Color {
    pub color: [f32; 4],
}

#[derive(Component)]
pub struct Albedo {
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

#[derive(Component)]
pub struct TextureBindGroup {
    pub texture: gltf::image::Data,
    pub bind_group: wgpu::BindGroup,
}

#[derive(Component)]
pub struct Transparency;

#[derive(Default)]
pub struct AssetWorld {
    pub world: World,
}

fn create_index(mesh_primitive: &MeshPrimitive, device: &wgpu::Device) -> Option<Indices> {
    use wgpu::IndexFormat::{Uint16, Uint32};

    let indices = mesh_primitive.indices.as_ref()?;

    if mesh_primitive.vertices.len() > u16::MAX as usize {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        Some(Indices {
            buffer,
            format: Uint32,
        })
    } else {
        let indices = indices
            .iter()
            .map(|i| u16::try_from(*i).unwrap())
            .collect::<Vec<u16>>();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        Some(Indices {
            buffer,
            format: Uint16,
        })
    }
}

impl AssetWorld {
    pub fn new(scenes: &mut [Scene], device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut world = World::default();

        let mut mesh_primitives = scenes
            .iter_mut()
            .flat_map(|scene| {
                scene
                    .nodes
                    .drain(..)
                    .flat_map(|node| node.into_iter().collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        for (mesh_primitive, transform) in &mut mesh_primitives {
            mesh_primitive.create_texture_and_vertex_buffers(device, queue, *transform);

            let mut entity = world.spawn((
                Vertices {
                    count: mesh_primitive.index_count,
                    buffer: mesh_primitive.vertex_buffer.take().unwrap(),
                    transform_bind_group: mesh_primitive.transform_bind_group.take().unwrap(),

                    #[cfg(feature = "debug_gltf")]
                    name: mesh_primitive.name.take(),
                },
                Albedo {
                    albedo: mesh_primitive.material.base_albedo,
                    metallic: mesh_primitive.material.base_metallic,
                    roughness: mesh_primitive.material.base_roughness,
                },
            ));

            if let Some(indices) = create_index(mesh_primitive, device) {
                entity.insert(indices);
            }

            if let Some(texture) = &mesh_primitive.material.texture {
                entity.insert(TextureBindGroup {
                    texture: texture.0.clone(),
                    bind_group: mesh_primitive.texture_bind_group.take().unwrap(),
                });
            }

            if mesh_primitive.material.blend_mode == wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING
            {
                entity.insert(Transparency);
            }
        }

        Self { world }
    }
}
