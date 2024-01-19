use crate::library::Library;
use crate::parser::{Ast, DocId, DocumentParseError, Literal, ParseError, Parser};
use crate::reader::FsReader;
use crate::resources::ResourceExt;
use crate::runtime::{Engine, RuntimeError, Value, WithStack};
use dslcad_storage::protocol::{Part, Render};
use log::trace;
use std::collections::HashMap;
use std::time::Instant;

pub mod library;
pub mod parser;
pub mod reader;
mod resources;
pub mod runtime;
mod trace;

pub fn parse(source: String) -> Result<Ast, ParseError> {
    let parse_time = Instant::now();

    let parser = parser::Parser::new(FsReader, DocId::new(source)).with_default_loaders();
    let ast = parser.parse();

    trace!("parse in {}s", parse_time.elapsed().as_secs_f64());

    ast
}

/// Parse arguments for use in DSLCAD.
/// Arguments take the form of name=literal
pub fn parse_arguments<'a>(
    arguments: impl Iterator<Item = &'a str>,
) -> Result<HashMap<&'a str, Literal>, DocumentParseError> {
    let parse_time = Instant::now();

    let parser = Parser::new((), DocId::new(String::new()));
    let arguments = parser.parse_arguments(arguments)?;

    trace!("arguments in {}s", parse_time.elapsed().as_secs_f64());

    Ok(arguments)
}

pub fn render(
    documents: Ast,
    arguments: HashMap<&str, Literal>,
    deflection: f64,
) -> Result<Render, WithStack<RuntimeError>> {
    let lib = Library::default();

    let mut engine = Engine::new(&lib, documents);

    let eval_time = Instant::now();
    let instance = engine.eval_root(arguments)?;
    trace!("eval in {}s", eval_time.elapsed().as_secs_f64());

    let render_time = Instant::now();

    let text = instance.to_text().unwrap_or_default();

    let parts: Vec<_> = instance.flatten().into_iter().cloned().collect();
    let output = values_to_output(parts, deflection);

    trace!("render in {}s", render_time.elapsed().as_secs_f64());

    Ok(Render {
        parts: output.map_err(|e| WithStack::from_err(e, &vec![]))?,
        stdout: text,
    })
}

#[cfg(feature = "rayon")]
fn values_to_output(values: Vec<Value>, deflection: f64) -> Result<Vec<Part>, RuntimeError> {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    values
        .into_par_iter()
        .map(|v| v.to_output(deflection))
        .collect()
}

#[cfg(not(feature = "rayon"))]
fn values_to_output(values: Vec<Value>, deflection: f64) -> Result<Vec<Part>, RuntimeError> {
    values
        .into_iter()
        .map(|v| v.to_output(deflection))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::library::Library;
    use crate::parser::{Ast, DocId, Reader};
    use crate::runtime::{Engine, Value};
    use crate::{parse_arguments, render};
    use std::collections::HashMap;
    use std::io::Error;
    use std::path::{Path, PathBuf};

    fn parse_str(code: &'static str) -> Ast {
        let reader = TestReader(code);
        let root = DocId::new("test".to_string());
        let parser = crate::parser::Parser::new(reader, root);
        parser.parse().unwrap()
    }

    fn run(code: &'static str) -> Value {
        let documents = parse_str(code);
        let lib = Library::default();
        let mut engine = Engine::new(&lib, documents);
        engine.eval_root(HashMap::new()).expect("failed to eval")
    }

    #[test]
    fn it_has_point() {
        run("point(x=10,y=10);");
        run("point(x=10,y=10).x;");
    }

    #[test]
    fn it_supports_order_of_operations() {
        assert_eq!(Ok(6.), run("5 / 5 + 5;").to_number());
    }

    #[test]
    fn it_has_boolean_algebra() {
        assert_eq!(Ok(true), run("true;").to_bool());
        assert_eq!(Ok(false), run("false;").to_bool());

        assert_eq!(Ok(false), run("true and false;").to_bool());
        assert_eq!(Ok(true), run("true and true;").to_bool());

        assert_eq!(Ok(true), run("true or false;").to_bool());
        assert_eq!(Ok(true), run("true or true;").to_bool());

        assert_eq!(Ok(true), run("not false;").to_bool());
        assert_eq!(Ok(true), run("not false or false;").to_bool());
    }

    #[test]
    fn it_has_math() {
        run("less_or_equal(left=10,right=10);");
        run("pi();");

        run("1+1;");
        run("1-1;");
        run("1*1;");
        run("1/1;");
        run("1%1;");
        run("1^1;");

        run("1>1;");
        run("1>=1;");
        run("1==1;");
        run("1!=1;");
        run("1<1;");
        run("1<=1;");
    }

    #[test]
    fn it_has_lines() {
        run("line(start=point(x=0,y=0), end=point(x=1,y=1));");
        run("arc(start=point(x=0,y=0),center=point(x=1,y=0), end=point(x=0,y=1));");
    }

    #[test]
    fn it_supports_arguments() {
        let args = parse_arguments(vec!["a=\"5\""].into_iter()).unwrap();

        let ast = parse_str("var a; a;");
        let res = render(ast, args, 0.001).unwrap();

        assert_eq!("5", &res.stdout);
    }

    #[test]
    fn it_supports_scopes() {
        assert_eq!(Ok(5.), run("{ 5; };").to_number());
        assert_eq!(Ok(5.), run("{ var t = 5; t; };").to_number());
        assert_eq!(Ok(5.), run("var t = 5; { t; };").to_number());
    }

    #[test]
    fn it_supports_functions() {
        assert_eq!(Ok(5.), run("var f = func { 5; }; f();").to_number());
    }

    #[test]
    fn it_has_if_statements() {
        assert_eq!(Ok(10.), run("if true: 10 else: 0;").to_number());
    }

    #[test]
    fn it_can_join_lines() {
        run(r"
line(start=point(x=0,y=0), end=point(x=1,y=1))
    ->left union(right=line(start=point(x=0,y=0), end=point(x=1,y=1)));
        ");
    }

    #[test]
    fn it_has_lists() {
        run("[1,2,3];");
    }

    #[test]
    fn it_supports_groups_of_parts() {
        let i = run(r"
[[cube(), cube()], [cube(), cube()]];
        ");
        let parts: Vec<_> = i
            .flatten()
            .iter()
            .map(|v| v.to_output(0.1).unwrap())
            .collect();
        assert_eq!(4, parts.len());
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
