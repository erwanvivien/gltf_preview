use crate::model::types::{Mesh, MeshMaterial, MeshPrimitive, Node, Scene, Texture, Vertex};

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
    #[cfg(feature = "debug_gltf")]
    #[rustfmt::skip]
    log::info!("      > Material#{:?}: {:?}", material.index(), material.name());

    let material_pbr = material.pbr_metallic_roughness();
    let texture = material_pbr
        .base_color_texture()
        .map(|texture| parse_texture(global_gltf, &texture.texture()));

    MeshMaterial {
        base_albedo: material_pbr.base_color_factor(),
        base_metallic: material_pbr.metallic_factor(),
        base_roughness: material_pbr.roughness_factor(),
        texture,
    }
}

fn parse_mesh_primitive(
    global_gltf: &GlobalGltf,
    primitive: gltf::Primitive,
) -> Result<MeshPrimitive, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("    > Primitive#{:?}", primitive.index());

    let reader =
        primitive.reader(|buffer| global_gltf.buffers.get(buffer.index()).map(|b| &b.0[..]));

    let positions = reader.read_positions().ok_or_else(|| ())?;
    let normals = reader.read_positions().ok_or_else(|| ())?;
    let tex_coords = reader
        .read_tex_coords(0)
        .ok_or_else(|| ())?
        .into_f32()
        .collect::<Vec<_>>();

    let vertices = positions
        .zip(normals)
        .zip(tex_coords)
        .map(|((position, normal), tex_coord)| Vertex::new(position, normal, tex_coord))
        .collect::<Vec<_>>();

    let indices = reader
        .read_indices()
        .map(|indices| indices.into_u32().collect::<Vec<_>>());

    // Use the default material if no material is set
    let material_index = primitive.material().index().unwrap_or(0);
    let material = parse_material(&global_gltf, &global_gltf.materials[material_index]);

    Ok(MeshPrimitive::new(vertices, indices, material))
}

fn parse_mesh(global_gltf: &GlobalGltf, mesh: gltf::Mesh) -> Result<Mesh, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("  > Mesh#{:?}: {:?}", mesh.index(), mesh.name());

    let mut primitives = Vec::new();
    for primitive in mesh.primitives() {
        let primitive = parse_mesh_primitive(global_gltf, primitive)?;
        primitives.push(primitive);
    }

    Ok(Mesh { primitives })
}

fn parse_node(global_gltf: &GlobalGltf, node: gltf::Node) -> Result<Node, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("> Node#{:?}: {:?}", node.index(), node.name());

    let mut meshes = Vec::new();
    if let Some(mesh) = node.mesh() {
        let mesh = parse_mesh(global_gltf, mesh)?;
        meshes.push(mesh);
    }

    use gltf::scene::Transform;
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

    let children: Vec<Node> = node
        .children()
        .map(|child| parse_node(global_gltf, child))
        .collect::<Result<_, _>>()?;

    Ok(Node {
        index: node.index(),
        meshes,
        transform,
        children,
    })
}

fn parse_scene(global_gltf: &GlobalGltf, scene: gltf::Scene) -> Result<Scene, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("Scene: {:?}", scene.name());

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
        let node = parse_node(&global_gltf, node)?;
        nodes.append(&mut get_children_nodes(&mut Vec::new(), node));
    }

    Ok(Scene { nodes })
}

pub fn load_scenes<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Scene>, ()> {
    #[cfg(feature = "debug_gltf")]
    log::info!("⏹ Loading gltf file: {:?}", path.as_ref());

    let (gltf, buffers, images) = gltf::import(&path).expect("Failed to load gltf file");

    let materials = gltf.materials().collect::<Vec<_>>();

    let global_gltf = GlobalGltf {
        buffers,
        materials,
        images,
    };

    let mut scenes = Vec::new();
    for scene in gltf.scenes() {
        let scene = parse_scene(&global_gltf, scene)?;
        scenes.push(scene);
    }

    #[cfg(feature = "debug_gltf")]
    log::info!("🆗 Loaded gltf file: {:?}", path.as_ref());

    Ok(scenes)
}
