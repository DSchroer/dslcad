use crate::syntax::Value;

pub trait Instance {
    fn get(&self, identifier: &str) -> Option<&Box<Value>>;
    fn value(&self) -> &Value;
    fn write_to_file(&mut self, path: &str) -> Result<(), std::io::Error>;
}
