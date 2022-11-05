mod shapes;

use std::collections::HashMap;
use crate::runtime::RuntimeError;
use crate::syntax::{Value};


type Function = dyn Fn(&HashMap<String, Value>) -> Result<Value, RuntimeError>;

pub struct Library;
impl Library {
    pub fn find(&self, name: &str) -> Option<&Function>{
        match name {
            "one" => Some(&|_|Ok(Value::Number(1 as f64))),
            "cube" => Some(&shapes::cube),
            "cylinder" => Some(&shapes::cylinder),
            "union" => Some(&shapes::union),
            "chamfer" => Some(&shapes::chamfer),
            "fillet" => Some(&shapes::fillet),
            "difference" => Some(&shapes::difference),
            _ => None
        }
    }
}
