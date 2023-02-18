use super::syntax_tree::Statement;
use crate::parser::source_store::DocId;
use std::collections::HashSet;
use std::slice::Iter;

#[derive(Debug)]
pub struct Document<'a> {
    id: &'a DocId,
    source: &'a str,
    identifiers: HashSet<&'a str>,
    statements: Vec<Statement<'a>>,
}

impl<'a> Document<'a> {
    pub fn new(
        id: &'a DocId,
        source: &'a str,
        identifiers: HashSet<&'a str>,
        statements: Vec<Statement<'a>>,
    ) -> Self {
        Document {
            id,
            source,
            identifiers,
            statements,
        }
    }

    pub fn id(&self) -> &str {
        self.id.to_str()
    }

    pub fn has_identifier(&self, name: &str) -> bool {
        self.identifiers.contains(name)
    }

    pub fn statements(&self) -> Iter<'_, Statement> {
        self.statements.iter()
    }

    pub fn source(&self) -> &str {
        self.source
    }
}
