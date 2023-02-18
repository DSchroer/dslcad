use crate::runtime::{RuntimeError, Value};
use std::f64::consts::PI;

pub fn round(number: f64) -> Result<f64, RuntimeError> {
    Ok(number.round())
}

pub fn ceil(number: f64) -> Result<f64, RuntimeError> {
    Ok(number.ceil())
}

pub fn floor(number: f64) -> Result<f64, RuntimeError> {
    Ok(number.floor())
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

pub fn rad_to_deg(radians: f64) -> Result<f64, RuntimeError> {
    Ok(radians * (180. / PI))
}

pub fn deg_to_rad(degrees: f64) -> Result<f64, RuntimeError> {
    Ok(degrees / (180. / PI))
}

pub fn sin_deg(degrees: f64) -> Result<f64, RuntimeError> {
    rad_to_deg(f64::sin(deg_to_rad(degrees)?))
}

pub fn cos_deg(degrees: f64) -> Result<f64, RuntimeError> {
    rad_to_deg(f64::cos(deg_to_rad(degrees)?))
}

pub fn tan_deg(degrees: f64) -> Result<f64, RuntimeError> {
    rad_to_deg(f64::tan(deg_to_rad(degrees)?))
}

pub fn sin_rad(radians: f64) -> Result<f64, RuntimeError> {
    Ok(f64::sin(radians))
}

pub fn cos_rad(radians: f64) -> Result<f64, RuntimeError> {
    Ok(f64::cos(radians))
}

pub fn tan_rad(radians: f64) -> Result<f64, RuntimeError> {
    Ok(f64::tan(radians))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn it_supports_trig() {
        assert_eq!(90., rad_to_deg(PI / 2.).unwrap());
        assert_eq!(PI / 2., deg_to_rad(90.).unwrap());
    }
}
