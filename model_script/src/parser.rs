mod document;
mod lexer;
mod parse_error;
mod reader;

use crate::syntax::*;
use lexer::{Lexer, Token};
use logos::Logos;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub use document::Document;
pub use parse_error::ParseError;
pub use reader::Reader;

pub struct Parser<'a, T> {
    reader: &'a T,
    path: PathBuf,
    documents: HashMap<String, Document>,
    variables: HashSet<String>,
}

#[derive(Debug)]
pub struct Ast {
    root: PathBuf,
    documents: HashMap<String, Document>,
}

impl Ast {
    pub fn root_document(&self) -> &Document {
        self.documents.get(self.root.to_str().unwrap()).unwrap()
    }

    pub fn documents(&self) -> &HashMap<String, Document> {
        &self.documents
    }
}

macro_rules! take {
    ($self: ident, $lexer: ident, $token: pat = $name: literal) => {
        match $lexer.next() {
            Some($token) => {},
            Some(_) => return Err(ParseError::Expected($name, $self.path.clone(), $lexer.span(), $self.source()?)),
            None => return Err(ParseError::UnexpectedEndOfFile($self.path.clone(), $self.source()?)),
        };
    };
    ($self: ident, $lexer: ident, $($token: pat = $name: literal => $case: expr), *) => {
        match $lexer.next() {
            $(Some($token) => $case,)*
            Some(_) => return Err(ParseError::ExpectedOneOf(vec![$($name,)*], $self.path.clone(), $lexer.span(), $self.source()?)),
            None => return Err(ParseError::UnexpectedEndOfFile($self.path.clone(), $self.source()?)),
        }
    };
}

impl<'a, T: Reader> Parser<'a, T> {
    pub fn new(path: &str, reader: &'a T) -> Self {
        let path = reader.normalize(Path::new(path));

        Parser {
            reader,
            path,
            documents: HashMap::new(),
            variables: HashSet::new(),
        }
    }

    fn source(&self) -> Result<String, ParseError> {
        let input = self.reader.read(self.path.as_path());
        if input.is_err() {
            return Err(ParseError::NoSuchFile(self.path.clone()));
        }
        Ok(input.unwrap())
    }

    pub fn parse(mut self) -> Result<Ast, ParseError> {
        let source = self.source()?;
        let mut lex = Token::lexer(&source);

        let mut seen_return = false;
        let mut statements = Vec::new();
        while let Some(_) = lex.clone().next() {
            let statement = self.parse_statement(&mut lex);
            match statement {
                Ok(s) => {
                    if matches!(s, Statement::Return(_)) {
                        if !seen_return {
                            seen_return = true;
                        } else {
                            return Err(ParseError::Expected(
                                "end of file",
                                self.path.clone(),
                                lex.span(),
                                self.source()?,
                            ));
                        }
                    }
                    statements.push(s)
                }
                Err(error) => return Err(error),
            }
        }

        self.documents.insert(
            String::from(self.path.to_str().unwrap()),
            Document::new(statements),
        );
        Ok(Ast {
            root: self.path,
            documents: self.documents,
        })
    }

    fn parse_statement(&mut self, lexer: &mut Lexer) -> Result<Statement, ParseError> {
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Var) => self.parse_variable_statement(lexer),
            Some(_) => self.parse_return_statement(lexer),
            None => Err(ParseError::UnexpectedEndOfFile(
                self.path.clone(),
                self.source()?,
            )),
        }
    }

    fn parse_return_statement(&mut self, lexer: &mut Lexer) -> Result<Statement, ParseError> {
        let expr = self.parse_expression(lexer)?;
        take!(self, lexer, Token::Semicolon = "semicolon");
        Ok(Statement::Return(expr))
    }

    fn parse_variable_statement(&mut self, lexer: &mut Lexer) -> Result<Statement, ParseError> {
        take!(self, lexer, Token::Var = "var");
        let name =
            take!(self, lexer, Token::Identifier = "identifier" => lexer.slice().to_string());

        if !self.variables.contains(&name) {
            self.variables.insert(name.clone());
        } else {
            return Err(ParseError::DuplicateVariableName(
                self.path.clone(),
                lexer.span(),
                self.source()?,
            ));
        }

        let expr = take!(self, lexer,
            Token::Semicolon = ";" => None,
            Token::Equal = "=" => {
                let expr = self.parse_expression(lexer)?;
                take!(self, lexer, Token::Semicolon = "semicolon");
                Some(expr)
            }
        );
        Ok(Statement::Variable { name, value: expr })
    }

    fn parse_call(&mut self, lexer: &mut Lexer) -> Result<Expression, ParseError> {
        let path = take!(self, lexer,
            Token::Identifier = "identifier" => lexer.slice().to_string(),
            Token::Path = "path" => {
                let path =  lexer.slice();

                let mut buf = std::path::PathBuf::new();
                buf.push(self.path.clone());
                let buf = buf.parent().unwrap();
                let buf = buf.join(path.to_string() + ".ex").canonicalize();
                let buf = match &buf {
                    Ok(buf) => buf.to_str().unwrap(),
                    Err(_) => return Err(ParseError::NoSuchFile(PathBuf::from(path)))
                };

                if !self.documents.contains_key(buf) && self.path.to_str().unwrap() != buf {
                    let ast = Parser::new(buf, self.reader).parse()?;
                    for (path, document) in ast.documents {
                        self.documents.insert(path, document);
                    }
                }

                buf.to_string()
            }
        );

        take!(self, lexer, Token::OpenBracket = "(");

        let mut args = HashMap::new();
        loop {
            let mut peek = lexer.clone();
            take!(self, peek,
                Token::CloseBracket = ")" => {
                    lexer.next();
                    break;
                },
                Token::Identifier = "identifier" => {
                    let (name, expression) = self.parse_argument(lexer)?;
                    args.insert(name, Box::new(expression));
                    take!(self, lexer,
                        Token::Comma = "," => {},
                        Token::CloseBracket = ")" => break
                    );
                }
            )
        }

        Ok(Expression::Invocation {
            path,
            arguments: args,
        })
    }

    fn parse_argument(&mut self, lexer: &mut Lexer) -> Result<(String, Expression), ParseError> {
        let name =
            take!(self, lexer, Token::Identifier = "identifier" => lexer.slice().to_string());
        take!(self, lexer, Token::Equal = "=");
        let expr = self.parse_expression(lexer)?;
        Ok((name, expr))
    }

    fn parse_reference(&self, lexer: &mut Lexer) -> Result<Expression, ParseError> {
        let name =
            take!(self, lexer, Token::Identifier = "identifier" => lexer.slice().to_string());

        if !self.variables.contains(&name) {
            return Err(ParseError::UndeclaredIdentifier(
                self.path.clone(),
                lexer.span(),
                self.source()?,
            ));
        }

        Ok(Expression::Reference(name))
    }

    fn parse_expression(&mut self, lexer: &mut Lexer) -> Result<Expression, ParseError> {
        let mut peek = lexer.clone();
        let unary = match peek.next() {
            Some(Token::Minus) => {
                lexer.next();
                Some(|e| Expression::Invocation {
                    path: String::from("subtract"),
                    arguments: HashMap::from([
                        (
                            "left".to_string(),
                            Box::new(Expression::Literal(Value::Number(0.0))),
                        ),
                        ("right".to_string(), Box::new(e)),
                    ]),
                })
            }
            Some(_) => None,
            None => {
                return Err(ParseError::UnexpectedEndOfFile(
                    self.path.clone(),
                    self.source()?,
                ))
            }
        };

        let mut peek = lexer.clone();
        let first = take!(self, peek,
            Token::Number = "number" => {
                lexer.next();
                let value = f64::from_str(lexer.slice()).unwrap();
                Expression::Literal(Value::Number(value))
            },
            Token::Bool = "boolean" => {
                lexer.next();
                let value = lexer.slice() == "true";
                Expression::Literal(Value::Bool(value))
            },
            Token::String = "string" => {
                lexer.next();
                let value = lexer.slice();
                Expression::Literal(Value::Text(value[1..value.len()-1].to_string()))
            },
            Token::Path = "path" => self.parse_call(lexer)?,
            Token::Identifier = "identifier" => {
                match peek.next() {
                    Some(Token::OpenBracket) => self.parse_call(lexer)?,
                    _ => self.parse_reference(lexer)?,
                }
            },
            Token::OpenBracket = "(" => {
                lexer.next();
                let expr = self.parse_expression(lexer)?;
                take!(self, lexer, Token::CloseBracket = ")" => expr)
            }
        );

        let first = match unary {
            Some(builder) => builder(first),
            None => first,
        };

        self.parse_expression_rhs(first, lexer)
    }

    fn parse_expression_rhs(
        &mut self,
        lhs: Expression,
        lexer: &mut Lexer,
    ) -> Result<Expression, ParseError> {
        let first = lhs;

        macro_rules! op_shorthand {
            ($name: literal, $left: ident, $lexer: ident) => {{
                $lexer.next();
                let l = Box::new($left);
                let r = Box::new(self.parse_expression(lexer)?);
                Ok(Expression::Invocation {
                    path: String::from($name),
                    arguments: HashMap::from([
                        (String::from("left"), l),
                        (String::from("right"), r),
                    ]),
                })
            }};
        }

        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Period) => {
                lexer.next();
                let l = Box::new(first);
                let r = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice().to_string());
                self.parse_expression_rhs(Expression::Access(l, r), lexer)
            }
            Some(Token::Inject) => {
                lexer.next();
                let prop = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice().to_string());

                let expr = self.parse_call(lexer)?;
                match expr {
                    Expression::Invocation {
                        path,
                        mut arguments,
                    } => {
                        arguments.insert(prop, Box::new(first));
                        self.parse_expression_rhs(Expression::Invocation { path, arguments }, lexer)
                    }
                    _ => panic!("parse_call failed to return invocation"),
                }
            }
            Some(Token::Plus) => op_shorthand!("add", first, lexer),
            Some(Token::Minus) => op_shorthand!("subtract", first, lexer),
            Some(Token::Multiply) => op_shorthand!("multiply", first, lexer),
            Some(Token::Divide) => op_shorthand!("divide", first, lexer),
            _ => Ok(first),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn it_can_parse_variable() {
        Parser::new("test", &TestReader("test();"))
            .parse_variable_statement(&mut Token::lexer("var x = 5;"))
            .unwrap();
        Parser::new("test", &TestReader("test();"))
            .parse_variable_statement(&mut Token::lexer("var x;"))
            .unwrap();
        Parser::new("test", &TestReader("test();"))
            .parse_variable_statement(&mut Token::lexer("var x = true;"))
            .unwrap();
    }

    #[test]
    fn it_can_parse_calls() {
        Parser::new("test", &TestReader("test();"))
            .parse_call(&mut Token::lexer("cube()"))
            .unwrap();
        Parser::new("test", &TestReader("test();"))
            .parse_call(&mut Token::lexer("cube(x=5)"))
            .unwrap();
    }

    #[test]
    fn it_can_parse() {
        Parser::new("test", &TestReader("test(x=10,y=10);"))
            .parse()
            .unwrap();
    }

    #[test]
    fn it_can_parse_adds() {
        Parser::new("test", &TestReader("2 + 2;")).parse().unwrap();
        Parser::new("test", &TestReader("var test; test.area + 10;"))
            .parse()
            .unwrap();
    }

    #[test]
    fn it_can_parse_divide() {
        Parser::new("test", &TestReader("var test; test(x=test / 2);"))
            .parse()
            .unwrap();
    }

    #[test]
    fn it_can_parse_unary_minus() {
        Parser::new("test", &TestReader("-2;")).parse().unwrap();
        Parser::new("test", &TestReader("var foo; -foo;"))
            .parse()
            .unwrap();
    }

    macro_rules! parse {
        ($code: literal) => {
            Parser::new("test", &TestReader($code)).parse()
        };
    }

    macro_rules! parse_statement {
        ($code: literal) => {{
            let parsed = Parser::new("test", &TestReader($code)).parse().unwrap();
            let doc = parsed.root_document();
            let statement = doc.statements().next();
            statement.unwrap().clone()
        }};
    }

    #[test]
    fn it_can_parse_inject() {
        Parser::new("test", &TestReader("5 ->value cube();"))
            .parse()
            .unwrap();

        let p = parse_statement!("5 ->value cube() ->test cube();");
        assert!(matches!(p, Statement::Return(
            Expression::Invocation { arguments: x, .. }
        ) if !x.contains_key("value")))
    }

    #[test]
    fn it_rejects_duplicate_returns() {
        parse!("5; 10;").expect_err("expected duplicate return error");
    }

    #[test]
    fn it_rejects_duplicate_variables() {
        parse!("var x; var x;").expect_err("expected duplicate variable error");
    }

    #[test]
    fn it_rejects_undeclared_variables() {
        parse!("x;").expect_err("expected undeclared identifier error");
    }

    #[test]
    fn it_can_parse_brackets() {
        Parser::new("test", &TestReader("3 - (3 + 2);"))
            .parse()
            .unwrap();
        Parser::new("test", &TestReader("(3 - 3) + 2;"))
            .parse()
            .unwrap();
    }

    #[test]
    fn it_can_parse_access() {
        Parser::new("test", &TestReader("var foo; foo.bar;"))
            .parse()
            .unwrap();
    }

    pub struct TestReader<'a>(pub &'a str);
    impl<'a> Reader for TestReader<'a> {
        fn read(&self, _: &Path) -> Result<String, std::io::Error> {
            Ok(self.0.to_string())
        }

        fn normalize(&self, path: &Path) -> PathBuf {
            PathBuf::from(path)
        }
    }
}
