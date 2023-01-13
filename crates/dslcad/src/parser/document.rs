use super::syntax_tree::Statement;
use std::collections::HashSet;
use std::slice::Iter;

#[derive(Debug)]
pub struct Document {
    id: String,
    source: String,
    identifiers: HashSet<String>,
    statements: Vec<Statement>,
}

impl Document {
    pub fn new(
        id: String,
        source: String,
        identifiers: HashSet<String>,
        statements: Vec<Statement>,
    ) -> Self {
        Document {
            id,
            source,
            identifiers,
            statements,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn has_identifier(&self, name: &str) -> bool {
        self.identifiers.contains(name)
    }

    pub fn statements(&self) -> Iter<'_, Statement> {
        self.statements.iter()
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}
