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
    parts: Vec<Value>,
}

impl ScriptInstance {
    pub fn from_scope(parts: Vec<Value>, scope: Scope) -> Result<Self, RuntimeError> {
        if parts.is_empty() {
            return Err(RuntimeError::NoReturnValue());
        }

        Ok(ScriptInstance {
            arguments: scope.arguments,
            variables: scope.variables,
            parts,
        })
    }

    pub fn parts(&self) -> Value {
        Value::List(self.parts.clone())
    }
}

impl Display for ScriptInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let parts = self.parts().to_output().map_err(|_| std::fmt::Error)?;
        for part in parts {
            match part {
                Part::Data { text } => f.write_str(&text)?,
                Part::Planar { .. } => panic!("can not display 2d as text"),
                Part::Object { .. } => panic!("can not display 3d as text"),
            }
        }
        Ok(())
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
