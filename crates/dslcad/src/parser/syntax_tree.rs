use logos::Span;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Statement {
    Variable {
        name: String,
        value: Option<Expression>,
        span: Span,
    },
    Return(Expression, Span),
}

impl Statement {
    pub fn span(&self) -> &Span {
        match self {
            Statement::Variable { span, .. } => span,
            Statement::Return(_, span) => span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal, Span),
    Reference(String, Span),
    Invocation {
        path: String,
        arguments: HashMap<String, Box<Expression>>,
        span: Span,
    },
    Access(Box<Expression>, String, Span),
    Index {
        target: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    Map {
        identifier: String,
        range: Box<Expression>,
        action: Box<Expression>,
        span: Span,
    },
    Reduce {
        left: String,
        right: String,
        root: Option<Box<Expression>>,
        range: Box<Expression>,
        action: Box<Expression>,
        span: Span,
    },
    If {
        condition: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Box<Expression>,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> &Span {
        match self {
            Expression::Literal(_, span) => span,
            Expression::Reference(_, span) => span,
            Expression::Invocation { span, .. } => span,
            Expression::Access(_, _, span) => span,
            Expression::Index { span, .. } => span,
            Expression::Map { span, .. } => span,
            Expression::Reduce { span, .. } => span,
            Expression::If { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    Text(String),
    List(Vec<Expression>),
}
