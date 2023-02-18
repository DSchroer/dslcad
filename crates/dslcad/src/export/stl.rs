use crate::Output;
use std::io::Write;
use stl_io::{Normal, Triangle, Vector};

pub fn export_stl(out: &Output, writer: &mut impl Write) -> Result<(), std::io::Error> {
    let mut triangles = Vec::new();
    let mesh = out.mesh();
    for (face, normal) in mesh.triangles_with_normals() {
        triangles.push(Triangle {
            vertices: [
                Vector::new(mesh.vertex_f32(face[0])),
                Vector::new(mesh.vertex_f32(face[1])),
                Vector::new(mesh.vertex_f32(face[2])),
            ],
            normal: Normal::new(normal.map(|n| n as f32)),
        })
    }

    stl_io::write_stl(writer, triangles.iter())
}
