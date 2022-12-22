use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};

pub fn stl_to_triangle_mesh(stl: &dslcad::Mesh) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let vertex_count = stl.triangles.len() * 3;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(vertex_count);

    for (i, (triangle, normal)) in stl.triangles_with_normals().enumerate() {
        for (j, vertex_index) in triangle.iter().enumerate().take(3) {
            let vertex = stl.vertices[*vertex_index];
            positions.push([vertex[0] as f32, vertex[1] as f32, vertex[2] as f32]);
            normals.push([normal[0] as f32, normal[1] as f32, normal[2] as f32]);
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
