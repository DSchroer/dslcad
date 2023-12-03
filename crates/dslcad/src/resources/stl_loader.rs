use crate::parser::{DocumentParseError, Reader};
use crate::resources::{Resource, ResourceLoader};
use crate::runtime::{RuntimeError, Value};
use opencascade::{Point, TriangleMeshBuilder};
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use stl_io::{IndexedMesh, Vector};

pub struct StlLoader;

impl<R: Reader> ResourceLoader<R> for StlLoader {
    fn load(&self, path: &str, reader: &R) -> Result<Arc<dyn Resource>, DocumentParseError> {
        let data = reader.read_bytes(Path::new(path)).unwrap();
        let mut cursor = Cursor::new(&data);
        let mesh =
            stl_io::read_stl(&mut cursor).map_err(|_| DocumentParseError::UnexpectedEndOfFile())?;
        Ok(Arc::new(mesh))
    }
}

impl Resource for IndexedMesh {
    fn to_instance(&self) -> Result<Value, RuntimeError> {
        let mut builder = TriangleMeshBuilder::new();

        for triangle in &self.faces {
            builder.add_triangle([
                vec_to_point(self.vertices[triangle.vertices[0]]),
                vec_to_point(self.vertices[triangle.vertices[1]]),
                vec_to_point(self.vertices[triangle.vertices[2]]),
            ])?;
        }

        Ok(builder.build()?.into())
    }
}

fn vec_to_point(vec: Vector<f32>) -> Point {
    Point::new(vec[0] as f64, vec[1] as f64, vec[2] as f64)
}
