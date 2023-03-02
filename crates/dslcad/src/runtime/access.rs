use super::value::Value;
use opencascade::{Point, Shape, Wire};

pub trait Access {
    fn get(&self, identifier: &str) -> Option<Value>;
}

impl Access for Wire {
    fn get(&self, identifier: &str) -> Option<Value> {
        match identifier {
            "center" => Some(self.center_of_mass().into()),
            _ => None,
        }
    }
}

impl Access for Shape {
    fn get(&self, identifier: &str) -> Option<Value> {
        match identifier {
            "center" => Some(self.center_of_mass().into()),
            _ => None,
        }
    }
}

impl Access for Point {
    fn get(&self, identifier: &str) -> Option<Value> {
        match identifier {
            "x" => Some(Value::Number(self.x())),
            "y" => Some(Value::Number(self.y())),
            "z" => Some(Value::Number(self.z())),
            _ => None,
        }
    }
}
