use super::{Access, Value};
use crate::runtime::scope::Scope;
use crate::runtime::RuntimeError;
use persistence::protocol::Part;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ScriptInstance {
    arguments: HashMap<String, Value>,
    variables: HashMap<String, Value>,
    value: Value,
}

impl ScriptInstance {
    pub fn from_scope(values: Vec<Value>, scope: Scope) -> Result<Self, RuntimeError> {
        if values.is_empty() {
            return Err(RuntimeError::NoReturnValue());
        }

        let value = if values.len() == 1 {
            values[0].clone()
        } else {
            Value::List(values)
        };

        Ok(ScriptInstance {
            arguments: scope.arguments,
            variables: scope.variables,
            value,
        })
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl Display for ScriptInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let part = self.value().to_output().map_err(|_| std::fmt::Error)?;
        match part {
            Part::Data { text } => f.write_str(&text),
            Part::Planar { .. } => panic!("can not display 2d as text"),
            Part::Object { .. } => panic!("can not display 3d as text"),
        }
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
