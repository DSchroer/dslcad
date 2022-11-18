use crate::syntax::Value;
use opencascade::{Point, Shape};

pub trait Accessible {
    fn get(&self, identifier: &str) -> Option<Value>;
}

impl Accessible for Shape {
    fn get(&self, _: &str) -> Option<Value> {
        None
    }
}

impl Accessible for Point {
    fn get(&self, identifier: &str) -> Option<Value> {
        match identifier {
            "x" => Some(Value::Number(self.x())),
            "y" => Some(Value::Number(self.y())),
            "z" => Some(Value::Number(self.z())),
            _ => None,
        }
    }
}
