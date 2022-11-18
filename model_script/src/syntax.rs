mod accessible;
mod output;
mod types;

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::rc::Rc;

use crate::runtime::ScriptInstance;
pub use accessible::Accessible;
use opencascade::{Edge, Point, Shape};
pub use types::Type;

pub use output::Output;

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
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Value {
    pub fn to_output(&self) -> Result<Output, io::Error> {
        Ok(match self {
            Value::Number(v) => Output::Value(v.to_string()),
            Value::Bool(v) => Output::Value(v.to_string()),
            Value::Text(v) => Output::Value(v.to_string()),

            Value::Script(s) => s.borrow().value().to_output()?,

            Value::Point(p) => {
                Output::Figure(vec![vec![[p.borrow().x(), p.borrow().y(), p.borrow().z()]]])
            }
            Value::Line(l) => Output::Figure(l.borrow_mut().points()),

            Value::Shape(s) => Output::Shape(s.borrow_mut().mesh()?),
        })
    }

    pub fn to_number(&self) -> Option<f64> {
        match self {
            Value::Number(f) => Some(*f),
            Value::Script(i) => i.borrow().value().to_number(),
            _ => None,
        }
    }

    pub fn to_script(&self) -> Option<Ref<dyn Accessible>> {
        match self {
            Value::Script(i) => Some(i.borrow()),
            Value::Shape(s) => Some(s.borrow()),
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

    pub fn to_line(&self) -> Option<Rc<RefCell<Edge>>> {
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

    pub fn get_type(&self) -> Type {
        match self {
            Value::Number(_) => Type::Number,
            Value::Bool(_) => Type::Bool,
            Value::Text(_) => Type::Text,
            Value::Script(i) => i.borrow().value().get_type(),
            Value::Point(_) => Type::Point,
            Value::Line(_) => Type::Edge,
            Value::Shape(_) => Type::Shape,
        }
    }
}
