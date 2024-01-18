use crate::resources::Resource;
use logos::Span;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::sync::Arc;

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

#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub enum Statement {
    Variable {
        name: String,
        value: Option<Expression>,
        span: Span,
    },
    CreatePart(Expression, Span),
}

impl Statement {
    pub fn span(&self) -> &Span {
        match self {
            Statement::Variable { span, .. } => span,
            Statement::CreatePart(_, span) => span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CallPath {
    Function(String),
    Document(DocId),
}

#[derive(Debug, Clone)]
pub enum Argument {
    Named(String, Box<Expression>),
    Unnamed(Box<Expression>),
}

impl Argument {
    pub fn has_name(&self, name: &str) -> bool {
        match self {
            Argument::Named(n, _) => n == name,
            Argument::Unnamed(_) => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal, Span),
    Reference(String, Span),
    Invocation {
        path: CallPath,
        arguments: VecDeque<Argument>,
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
    Scope {
        statements: Vec<Statement>,
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
            Expression::Scope { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    Text(String),
    List(Vec<Expression>),
    Resource(Arc<dyn Resource>),
    Function(Vec<Statement>),
}
