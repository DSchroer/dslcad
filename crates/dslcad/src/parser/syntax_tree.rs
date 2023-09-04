use logos::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DocId {
    path: String,
}

impl DocId {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn to_path(&self) -> &Path {
        return Path::new(&self.path);
    }

    pub fn to_str(&self) -> &str {
        &self.path
    }
}

impl Display for DocId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ast {
    root: DocId,
    pub documents: HashMap<DocId, Vec<Statement>>,
}

impl Ast {
    pub fn new(root: DocId) -> Self {
        Self {
            root,
            documents: HashMap::new(),
        }
    }
    pub fn root(&self) -> &DocId {
        &self.root
    }
    pub fn root_document(&self) -> &Vec<Statement> {
        self.documents.get(&self.root).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CallPath {
    String(String),
    Document(DocId),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum ArgName {
    Named(String),
    Default,
}

impl From<String> for ArgName {
    fn from(value: String) -> Self {
        ArgName::Named(value)
    }
}

impl From<&'static str> for ArgName {
    fn from(value: &'static str) -> Self {
        ArgName::Named(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal, Span),
    Reference(String, Span),
    Invocation {
        path: CallPath,
        arguments: HashMap<ArgName, Box<Expression>>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    Text(String),
    List(Vec<Expression>),
}
