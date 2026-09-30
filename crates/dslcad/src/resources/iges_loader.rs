use crate::parser::{DocumentParseError, Literal, Reader};
use crate::resources::{Resource, ResourceLoader};
use crate::runtime::{RuntimeError, Value};
use dslcad_occt::Shape;
use std::collections::HashMap;
use std::rc::Rc;

pub struct IgesLoader;

impl ResourceLoader for IgesLoader {
    fn load(
        &self,
        path: &str,
        _reader: &dyn Reader,
        _arguments: &HashMap<String, Literal>,
    ) -> Result<Box<dyn Resource>, DocumentParseError> {
        Ok(Box::new(IgesShape {
            path: path.to_string(),
        }))
    }
}

/// An IGES file read when the resource is used. Keeping the path instead of the
/// shape lets the resource stay `Send + Sync` and delays the OCCT reader until
/// evaluation.
#[derive(Debug)]
struct IgesShape {
    path: String,
}

impl Resource for IgesShape {
    fn to_instance(&self) -> Result<Value, RuntimeError> {
        Ok(Value::Shape(Rc::new(Shape::read_iges(&self.path)?)))
    }
}
