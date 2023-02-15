use std::cell::{Ref, RefCell};

use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use super::Access;
use super::Output;
use super::Type;
use crate::runtime::ScriptInstance;
use opencascade::{Point, Shape, Wire};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Text(String),

    List(Vec<Value>),

    Script(Rc<RefCell<ScriptInstance>>),

    Point(Rc<RefCell<Point>>),
    Line(Rc<RefCell<Wire>>),
    Shape(Rc<RefCell<Shape>>),
}

impl From<Shape> for Value {
    fn from(value: Shape) -> Self {
        Value::Shape(Rc::new(RefCell::new(value)))
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
    pub fn to_output(&self) -> Result<Output, opencascade::Error> {
        Ok(match self {
            Value::Number(v) => v.into(),
            Value::Bool(v) => v.into(),
            Value::Text(v) => v.into(),
            Value::List(v) => v.into(),

            Value::Script(s) => s.borrow().value().to_output()?,

            Value::Point(p) => p.borrow().into(),
            Value::Line(l) => l.borrow_mut().try_into()?,
            Value::Shape(s) => s.borrow_mut().try_into()?,
        })
    }

    pub fn to_number(&self) -> Option<f64> {
        match self {
            Value::Number(f) => Some(*f),
            Value::Script(i) => i.borrow().value().to_number(),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(f) => Some(*f),
            Value::Script(i) => i.borrow().value().to_bool(),
            _ => None,
        }
    }

    pub fn to_accessible(&self) -> Option<Ref<dyn Access>> {
        match self {
            Value::Script(i) => Some(i.borrow()),
            Value::Shape(s) => Some(s.borrow()),
            Value::Point(p) => Some(p.borrow()),
            _ => None,
        }
    }

    pub fn to_point(&self) -> Option<Rc<RefCell<Point>>> {
        match self {
            Value::Point(s) => Some(s.clone()),
            Value::Script(i) => i.borrow().value().to_point(),
            _ => None,
        }
    }

    pub fn to_line(&self) -> Option<Rc<RefCell<Wire>>> {
        match self {
            Value::Line(s) => Some(s.clone()),
            Value::Script(i) => i.borrow().value().to_line(),
            _ => None,
        }
    }

    pub fn to_shape(&self) -> Option<Rc<RefCell<Shape>>> {
        match self {
            Value::Shape(s) => Some(s.clone()),
            Value::Script(i) => i.borrow().value().to_shape(),
            _ => None,
        }
    }

    pub fn to_list(&self) -> Option<Vec<Value>> {
        match self {
            Value::List(s) => Some(s.clone()),
            Value::Script(i) => i.borrow().value().to_list(),
            _ => None,
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            Value::Number(_) => Type::Number,
            Value::Bool(_) => Type::Bool,
            Value::Text(_) => Type::Text,
            Value::List(_) => Type::List,
            Value::Script(i) => i.borrow().value().get_type(),
            Value::Point(_) => Type::Point,
            Value::Line(_) => Type::Edge,
            Value::Shape(_) => Type::Shape,
        }
    }
}
