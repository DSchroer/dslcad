use crate::syntax::Value;
use std::fmt::Debug;

pub trait Instance {
    fn get(&self, identifier: &str) -> Option<&Box<Value>>;
    fn value(&self) -> &Box<Value>;
    fn write_to_file(&mut self, path: &str) -> Result<(), std::io::Error>;
}
