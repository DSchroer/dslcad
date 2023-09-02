use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};

pub fn stl_to_triangle_mesh(stl: &dslcad_api::protocol::Mesh) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let vertex_count = stl.triangles.len() * 3;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(vertex_count);

    for i in 0..stl.triangles.len() {
        for (j, vertex) in stl.triangles[i].iter().enumerate() {
            positions.push(stl.vertices[*vertex].map(|v| v as f32));
            normals.push(stl.normals[i].map(|v| v as f32));
            indices.push((i * 3 + j) as u32);
        }
    }

    let uvs = vec![[0.0, 0.0]; vertex_count];

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}
