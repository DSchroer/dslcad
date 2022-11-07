use super::*;
use crate::runtime::RuntimeError;
use crate::syntax::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn numeric(
    args: &HashMap<String, Value>,
    op: impl FnOnce(f64, f64) -> f64,
) -> Result<Value, RuntimeError> {
    let left = number!(args, "left");
    let right = number!(args, "right");

    Ok(Value::Number(op(left, right)))
}
