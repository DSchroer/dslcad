use super::{Access, Value};
use crate::runtime::scope::Scope;
use crate::runtime::RuntimeError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScriptInstance {
    variables: HashMap<String, Value>,
    parts: Value,
}

impl ScriptInstance {
    pub fn from_scope(parts: Vec<Value>, scope: Scope) -> Result<Self, RuntimeError> {
        Self::new(parts, scope.variables)
    }

    pub fn new(
        mut parts: Vec<Value>,
        variables: HashMap<String, Value>,
    ) -> Result<Self, RuntimeError> {
        let reduced_parts = match parts.len() {
            0 => return Err(RuntimeError::NoReturnValue()),
            1 => parts.pop().unwrap(),
            _ => Value::List(parts),
        };

        Ok(ScriptInstance {
            variables,
            parts: reduced_parts,
        })
    }

    pub fn value(&self) -> &Value {
        &self.parts
    }
}

impl Access for ScriptInstance {
    fn get(&self, identifier: &str) -> Option<Value> {
        self.variables.get(identifier).cloned()
    }
}
