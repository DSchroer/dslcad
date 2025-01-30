use crate::parser::syntax_visitor::{ExpressionVisitor, LiteralVisitor, StatementVisitor};
use crate::resources::Resource;
use logos::Span;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::rc::Rc;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DocId {
    id: String,
    path: Option<String>,
}

impl DocId {
    pub fn new(path: String) -> Self {
        Self { id: path, path: None }
    }

    pub fn new_with_path(id: &'static str, path: Option<String>) -> Self {
        Self { id: id.to_string(), path }
    }

    pub fn to_path(&self) -> &Path {
        Path::new(self.path.as_ref().unwrap_or(&self.id))
    }

    pub fn to_str(&self) -> &str {
        &self.id
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

#[derive(Debug)]
pub enum Statement {
    Variable(Variable, Span),
    CreatePart(Expression, Span),
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: Option<Expression>,
}

impl Statement {
    pub fn walk_statement<T: StatementVisitor>(&self, visitor: &mut T) -> T::Result {
        match self {
            Statement::Variable(v, s) => visitor.visit_variable(v, s),
            Statement::CreatePart(v, s) => visitor.visit_create_part(v, s),
        }
    }
}

impl Statement {
    pub fn span(&self) -> &Span {
        match self {
            Statement::Variable(_, s) => s,
            Statement::CreatePart(_, s) => s,
        }
    }
}

#[derive(Debug)]
pub enum CallPath {
    Function(Box<Expression>),
    Document(DocId),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Expression {
    Literal(Literal, Span),
    Reference(Reference, Span),
    Invocation(Invocation, Span),
    Property(Property, Span),
    Index(Index, Span),
    Map(Map, Span),
    Reduce(Reduce, Span),
    If(If, Span),
    Scope(NestedScope, Span),
}

#[derive(Debug)]
pub struct Invocation {
    pub path: CallPath,
    pub arguments: VecDeque<Argument>,
}

#[derive(Debug)]
pub struct Reference {
    pub name: String,
}

#[derive(Debug)]
pub struct Property {
    pub target: Box<Expression>,
    pub name: String,
}

#[derive(Debug)]
pub struct Index {
    pub target: Box<Expression>,
    pub index: Box<Expression>,
}

#[derive(Debug)]
pub struct Map {
    pub identifier: String,
    pub range: Box<Expression>,
    pub action: Box<Expression>,
}

#[derive(Debug)]
pub struct Reduce {
    pub left: String,
    pub right: String,
    pub root: Option<Box<Expression>>,
    pub range: Box<Expression>,
    pub action: Box<Expression>,
}

#[derive(Debug)]
pub struct If {
    pub condition: Box<Expression>,
    pub if_true: Box<Expression>,
    pub if_false: Box<Expression>,
}

#[derive(Debug)]
pub struct NestedScope {
    pub statements: Vec<Statement>,
}

impl Expression {
    pub fn walk_expression<T: ExpressionVisitor>(&self, visitor: &mut T) -> T::Result {
        match self {
            Expression::Literal(v, s) => visitor.visit_literal(v, s),
            Expression::Reference(v, s) => visitor.visit_reference(v, s),
            Expression::Invocation(v, s) => visitor.visit_invocation(v, s),
            Expression::Property(v, s) => visitor.visit_property(v, s),
            Expression::Index(v, s) => visitor.visit_index(v, s),
            Expression::Map(v, s) => visitor.visit_map(v, s),
            Expression::Reduce(v, s) => visitor.visit_reduce(v, s),
            Expression::If(v, s) => visitor.visit_if(v, s),
            Expression::Scope(v, s) => visitor.visit_scope(v, s),
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            Expression::Literal(_, span) => span,
            Expression::Reference(_, span) => span,
            Expression::Invocation(_, span) => span,
            Expression::Property(_, span) => span,
            Expression::Index(_, span) => span,
            Expression::Map(_, span) => span,
            Expression::Reduce(_, span) => span,
            Expression::If(_, span) => span,
            Expression::Scope(_, span) => span,
        }
    }
}

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    Text(String),
    List(Vec<Expression>),
    Resource(Box<dyn Resource>),
    Function(Rc<Vec<Statement>>),
}

impl Literal {
    pub fn walk_literal<T: LiteralVisitor>(&self, visitor: &mut T) -> T::Result {
        match self {
            Literal::Number(v) => visitor.visit_number(v),
            Literal::Bool(v) => visitor.visit_bool(v),
            Literal::Text(v) => visitor.visit_text(v),
            Literal::List(v) => visitor.visit_list(v),
            Literal::Resource(v) => visitor.visit_resource(v.as_ref()),
            Literal::Function(v) => visitor.visit_function(v),
        }
    }
}
