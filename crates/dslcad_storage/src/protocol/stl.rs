use std::io::Write;
use thiserror::Error;
use crate::protocol::{Part, Render};

#[derive(Debug, Error)]
pub enum StlError {
    #[error("render did not include any parts to output")]
    MissingPart(),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Render {
    pub fn to_stl(&self, writer: &mut impl Write) -> Result<(), StlError> {
        let part  = self.parts.first().ok_or(StlError::MissingPart())?;
        let mesh = match part {
            Part::Empty => return Err(StlError::MissingPart()),
            Part::Planar { .. } => return Err(StlError::MissingPart()),
            Part::Object { mesh, .. } => mesh
        };

        let triangles = mesh.triangles.iter().map(|t| stl_io::Triangle{
            normal: stl_io::Vertex::new(mesh.normals[t[0]].map(|f| f as f32)),
            vertices: [
                stl_io::Vertex::new(mesh.vertices[t[0]].map(|f| f as f32)),
                stl_io::Vertex::new(mesh.vertices[t[1]].map(|f| f as f32)),
                stl_io::Vertex::new(mesh.vertices[t[2]].map(|f| f as f32))
            ]
        });
        stl_io::write_stl(writer, triangles)?;

        Ok(())
    }
}