mod instance;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::runtime::ScriptInstance;
pub use instance::Instance;
use opencascade::{Edge, Point, Shape};

#[derive(Debug, Clone)]
pub enum Statement {
    Variable {
        name: String,
        value: Option<Expression>,
    },
    Return(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Value),
    Reference(String),
    Invocation {
        path: String,
        arguments: HashMap<String, Box<Expression>>,
    },
    Access(Box<Expression>, String),
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Text(String),

    Script(Rc<RefCell<ScriptInstance>>),

    Point(Rc<RefCell<Point>>),
    Line(Rc<RefCell<Edge>>),
    Shape(Rc<RefCell<Shape>>),

    Empty,
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => Display::fmt(n, f),
            Value::Bool(n) => Display::fmt(n, f),
            Value::Text(n) => Display::fmt(n, f),
            Value::Script(_) => Display::fmt("INSTANCE", f),
            Value::Shape(_) => Display::fmt("SHAPE", f),
            Value::Point(_) => Display::fmt("POINT", f),
            Value::Line(_) => Display::fmt("LINE", f),
            Value::Empty => f.write_str("()"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Value {
    pub fn to_number(&self) -> Option<f64> {
        match self {
            Value::Number(f) => Some(*f),
            Value::Script(i) => i.borrow().value().to_number(),
            _ => None,
        }
    }

    pub fn to_script(&self) -> Option<&Rc<RefCell<ScriptInstance>>> {
        match self {
            Value::Script(i) => Some(i),
            _ => None,
        }
    }

    pub fn to_point(&self) -> Option<&Rc<RefCell<Point>>> {
        match self {
            Value::Point(s) => Some(s),
            _ => None,
        }
    }

    pub fn to_line(&self) -> Option<&Rc<RefCell<Edge>>> {
        match self {
            Value::Line(s) => Some(s),
            _ => None,
        }
    }

    pub fn to_shape(&self) -> Option<&Rc<RefCell<Shape>>> {
        match self {
            Value::Shape(s) => Some(s),
            _ => None,
        }
    }
}
