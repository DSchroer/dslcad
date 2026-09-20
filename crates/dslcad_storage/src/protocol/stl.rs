use crate::protocol::{Part, Render};
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StlError {
    #[error("render did not include any parts to output")]
    MissingPart(),
    #[error("render contained a mesh with out-of-range vertex indices")]
    InvalidMesh(),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Render {
    pub fn to_stl(&self, writer: &mut impl Write) -> Result<(), StlError> {
        let part = self.parts.first().ok_or(StlError::MissingPart())?;
        let mesh = match part {
            Part::Empty => return Err(StlError::MissingPart()),
            Part::Planar { .. } => return Err(StlError::MissingPart()),
            Part::Object { mesh, .. } => mesh,
        };

        let mut triangles = Vec::with_capacity(mesh.triangles.len());
        for (i, t) in mesh.triangles.iter().enumerate() {
            let vertex = |index: usize| {
                mesh.vertices
                    .get(index)
                    .map(|v| v.map(|f| f as f32))
                    .ok_or(StlError::InvalidMesh())
            };
            let vertices = [vertex(t[0])?, vertex(t[1])?, vertex(t[2])?];
            let normal = mesh
                .normals
                .get(i)
                .map(|n| n.map(|f| f as f32))
                .unwrap_or_else(|| triangle_normal(&vertices));

            triangles.push(stl_io::Triangle {
                normal: stl_io::Vertex::new(normal),
                vertices: vertices.map(stl_io::Vertex::new),
            });
        }
        stl_io::write_stl(writer, triangles.into_iter())?;

        Ok(())
    }
}

fn triangle_normal(vertices: &[[f32; 3]; 3]) -> [f32; 3] {
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];

    let normal = cross(sub(vertices[1], vertices[0]), sub(vertices[2], vertices[0]));
    let length = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
    if length == 0.0 {
        [0.0, 0.0, 1.0]
    } else {
        [normal[0] / length, normal[1] / length, normal[2] / length]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Mesh, Point};

    fn render(mesh: Mesh) -> Render {
        Render {
            parts: vec![Part::Object {
                points: vec![],
                lines: vec![],
                mesh,
            }],
            stdout: String::new(),
        }
    }

    #[test]
    fn it_writes_a_mesh_with_vertex_indices_larger_than_the_triangle_count() {
        let mut mesh = Mesh {
            vertices: (0..4).map(|i| Point::from((i as f64, 0.0, 0.0))).collect(),
            triangles: vec![],
            normals: vec![],
        };

        // A cube-like mesh: 12 triangles referencing vertices by index while
        // only storing one normal per triangle.
        for i in 0..12 {
            mesh.triangles.push([i % 4, (i + 1) % 4, (i + 2) % 4]);
            mesh.normals.push([0.0, 0.0, 1.0]);
        }

        let mut out = Vec::new();
        render(mesh).to_stl(&mut out).expect("failed to write stl");
        assert!(!out.is_empty());
    }

    #[test]
    fn it_errors_on_out_of_range_vertex_indices() {
        let mesh = Mesh {
            vertices: vec![Point::from((0.0, 0.0, 0.0))],
            triangles: vec![[0, 1, 2]],
            normals: vec![[0.0, 1.0, 0.0]],
        };

        let mut out = Vec::new();
        assert!(matches!(
            render(mesh).to_stl(&mut out),
            Err(StlError::InvalidMesh())
        ));
    }
}
