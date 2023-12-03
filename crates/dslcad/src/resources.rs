mod stl_loader;

use crate::parser::{DocumentParseError, Reader};
use crate::runtime::{RuntimeError, Value};
use std::fmt::Debug;
use std::sync::Arc;

pub use stl_loader::StlLoader;

pub trait ResourceLoader<TReader: Reader> {
    fn load(&self, path: &str, reader: &TReader) -> Result<Arc<dyn Resource>, DocumentParseError>;
}

pub trait Resource: Debug {
    fn to_instance(&self) -> Result<Value, RuntimeError>;
}
