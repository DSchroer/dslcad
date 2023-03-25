use dslcad_api::protocol::Mesh;
use std::io::Write;
use stl_io::{Normal, Triangle, Vector};

pub fn export_stl(mesh: &Mesh, writer: &mut impl Write) -> Result<(), std::io::Error> {
    let mut triangles = Vec::new();
    for (i, triangle) in mesh.triangles.iter().enumerate() {
        triangles.push(Triangle {
            vertices: [
                Vector::new([
                    mesh.vertices[triangle[0]][0] as f32,
                    mesh.vertices[triangle[0]][1] as f32,
                    mesh.vertices[triangle[0]][2] as f32,
                ]),
                Vector::new([
                    mesh.vertices[triangle[1]][0] as f32,
                    mesh.vertices[triangle[1]][1] as f32,
                    mesh.vertices[triangle[1]][2] as f32,
                ]),
                Vector::new([
                    mesh.vertices[triangle[2]][0] as f32,
                    mesh.vertices[triangle[2]][1] as f32,
                    mesh.vertices[triangle[2]][2] as f32,
                ]),
            ],
            normal: Normal::new([
                mesh.normals[i][0] as f32,
                mesh.normals[i][1] as f32,
                mesh.normals[i][2] as f32,
            ]),
        })
    }

    stl_io::write_stl(writer, triangles.iter())
}
