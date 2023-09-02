mod lexer;
mod parse_error;
mod reader;
mod span_builder;
mod syntax_tree;

use lexer::{Lexer, Token};
use logos::Logos;

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use crate::parser::span_builder::SpanBuilder;
pub use parse_error::{DocumentParseError, ParseError};
pub use reader::Reader;
pub use syntax_tree::*;

pub struct Parser<R> {
    reader: R,
    current_id: DocId,
    variables: HashSet<String>,
    to_parse: Vec<DocId>,
}

macro_rules! take {
    ($self: ident, $lexer: ident, $token: pat = $name: literal) => {
        match $lexer.next() {
            Some($token) => {},
            Some(_) => return Err(DocumentParseError::Expected($name, $lexer.span())),
            None => return Err(DocumentParseError::UnexpectedEndOfFile()),
        };
    };
    ($self: ident, $lexer: ident, $($token: pat = $name: literal => $case: expr), *) => {
        match $lexer.next() {
            $(Some($token) => $case,)*
            #[allow(unreachable_patterns)]
            Some(_) => return Err(DocumentParseError::ExpectedOneOf(vec![$($name,)*], $lexer.span())),
            None => return Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    };
}

impl<R: Reader> Parser<R> {
    pub fn new(reader: R, root: DocId) -> Self {
        Parser {
            reader,
            current_id: root,
            variables: HashSet::new(),
            to_parse: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Result<Ast, ParseError> {
        self.to_parse.push(self.current_id.clone());
        let mut ast = Ast::new(self.current_id.clone());

        while let Some(doc) = self.to_parse.pop() {
            if ast.documents.contains_key(&doc) {
                continue;
            }

            self.current_id = doc.clone();
            self.variables.clear();
            let source = self
                .reader
                .read(doc.to_path())
                .map_err(|_| ParseError::NoSuchFile { file: doc.clone() })?;
            let document = self
                .parse_document(&source)
                .map_err(|e| e.with_source(doc.clone(), source))?;
            ast.documents.insert(doc, document);
        }

        Ok(ast)
    }

    fn parse_document(&mut self, source: &str) -> Result<Vec<Statement>, DocumentParseError> {
        let mut lex = Token::lexer(source);

        let mut statements = Vec::new();
        while let Some(_) = lex.clone().next() {
            let statement = self.parse_statement(&mut lex)?;
            statements.push(statement);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self, lexer: &mut Lexer) -> Result<Statement, DocumentParseError> {
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Var) => self.parse_variable_statement(lexer),
            Some(_) => self.parse_return_statement(lexer),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_return_statement(
        &mut self,
        lexer: &mut Lexer,
    ) -> Result<Statement, DocumentParseError> {
        let expr = self.parse_expression(lexer)?;
        let sb = SpanBuilder::from_expr(&expr);
        take!(self, lexer, Token::Semicolon = "semicolon");
        Ok(Statement::Return(expr, sb.to(lexer)))
    }

    fn parse_variable_statement(
        &mut self,
        lexer: &mut Lexer,
    ) -> Result<Statement, DocumentParseError> {
        take!(self, lexer, Token::Var = "var");
        let sb = SpanBuilder::from(lexer);
        let name = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());

        if !self.variables.contains(name) {
            self.variables.insert(name.to_string());
        } else {
            return Err(DocumentParseError::DuplicateVariableName(lexer.span()));
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
            name: name.to_string(),
            value: expr,
            span: sb.to(lexer),
        })
    }

    fn parse_call(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let path = take!(self, lexer,
            Token::Identifier = "identifier" => CallPath::String(lexer.slice().to_string()),
            Token::Path = "path" => {
                let path =  lexer.slice();

                let mut buf = std::path::PathBuf::new();
                buf.push(self.current_id.to_path());
                let buf = buf.parent().unwrap();
                let buf = buf.join(path.to_string() + ".ds");

                let id = DocId::new(buf.to_str().unwrap().to_string());
                self.to_parse.push(id.clone());
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
                    args.insert(name.to_string(), Box::new(expression));
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
        lexer: &mut Lexer,
    ) -> Result<(String, Expression), DocumentParseError> {
        let name = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
        take!(self, lexer, Token::Equal = "=");
        let expr = self.parse_expression(lexer)?;
        Ok((name.to_string(), expr))
    }

    fn parse_reference(&self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let name = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
        let sb = SpanBuilder::from(lexer);

        if !self.variables.contains(name) {
            return Err(DocumentParseError::UndeclaredIdentifier(lexer.span()));
        }

        Ok(Expression::Reference(name.to_string(), sb.to(lexer)))
    }

    fn parse_list(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
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

    fn parse_map(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        take!(self, lexer, Token::Map = "map");
        let sb = SpanBuilder::from(lexer);

        let range = self.parse_expression(lexer)?;
        take!(self, lexer, Token::As = "as");
        let ident = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
        take!(self, lexer, Token::Colon = ":");

        self.variables.insert(ident.to_string());
        let action = self.parse_expression(lexer)?;
        self.variables.remove(ident);

        Ok(Expression::Map {
            identifier: ident.to_string(),
            range: Box::new(range),
            action: Box::new(action),
            span: sb.to(lexer),
        })
    }

    fn parse_reduce(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
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

        self.variables.insert(left.to_string());
        self.variables.insert(right.to_string());
        let action = self.parse_expression(lexer)?;
        self.variables.remove(left);
        self.variables.remove(right);

        Ok(Expression::Reduce {
            left: left.to_string(),
            right: right.to_string(),
            root,
            range: Box::new(range),
            action: Box::new(action),
            span: sb.to(lexer),
        })
    }

    fn parse_if(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
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

    fn operator(
        &mut self,
        lexer: &mut Lexer,
        name: &'static str,
        left: Expression,
        parse_right: impl Fn(&mut Self, &mut Lexer) -> Result<Expression, DocumentParseError>,
    ) -> Result<Expression, DocumentParseError> {
        lexer.next();
        let sb = SpanBuilder::from(lexer);
        let right = parse_right(self, lexer)?;
        Ok(Expression::Invocation {
            path: CallPath::String(name.to_string()),
            arguments: HashMap::from([
                ("left".to_string(), left.into()),
                ("right".to_string(), right.into()),
            ]),
            span: sb.to(lexer),
        })
    }

    fn parse_expression(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        self.parse_or(lexer)
    }

    fn parse_or(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_and(lexer)?;
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Or) => self.operator(lexer, "or", first, |s, l| s.parse_or(l)),
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_and(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_equality(lexer)?;
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::And) => self.operator(lexer, "and", first, |s, l| s.parse_and(l)),
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_equality(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_comparison(lexer)?;
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Equals) => {
                self.operator(lexer, "equals", first, |s, l| s.parse_equality(l))
            }
            Some(Token::NotEquals) => {
                self.operator(lexer, "not_equals", first, |s, l| s.parse_equality(l))
            }
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_comparison(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_add_sub(lexer)?;
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Less) => self.operator(lexer, "less", first, |s, l| s.parse_comparison(l)),
            Some(Token::LessEquals) => {
                self.operator(lexer, "less_or_equal", first, |s, l| s.parse_comparison(l))
            }
            Some(Token::Greater) => {
                self.operator(lexer, "greater", first, |s, l| s.parse_comparison(l))
            }
            Some(Token::GreaterEquals) => {
                self.operator(lexer, "greater_or_equal", first, |s, l| {
                    s.parse_comparison(l)
                })
            }
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_add_sub(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_mul_div_mod(lexer)?;
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Plus) => self.operator(lexer, "add", first, |s, l| s.parse_add_sub(l)),
            Some(Token::Minus) => {
                self.operator(lexer, "subtract", first, |s, l| s.parse_add_sub(l))
            }
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_mul_div_mod(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_pow(lexer)?;
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Divide) => {
                self.operator(lexer, "divide", first, |s, l| s.parse_mul_div_mod(l))
            }
            Some(Token::Multiply) => {
                self.operator(lexer, "multiply", first, |s, l| s.parse_mul_div_mod(l))
            }
            Some(Token::Modulo) => {
                self.operator(lexer, "modulo", first, |s, l| s.parse_mul_div_mod(l))
            }
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_pow(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_inject(lexer)?;
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Power) => self.operator(lexer, "power", first, |s, l| s.parse_pow(l)),
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_inject(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_spanning(lexer)?;
        self.try_add_inject(lexer, first)
    }

    fn try_add_inject(
        &mut self,
        lexer: &mut Lexer,
        first: Expression,
    ) -> Result<Expression, DocumentParseError> {
        let mut peek = lexer.clone();
        match peek.next() {
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
                        arguments.insert(prop.to_string(), Box::new(first));
                        self.try_add_inject(
                            lexer,
                            Expression::Invocation {
                                path,
                                arguments,
                                span: first_span.start..sb.to(lexer).end,
                            },
                        )
                    }
                    _ => panic!("parse_call failed to return invocation"),
                }
            }
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_spanning(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let first = self.parse_terminal_expression(lexer)?;
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Period) => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let l = Box::new(first);
                let r = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
                Ok(Expression::Access(l, r.to_string(), sb.to(lexer)))
            }
            Some(Token::OpenList) => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let r = self.parse_expression(lexer)?;
                take!(self, lexer, Token::CloseList = "]");
                Ok(Expression::Index {
                    target: first.into(),
                    index: r.into(),
                    span: sb.to(lexer),
                })
            }
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_terminal_expression(
        &mut self,
        lexer: &mut Lexer,
    ) -> Result<Expression, DocumentParseError> {
        let mut peek = lexer.clone();
        Ok(take!(self, peek,
            Token::Minus = "-" => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let expr = self.parse_terminal_expression(lexer)?;
                let span = sb.to(lexer);
                Expression::Invocation {
                    path: CallPath::String("subtract".to_string()),
                    arguments: HashMap::from([
                        (
                            "left".to_string(),
                            Box::new(Expression::Literal(Literal::Number(0.0), span.clone())),
                        ),
                        ("right".to_string(), Box::new(expr)),
                    ]),
                    span
                }
            },
            Token::Not = "not" => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let expr = self.parse_terminal_expression(lexer)?;
                let span = sb.to(lexer);
                Expression::Invocation {
                    path: CallPath::String("not".to_string()),
                    arguments: HashMap::from([
                         ("value".to_string(), Box::new(expr)),
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

    fn parse(code: &'static str, action: impl for<'a> FnOnce(Result<Ast, ParseError>)) {
        let res = Parser::new(TestReader(code), DocId::new("test".to_string())).parse();
        action(res)
    }

    fn parse_statement(code: &'static str, action: impl for<'a> FnOnce(&Statement)) {
        let root_id = DocId::new("test".to_string());
        let parsed = Parser::new(TestReader(code), root_id).parse().unwrap();
        let doc = parsed.root_document();
        let statement = doc.iter().next();
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

        parse_statement("5 ->value a() ->test b();", |p| {
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
