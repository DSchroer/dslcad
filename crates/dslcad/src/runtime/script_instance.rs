use super::{Access, Value};
use crate::runtime::scope::Scope;
use crate::runtime::{RuntimeError, Type};
use opencascade::{DsShape, Shape, Wire, WireFactory};
use persistence::protocol::Part;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ScriptInstance {
    arguments: HashMap<String, Value>,
    variables: HashMap<String, Value>,
    value_type: Type,
    value: Value,
    values: Vec<Value>,
}

impl ScriptInstance {
    pub fn from_scope(values: Vec<Value>, scope: Scope) -> Result<Self, RuntimeError> {
        if values.is_empty() {
            return Err(RuntimeError::NoReturnValue());
        }

        let value_type = values[0].get_type();
        if !values.iter().all(|v| v.get_type() == value_type) {
            return Err(RuntimeError::MismatchedPartTypes);
        }

        let value = Self::union_value(&values, value_type)?;

        Ok(ScriptInstance {
            arguments: scope.arguments,
            variables: scope.variables,
            value_type,
            value,
            values,
        })
    }

    pub fn value_type(&self) -> Type {
        self.value_type
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }

    fn union_value(values: &Vec<Value>, value_type: Type) -> Result<Value, RuntimeError> {
        if values.len() == 1 {
            return Ok(values[0].clone());
        }

        match value_type {
            Type::Number | Type::Bool | Type::Text | Type::List | Type::Point => {
                Err(RuntimeError::InvalidMultiPartType)
            }
            Type::Edge => {
                let lines: Vec<&Rc<Wire>> = values.iter().map(|v| v.to_line().unwrap()).collect();
                let mut combined = WireFactory::new();
                for line in lines {
                    combined.add_wire(line)
                }
                Ok(Into::<Value>::into(combined.build()?))
            }
            Type::Shape => {
                let shapes: Vec<&Rc<Shape>> =
                    values.iter().map(|v| v.to_shape().unwrap()).collect();
                let mut combined = shapes[0].clone().fuse(shapes[1])?;
                for shape in &shapes[2..] {
                    combined = combined.fuse(shape)?
                }
                Ok(Into::<Value>::into(combined))
            }
        }
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
