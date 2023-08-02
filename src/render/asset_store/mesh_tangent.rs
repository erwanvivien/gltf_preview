use crate::render::asset_store::mesh::PrimitiveVertex;

const VERTEX_PER_FACE: usize = 3;

struct MeshTangentUtil<'a> {
    faces: Vec<[u32; 3]>,
    vertices: &'a mut Vec<PrimitiveVertex>,
}

impl<'a> MeshTangentUtil<'a> {
    fn get_vertex(&self, face: usize, vertex: usize) -> &PrimitiveVertex {
        let face = self.faces[face];
        let vertex = face[vertex] as usize;
        &self.vertices[vertex]
    }

    fn get_vertex_mut(&mut self, face: usize, vertex: usize) -> &mut PrimitiveVertex {
        let face = self.faces[face];
        let vertex = face[vertex] as usize;
        &mut self.vertices[vertex]
    }
}

impl<'a> mikktspace::Geometry for MeshTangentUtil<'a> {
    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.get_vertex(face, vert).normal.into()
    }

    fn num_faces(&self) -> usize {
        self.faces.len()
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        VERTEX_PER_FACE
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.get_vertex(face, vert).position.into()
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.get_vertex(face, vert).tex_coord_0.into()
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let vertex = self.get_vertex_mut(face, vert);
        vertex.tangent = tangent.into();
    }
}

pub(super) fn generate_tangents(indices: Option<&Vec<u32>>, vertices: &mut Vec<PrimitiveVertex>) {
    let vertex_count = vertices.len();

    if vertex_count == 0 {
        return;
    }

    if indices.is_none() && vertex_count % VERTEX_PER_FACE != 0 {
        log::warn!("Invalid vert count for tangents gen: {}", vertex_count);
        return;
    }

    let indices_count = indices.map(Vec::len).unwrap_or(0);
    if indices_count != 0 && indices_count % VERTEX_PER_FACE != 0 {
        log::warn!("Invalid indices count for tangents gen: {}", indices_count);
        return;
    }

    let faces: Vec<[u32; 3]> = if let Some(indices) = indices {
        (0..indices.len())
            .step_by(VERTEX_PER_FACE)
            .map(|i| [indices[i], indices[i + 1], indices[i + 2]])
            .collect()
    } else {
        (0..vertex_count)
            .step_by(VERTEX_PER_FACE)
            .map(|i| [i as u32, i as u32 + 1, i as u32 + 2])
            .collect()
    };

    let mut mesh = MeshTangentUtil { faces, vertices };
    mikktspace::generate_tangents(&mut mesh);
}
