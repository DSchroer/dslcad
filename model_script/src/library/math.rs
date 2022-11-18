use crate::runtime::RuntimeError;
use crate::syntax::Value;

pub fn add(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Number(left + right))
}

pub fn subtract(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Number(left - right))
}

pub fn multiply(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Number(left * right))
}

pub fn divide(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Number(left / right))
}
