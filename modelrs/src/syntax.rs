mod instance;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub use instance::Instance;
use opencascade::Shape;
use crate::runtime::ScriptInstance;

#[derive(Debug, Clone)]
pub enum Statement {
    Variable{ name: String, value: Option<Expression> },
    Return(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Value),
    Reference(String),
    Invocation{ path: String, arguments: HashMap<String, Box<Expression>> },

    Access(Box<Expression>, String),

    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Text(String),

    Script(Rc<RefCell<ScriptInstance>>),
    Shape(Rc<RefCell<Shape>>),

    Empty
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => Display::fmt(n, f),
            Value::Bool(n) => Display::fmt(n, f),
            Value::Text(n) => Display::fmt(n, f),
            Value::Script(i) => Display::fmt("INSTANCE", f),
            Value::Shape(s) => Display::fmt("SHAPE", f),
            Value::Empty => f.write_str("()")
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => Display::fmt(n, f),
            Value::Bool(n) => Display::fmt(n, f),
            Value::Text(n) => Display::fmt(n, f),
            Value::Script(i) => Display::fmt("INSTANCE", f),
            Value::Shape(s) => Display::fmt("SHAPE", f),
            Value::Empty => f.write_str("()")
        }
    }
}

impl Value {
    pub fn to_number(&self) -> Option<f64> {
        match self {
            Value::Number(f) => Some(*f),
            Value::Script(i) => i.borrow().value().to_number(),
            _ => None
        }
    }

    pub fn to_script(&self) -> Option<&Rc<RefCell<ScriptInstance>>> {
        match self {
            Value::Script(i) => Some(i),
            _ => None
        }
    }

    pub fn to_shape(&self) -> Option<&Rc<RefCell<Shape>>> {
        match self {
            Value::Shape(s) => Some(s),
            _ => None
        }
    }
}
