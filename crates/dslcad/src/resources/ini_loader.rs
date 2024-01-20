use crate::parser::{DocumentParseError, Reader};
use crate::resources::{Resource, ResourceLoader};
use crate::runtime::{RuntimeError, ScriptInstance, Value};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

pub struct IniLoader;

impl<R: Reader> ResourceLoader<R> for IniLoader {
    fn load(&self, path: &str, reader: &R) -> Result<Box<dyn Resource>, DocumentParseError> {
        let data = reader.read(Path::new(path)).unwrap();

        let mut values = HashMap::new();
        for line in data.lines() {
            let mut parts = line.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                values.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(Box::new(values))
    }
}

impl Resource for HashMap<String, String> {
    fn to_instance(&self) -> Result<Value, RuntimeError> {
        let variables = self
            .clone()
            .drain()
            .map(|(k, v)| (k, Value::Text(v)))
            .collect();

        Ok(Value::Script(Rc::new(ScriptInstance::new(
            vec![Value::List(Vec::new())],
            variables,
        )?)))
    }
}
