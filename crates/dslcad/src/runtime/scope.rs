use super::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Scope {
    pub arguments: HashMap<String, Box<Value>>,
    pub variables: HashMap<String, Box<Value>>,
}

impl Scope {
    pub fn new(arguments: HashMap<&str, Value>) -> Self {
        Scope {
            arguments: arguments
                .into_iter()
                .map(|(k, v)| (k.to_owned(), Box::new(v)))
                .collect(),
            variables: HashMap::new(),
        }
    }

    pub fn get(&self, identifier: &str) -> Option<&Value> {
        let val = self
            .arguments
            .get(identifier)
            .or_else(|| self.variables.get(identifier))?;
        Some(val)
    }

    pub fn set(&mut self, identifier: String, value: Value) {
        self.variables.insert(identifier, Box::new(value));
    }
}
