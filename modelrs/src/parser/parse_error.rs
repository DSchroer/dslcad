use std::path::PathBuf;
use logos::{Source, Span};
use thiserror::Error;
use crate::parser::lexer::Token;
use crate::parser::Reader;

#[derive(Error, Debug)]
pub enum ParseError{
    #[error("NoSuchFile {0}")]
    NoSuchFile(PathBuf),
    #[error("Aggregate Error {0:?}")]
    AggregateError(Vec<ParseError>),
    #[error("UnexpectedEndOfFile")]
    UnexpectedEndOfFile(PathBuf),
    #[error("Expected {0:?} but found {1:?}")]
    Expected(&'static str, Token, PathBuf, Span),
    #[error("Expected {0:?} but found {1:?}")]
    ExpectedOneOf(Vec<&'static str>, Token, PathBuf, Span),
}

impl ParseError {
    pub fn print(&self, reader: &impl Reader) {
        match self {
            ParseError::NoSuchFile(file) => {
                println!("error: {}", file.display());
                println!("unable to read file");
            },
            ParseError::AggregateError(errors) => {
                for error in errors {
                    error.print(reader)
                }
            }
            ParseError::UnexpectedEndOfFile(file) => {
                let text = reader.read(file).unwrap();
                let last = text.split("\n").enumerate().last();

                println!("error: {}", file.display());
                match last {
                    None => println!("unexpected end of line [0]:"),
                    Some((line, text)) => println!("unexpected end of line [{}]: {}", line, text)
                }
            }
            ParseError::Expected(expected, found, file, span) => {
                let text = reader.read(file).unwrap();

                println!("error: {}", file.display());
                println!("expected {} but found {:?}: {}", expected, found, text.slice(span.clone()).unwrap())
            }
            ParseError::ExpectedOneOf(expected, found, file, span) => {
                let text = reader.read(file).unwrap();

                println!("error: {}", file.display());
                println!("expected one of {} but found {:?}: {}", expected.join(" or "), found, text.slice(span.clone()).unwrap())
            }
        }
    }
}
