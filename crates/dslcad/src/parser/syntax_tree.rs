use crate::parser::DocId;
use logos::Span;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Variable {
        name: &'a str,
        value: Option<Expression<'a>>,
        span: Span,
    },
    Return(Expression<'a>, Span),
}

impl Statement<'_> {
    pub fn span(&self) -> &Span {
        match self {
            Statement::Variable { span, .. } => span,
            Statement::Return(_, span) => span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CallPath<'a> {
    String(&'a str),
    Document(&'a DocId),
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Literal(Literal<'a>, Span),
    Reference(&'a str, Span),
    Invocation {
        path: CallPath<'a>,
        arguments: HashMap<&'a str, Box<Expression<'a>>>,
        span: Span,
    },
    Access(Box<Expression<'a>>, &'a str, Span),
    Index {
        target: Box<Expression<'a>>,
        index: Box<Expression<'a>>,
        span: Span,
    },
    Map {
        identifier: &'a str,
        range: Box<Expression<'a>>,
        action: Box<Expression<'a>>,
        span: Span,
    },
    Reduce {
        left: &'a str,
        right: &'a str,
        root: Option<Box<Expression<'a>>>,
        range: Box<Expression<'a>>,
        action: Box<Expression<'a>>,
        span: Span,
    },
    If {
        condition: Box<Expression<'a>>,
        if_true: Box<Expression<'a>>,
        if_false: Box<Expression<'a>>,
        span: Span,
    },
}

impl Expression<'_> {
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
pub enum Literal<'a> {
    Number(f64),
    Bool(bool),
    Text(String),
    List(Vec<Expression<'a>>),
}
