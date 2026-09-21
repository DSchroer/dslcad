mod ini_loader;
mod stl_loader;
mod svg_loader;
mod ttf_loader;

use crate::parser::{Argument, DocumentParseError, Literal, Parser, Reader};
use crate::runtime::{RuntimeError, Value};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::resources::ini_loader::IniLoader;
pub use stl_loader::StlLoader;
pub use svg_loader::SvgLoader;
pub use ttf_loader::TtfLoader;

pub trait ResourceLoader {
    fn load(
        &self,
        path: &str,
        reader: &dyn Reader,
        arguments: &HashMap<String, Literal>,
    ) -> Result<Box<dyn Resource>, DocumentParseError>;
}

pub trait Resource: Debug + Send + Sync {
    fn to_instance(&self) -> Result<Value, RuntimeError>;
}

/// A deferred resource load. Resources used to be loaded while parsing, which
/// meant their arguments had to be literal values. Keeping the path, argument
/// expressions, reader and loader around lets the runtime evaluate the
/// arguments first and load the resource on demand.
pub struct ResourceFactory {
    path: String,
    arguments: VecDeque<Argument>,
    loader: Rc<dyn ResourceLoader>,
    reader: Rc<dyn Reader>,
}

impl ResourceFactory {
    pub fn new(
        path: String,
        arguments: VecDeque<Argument>,
        loader: Rc<dyn ResourceLoader>,
        reader: Rc<dyn Reader>,
    ) -> Self {
        Self {
            path,
            arguments,
            loader,
            reader,
        }
    }

    /// The argument expressions to evaluate before loading the resource.
    pub fn arguments(&self) -> &VecDeque<Argument> {
        &self.arguments
    }

    pub fn load(
        &self,
        arguments: &HashMap<String, Literal>,
    ) -> Result<Box<dyn Resource>, DocumentParseError> {
        self.loader
            .load(&self.path, self.reader.as_ref(), arguments)
    }
}

impl Debug for ResourceFactory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceFactory")
            .field("path", &self.path)
            .finish()
    }
}

pub trait ResourceExt {
    fn with_default_loaders(self) -> Self;
}

impl<R: Reader + 'static> ResourceExt for Parser<R> {
    fn with_default_loaders(self) -> Self {
        self.with_loader("stl", StlLoader)
            .with_loader("svg", SvgLoader)
            .with_loader("ttf", TtfLoader)
            .with_loader("otf", TtfLoader)
            .with_loader("ini", IniLoader)
    }
}
