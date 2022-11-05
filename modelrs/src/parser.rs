mod lexer;
mod document;
mod reader;

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use path_absolutize::*;
use crate::syntax::{*};
use logos::{Logos, Span};
use thiserror::Error;
use lexer::{Token, Lexer};

pub use document::Document;
pub use reader::Reader;

pub struct Parser<'a, T: Reader> {
    path: Cow<'a, Path>,
    reader: &'a T,
    documents: HashMap<String, Document>,
}

#[derive(Debug)]
pub enum ParseResult {
    Success(HashMap<String, Document>),
    Failure(Vec<ParseError>)
}

#[derive(Error, Debug)]
pub enum ParseError{
    #[error("NoSuchFile {0}")]
    NoSuchFile(String),
    #[error("SubparseFailure")]
    SubparseFailure,
    #[error("UnexpectedEndOfLine")]
    UnexpectedEndOfLine,
    #[error("UnexpectedToken {0:?} at {1:?}")]
    UnexpectedToken(Token, Span)
}

macro_rules! take {
    ($lexer: ident, $token: pat) => {
        match $lexer.next() {
            Some($token) => {},
            None => return Err(ParseError::UnexpectedEndOfLine),
            Some(t) => return Err(ParseError::UnexpectedToken(t, $lexer.span())),
        };
    };
    ($lexer: ident, $($token: pat => $case: expr), *) => {
        match $lexer.next() {
            $(Some($token) => $case,)*
            Some(t) => return Err(ParseError::UnexpectedToken(t, $lexer.span())),
            None => return Err(ParseError::UnexpectedEndOfLine),
        }
    };
}

impl<'a, T: Reader> Parser<'a, T> {
    pub fn new(path: &'a str, reader: &'a T) -> Self {
        let path = Path::new(path).absolutize().unwrap();
        Parser { path, reader, documents: HashMap::new() }
    }

    pub fn parse(mut self) -> ParseResult {
        let input = self.reader.read(self.path.to_str().unwrap());
        let mut lex = Token::lexer(&input);

        let mut statements = Vec::new();
        loop {
            let statement = match lex.clone().next() {
                Some(_) => self.parse_statement(&mut lex),
                None => break
            };
            match statement {
                Ok(s) => statements.push(s),
                Err(error) => return ParseResult::Failure(vec![error])
            }
        }

        self.documents.insert(String::from(self.path.to_str().unwrap()), Document::new(statements));
        return ParseResult::Success(self.documents);
    }

    fn parse_statement(&mut self, lexer: &mut Lexer) -> Result<Statement, ParseError> {
        let mut peek = lexer.clone();
        take!(peek,
            Token::Var => self.parse_variable_statement(lexer),
            _ => self.parse_return_statement(lexer)
        )
    }

    fn parse_return_statement(&mut self, lexer: &mut Lexer) -> Result<Statement, ParseError> {
        let expr = self.parse_expression(lexer)?;
        take!(lexer, Token::Semicolon);
        Ok(Statement::Return(expr))
    }

    fn parse_variable_statement(&mut self, lexer: &mut Lexer) -> Result<Statement, ParseError> {
        take!(lexer, Token::Var);
        let name = take!(lexer, Token::Identifier => lexer.slice().to_string());
        let expr = take!(lexer,
            Token::Semicolon => None,
            Token::Equal => {
                let expr = self.parse_expression(lexer)?;
                take!(lexer, Token::Semicolon);
                Some(expr)
            }
        );
        Ok(Statement::Variable { name, value: expr })
    }

    fn parse_call(&mut self, lexer: &mut Lexer) -> Result<Expression, ParseError> {
        let path = take!(lexer,
            Token::Identifier => lexer.slice().to_string(),
            Token::Path => {
                let path =  lexer.slice();

                let mut buf = std::path::PathBuf::new();
                buf.push(self.path.clone());
                let buf = buf.parent().unwrap();
                let buf = buf.join(path.to_string() + ".ex").canonicalize();
                let buf = match &buf {
                    Ok(buf) => buf.to_str().unwrap(),
                    Err(_) => return Err(ParseError::NoSuchFile(path.to_string()))
                };

                if !self.documents.contains_key(buf) && self.path.to_str().unwrap() != buf {
                    let r = Parser::new(buf, self.reader).parse();
                    match r {
                        ParseResult::Success(docs) => {
                            for (path, document) in docs {
                                self.documents.insert(path, document);
                            }
                        },
                        ParseResult::Failure(_) => return Err(ParseError::SubparseFailure)
                    }
                }

                buf.to_string()
            }
        );

        take!(lexer, Token::OpenBracket);

        let mut args = HashMap::new();
        loop {
            let mut peek = lexer.clone();
            take!(peek,
                Token::CloseBracket => {
                    lexer.next();
                    break;
                },
                Token::Identifier => {
                    let (name, expression) = self.parse_argument(lexer)?;
                    args.insert(name, Box::new(expression));
                    take!(lexer,
                        Token::Comma => {},
                        Token::CloseBracket => break
                    );
                }
            )
        }

        Ok(Expression::Invocation { path, arguments: args })
    }

    fn parse_argument(&mut self, lexer: &mut Lexer) -> Result<(String, Expression), ParseError> {
        let name = take!(lexer, Token::Identifier => lexer.slice().to_string());
        take!(lexer, Token::Equal);
        let expr = self.parse_expression(lexer)?;
        Ok((name, expr))
    }

    fn parse_reference(&self, lexer: &mut Lexer) -> Result<Expression, ParseError> {
        Ok(take!(lexer, Token::Identifier => Expression::Reference(lexer.slice().to_string())))
    }

    fn parse_expression(&mut self, lexer: &mut Lexer) -> Result<Expression, ParseError> {
        let mut peek = lexer.clone();
        let first = take!(peek,
            Token::Number => {
                lexer.next();
                let value = f64::from_str(lexer.slice()).unwrap();
                Expression::Literal(Value::Number(value))
            },
            Token::Bool => {
                lexer.next();
                let value = lexer.slice() == "true";
                Expression::Literal(Value::Bool(value))
            },
            Token::String => {
                lexer.next();
                let value = lexer.slice();
                Expression::Literal(Value::Text(value[1..value.len()-1].to_string()))
            },
            Token::Path => self.parse_call(lexer)?,
            Token::Identifier => {
                match peek.next() {
                    Some(Token::OpenBracket) => self.parse_call(lexer)?,
                    _ => self.parse_reference(lexer)?,
                }
            },
            Token::OpenBracket => {
                lexer.next();
                let expr = self.parse_expression(lexer)?;
                take!(lexer, Token::CloseBracket => expr)
            }
        );

        let mut peek = lexer.clone();
        let first = match peek.next() {
            Some(Token::Period) => {
                lexer.next();
                let l = Box::new(first);
                take!(lexer, Token::Identifier);
                let r = lexer.slice().to_string();
                Expression::Access(l, r)
            },
            _ => first,
        };

        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Plus) => {
                lexer.next();
                let l = Box::new(first);
                let r = Box::new(self.parse_expression(lexer)?);
                Ok(Expression::Add(l, r))
            },
            Some(Token::Minus) => {
                lexer.next();
                let l = Box::new(first);
                let r = Box::new(self.parse_expression(lexer)?);
                Ok(Expression::Subtract(l, r))
            },
            Some(Token::Multiply) => {
                lexer.next();
                let l = Box::new(first);
                let r = Box::new(self.parse_expression(lexer)?);
                Ok(Expression::Multiply(l, r))
            },
            Some(Token::Divide) => {
                lexer.next();
                let l = Box::new(first);
                let r = Box::new(self.parse_expression(lexer)?);
                Ok(Expression::Divide(l, r))
            }
            _ => Ok(first)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{*};

    impl ParseResult {
        #[inline]
        #[track_caller]
        pub fn unwrap(self) -> HashMap<String, Document>
        {
            match self {
                ParseResult::Success(t) => t,
                ParseResult::Failure(e) => panic!("called `Result::unwrap()` on an `Err` value {:?}", e),
            }
        }
    }

    #[test]
    fn it_can_parse_variable() {
        Parser::new("test", &TestReader("test();"))
            .parse_variable_statement(&mut Token::lexer("var x = 5;")).unwrap();
        Parser::new("test", &TestReader("test();"))
            .parse_variable_statement(&mut Token::lexer("var x;")).unwrap();
        Parser::new("test", &TestReader("test();"))
            .parse_variable_statement(&mut Token::lexer("var x = true;")).unwrap();
    }

    #[test]
    fn it_can_parse_calls() {
        Parser::new("test", &TestReader("test();"))
            .parse_call(&mut Token::lexer("cube()")).unwrap();
        Parser::new("test", &TestReader("test();"))
            .parse_call(&mut Token::lexer("cube(x=5)")).unwrap();
    }

    #[test]
    fn it_can_parse() {
        Parser::new("test", &TestReader("test();")).parse().unwrap();
    }

    #[test]
    fn it_can_parse_adds() {
        Parser::new("test", &TestReader("2 + 2;")).parse().unwrap();
        Parser::new("test", &TestReader("test.area + 10;")).parse().unwrap();
    }

    #[test]
    fn it_can_parse_brackets() {
        Parser::new("test", &TestReader("3 - (3 + 2);")).parse().unwrap();
        Parser::new("test", &TestReader("(3 - 3) + 2;")).parse().unwrap();
    }

    #[test]
    fn it_can_parse_access() {
        Parser::new("test", &TestReader("foo.bar;")).parse().unwrap();
    }

    struct TestReader<'a>(&'a str);
    impl<'a> Reader for TestReader<'a> {
        fn read(&self, _: &str) -> String {
            self.0.to_string()
        }
    }
}