use crate::runtime::{RuntimeError, Value};

pub fn length(list: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(list.len() as f64))
}

pub fn range(from: Option<f64>, to: f64) -> Result<Value, RuntimeError> {
    let from = from.unwrap_or(0.) as i64;
    let to = to as i64;
    Ok(Value::List(
        (from..to).map(|v| Value::Number(v as f64)).collect(),
    ))
}
