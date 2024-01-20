use crate::parser::{DocumentParseError, Reader};
use crate::resources::{Resource, ResourceLoader};
use crate::runtime::{RuntimeError, Value};
use dslcad_occt::{Point, TriangleMesh};
use std::io::Cursor;
use std::path::Path;
use std::rc::Rc;
use stl_io::IndexedMesh;

pub struct StlLoader;

impl<R: Reader> ResourceLoader<R> for StlLoader {
    fn load(&self, path: &str, reader: &R) -> Result<Box<dyn Resource>, DocumentParseError> {
        let data = reader.read_bytes(Path::new(path)).unwrap();
        let mut cursor = Cursor::new(&data);
        let mesh =
            stl_io::read_stl(&mut cursor).map_err(|_| DocumentParseError::UnexpectedEndOfFile())?;
        Ok(Box::new(mesh))
    }
}

impl Resource for IndexedMesh {
    fn to_instance(&self) -> Result<Value, RuntimeError> {
        let mesh = TriangleMesh::new(
            self.vertices
                .iter()
                .map(|v| Point::new(v[0] as f64, v[1] as f64, v[2] as f64)),
            self.faces.iter().map(|f| f.vertices),
        );

        Ok(Value::Shape(Rc::new(mesh.try_into()?)))
    }
}
