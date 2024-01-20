mod lexer;
mod parse_error;
mod reader;
mod span_builder;
mod syntax_tree;
mod syntax_visitor;

use lexer::{Lexer, Token};
use logos::Logos;

use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

use crate::library::Library;
use crate::parser::span_builder::SpanBuilder;
use crate::parser::Literal::Resource;
use crate::resources::ResourceLoader;
pub use parse_error::{DocumentParseError, ParseError};
pub use reader::Reader;
pub use syntax_tree::*;
pub use syntax_visitor::*;

pub struct Parser<R> {
    reader: R,
    current_id: DocId,
    variables: HashSet<String>,
    to_parse: Vec<DocId>,
    resource_loaders: HashMap<&'static str, Box<dyn ResourceLoader<R>>>,
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

impl<T> Parser<T> {
    pub fn new(reader: T, root: DocId) -> Self {
        Parser {
            reader,
            current_id: root,
            variables: HashSet::new(),
            to_parse: Vec::new(),
            resource_loaders: HashMap::new(),
        }
    }

    pub fn parse_arguments<'a>(
        self,
        arguments: impl Iterator<Item = &'a str>,
    ) -> Result<HashMap<&'a str, Literal>, DocumentParseError> {
        let mut ret = HashMap::new();
        for argument in arguments {
            let mut lexer = Token::lexer(argument);

            let name = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
            take!(self, lexer, Token::Equal = "=");
            let value = take!(self, lexer,
                Token::Minus = "-" => {
                    let number = take!(self, lexer, Token::Number = "number" => lexer.slice());
                    let value = f64::from_str(number).unwrap();
                    Literal::Number(-value)
                },
                Token::Number = "number" => {
                    let value = f64::from_str(lexer.slice()).unwrap();
                    Literal::Number(value)
                },
                Token::Bool = "boolean" => {
                    let value = lexer.slice() == "true";
                    Literal::Bool(value)
                },
                Token::String = "string" => {
                    let value = lexer.slice();
                    Literal::Text(escape_string(value))
                }
            );
            ret.insert(name, value);
        }
        Ok(ret)
    }
}

impl<R: Reader> Parser<R> {
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
            let mut lexer = Token::lexer(&source);
            let document = self
                .parse_document(&mut lexer, None, true)
                .map_err(|e| e.with_source(doc.clone(), source))?;
            ast.documents.insert(doc, document);
        }

        Ok(ast)
    }

    pub fn with_loader(
        mut self,
        ext: &'static str,
        loader: impl ResourceLoader<R> + 'static,
    ) -> Self {
        self.resource_loaders.insert(ext, Box::new(loader));
        self
    }

    fn parse_document(
        &mut self,
        lexer: &mut Lexer,
        terminal: Option<Token>,
        allow_parameters: bool,
    ) -> Result<Vec<Statement>, DocumentParseError> {
        let mut statements = Vec::new();
        while let Some(n) = lexer.clone().next() {
            if Some(n) == terminal {
                break;
            }

            let statement = self.parse_statement(lexer, allow_parameters)?;
            statements.push(statement);
        }

        Ok(statements)
    }

    fn parse_statement(
        &mut self,
        lexer: &mut Lexer,
        allow_parameters: bool,
    ) -> Result<Statement, DocumentParseError> {
        let mut peek = lexer.clone();
        match peek.next() {
            Some(Token::Var) => self.parse_variable_statement(lexer, allow_parameters),
            Some(_) => self.parse_return_statement(lexer),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_return_statement(
        &mut self,
        lexer: &mut Lexer,
    ) -> Result<Statement, DocumentParseError> {
        let sb = SpanBuilder::from(lexer);
        let expr = self.parse_expression(lexer)?;
        take!(self, lexer, Token::Semicolon = "semicolon");
        Ok(Statement::CreatePart(expr, sb.to(lexer)))
    }

    fn parse_variable_statement(
        &mut self,
        lexer: &mut Lexer,
        allow_parameters: bool,
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

        if !allow_parameters && expr.is_none() {
            return Err(DocumentParseError::ParametersNotAllowedInScopes(
                sb.to(lexer),
            ));
        }

        Ok(Statement::Variable(
            Variable {
                name: name.to_string(),
                value: expr,
            },
            sb.to(lexer),
        ))
    }

    fn parse_call(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let path = take!(self, lexer,
            Token::Path = "path" => {
                let path =  lexer.slice();

                let mut buf = std::path::PathBuf::new();
                buf.push(self.current_id.to_path());
                let buf = buf.parent().unwrap();
                let buf = buf.join(path);

                match buf.extension().unwrap_or(OsStr::new("ds")).to_str().unwrap() {
                    "ds" => {
                        let id = DocId::new(self.reader.normalize(Path::new(&buf.with_extension("ds"))).to_str().unwrap().to_string());
                        self.to_parse.push(id.clone());
                        CallPath::Document(id)
                    },
                    extension => {
                        if let Some(loader) = self.resource_loaders.get(extension) {
                            take!(self, lexer, Token::OpenBracket = "(");
                            take!(self, lexer, Token::CloseBracket = ")");

                            return Ok(Expression::Literal(Resource(loader.load(buf.to_str().unwrap(), &self.reader)?), lexer.span()))
                        } else {
                            return Err(DocumentParseError::UnknownResourceType(extension.to_owned(), lexer.span()))
                        }
                    }
                }
            }
        );
        let sb = SpanBuilder::from(lexer);

        let args = self.parse_call_arguments(lexer)?;

        Ok(Expression::Invocation(
            Invocation {
                path,
                arguments: args,
            },
            sb.to(lexer),
        ))
    }

    fn parse_call_arguments(
        &mut self,
        lexer: &mut Lexer,
    ) -> Result<VecDeque<Argument>, DocumentParseError> {
        take!(self, lexer, Token::OpenBracket = "(");

        let mut args = VecDeque::new();
        loop {
            let mut peek = lexer.clone();
            take!(self, peek,
                Token::CloseBracket = ")" => {
                    lexer.next();
                    break;
                },
                _ = "expression" => {

                    let mut arg_lexer = lexer.clone();
                    if let Ok((name, expression)) = self.parse_argument(&mut arg_lexer) {
                        *lexer = arg_lexer;
                        args.push_back(Argument::Named(name, Box::new(expression)));
                    } else {
                        let mut expr_lexer = lexer.clone();
                        let expression = self.parse_expression(&mut expr_lexer)?;
                        *lexer = expr_lexer;
                        args.push_back(Argument::Unnamed(Box::new(expression)));
                    }

                     take!(self, lexer,
                        Token::Comma = "," => {},
                        Token::CloseBracket = ")" => break
                    );
                }
            )
        }
        Ok(args)
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

        if !self.variables.contains(name) && !Library::default().contains(name) {
            return Err(DocumentParseError::UndeclaredIdentifier(lexer.span()));
        }

        Ok(Expression::Reference(
            Reference {
                name: name.to_string(),
            },
            sb.to(lexer),
        ))
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

        Ok(Expression::Map(
            Map {
                identifier: ident.to_string(),
                range: Box::new(range),
                action: Box::new(action),
            },
            sb.to(lexer),
        ))
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

        Ok(Expression::Reduce(
            Reduce {
                left: left.to_string(),
                right: right.to_string(),
                root,
                range: Box::new(range),
                action: Box::new(action),
            },
            sb.to(lexer),
        ))
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

        Ok(Expression::If(
            If {
                condition: condition.into(),
                if_true: if_true.into(),
                if_false: if_false.into(),
            },
            sb.to(lexer),
        ))
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
        Ok(Expression::Invocation(
            Invocation {
                path: CallPath::Function(
                    Expression::Reference(
                        Reference {
                            name: name.to_string(),
                        },
                        0..0,
                    )
                    .into(),
                ),
                arguments: vec![
                    Argument::Named("left".into(), left.into()),
                    Argument::Named("right".into(), right.into()),
                ]
                .into(),
            },
            sb.to(lexer),
        ))
    }

    fn parse_scope(
        &mut self,
        lexer: &mut Lexer,
        allow_parameters: bool,
    ) -> Result<Vec<Statement>, DocumentParseError> {
        take!(self, lexer, Token::OpenScope = "{");

        let outer = self.variables.clone();
        let statements = self.parse_document(lexer, Some(Token::CloseScope), allow_parameters)?;
        self.variables = outer;

        take!(self, lexer, Token::CloseScope = "}");
        Ok(statements)
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

                let name = &lexer.slice()[2..];
                let arg_name = if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                };

                let expr = self.parse_spanning(lexer)?;
                match expr {
                    Expression::Invocation(
                        Invocation {
                            path,
                            mut arguments,
                            ..
                        },
                        _,
                    ) => {
                        arguments.push_front(if let Some(name) = arg_name {
                            Argument::Named(name, Box::new(first))
                        } else {
                            Argument::Unnamed(Box::new(first))
                        });

                        self.try_add_inject(
                            lexer,
                            Expression::Invocation(
                                Invocation { path, arguments },
                                first_span.start..sb.to(lexer).end,
                            ),
                        )
                    }
                    _ => Err(DocumentParseError::ExpectedOneOf(
                        vec!["function"],
                        sb.to(lexer),
                    )),
                }
            }
            Some(_) => Ok(first),
            None => Err(DocumentParseError::UnexpectedEndOfFile()),
        }
    }

    fn parse_spanning(&mut self, lexer: &mut Lexer) -> Result<Expression, DocumentParseError> {
        let mut first = self.parse_terminal_expression(lexer)?;
        loop {
            let mut peek = lexer.clone();
            first = match peek.next() {
                Some(Token::OpenBracket) => {
                    let arguments = self.parse_call_arguments(lexer)?;
                    Expression::Invocation(
                        Invocation {
                            path: CallPath::Function(first.into()),
                            arguments,
                        },
                        Default::default(),
                    )
                }
                Some(Token::Period) => {
                    lexer.next();
                    let sb = SpanBuilder::from(lexer);
                    let l = Box::new(first);
                    let r = take!(self, lexer, Token::Identifier = "identifier" => lexer.slice());
                    Expression::Property(
                        Property {
                            target: l,
                            name: r.to_string(),
                        },
                        sb.to(lexer),
                    )
                }
                Some(Token::OpenList) => {
                    lexer.next();
                    let sb = SpanBuilder::from(lexer);
                    let r = self.parse_expression(lexer)?;
                    take!(self, lexer, Token::CloseList = "]");
                    Expression::Index(
                        Index {
                            target: first.into(),
                            index: r.into(),
                        },
                        sb.to(lexer),
                    )
                }
                Some(_) => break,
                None => return Err(DocumentParseError::UnexpectedEndOfFile()),
            }
        }
        Ok(first)
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
                Expression::Invocation( Invocation {
                    path: CallPath::Function(Expression::Reference( Reference { name: "subtract".to_string() } , 0..0).into()),
                    arguments: VecDeque::from([
                        Argument::Named(
                            "left".into(),
                            Box::new(Expression::Literal(Literal::Number(0.0), span.clone())),
                        ),
                        Argument::Named("right".into(), Box::new(expr)),
                    ]),
                }, span)
            },
            Token::Not = "not" => {
                lexer.next();
                let sb = SpanBuilder::from(lexer);
                let expr = self.parse_terminal_expression(lexer)?;
                let span = sb.to(lexer);
                Expression::Invocation( Invocation {
                    path: CallPath::Function(Expression::Reference(Reference{ name: "not".to_string() }, 0..0).into()),
                    arguments: VecDeque::from([
                        Argument::Named("value".into(), Box::new(expr)),
                    ]),
                }, span)
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
            Token::Identifier = "identifier" => self.parse_reference(lexer)?,
            Token::OpenBracket = "(" => {
                lexer.next();
                let expr = self.parse_expression(lexer)?;
                take!(self, lexer, Token::CloseBracket = ")" => expr)
            },
            Token::OpenList = "[" => {
                self.parse_list(lexer)?
            },
            Token::OpenScope = "{" => {
                let sb = SpanBuilder::from(lexer);
                let statements = self.parse_scope(lexer, false)?;
                Expression::Scope( NestedScope { statements }, sb.to(lexer))
            },
            Token::Function = "func" => {
                let sb = SpanBuilder::from(lexer);
                lexer.next();
                let statements = self.parse_scope(lexer, true)?;
                Expression::Literal(Literal::Function(statements.into()), sb.to(lexer))
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
    use crate::resources::Resource;
    use crate::runtime::{RuntimeError, Value};
    use std::io::Error;
    use std::path::{Path, PathBuf};

    macro_rules! parse {
        ($code: literal) => {{}};
    }

    #[derive(Debug, Clone)]
    struct TestRes;

    impl<R: Reader> ResourceLoader<R> for TestRes {
        fn load(&self, _: &str, _: &R) -> Result<Box<dyn Resource>, DocumentParseError> {
            Ok(Box::new(self.clone()))
        }
    }

    impl Resource for TestRes {
        fn to_instance(&self) -> Result<Value, RuntimeError> {
            todo!()
        }
    }

    fn parse(code: &'static str, action: impl for<'a> FnOnce(Result<Ast, ParseError>)) {
        let res = Parser::new(TestReader(code), DocId::new("test".to_string()))
            .with_loader("stl", TestRes)
            .parse();
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
    fn it_parses_arguments() {
        let p = Parser::new((), DocId::new(String::new()));
        let parsed = p
            .parse_arguments(vec!["a=5", "b=true", "c=\"hi\"", "d=-5"].into_iter())
            .unwrap();

        assert!(matches!(parsed.get("a").unwrap(), Literal::Number(_)));
        assert!(matches!(parsed.get("b").unwrap(), Literal::Bool(true)));
        assert!(matches!(parsed.get("c").unwrap(), Literal::Text(_)));
        assert!(matches!(parsed.get("d").unwrap(), Literal::Number(_)));
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
        parse("cube(5 + 5);", |a| {
            a.unwrap();
        });
        parse("cube.foo(5 + 5);", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse_resource_calls() {
        parse("./cube.stl();", |a| {
            a.unwrap();
        });
    }

    #[test]
    fn it_can_parse() {
        parse("cube(x=10,y=10);", |a| {
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

        parse("5 -> cube();", |a| {
            a.unwrap();
        });

        parse_statement("5 ->value cube() ->test cube();", |p| {
            assert!(matches!(p, Statement::CreatePart(
            Expression::Invocation ( Invocation { arguments: x, .. }, _)
            , ..
        ) if !x.iter().any(|a| a.has_name("value"))))
        });
    }

    #[test]
    fn it_can_load_inject_spans() {
        parse_statement("5 ->value cube();", |p| {
            if let Statement::CreatePart(expr, ..) = p {
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
        parse("var foo; foo.bar.baz;", |a| {
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
    fn it_can_parse_scopes() {
        parse("var s = {};", |a| {
            a.unwrap();
        });
        parse("var s = { 5; };", |a| {
            a.unwrap();
        });

        parse("{ var t = 0; }; t;", |a| {
            a.unwrap_err();
        });
        parse("{ var t; };", |a| {
            a.unwrap_err();
        });
    }

    #[test]
    fn it_can_parse_functions() {
        parse("var s = func {};", |a| {
            a.unwrap();
        });
        parse("var s = func { 5; };", |a| {
            a.unwrap();
        });
        parse("func { var t; };", |a| {
            a.unwrap();
        });

        parse("func { var t = 0; }; t;", |a| {
            a.unwrap_err();
        });
    }

    #[test]
    fn it_can_parse_reduce() {
        parse("var foo = reduce [] as a,b: a;", |a| {
            a.unwrap();
        });
        parse("var foo = reduce [] from cube() as a,b: a;", |a| {
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
        parse("var foo = [cube(), 2];", |a| {
            a.unwrap();
        });
    }

    pub struct TestReader(pub &'static str);
    impl Reader for TestReader {
        fn read_bytes(&self, _: &Path) -> Result<Vec<u8>, Error> {
            todo!()
        }

        fn read(&self, _: &Path) -> Result<String, std::io::Error> {
            Ok(self.0.to_string())
        }

        fn normalize(&self, path: &Path) -> PathBuf {
            PathBuf::from(path)
        }
    }
}
