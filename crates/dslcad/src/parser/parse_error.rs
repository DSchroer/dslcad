use crate::parser::DocId;
use crate::source::LineColExt;
use logos::Span;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[derive(Debug)]
pub struct ParseError {
    pub file: DocId,
    pub error: DocumentParseError,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl Error for ParseError {}

#[derive(Debug, Error)]
pub enum DocumentParseError {
    #[error("file not found")]
    NoSuchFile(),
    #[error("unexpected end of file")]
    UnexpectedEndOfFile(),
    #[error("unknown resource extension {0}")]
    UnknownResourceType(String, Span),
    #[error("use of undeclared identifier {0}")]
    UndeclaredIdentifier(String, Span),
    #[error("a variable already exists with the name {0}")]
    DuplicateVariableName(String, Span),
    #[error("parameters are not allowed in scopes")]
    ParametersNotAllowedInScopes(Span),
    #[error("expected {0} but found {1}")]
    Expected(&'static str, String, Span),
    #[error("expected one of {} but found {1}", one_of_list(.0))]
    ExpectedOneOf(Vec<&'static str>, String, Span),
}

fn one_of_list(list: &[&'static str]) -> String {
    list.join(" or ")
}

impl DocumentParseError {
    pub fn line_col(&self, text: &str) -> (usize, Span) {
        match self {
            DocumentParseError::NoSuchFile() => (0, 0..0),
            DocumentParseError::UnexpectedEndOfFile() => {
                let (i, line) = text.split('\n').enumerate().last().unwrap_or_default();
                (i, line.len()..line.len())
            }
            DocumentParseError::UnknownResourceType(_, span)
            | DocumentParseError::UndeclaredIdentifier(_, span)
            | DocumentParseError::DuplicateVariableName(_, span)
            | DocumentParseError::ParametersNotAllowedInScopes(span)
            | DocumentParseError::Expected(_, _, span)
            | DocumentParseError::ExpectedOneOf(_, _, span) => span.line_col(text),
        }
    }

    pub fn with_source(self, file: DocId) -> ParseError {
        ParseError { error: self, file }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn it_can_print_lines() {
        let source = "abc\nabc\nabv";
        let range = 10..11;
        let error = DocumentParseError::Expected("foo", "test".to_string(), range.clone())
            .with_source(DocId::new("test.txt".to_string()));

        assert_eq!("v", source[range].to_string());
        assert_eq!("expected foo but found test", format!("{error}"))
    }
}
