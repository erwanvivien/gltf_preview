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

    let positions = reader
        .read_positions()
        .ok_or_else(|| ())?
        .collect::<Vec<_>>();
    let normals = reader
        .read_positions()
        .ok_or_else(|| ())?
        .collect::<Vec<_>>();
    let tex_coords = reader
        .read_tex_coords(0)
        .map(|tex| tex.into_f32().collect::<Vec<_>>());
    let colors = reader
        .read_colors(0)
        .map(|color| color.into_rgb_f32().collect::<Vec<_>>());

    let mut vertices = Vec::with_capacity(positions.len());
    for i in 0..positions.len() {
        let vertex = Vertex {
            position: positions[i],
            normal: normals[i],
            tex_coord: tex_coords.as_ref().map(|tex| tex[i]),
            color: colors.as_ref().map(|color| color[i]),
        };

        vertices.push(vertex);
    }

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
        primitives.push(MeshPrimitive {
            #[cfg(feature = "debug_gltf")]
            name: mesh.name().map(|s| s.to_string()),
            ..primitive
        });
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

        #[cfg(feature = "debug_gltf")]
        name: node.name().map(String::from),
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

    Ok(Scene {
        nodes,

        #[cfg(feature = "debug_gltf")]
        name: scene.name().map(String::from),
    })
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

        #[cfg(feature = "debug_gltf")]
        dbg!(&scene);
        scenes.push(scene);
    }

    #[cfg(feature = "debug_gltf")]
    log::info!("🆗 Loaded gltf file: {:?}", path.as_ref());

    Ok(scenes)
}
