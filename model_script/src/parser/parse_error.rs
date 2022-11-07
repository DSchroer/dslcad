use crate::parser::lexer::Token;
use crate::parser::Reader;
use logos::{Source, Span};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("NoSuchFile {0}")]
    NoSuchFile(PathBuf),
    #[error("Aggregate Error {0:?}")]
    AggregateError(Vec<ParseError>),
    #[error("UnexpectedEndOfFile")]
    UnexpectedEndOfFile(PathBuf),
    #[error("Expected {0:?} but found {1:?}")]
    Expected(&'static str, PathBuf, Span),
    #[error("Expected {0:?} but found {1:?}")]
    ExpectedOneOf(Vec<&'static str>, PathBuf, Span),
}

impl ParseError {
    pub fn print(&self, reader: &impl Reader) {
        match self {
            ParseError::NoSuchFile(file) => {
                println!("error: {}", file.display());
                println!("unable to read file");
            }
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
                    Some((line, text)) => println!("unexpected end of line [{}]: {}", line, text),
                }
            }
            ParseError::Expected(expected, file, span) => {
                let text = reader.read(file).unwrap();

                let (line, col) = line_col(&text, &span);
                println!("error: {}[{}:{}]", file.display(), line, col.start);
                println!(
                    "expected {} but found {}",
                    expected,
                    text.slice(span.clone()).unwrap()
                )
            }
            ParseError::ExpectedOneOf(expected, file, span) => {
                let text = reader.read(file).unwrap();

                let (line, col) = line_col(&text, &span);
                println!("error: {}[{}:{}]", file.display(), line, col.start);
                println!(
                    "expected one of {} but found {}",
                    expected.join(" or "),
                    text.slice(span.clone()).unwrap()
                )
            }
        }
    }
}

fn line_col(text: &str, span: &Span) -> (usize, Span) {
    let mut target = span.clone();
    for (i, line) in text.split("\n").enumerate() {
        let len = line.len();
        if target.start > len {
            target.start -= len;
            target.end -= len;
        } else {
            return (i, target);
        }
    }
    return (0, target);
}
