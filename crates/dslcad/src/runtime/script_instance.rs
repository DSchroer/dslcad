use super::{Access, Value};
use crate::runtime::scope::Scope;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ScriptInstance {
    arguments: HashMap<String, Box<Value>>,
    variables: HashMap<String, Box<Value>>,
    value: Box<Value>,
}

impl ScriptInstance {
    pub fn from_scope(value: Value, scope: Scope) -> Self {
        ScriptInstance {
            arguments: scope.arguments,
            variables: scope.variables,
            value: Box::new(value),
        }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl Access for ScriptInstance {
    fn get(&self, identifier: &str) -> Option<Value> {
        let val = self
            .arguments
            .get(identifier)
            .or_else(|| self.variables.get(identifier))?;
        Some(*val.clone())
    }
}
