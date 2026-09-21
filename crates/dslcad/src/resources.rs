mod ini_loader;
mod stl_loader;
mod svg_loader;
mod ttf_loader;

use crate::parser::{DocumentParseError, Literal, Parser, Reader};
use crate::runtime::{RuntimeError, Value};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::resources::ini_loader::IniLoader;
pub use stl_loader::StlLoader;
pub use svg_loader::SvgLoader;
pub use ttf_loader::TtfLoader;

pub trait ResourceLoader<TReader: Reader> {
    fn load(
        &self,
        path: &str,
        reader: &TReader,
        arguments: &HashMap<String, Literal>,
    ) -> Result<Box<dyn Resource>, DocumentParseError>;
}

pub trait Resource: Debug + Send + Sync {
    fn to_instance(&self) -> Result<Value, RuntimeError>;
}

pub trait ResourceExt {
    fn with_default_loaders(self) -> Self;
}

impl<R: Reader> ResourceExt for Parser<R> {
    fn with_default_loaders(self) -> Self {
        self.with_loader("stl", StlLoader)
            .with_loader("svg", SvgLoader)
            .with_loader("ttf", TtfLoader)
            .with_loader("otf", TtfLoader)
            .with_loader("ini", IniLoader)
    }
}
