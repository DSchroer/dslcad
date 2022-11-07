mod math;
mod shapes;

use crate::runtime::RuntimeError;
use crate::syntax::Value;
use std::collections::HashMap;
use std::ops::{Add, Deref, Div, Mul, Sub};

type Function = dyn Fn(&HashMap<String, Value>) -> Result<Value, RuntimeError>;

pub struct Library;
impl Library {
    pub fn find(&self, name: &str) -> Option<&Function> {
        match name {
            "add" => Some(&|a| math::numeric(a, f64::add)),
            "subtract" => Some(&|a| math::numeric(a, f64::sub)),
            "multiply" => Some(&|a| math::numeric(a, f64::mul)),
            "divide" => Some(&|a| math::numeric(a, f64::div)),

            "cube" => Some(&shapes::cube),
            "cylinder" => Some(&shapes::cylinder),
            "union" => Some(&shapes::union),
            "chamfer" => Some(&shapes::chamfer),
            "fillet" => Some(&shapes::fillet),
            "difference" => Some(&shapes::difference),
            "translate" => Some(&shapes::translate),
            "rotate" => Some(&shapes::rotate),
            "scale" => Some(&shapes::scale),
            _ => None,
        }
    }
}

macro_rules! number {
    ($args: ident, $name: literal) => {{
        let value = $args
            .get($name)
            .ok_or(RuntimeError::UnsetParameter(String::from($name)))?;
        value
            .to_number()
            .ok_or(RuntimeError::UnexpectedType(value.clone()))?
    }};
    ($args: ident, $name: literal, $default: literal) => {
        match $args.get($name) {
            Some(value) => value
                .to_number()
                .ok_or(RuntimeError::UnexpectedType(value.clone()))?,
            None => $default,
        }
    };
}
macro_rules! shape {
    ($args: ident, $name: literal) => {{
        let value = $args
            .get($name)
            .ok_or(RuntimeError::UnsetParameter(String::from($name)))?;
        value
            .to_shape()
            .ok_or(RuntimeError::UnexpectedType(value.clone()))?
    }};
}
pub(crate) use number;
pub(crate) use shape;
