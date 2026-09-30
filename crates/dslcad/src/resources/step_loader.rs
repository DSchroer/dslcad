use crate::parser::{DocumentParseError, Literal, Reader};
use crate::resources::{Resource, ResourceLoader};
use crate::runtime::{RuntimeError, Value};
use dslcad_occt::Shape;
use std::collections::HashMap;
use std::rc::Rc;

pub struct StepLoader;

impl ResourceLoader for StepLoader {
    fn load(
        &self,
        path: &str,
        _reader: &dyn Reader,
        _arguments: &HashMap<String, Literal>,
    ) -> Result<Box<dyn Resource>, DocumentParseError> {
        Ok(Box::new(StepShape {
            path: path.to_string(),
        }))
    }
}

/// A STEP file read when the resource is used. Keeping the path instead of the
/// shape lets the resource stay `Send + Sync` and delays the OCCT reader until
/// evaluation.
#[derive(Debug)]
struct StepShape {
    path: String,
}

impl Resource for StepShape {
    fn to_instance(&self) -> Result<Value, RuntimeError> {
        Ok(Value::Shape(Rc::new(Shape::read_step(&self.path)?)))
    }
}
