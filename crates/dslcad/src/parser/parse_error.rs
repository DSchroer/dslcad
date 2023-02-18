use crate::parser::source_store::DocId;
use logos::{Source, Span};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};
use std::path::PathBuf;

pub enum ParseError {
    NoSuchFile(PathBuf),
    AggregateError(Vec<ParseError>),
    UnexpectedEndOfFile(DocId, String),
    UndeclaredIdentifier(DocId, Span, String),
    DuplicateVariableName(DocId, Span, String),
    Expected(&'static str, DocId, Span, String),
    ExpectedOneOf(Vec<&'static str>, DocId, Span, String),
}

impl Error for ParseError {}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NoSuchFile(file) => {
                f.write_fmt(format_args!("error: {}\n", file.display()))?;
                f.write_str("unable to read file")?;
            }
            ParseError::AggregateError(errors) => {
                for error in errors {
                    Display::fmt(error, f)?;
                }
            }
            ParseError::UnexpectedEndOfFile(file, text) => {
                let last = text.split('\n').enumerate().last();

                f.write_fmt(format_args!("error: {}\n", file))?;
                match last {
                    None => f.write_str("unexpected end of line [0]:")?,
                    Some((line, text)) => {
                        f.write_fmt(format_args!("unexpected end of line [{line}]: {text}"))?
                    }
                }
            }
            ParseError::UndeclaredIdentifier(file, span, text) => {
                let (line, col) = line_col(text, span);
                f.write_fmt(format_args!("error: {}[{}:{}]\n", file, line, col.start))?;
                f.write_fmt(format_args!(
                    "undeclared identifier {}",
                    text.slice(span.clone()).unwrap()
                ))?;
            }
            ParseError::DuplicateVariableName(file, span, text) => {
                let (line, col) = line_col(text, span);
                f.write_fmt(format_args!("error: {}[{}:{}]\n", file, line, col.start))?;
                f.write_fmt(format_args!(
                    "duplicate variable name {}",
                    text.slice(span.clone()).unwrap()
                ))?;
            }
            ParseError::Expected(expected, file, span, text) => {
                let (line, col) = line_col(text, span);
                f.write_fmt(format_args!("error: {}[{}:{}]\n", file, line, col.start))?;
                f.write_fmt(format_args!(
                    "expected {} but found {}",
                    expected,
                    text.slice(span.clone()).unwrap()
                ))?;
            }
            ParseError::ExpectedOneOf(expected, file, span, text) => {
                let (line, col) = line_col(text, span);
                f.write_fmt(format_args!("error: {}[{}:{}]\n", file, line, col.start))?;
                f.write_fmt(format_args!(
                    "expected one of {} but found {}",
                    expected.join(" or "),
                    text.slice(span.clone()).unwrap()
                ))?
            }
        }
        f.write_char('\n')?;
        Ok(())
    }
}

fn line_col(text: &str, span: &Span) -> (usize, Span) {
    let mut target = span.clone();
    for (i, line) in text.split('\n').enumerate() {
        let len = line.len();
        if target.start > len {
            target.start -= len + 1;
            target.end -= len + 1;
        } else {
            return (i + 1, target);
        }
    }
    (1, target)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::parser::tests::TestReader;
    use crate::parser::SourceStore;

    #[test]
    fn it_can_print_lines() {
        let source = "abc\nabc\nabv";
        let range = 10..11;
        let store = SourceStore::new(Box::new(TestReader("foo")));
        let error = ParseError::Expected(
            "foo",
            store.forge_id("test.txt".to_string()).unwrap().clone(),
            range.clone(),
            source.to_string(),
        );

        assert_eq!("v", source[range].to_string());
        assert_eq!(
            "error: test.txt[3:2]\nexpected foo but found v\n",
            format!("{error}")
        )
    }
}
