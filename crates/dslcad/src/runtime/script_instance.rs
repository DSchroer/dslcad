use super::{Access, Value};
use crate::runtime::scope::Scope;
use crate::runtime::RuntimeError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScriptInstance {
    arguments: HashMap<String, Value>,
    variables: HashMap<String, Value>,
    parts: Value,
}

impl ScriptInstance {
    pub fn from_scope(mut parts: Vec<Value>, scope: Scope) -> Result<Self, RuntimeError> {
        let reduced_parts = match parts.len() {
            0 => return Err(RuntimeError::NoReturnValue()),
            1 => parts.pop().unwrap(),
            _ => Value::List(parts),
        };

        Ok(ScriptInstance {
            arguments: scope.arguments,
            variables: scope.variables,
            parts: reduced_parts,
        })
    }

    pub fn value(&self) -> &Value {
        &self.parts
    }
}

impl Access for ScriptInstance {
    fn get(&self, identifier: &str) -> Option<Value> {
        let val = self
            .arguments
            .get(identifier)
            .or_else(|| self.variables.get(identifier))?;
        Some(val.clone())
    }
}
