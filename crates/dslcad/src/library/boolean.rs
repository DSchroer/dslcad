use crate::runtime::{RuntimeError, Value};

pub fn and(left: bool, right: bool) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(left && right))
}

pub fn or(left: bool, right: bool) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(left || right))
}

pub fn not(value: bool) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(!value))
}
