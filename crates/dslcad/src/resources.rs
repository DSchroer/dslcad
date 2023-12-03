mod ini_loader;
mod stl_loader;

use crate::parser::{DocumentParseError, Parser, Reader};
use crate::runtime::{RuntimeError, Value};
use std::fmt::Debug;
use std::sync::Arc;

use crate::resources::ini_loader::IniLoader;
pub use stl_loader::StlLoader;

pub trait ResourceLoader<TReader: Reader> {
    fn load(&self, path: &str, reader: &TReader) -> Result<Arc<dyn Resource>, DocumentParseError>;
}

pub trait Resource: Debug {
    fn to_instance(&self) -> Result<Value, RuntimeError>;
}

pub trait ResourceExt {
    fn with_default_loaders(self) -> Self;
}

impl<R: Reader> ResourceExt for Parser<R> {
    fn with_default_loaders(self) -> Self {
        self.with_loader("stl", StlLoader)
            .with_loader("ini", IniLoader)
    }
}
