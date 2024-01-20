use super::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub arguments: HashMap<String, Value>,
    pub variables: HashMap<String, Value>,
}

impl Scope {
    pub fn new(arguments: HashMap<&str, Value>) -> Self {
        Scope {
            arguments: arguments
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
            variables: HashMap::new(),
        }
    }

    pub fn set_arguments(&mut self, arguments: HashMap<&str, Value>) {
        self.arguments = arguments
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect();
    }

    pub fn get(&self, identifier: &str) -> Option<&Value> {
        self.arguments
            .get(identifier)
            .or_else(|| self.variables.get(identifier))
    }

    pub fn set(&mut self, identifier: String, value: Value) {
        self.variables.insert(identifier, value);
    }
}
