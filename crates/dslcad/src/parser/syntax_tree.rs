use std::collections::HashMap;
use std::fmt::Debug;

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
    Map {
        identifier: String,
        range: Box<Expression>,
        action: Box<Expression>,
    },
    Reduce {
        left: String,
        right: String,
        root: Option<Box<Expression>>,
        range: Box<Expression>,
        action: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    Text(String),
    List(Vec<Expression>),
}
