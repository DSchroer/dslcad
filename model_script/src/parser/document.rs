use crate::syntax::Statement;
use std::slice::Iter;

#[derive(Debug)]
pub struct Document {
    statements: Vec<Statement>,
}

impl Document {
    pub fn new(statements: Vec<Statement>) -> Self {
        Document { statements }
    }

    pub fn statements(&self) -> Iter<'_, Statement> {
        self.statements.iter()
    }
}
