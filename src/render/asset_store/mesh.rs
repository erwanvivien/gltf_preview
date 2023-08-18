#[cfg(feature = "debug_gltf")]
use crate::render::asset_store::utils::indent;
use crate::render::{
    asset_store::{
        material::Material, mesh_tangent::generate_tangents, MeshIndex, NodeIndex, NodeLayout,
    },
    shaders::kind::ShaderKinds,
};

use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Aabb {
    min: glam::Vec3,
    max: glam::Vec3,
}

impl Aabb {
    pub const ZERO: Self = Self {
        min: glam::Vec3::ZERO,
        max: glam::Vec3::ZERO,
    };

    pub fn unions(aabbs: &[Self]) -> Self {
        if aabbs.is_empty() {
            return Self::ZERO;
        }

        let mut min = glam::Vec3::splat(f32::MAX);
        let mut max = glam::Vec3::splat(f32::MIN);

        for aabb in aabbs {
            min = min.min(aabb.min);
            max = max.max(aabb.max);
        }

        Self { min, max }
    }

    pub fn center(&self) -> glam::Vec3 {
        (self.min + self.max) / 2.0
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

#[repr(align(16), C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveVertex {
    pub position: glam::Vec3,
    _padding_0: f32,
    pub normal: glam::Vec3,
    _padding_1: f32,
    pub tex_coord_0: glam::Vec2,
    pub tex_coord_1: glam::Vec2,
    pub tangent: glam::Vec4,
    pub weights: glam::Vec4,
    pub joints: glam::UVec4,
    pub color: glam::Vec4,
    pub shader_kinds: ShaderKinds,
    _padding_2: [f32; 3],
}

impl PrimitiveVertex {
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 11] = wgpu::vertex_attr_array![
            0 => Float32x3, 1 => Float32, 2 => Float32x3,
            3 => Float32, 4 => Float32x2, 5 => Float32x2,
            6 => Float32x4, 7 => Float32x4, 8 => Uint32x4,
            9 => Float32x4, 10 => Uint32
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl PrimitiveVertex {
    #[rustfmt::skip]
    pub fn new<V3: Into<glam::Vec3> + Copy, V2: Into<glam::Vec2> + Copy,
                V4: Into<glam::Vec4> + Copy, U4: Into<glam::UVec4> + Copy>(
        position: V3, normal: V3, tex_coord_0: V2,
        tex_coord_1: V2, tangent: V4, weights: V4,
        joints: U4, color: V4, shader_kinds: ShaderKinds
    ) -> Self {
        Self {
            position: position.into(),
            normal: normal.into(),
            tex_coord_0: tex_coord_0.into(),
            tex_coord_1: tex_coord_1.into(),
            tangent: tangent.into(),
            weights: weights.into(),
            joints: joints.into(),
            color: color.into(),
            shader_kinds,
            _padding_0: 0.0,
            _padding_1: 0.0,
            _padding_2: [0.0; 3],
        }
    }
}

impl PrimitiveVertex {
    pub const DEFAULT_NORMAL: [f32; 3] = [1f32; 3];
    pub const DEFAULT_TEX: [f32; 2] = [0f32; 2];
    pub const DEFAULT_TANGENT: [f32; 4] = [1f32; 4];
    pub const DEFAULT_WEIGHTS: [f32; 4] = [0f32; 4];
    pub const DEFAULT_JOINTS: [u32; 4] = [0u32; 4];
    pub const DEFAULT_COLOR: [f32; 4] = [1f32; 4];
}

pub struct Primitive {
    pub index: usize,
    pub vertices: Vec<PrimitiveVertex>,
    pub indices: Option<Vec<u32>>,
    pub material: Material,
    pub aabb: Aabb,
    pub instance_transforms: Vec<glam::Mat4>,
    pub instance_count: u32,

    #[cfg(feature = "debug_gltf")]
    pub instance_node_indices: Vec<NodeIndex>,
}

pub struct Mesh {
    #[cfg(feature = "debug_gltf")]
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
    pub aabb: Aabb,
}

static mut PRIMITIVE_COUNT: AtomicUsize = AtomicUsize::new(0);

impl Mesh {
    fn new(primitives: Vec<Primitive>, aabb: Aabb, _name: Option<String>) -> Self {
        Self {
            #[cfg(feature = "debug_gltf")]
            name: _name,
            primitives,
            aabb,
        }
    }

    pub fn parse(
        node_layout: &NodeLayout,
        mesh: &gltf::Mesh,
        buffers: &[gltf::buffer::Data],
    ) -> Self {
        #[cfg(feature = "debug_gltf")]
        log::info!("{}Mesh#{}: {:?}", indent(), mesh.index(), mesh.name());

        let mut primitives: Vec<Primitive> = Vec::new();
        let mut global_aabb = Aabb::ZERO;

        for primitive in mesh.primitives() {
            let index = unsafe { PRIMITIVE_COUNT.fetch_add(1, Ordering::Relaxed) };
            let material: Material = primitive.material().into();

            let mesh_index = u32::try_from(mesh.index()).expect("Mesh index overflow");
            let mesh_index = MeshIndex(mesh_index);
            let mesh_nodes = node_layout.mesh_nodes.get(&mesh_index).unwrap();

            let instance_transforms = mesh_nodes
                .iter()
                .map(|node_index| node_layout.get_node_transform(*node_index))
                .collect::<Vec<_>>();
            let instance_count =
                u32::try_from(instance_transforms.len()).expect("Instance count overflow");

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let aabb = {
                let bounds = primitive.bounding_box();
                let min = bounds.min.into();
                let max = bounds.max.into();

                Aabb { min, max }
            };

            global_aabb = global_aabb.union(&aabb);

            let mut shader_kinds = ShaderKinds::NONE;

            let positions = read_positions(&reader);
            let positions_len = positions.len();

            let normals = read_normals(&reader);
            let tangents = read_tangents(&reader);
            let tex_coords_0 = read_tex_coords(&reader, 0);
            let tex_coords_1 = read_tex_coords(&reader, 1);
            let weights = read_weights(&reader);
            let joints = read_joints(&reader);
            let colors = read_colors(&reader);

            let has_normals = (normals.is_some(), ShaderKinds::NORMAL);
            let has_tangents = (tangents.is_some(), ShaderKinds::TANGENT);
            let has_tex_coords_0 = (tex_coords_0.is_some(), ShaderKinds::TEX_COORD_0);
            let has_tex_coords_1 = (tex_coords_1.is_some(), ShaderKinds::TEX_COORD_1);
            let has_weights = (weights.is_some(), ShaderKinds::WEIGHT);
            let has_joints = (joints.is_some(), ShaderKinds::JOINT);
            let has_colors = (colors.is_some(), ShaderKinds::COLOR);

            #[rustfmt::skip]
            let has_kind = [
                has_normals, has_tangents, has_tex_coords_0,
                has_tex_coords_1, has_weights, has_joints, has_colors,
            ];
            for (has, kind) in has_kind {
                if has {
                    shader_kinds = shader_kinds | kind;
                }
            }

            // TODO: Remove this hack
            if material.color != [1f32; 4] {
                shader_kinds = shader_kinds | ShaderKinds::COLOR;
            }

            let normals =
                normals.unwrap_or_else(|| vec![PrimitiveVertex::DEFAULT_NORMAL; positions_len]);
            let tangents =
                tangents.unwrap_or_else(|| vec![PrimitiveVertex::DEFAULT_TANGENT; positions_len]);
            let tex_coords_0 =
                tex_coords_0.unwrap_or_else(|| vec![PrimitiveVertex::DEFAULT_TEX; positions_len]);
            let tex_coords_1 =
                tex_coords_1.unwrap_or_else(|| vec![PrimitiveVertex::DEFAULT_TEX; positions_len]);
            let weights =
                weights.unwrap_or_else(|| vec![PrimitiveVertex::DEFAULT_WEIGHTS; positions_len]);
            let joints =
                joints.unwrap_or_else(|| vec![PrimitiveVertex::DEFAULT_JOINTS; positions_len]);
            let colors = colors.unwrap_or_else(|| vec![material.color; positions_len]);

            let mut vertices = Vec::with_capacity(positions.len());
            for i in 0..positions.len() {
                vertices.push(PrimitiveVertex::new(
                    positions[i],
                    normals[i],
                    tex_coords_0[i],
                    tex_coords_1[i],
                    tangents[i],
                    weights[i],
                    joints[i],
                    colors[i],
                    shader_kinds,
                ));
            }

            let indices = read_indices(&reader);
            if !positions.is_empty()
                && shader_kinds.is_normal()
                && shader_kinds.is_tex_coord()
                && !shader_kinds.is_tangent()
            {
                generate_tangents(indices.as_ref(), &mut vertices);
            } else {
                #[cfg(feature = "debug_gltf")]
                log::warn!("Mesh#{}: Failed tangent generation", mesh.index());
            }

            let primitive = Primitive {
                index,
                vertices,
                indices,
                material,
                aabb,
                instance_transforms,
                instance_count,

                #[cfg(feature = "debug_gltf")]
                instance_node_indices: mesh_nodes.clone(),
            };
            primitives.push(primitive);
        }

        #[cfg(feature = "debug_gltf")]
        let name = mesh.name().map(|s| s.to_string());
        #[cfg(not(feature = "debug_gltf"))]
        let name = None;

        Mesh::new(primitives, global_aabb, name)
    }
}

// From https://github.com/adrien-ben/gltf-viewer-rs/blob/eebdd3/crates/libs/model/src/mesh.rs#L248-L332
fn read_indices<'a, 's, F>(reader: &gltf::mesh::Reader<'a, 's, F>) -> Option<Vec<u32>>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_indices()
        .map(|indices| indices.into_u32().collect::<Vec<_>>())
}

fn read_positions<'a, 's, F>(reader: &gltf::mesh::Reader<'a, 's, F>) -> Vec<[f32; 3]>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_positions()
        .expect("Position primitives should be present")
        .collect()
}

fn read_normals<'a, 's, F>(reader: &gltf::mesh::Reader<'a, 's, F>) -> Option<Vec<[f32; 3]>>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader.read_normals().map(|normals| normals.collect())
}

fn read_tex_coords<'a, 's, F>(
    reader: &gltf::mesh::Reader<'a, 's, F>,
    channel: u32,
) -> Option<Vec<[f32; 2]>>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_tex_coords(channel)
        .map(|coords| coords.into_f32().collect())
}

fn read_tangents<'a, 's, F>(reader: &gltf::mesh::Reader<'a, 's, F>) -> Option<Vec<[f32; 4]>>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader.read_tangents().map(|tangents| tangents.collect())
}

fn read_weights<'a, 's, F>(reader: &gltf::mesh::Reader<'a, 's, F>) -> Option<Vec<[f32; 4]>>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_weights(0)
        .map(|weights| weights.into_f32().collect())
}

fn read_joints<'a, 's, F>(reader: &gltf::mesh::Reader<'a, 's, F>) -> Option<Vec<[u32; 4]>>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader.read_joints(0).map(|joints| {
        joints
            .into_u16()
            .map(|[a, b, c, d]| [u32::from(a), u32::from(b), u32::from(c), u32::from(d)])
            .collect()
    })
}

fn read_colors<'a, 's, F>(reader: &gltf::mesh::Reader<'a, 's, F>) -> Option<Vec<[f32; 4]>>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_colors(0)
        .map(|colors| colors.into_rgba_f32().collect())
}
