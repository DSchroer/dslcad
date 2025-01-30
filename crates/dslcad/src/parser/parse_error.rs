use crate::parser::DocId;
use logos::{Source, Span};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};

#[derive(Debug)]
pub enum ParseError {
    NoSuchFile {
        file: DocId,
    },
    DocumentError {
        error: DocumentParseError,
        file: DocId,
        source: String,
    },
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NoSuchFile { file } => {
                f.write_fmt(format_args!("error: {}\n", file.to_path().display()))?;
                f.write_str("unable to read file")?;
            }
            ParseError::DocumentError {
                error,
                file,
                source,
            } => {
                let text = source;

                let (line, col) = error.line_col(text);
                f.write_fmt(format_args!(
                    "error: {}[{}:{}-{}]\n",
                    file, line, col.start, col.end
                ))?;

                match error {
                    DocumentParseError::UnexpectedEndOfFile() => {
                        f.write_fmt(format_args!("unexpected end of file"))?;
                    }
                    DocumentParseError::UndeclaredIdentifier(span) => {
                        f.write_fmt(format_args!(
                            "undeclared identifier {}",
                            text.slice(span.clone()).unwrap()
                        ))?;
                    }
                    DocumentParseError::DuplicateVariableName(span) => {
                        f.write_fmt(format_args!(
                            "duplicate variable name {}",
                            text.slice(span.clone()).unwrap()
                        ))?;
                    }
                    DocumentParseError::Expected(expected, span) => {
                        f.write_fmt(format_args!(
                            "expected {} but found {}",
                            expected,
                            text.slice(span.clone()).unwrap()
                        ))?;
                    }
                    DocumentParseError::ExpectedOneOf(expected, span) => {
                        f.write_fmt(format_args!(
                            "expected one of {} but found {}",
                            expected.join(" or "),
                            text.slice(span.clone()).unwrap()
                        ))?;
                    }
                    DocumentParseError::UnknownResourceType(extension, _) => {
                        f.write_fmt(format_args!("unknown resource extension {}", extension))?;
                    }
                    DocumentParseError::ParametersNotAllowedInScopes(_) => {
                        f.write_fmt(format_args!("parameters are not allowed in scopes"))?;
                    }
                }
            }
        }
        f.write_char('\n')?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum DocumentParseError {
    UnexpectedEndOfFile(),
    UnknownResourceType(String, Span),
    UndeclaredIdentifier(Span),
    DuplicateVariableName(Span),
    ParametersNotAllowedInScopes(Span),
    Expected(&'static str, Span),
    ExpectedOneOf(Vec<&'static str>, Span),
}

impl DocumentParseError {
    pub fn line_col(&self, text: &str) -> (usize, Span) {
        match self {
            DocumentParseError::UnexpectedEndOfFile() => {
                let (i, line) = text.split('\n').enumerate().last().unwrap_or_default();
                (i, line.len()..line.len())
            }
            DocumentParseError::UnknownResourceType(_, span)
            | DocumentParseError::UndeclaredIdentifier(span)
            | DocumentParseError::DuplicateVariableName(span)
            | DocumentParseError::ParametersNotAllowedInScopes(span)
            | DocumentParseError::Expected(_, span)
            | DocumentParseError::ExpectedOneOf(_, span) => line_col(text, span),
        }
    }

    pub fn with_source(self, file: DocId, source: String) -> ParseError {
        ParseError::DocumentError {
            error: self,
            file,
            source,
        }
    }
}

impl Error for DocumentParseError {}

impl Display for DocumentParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
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

    #[test]
    fn it_can_print_lines() {
        let source = "abc\nabc\nabv";
        let range = 10..11;
        let error = DocumentParseError::Expected("foo", range.clone())
            .with_source(DocId::new("test.txt".to_string()), source.to_string());

        assert_eq!("v", source[range].to_string());
        assert_eq!(
            "error: test.txt[3:2-3]\nexpected foo but found v\n",
            format!("{error}")
        )
    }
}
