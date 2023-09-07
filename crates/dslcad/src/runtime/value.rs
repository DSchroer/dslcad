use crate::runtime::output::IntoPart;
use persistence::protocol::Part;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use super::Access;
use super::Type;
use crate::runtime::ScriptInstance;
use opencascade::{Point, Shape, Wire};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Text(String),

    List(Vec<Value>),

    Script(Rc<ScriptInstance>),

    Point(Rc<Point>),
    Line(Rc<Wire>),
    Shape(Rc<Shape>),
}

impl From<Point> for Value {
    fn from(value: Point) -> Self {
        Value::Point(Rc::new(value))
    }
}

impl From<Shape> for Value {
    fn from(value: Shape) -> Self {
        Value::Shape(Rc::new(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Value::Bool(n) => f.debug_tuple("Bool").field(n).finish(),
            Value::Text(n) => f.debug_tuple("Text").field(n).finish(),
            Value::List(_) => f.debug_tuple("List").finish(),
            Value::Script(_) => f.debug_tuple("Script").finish(),
            Value::Shape(_) => f.debug_tuple("Shape").finish(),
            Value::Point(_) => f.debug_tuple("Point").finish(),
            Value::Line(_) => f.debug_tuple("Line").finish(),
        }
    }
}

impl Value {
    pub fn to_output(&self) -> Result<Part, opencascade::Error> {
        Ok(match self {
            Value::Number(v) => v.into_part()?,
            Value::Bool(v) => v.into_part()?,
            Value::Text(v) => v.into_part()?,
            Value::List(v) => v.into_part()?,

            Value::Script(s) => s.value().to_output()?,

            Value::Point(p) => p.into_part()?,
            Value::Line(l) => l.into_part()?,
            Value::Shape(s) => s.into_part()?,
        })
    }

    pub fn to_number(&self) -> Option<f64> {
        match self {
            Value::Number(f) => Some(*f),
            Value::Script(i) => i.value().to_number(),
            _ => None,
        }
    }

    pub fn to_text(&self) -> Option<String> {
        match self {
            Value::Text(f) => Some(f.clone()),
            Value::Script(i) => i.value().to_text(),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(f) => Some(*f),
            Value::Script(i) => i.value().to_bool(),
            _ => None,
        }
    }

    pub fn to_accessible(&self) -> Option<&dyn Access> {
        match self {
            Value::Script(i) => Some(i.as_ref()),
            Value::Line(w) => Some(w.as_ref()),
            Value::Shape(s) => Some(s.as_ref()),
            Value::Point(p) => Some(p.as_ref()),
            _ => None,
        }
    }

    pub fn to_point(&self) -> Option<Rc<Point>> {
        match self {
            Value::Point(s) => Some(s.clone()),
            Value::Script(i) => i.value().to_point(),
            _ => None,
        }
    }

    pub fn to_line(&self) -> Option<Rc<Wire>> {
        match self {
            Value::Line(s) => Some(s.clone()),
            Value::Script(i) => i.value().to_line(),
            _ => None,
        }
    }

    pub fn to_shape(&self) -> Option<Rc<Shape>> {
        match self {
            Value::Shape(s) => Some(s.clone()),
            Value::Script(i) => i.value().to_shape(),
            _ => None,
        }
    }

    pub fn to_list(&self) -> Option<Vec<Value>> {
        match self {
            Value::List(s) => Some(s.clone()),
            Value::Script(i) => i.value().to_list(),
            _ => None,
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            Value::Number(_) => Type::Number,
            Value::Bool(_) => Type::Bool,
            Value::Text(_) => Type::Text,
            Value::List(_) => Type::List,
            Value::Script(i) => i.value().get_type(),
            Value::Point(_) => Type::Point,
            Value::Line(_) => Type::Edge,
            Value::Shape(_) => Type::Shape,
        }
    }
}
