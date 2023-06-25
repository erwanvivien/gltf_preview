use crate::model::{MeshPrimitive, Scene};

use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;

#[derive(Component)]
pub struct Vertices {
    pub count: u32,
    pub buffer: wgpu::Buffer,
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

#[derive(Default)]
pub struct AssetWorld {
    pub world: World,
}

fn create_index(mesh_primitive: &MeshPrimitive, device: &wgpu::Device) -> Option<Indices> {
    if mesh_primitive.indices.is_none() {
        return None;
    }

    let indices = mesh_primitive.indices.as_ref().unwrap();

    use wgpu::IndexFormat::{Uint16, Uint32};
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
        let indices = indices.iter().map(|i| *i as u16).collect::<Vec<u16>>();
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
            .flat_map(|scene| scene.nodes.drain(..))
            .flat_map(|node| node.meshes)
            .flat_map(|mesh| mesh.primitives)
            .collect::<Vec<_>>();

        for mesh_primitive in &mut mesh_primitives {
            mesh_primitive.create_texture_and_vertex_buffers(&device, &queue);

            let mut entity = world.spawn((
                Vertices {
                    count: mesh_primitive.index_count,
                    buffer: mesh_primitive.vertex_buffer.take().unwrap(),
                },
                Albedo {
                    albedo: mesh_primitive.material.base_albedo,
                    metallic: mesh_primitive.material.base_metallic,
                    roughness: mesh_primitive.material.base_roughness,
                },
            ));

            if let Some(indices) = create_index(mesh_primitive, &device) {
                entity.insert(indices);
            }

            if let Some(texture) = &mesh_primitive.material.texture {
                entity.insert(TextureBindGroup {
                    texture: texture.0.clone(),
                    bind_group: mesh_primitive.texture_bind_group.take().unwrap(),
                });
            }
        }

        Self { world }
    }
}
