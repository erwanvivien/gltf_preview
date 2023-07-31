use crate::{
    render::asset_store::types::{Mesh, MeshMaterial, MeshPrimitive, Node, Scene, Texture, Vertex},
    utils::load_file_buffer,
};

#[cfg(feature = "debug_gltf")]
static mut INDENT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[cfg(feature = "debug_gltf")]
use std::sync::atomic::Ordering;

#[cfg(feature = "debug_gltf")]
fn indent() -> String {
    let indentation = unsafe { INDENT.load(Ordering::Relaxed) };
    if indentation == 0 {
        return String::new();
    }

    format!("{}└", " ".repeat(indentation - 1))
}

struct GlobalGltf<'a> {
    buffers: Vec<gltf::buffer::Data>,
    materials: Vec<gltf::Material<'a>>,
    images: Vec<gltf::image::Data>,
}

fn parse_texture(global_gltf: &GlobalGltf, texture: &gltf::Texture) -> Texture {
    let image_index = texture.source().index();
    let image = &global_gltf.images[image_index];

    Texture(image.clone())
}

fn parse_material(global_gltf: &GlobalGltf, material: &gltf::Material) -> MeshMaterial {
    use gltf::material::AlphaMode::{Blend, Mask, Opaque};

    #[cfg(feature = "debug_gltf")]
    #[rustfmt::skip]
    log::info!("      {}> Material#{:?}: {:?}", indent(), material.index(), material.name());

    let material_pbr = material.pbr_metallic_roughness();
    let texture = material_pbr
        .base_color_texture()
        .map(|texture| parse_texture(global_gltf, &texture.texture()));

    let blend_mode = match material.alpha_mode() {
        Opaque | Mask => wgpu::BlendState::REPLACE,
        Blend => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
    };

    MeshMaterial {
        base_albedo: material_pbr.base_color_factor(),
        base_metallic: material_pbr.metallic_factor(),
        base_roughness: material_pbr.roughness_factor(),
        blend_mode,
        texture,
    }
}

fn parse_mesh_primitive(
    global_gltf: &GlobalGltf,
    primitive: &gltf::Primitive,
) -> Result<MeshPrimitive, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("    {}> Primitive#{:?}", indent(), primitive.index());

    let reader =
        primitive.reader(|buffer| global_gltf.buffers.get(buffer.index()).map(|b| &b.0[..]));

    // Use the default material if no material is set
    let material_index = primitive.material().index().unwrap_or(0);
    let material = parse_material(global_gltf, &global_gltf.materials[material_index]);

    let positions = reader.read_positions().ok_or(())?.collect::<Vec<_>>();
    let normals = reader.read_positions().ok_or(())?.collect::<Vec<_>>();
    let tex_coords = reader
        .read_tex_coords(0)
        .map(|tex| tex.into_f32().collect::<Vec<_>>());
    let colors = reader
        .read_colors(0)
        .map(|color| color.into_rgb_f32().collect::<Vec<_>>());

    let mut vertices = Vec::with_capacity(positions.len());
    let default_albedo = material.base_albedo[0..3].try_into().unwrap();
    for i in 0..positions.len() {
        let vertex = Vertex {
            position: positions[i],
            normal: normals[i],
            tex_coord: tex_coords.as_ref().map(|tex| tex[i]),
            color: colors.as_ref().map_or(default_albedo, |color| color[i]),
        };

        vertices.push(vertex);
    }

    let indices = reader
        .read_indices()
        .map(|indices| indices.into_u32().collect::<Vec<_>>());

    Ok(MeshPrimitive::new(vertices, indices, material))
}

fn parse_mesh(global_gltf: &GlobalGltf, mesh: &gltf::Mesh) -> Result<Mesh, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("  {}> Mesh#{:?}: {:?}", indent(), mesh.index(), mesh.name());

    let mut primitives = Vec::new();
    for primitive in mesh.primitives() {
        let primitive = parse_mesh_primitive(global_gltf, &primitive)?;
        primitives.push(MeshPrimitive {
            #[cfg(feature = "debug_gltf")]
            name: mesh.name().map(|s| s.to_string()),
            ..primitive
        });
    }

    Ok(Mesh { primitives })
}

fn parse_node(global_gltf: &GlobalGltf, node: &gltf::Node) -> Result<Node, ()> {
    use gltf::scene::Transform;

    #[cfg(feature = "debug_gltf")]
    log::info!("{}> Node#{:?}: {:?}", indent(), node.index(), node.name());

    let mut meshes = Vec::new();
    if let Some(mesh) = node.mesh() {
        let mesh = parse_mesh(global_gltf, &mesh)?;
        meshes.push(mesh);
    }

    let transform = match node.transform() {
        Transform::Matrix { matrix } => glam::Mat4::from_cols_array_2d(&matrix),
        Transform::Decomposed {
            translation,
            rotation,
            scale,
        } => {
            let rotation = glam::Quat::from_array(rotation);
            glam::Mat4::from_scale_rotation_translation(scale.into(), rotation, translation.into())
        }
    };

    #[cfg(feature = "debug_gltf")]
    let _ = unsafe { INDENT.fetch_add(1, Ordering::Relaxed) };

    let children: Vec<Node> = node
        .children()
        .map(|child| parse_node(global_gltf, &child))
        .collect::<Result<_, _>>()?;

    #[cfg(feature = "debug_gltf")]
    let _ = unsafe { INDENT.fetch_sub(1, Ordering::Relaxed) };

    Ok(Node {
        index: node.index(),
        meshes,
        transform,
        children,

        #[cfg(feature = "debug_gltf")]
        name: node.name().map(String::from),
    })
}

fn parse_scene(global_gltf: &GlobalGltf, scene: &gltf::Scene) -> Result<Scene, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("Scene: {:?}", scene.name());
    #[cfg(feature = "debug_gltf")]
    let _ = unsafe { INDENT.store(1, Ordering::Relaxed) };

    fn get_children_nodes(nodes: &mut Vec<Node>, mut current: Node) -> &mut Vec<Node> {
        for child in current.children.drain(..) {
            let child = Node {
                transform: current.transform * child.transform,
                ..child
            };
            get_children_nodes(nodes, child);
        }

        nodes.push(current);
        nodes
    }

    let mut nodes = Vec::new();
    for node in scene.nodes() {
        let node = parse_node(global_gltf, &node)?;
        nodes.append(get_children_nodes(&mut Vec::new(), node));
    }

    Ok(Scene {
        nodes,

        #[cfg(feature = "debug_gltf")]
        name: scene.name().map(String::from),
    })
}

/// Load a gltf file and return a list of scenes
///
/// # Errors
///
/// Returns `Err` if the file cannot be loaded or parsed
pub async fn load_scenes<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Scene>, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("⏹ Loading gltf file: {:?}", path.as_ref());

    let file_buffer = load_file_buffer(&path).await.map_err(|_| ())?;
    let (gltf, buffers, images) =
        gltf::import_slice(&file_buffer).expect("Failed to load gltf file");

    let materials = gltf.materials().collect::<Vec<_>>();

    let global_gltf = GlobalGltf {
        buffers,
        materials,
        images,
    };

    let mut scenes = Vec::new();
    for scene in gltf.scenes() {
        let scene = parse_scene(&global_gltf, &scene)?;
        scenes.push(scene);
    }

    #[cfg(feature = "debug_gltf")]
    log::info!("🆗 Loaded gltf file: {:?}", path.as_ref());

    Ok(scenes)
}
