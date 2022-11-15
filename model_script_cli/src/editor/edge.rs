use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};

pub fn line_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineList);

    let positions = vec![[0.,0.,0.], [10.,10.,10.]];
    let normals = vec![[0.,0.,0.], [0.,0.,0.]];
    let indices = vec![0, 1];

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}
