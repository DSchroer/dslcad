use std::collections::HashSet;
use crate::syntax::Statement;
use std::slice::Iter;

#[derive(Debug)]
pub struct Document {
    identifiers: HashSet<String>,
    statements: Vec<Statement>,
}

impl Document {
    pub fn new(identifiers: HashSet<String>, statements: Vec<Statement>) -> Self {
        Document { identifiers, statements }
    }

    pub fn has_identifier(&self, name: &str) -> bool {
        self.identifiers.contains(name)
    }

    pub fn statements(&self) -> Iter<'_, Statement> {
        self.statements.iter()
    }
}
