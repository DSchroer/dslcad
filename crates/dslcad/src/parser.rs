mod document;
mod lexer;
mod parse_error;
mod reader;
mod source_store;
mod span_builder;
mod syntax_tree;

use lexer::{Lexer, Token};
use logos::Logos;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;

use crate::parser::span_builder::SpanBuilder;
pub use document::Document;
pub use parse_error::ParseError;
pub use reader::Reader;
pub use source_store::{DocId, SourceStore};
pub use syntax_tree::{CallPath, Expression, Literal, Statement};

pub struct Parser<'a> {
    store: &'a SourceStore,
    path: &'a DocId,
    documents: HashMap<&'a DocId, Document<'a>>,
    variables: HashSet<&'a str>,
}

#[derive(Debug)]
pub struct Ast<'a> {
    root: &'a DocId,
    documents: HashMap<&'a DocId, Document<'a>>,
}

impl<'a> Ast<'a> {
    pub fn root_document(&self) -> &Document {
        self.documents.get(&self.root).unwrap()
    }

    pub fn documents(&self) -> &HashMap<&'a DocId, Document> {
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
            #[allow(unreachable_patterns)]
            Some(_) => return Err(ParseError::ExpectedOneOf(vec![$($name,)*], $self.path.clone(), $lexer.span(), $self.source()?)),
            None => return Err(ParseError::UnexpectedEndOfFile($self.path.clone(), $self.source()?)),
        }
    };
}

impl<'a> Parser<'a> {
    pub fn new(path: &'a DocId, store: &'a SourceStore) -> Self {
        Parser {
            store,
            path,
            documents: HashMap::new(),
            variables: HashSet::new(),
        }
    }

    fn source(&self) -> Result<String, ParseError> {
        Ok(self.store.read(self.path).to_string())
    }

    pub fn parse(mut self) -> Result<Ast<'a>, ParseError> {
        let mut lex = Token::lexer(self.store.read(self.path));

        let mut statements = Vec::new();
        while let Some(_) = lex.clone().next() {
            let statement = self.parse_statement(&mut lex)?;
            statements.push(statement);
        }

        self.documents.insert(
            self.path,
            Document::new(
                self.path,
                self.store.read(self.path),
                self.variables,
                statements,
            ),
        );
        Ok(Ast {
            root: self.path,
            documents: self.documents,
        })
    }

    fn parse_statement(&mut self, lexer: &mut Lexer<'a>) -> Result<Statement<'a>, ParseError> {
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

    fn parse_return_statement(
        &mut self,
        lexer: &mut Lexer<'a>,
    ) -> Result<Statement<'a>, ParseError> {
        let expr = self.parse_expression(lexer)?;
        let sb = SpanBuilder::from_expr(&expr);
        take!(self, lexer, Token::Semicolon = "semicolon");
        Ok(Statement::Return(expr, sb.to(lexer)))
    }

    fn parse_variable_statement(
        &mut self,
        lexer: &mut Lexer<'a>,
    ) -> Result<Statement<'a>, ParseError> {
        take!(self, lexer, Token::Var = "var");
        let sb = SpanBuilder::from(lexer);
        let name = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());

        if !self.variables.contains(name) {
            self.variables.insert(name);
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
        Ok(Statement::Variable {
            name,
            value: expr,
            span: sb.to(lexer),
        })
    }

    fn parse_call(&mut self, lexer: &mut Lexer<'a>) -> Result<Expression<'a>, ParseError> {
        let path = take!(self, lexer,
            Token::Identifier = "identifier" => CallPath::String(lexer.slice()),
            Token::Path = "path" => {
                let path =  lexer.slice();

                let mut buf = std::path::PathBuf::new();
                buf.push(self.path.to_path());
                let buf = buf.parent().unwrap();
                let buf = buf.join(path.to_string() + "." + crate::constants::FILE_EXTENSION).canonicalize();
                let buf = match &buf {
                    Ok(buf) => buf.to_str().unwrap(),
                    Err(_) => return Err(ParseError::NoSuchFile(PathBuf::from(path)))
                };

                let id = self.store.forge_id(buf.to_string())?;
                if !self.documents.contains_key(&id) && self.path.to_str() != buf {
                    let ast = Parser::new(id, self.store).parse()?;
                    for (path, document) in ast.documents {
                        self.documents.insert(path, document);
                    }
                }
                CallPath::Document(id)
            }
        );
        let sb = SpanBuilder::from(lexer);

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
            span: sb.to(lexer),
        })
    }

    fn parse_argument(
        &mut self,
        lexer: &mut Lexer<'a>,
    ) -> Result<(&'a str, Expression<'a>), ParseError> {
        let name = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
        take!(self, lexer, Token::Equal = "=");
        let expr = self.parse_expression(lexer)?;
        Ok((name, expr))
    }

    fn parse_reference(&self, lexer: &mut Lexer<'a>) -> Result<Expression<'a>, ParseError> {
        let name = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
        let sb = SpanBuilder::from(lexer);

        if !self.variables.contains(name) {
            return Err(ParseError::UndeclaredIdentifier(
                self.path.clone(),
                lexer.span(),
                self.source()?,
            ));
        }

        Ok(Expression::Reference(name, sb.to(lexer)))
    }

    fn parse_list(&mut self, lexer: &mut Lexer<'a>) -> Result<Expression<'a>, ParseError> {
        take!(self, lexer, Token::OpenList = "[");
        let sb = SpanBuilder::from(lexer);

        let mut items = Vec::new();
        loop {
            let mut peek = lexer.clone();
            take!(self, peek,
                Token::CloseList = "]" => {
                    lexer.next();
                    break;
                },
                _ = "expression" =>  items.push(self.parse_expression(lexer)?)
            );

            take!(self, lexer,
                Token::CloseList = "]" => break,
                Token::Comma = "," => {}
            );
        }

        Ok(Expression::Literal(Literal::List(items), sb.to(lexer)))
    }

    fn parse_map(&mut self, lexer: &mut Lexer<'a>) -> Result<Expression<'a>, ParseError> {
        take!(self, lexer, Token::Map = "map");
        let sb = SpanBuilder::from(lexer);

        let range = self.parse_expression(lexer)?;
        take!(self, lexer, Token::As = "as");
        let ident = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
        take!(self, lexer, Token::Colon = ":");

        self.variables.insert(ident);
        let action = self.parse_expression(lexer)?;
        self.variables.remove(ident);

        Ok(Expression::Map {
            identifier: ident,
            range: Box::new(range),
            action: Box::new(action),
            span: sb.to(lexer),
        })
    }

    fn parse_reduce(&mut self, lexer: &mut Lexer<'a>) -> Result<Expression<'a>, ParseError> {
        take!(self, lexer, Token::Reduce = "reduce");
        let sb = SpanBuilder::from(lexer);

        let range = self.parse_expression(lexer)?;
        let root = take!(self, lexer,
            Token::As = "as" => None,
            Token::From = "from" => {
                let root = self.parse_expression(lexer)?;
                take!(self, lexer, Token::As = "as");
                Some(Box::new(root))
            }
        );
        let left = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
        take!(self, lexer, Token::Comma = ",");
        let right = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
        take!(self, lexer, Token::Colon = ":");

        self.variables.insert(left);
        self.variables.insert(right);
        let action = self.parse_expression(lexer)?;
        self.variables.remove(left);
        self.variables.remove(right);

        Ok(Expression::Reduce {
            left,
            right,
            root,
            range: Box::new(range),
            action: Box::new(action),
            span: sb.to(lexer),
        })
    }

    fn parse_if(&mut self, lexer: &mut Lexer<'a>) -> Result<Expression<'a>, ParseError> {
        take!(self, lexer, Token::If = "if");
        let sb = SpanBuilder::from(lexer);

        let condition = self.parse_expression(lexer)?;
        take!(self, lexer, Token::Colon = ":");
        let if_true = self.parse_expression(lexer)?;
        take!(self, lexer, Token::Else = "else");

        let mut peek = lexer.clone();
        let if_false = take!(self, peek,
            Token::Colon = ":" => {
                lexer.next();
                self.parse_expression(lexer)?
            },
            Token::If = "if" => self.parse_if(lexer)?
        );

        Ok(Expression::If {
            condition: condition.into(),
            if_true: if_true.into(),
            if_false: if_false.into(),
            span: sb.to(lexer),
        })
    }

    fn parse_expression(&mut self, lexer: &mut Lexer<'a>) -> Result<Expression<'a>, ParseError> {
        let first = self.parse_expression_lhs(lexer)?;
        self.parse_expression_rhs(first, lexer)
    }

    fn parse_expression_lhs(
        &mut self,
        lexer: &mut Lexer<'a>,
    ) -> Result<Expression<'a>, ParseError> {
        let mut peek = lexer.clone();
        Ok(take!(self, peek,
            Token::Minus = "-" => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let expr = self.parse_expression_lhs(lexer)?;
                let span = sb.to(lexer);
                Expression::Invocation {
                    path: CallPath::String("subtract"),
                    arguments: HashMap::from([
                        (
                            "left",
                            Box::new(Expression::Literal(Literal::Number(0.0), span.clone())),
                        ),
                        ("right", Box::new(expr)),
                    ]),
                    span
                }
            },
            Token::Not = "not" => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let expr = self.parse_expression_lhs(lexer)?;
                let span = sb.to(lexer);
                Expression::Invocation {
                    path: CallPath::String("not"),
                    arguments: HashMap::from([
                         ("value", Box::new(expr)),
                    ]),
                    span
                }
            },
            Token::Number = "number" => {
                lexer.next();
                let value = f64::from_str(lexer.slice()).unwrap();
                Expression::Literal(Literal::Number(value), lexer.span())
            },
            Token::Bool = "boolean" => {
                lexer.next();
                let value = lexer.slice() == "true";
                Expression::Literal(Literal::Bool(value), lexer.span())
            },
            Token::String = "string" => {
                lexer.next();
                let value = lexer.slice();
                Expression::Literal(Literal::Text(escape_string(value)), lexer.span())
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
            },
            Token::OpenList = "[" => {
                self.parse_list(lexer)?
            },
            Token::Map = "map" => {
                self.parse_map(lexer)?
            },
            Token::Reduce = "reduce" => {
                self.parse_reduce(lexer)?
            },
            Token::If = "if" => {
                self.parse_if(lexer)?
            }
        ))
    }

    fn parse_expression_rhs(
        &mut self,
        lhs: Expression<'a>,
        lexer: &mut Lexer<'a>,
    ) -> Result<Expression<'a>, ParseError> {
        let first = lhs;

        macro_rules! op_shorthand {
            ($name: literal, $left: ident, $lexer: ident) => {{
                $lexer.next();
                let sb = SpanBuilder::from($lexer);
                let l = Box::new($left);
                let r = Box::new(self.parse_expression(lexer)?);
                Ok(Expression::Invocation {
                    path: CallPath::String($name),
                    arguments: HashMap::from([("left", l), ("right", r)]),
                    span: sb.to($lexer),
                })
            }};
        }

        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Period) => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let l = Box::new(first);
                let r = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
                self.parse_expression_rhs(Expression::Access(l, r, sb.to(lexer)), lexer)
            }
            Some(Token::OpenList) => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let r = self.parse_expression(lexer)?;
                take!(self, lexer, Token::CloseList = "]");
                self.parse_expression_rhs(
                    Expression::Index {
                        target: first.into(),
                        index: r.into(),
                        span: sb.to(lexer),
                    },
                    lexer,
                )
            }
            Some(Token::Inject) => {
                lexer.next();
                let first_span = first.span().clone();
                let sb = SpanBuilder::from(lexer);
                let prop = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());

                let expr = self.parse_call(lexer)?;
                match expr {
                    Expression::Invocation {
                        path,
                        mut arguments,
                        ..
                    } => {
                        arguments.insert(prop, Box::new(first));
                        self.parse_expression_rhs(
                            Expression::Invocation {
                                path,
                                arguments,
                                span: first_span.start..sb.to(lexer).end,
                            },
                            lexer,
                        )
                    }
                    _ => panic!("parse_call failed to return invocation"),
                }
            }
            Some(Token::Plus) => op_shorthand!("add", first, lexer),
            Some(Token::Minus) => op_shorthand!("subtract", first, lexer),
            Some(Token::Multiply) => op_shorthand!("multiply", first, lexer),
            Some(Token::Divide) => op_shorthand!("divide", first, lexer),
            Some(Token::Modulo) => op_shorthand!("modulo", first, lexer),
            Some(Token::Power) => op_shorthand!("power", first, lexer),
            Some(Token::Less) => op_shorthand!("less", first, lexer),
            Some(Token::LessEquals) => op_shorthand!("less_or_equal", first, lexer),
            Some(Token::Equals) => op_shorthand!("equals", first, lexer),
            Some(Token::NotEquals) => op_shorthand!("not_equals", first, lexer),
            Some(Token::Greater) => op_shorthand!("greater", first, lexer),
            Some(Token::GreaterEquals) => op_shorthand!("greater_or_equal", first, lexer),
            Some(Token::And) => op_shorthand!("and", first, lexer),
            Some(Token::Or) => op_shorthand!("or", first, lexer),
            _ => Ok(first),
        }
    }
}

fn escape_string(input: &str) -> String {
    let source = input[1..input.len() - 1].to_string();
    source
        .replace(r"\r", "\r")
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
        .replace("\\\"", "\"")
        .replace(r"\\", r"\")
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    macro_rules! parse {
        ($code: literal) => {{}};
    }

    fn parse(code: &'static str, action: impl for<'a> FnOnce(Result<Ast<'a>, ParseError>)) {
        let store = SourceStore::new(Box::new(TestReader(code)));
        let res = Parser::new(store.forge_id("test".to_string()).unwrap(), &store).parse();
        action(res)
    }

    fn parse_statement(code: &'static str, action: impl for<'a> FnOnce(&Statement<'a>)) {
        let store = SourceStore::new(Box::new(TestReader(code)));
        let root_id = store.forge_id("test".to_string()).unwrap();
        let parsed = Parser::new(root_id, &store).parse().unwrap();
        let doc = parsed.root_document();
        let statement = doc.statements().next();
        action(statement.unwrap());
    }

    #[test]
    fn it_escapes_strings() {
        assert_eq!("test", escape_string("\"test\""));
        assert_eq!("te\tst", escape_string("\"te\\tst\""));
        assert_eq!("te\nst", escape_string("\"te\\nst\""));
        assert_eq!("te\rst", escape_string("\"te\\rst\""));
        assert_eq!("te\\st", escape_string("\"te\\\\st\""));
        assert_eq!("te\"st", escape_string("\"te\\\"st\""));
    }

    #[test]
    fn it_can_parse_variable() {
        parse!("var x = 5;");
        parse!("var x;");
        parse!("var x = true;");
    }

    #[test]
    fn it_can_parse_calls() {
        parse("cube();", |a| {
            a.unwrap();
        });
        parse("cube(x=5);", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse() {
        parse("test(x=10,y=10);", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_adds() {
        parse("2 + 2;", |a| {
            a.unwrap();
        });
        parse("var test; test.area + 10;", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_divide() {
        parse("var test; test(x=test / 2);", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_unary_minus() {
        parse("-2;", |a| {
            a.unwrap();
        });
        parse("var foo; -foo;", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_unary_not() {
        parse_statement("not true;", |_| {});
        parse_statement("not not true;", |_| {});
    }

    #[test]
    fn it_can_parse_inject() {
        parse("5 ->value cube();", |a| {
            a.unwrap();
        });

        parse_statement("5 ->value cube() ->test cube();", |p| {
            assert!(matches!(p, Statement::Return(
            Expression::Invocation { arguments: x, .. }
            , ..
        ) if !x.contains_key("value")))
        });
    }

    #[test]
    fn it_can_load_inject_spans() {
        parse_statement("5 ->value cube();", |p| {
            if let Statement::Return(expr, ..) = p {
                assert_eq!(0..16, *expr.span())
            } else {
                unreachable!();
            }
        });
    }

    #[test]
    fn it_allows_duplicate_returns() {
        parse("5; 10;", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_rejects_duplicate_variables() {
        parse("var x; var x;", |a| {
            a.expect_err("expected duplicate variable error");
        });
    }

    #[test]
    fn it_rejects_undeclared_variables() {
        parse("x;", |a| {
            a.expect_err("expected undeclared identifier error");
        });
    }

    #[test]
    fn it_can_parse_brackets() {
        parse("3 - (3 + 2);", |a| {
            a.unwrap();
        });
        parse("(3 - 3) + 2;", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_access() {
        parse("var foo; foo.bar;", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_index() {
        parse("var foo; foo[0];", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_map() {
        parse("var foo = map [] as x: x;", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_reduce() {
        parse("var foo = reduce [] as a,b: a;", |a| {
            a.unwrap();
        });
        parse("var foo = reduce [] from t() as a,b: a;", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_if() {
        parse("var foo = if true: 1 else: 0;", |a| {
            a.unwrap();
        });
        parse("var foo = if true: 1 else if false: 2 else: 3;", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_list_literal() {
        parse("var foo = [];", |a| {
            a.unwrap();
        });
        parse("var foo = [1];", |a| {
            a.unwrap();
        });
        parse("var foo = [1 2];", |a| {
            a.expect_err("should not parse lists without commas");
        });
        parse("var foo = [test(), 2];", |a| {
            a.unwrap();
        });
    }

    pub struct TestReader(pub &'static str);
    impl Reader for TestReader {
        fn read(&self, _: &Path) -> Result<String, std::io::Error> {
            Ok(self.0.to_string())
        }

        fn normalize(&self, path: &Path) -> PathBuf {
            PathBuf::from(path)
        }
    }
}
