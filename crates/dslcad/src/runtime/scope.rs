use super::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Scope {
    pub variables: HashMap<String, Value>,
}

impl Scope {
    pub fn new(arguments: HashMap<&str, Value>) -> Self {
        Scope {
            variables: arguments
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
        }
    }

    pub fn get(&self, identifier: &str) -> Option<&Value> {
        self.variables.get(identifier)
    }

    pub fn set(&mut self, identifier: String, value: Value) {
        self.variables.insert(identifier, value);
    }
}
