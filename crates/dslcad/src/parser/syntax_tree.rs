use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io;
use std::rc::Rc;

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
    Literal(Literal),
    Reference(String),
    Invocation {
        path: String,
        arguments: HashMap<String, Box<Expression>>,
    },
    Access(Box<Expression>, String),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    Text(String),
    List(Vec<Expression>),
}
