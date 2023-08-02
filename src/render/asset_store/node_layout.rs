use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MeshIndex(pub u32);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeIndex(pub u32);

#[derive(Clone)]
pub(super) struct NodeData {
    #[cfg(feature = "debug_gltf")]
    name: Option<String>,
    pub(super) index: NodeIndex,
    transform_local: glam::Mat4,
    transform_global: glam::Mat4,
    parent: Option<NodeIndex>,
    children_index: Vec<NodeIndex>,
}

impl std::fmt::Debug for NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        let title = if self.parent.is_some() { "NodeData" } else { "NodeData(root)" };

        let mut debug_struct = f.debug_struct(title);

        debug_struct.field("index", &self.index.0);
        #[cfg(feature = "debug_gltf")]
        debug_struct.field("name", &self.name.as_ref().unwrap_or(&String::from("None")));
        if let Some(parent) = self.parent {
            debug_struct.field("parent", &parent.0);
        }

        let matrix = gltf::scene::Transform::Matrix {
            matrix: self.transform_global.to_cols_array_2d(),
        };
        let (translation, rotation, scale) = matrix.decomposed();

        debug_struct.field("transform.translation", &format!("{:.2?}", translation));
        debug_struct.field("transform.rotation", &format!("{:.2?}", rotation));
        debug_struct.field("transform.scale", &format!("{:.2?}", scale));

        debug_struct.finish()
    }
}

pub struct NodeLayout {
    pub(super) mesh_nodes: HashMap<MeshIndex, Vec<NodeIndex>>,
    pub(super) node_mesh: HashMap<NodeIndex, MeshIndex>,
    pub(super) nodes: Vec<NodeData>,
}

impl NodeLayout {
    pub fn from_gltf(gltf_nodes: gltf::iter::Nodes) -> Self {
        let mut mesh_nodes = HashMap::<_, Vec<_>>::new();
        let mut node_mesh = HashMap::new();
        let mut nodes = Vec::new();
        let mut parent = HashMap::new();

        for node in gltf_nodes {
            let node_index = u32::try_from(node.index()).expect("Node index overflow");
            let node_index = NodeIndex(node_index);
            let transform_matrix = node.transform().matrix();

            let transform_local = glam::Mat4::from_cols_array_2d(&transform_matrix);
            let transform_global = transform_local;

            if let Some(mesh) = node.mesh() {
                let mesh_index = u32::try_from(mesh.index()).expect("Mesh index overflow");
                mesh_nodes
                    .entry(MeshIndex(mesh_index))
                    .or_default()
                    .push(node_index);
                node_mesh.insert(node_index, MeshIndex(mesh_index));
            }

            let mut children_index = Vec::new();
            for child in node.children() {
                let child_index = u32::try_from(child.index()).expect("Child index overflow");
                let child_index = NodeIndex(child_index);
                children_index.push(child_index);
                parent.insert(child_index, node_index);
            }

            nodes.push(NodeData {
                #[cfg(feature = "debug_gltf")]
                name: node.name().map(ToOwned::to_owned),
                index: node_index,
                transform_local,
                transform_global,
                parent: None,
                children_index,
            });
        }

        for node in &mut nodes {
            node.parent = parent.get(&node.index).copied();
        }

        Self {
            mesh_nodes,
            node_mesh,
            nodes,
        }
    }

    pub fn get_node_transform(&self, node_index: NodeIndex) -> glam::Mat4 {
        let mut index = usize::try_from(node_index.0).expect("Node index overflow");

        let mut transform = self.nodes[index].transform_global;

        while let Some(parent_index) = self.nodes[index].parent {
            let parent_index = usize::try_from(parent_index.0).expect("Node index overflow");
            let parent = &self.nodes[parent_index];
            transform = parent.transform_global * transform;
            index = usize::try_from(parent_index).expect("Parent node index overflow");
        }

        transform
    }
}
