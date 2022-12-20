use crate::runtime::scope::Scope;
use super::{Accessible, Value};
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

impl Display for ScriptInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value
            .to_output()
            .map_err(|_| std::fmt::Error::default())?
            .fmt(f)
    }
}

impl Accessible for ScriptInstance {
    fn get(&self, identifier: &str) -> Option<Value> {
        let val = self
            .arguments
            .get(identifier)
            .or_else(|| self.variables.get(identifier))?;
        Some(*val.clone())
    }
}
