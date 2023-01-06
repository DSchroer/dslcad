use crate::runtime::{RuntimeError, Value};

pub fn round(number: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Number(number as i64 as f64))
}

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

pub fn modulo(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Number(left % right))
}

pub fn power(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Number(left.powf(right)))
}

pub fn pi() -> Result<Value, RuntimeError> {
    Ok(Value::Number(std::f64::consts::PI))
}

pub fn less(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(left < right))
}

pub fn less_or_equal(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(left <= right))
}

pub fn equals(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(left == right))
}

pub fn not_equals(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(left != right))
}

pub fn greater(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(left > right))
}

pub fn greater_or_equal(left: f64, right: f64) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(left >= right))
}
