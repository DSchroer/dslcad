use crate::library::Library;
use crate::parser::{Ast, DocId, DocumentParseError, Literal, ParseError, Parser};
use crate::reader::FsReader;
use crate::resources::ResourceExt;
use crate::runtime::{Engine, RuntimeError, Value, WithStack};
use dslcad_storage::protocol::{Part, Render};
use log::trace;
use std::collections::HashMap;
use std::time::Instant;

pub mod error_printer;
pub mod library;
pub mod parser;
pub mod reader;
mod resources;
pub mod runtime;
mod source;
mod trace;

pub fn parse(source: String) -> Result<Ast, ParseError> {
    let parse_time = Instant::now();

    let parser = Parser::new(FsReader, DocId::new(source)).with_default_loaders();
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

pub fn eval(
    documents: Ast,
    arguments: HashMap<&str, Literal>,
) -> Result<Value, WithStack<RuntimeError>> {
    let lib = Library::default();

    let mut engine = Engine::new(&lib, &documents);

    let eval_time = Instant::now();
    let instance = engine.eval_root(arguments)?;
    trace!("eval in {}s", eval_time.elapsed().as_secs_f64());

    Ok(instance)
}

pub fn render(instance: Value, deflection: f64) -> Result<Render, RuntimeError> {
    let render_time = Instant::now();

    let text = instance.to_text().unwrap_or_default();

    let parts: Vec<_> = instance.flatten().into_iter().cloned().collect();
    let output = values_to_output(parts, deflection)?;

    trace!("render in {}s", render_time.elapsed().as_secs_f64());

    Ok(Render {
        parts: output,
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
    use super::*;
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
        let parser = Parser::new(reader, root);
        parser.parse().unwrap()
    }

    fn try_run(code: &'static str) -> Result<Value, WithStack<RuntimeError>> {
        let documents = parse_str(code);
        let lib = Library::default();
        let mut engine = Engine::new(&lib, &documents);
        engine.eval_root(HashMap::new())
    }

    fn run(code: &'static str) -> Value {
        try_run(code).expect("failed to run")
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
    fn it_distinguishes_lines_and_planes() {
        let line = run("line(start=point(x=0,y=0), end=point(x=1,y=1));");
        assert!(line.to_line().is_ok());
        assert!(line.to_plane().is_err());

        let arc = run("arc(start=point(x=0,y=0),center=point(x=1,y=0), end=point(x=0,y=1));");
        assert!(arc.to_line().is_ok());
        assert!(arc.to_plane().is_err());

        let square = run("square();");
        assert!(square.to_plane().is_ok());
        assert!(square.to_line().is_err());

        let circle = run("circle();");
        assert!(circle.to_plane().is_ok());
        assert!(circle.to_line().is_err());

        let joined = run(r"line(start=point(x=0,y=0), end=point(x=1,y=1))
                ->left union(right=line(start=point(x=1,y=1), end=point(x=2,y=2)));");
        assert!(joined.to_line().is_ok());
        assert!(joined.to_plane().is_err());

        let translated_line =
            run("line(start=point(x=0,y=0), end=point(x=1,y=1)) -> translate(x=1);");
        assert!(translated_line.to_line().is_ok());
        assert!(translated_line.to_plane().is_err());

        let translated_plane = run("square() -> translate(x=1);");
        assert!(translated_plane.to_plane().is_ok());
        assert!(translated_plane.to_line().is_err());
    }

    #[test]
    fn it_only_extrudes_and_revolves_planes() {
        run("square() -> extrude(z=1);");
        run("circle() -> revolve(y=360);");
        run(r"face(parts=[point(x=0,y=0), point(x=1,y=0), point(x=0,y=1)]) -> extrude(z=1);");

        assert!(
            try_run("line(start=point(x=0,y=0), end=point(x=1,y=1)) -> extrude(z=1);").is_err()
        );
        assert!(try_run(
            "arc(start=point(x=0,y=0),center=point(x=1,y=0), end=point(x=0,y=1)) -> revolve(x=360);"
        )
        .is_err());
    }

    #[test]
    fn it_can_thicken_a_line_into_a_plane() {
        let perpendicular =
            run("line(start=point(x=0,y=0), end=point(x=10,y=0)) -> thicken(1) -> extrude(z=1);");
        assert!((perpendicular.to_shape().unwrap().volume() - 10.).abs() < 0.01);

        let horizontal =
            run("line(start=point(x=0,y=0), end=point(x=10,y=0)) -> thicken(y=1) -> extrude(z=1);");
        assert!((horizontal.to_shape().unwrap().volume() - 10.).abs() < 0.01);

        let vertical =
            run("line(start=point(x=0,y=0), end=point(x=0,y=10)) -> thicken(x=1) -> extrude(z=1);");
        assert!((vertical.to_shape().unwrap().volume() - 10.).abs() < 0.01);

        assert!(try_run("line(start=point(x=0,y=0), end=point(x=1,y=1)) -> offset(1);").is_err());
    }

    #[test]
    fn it_only_offsets_planes() {
        assert!(run("square() -> offset(1);").to_plane().is_ok());

        assert!(try_run("square() -> thicken(1);").is_err());
    }

    #[test]
    fn it_can_bend_shapes() {
        let bent = run("cube(x=10, y=1, z=1) ->shape bend(y=90);");
        let volume = bent.to_shape().unwrap().volume();
        assert!((volume - 10.0).abs() < 0.1, "unexpected volume {volume}");

        let unbent = run("cube(x=10, y=1, z=1) ->shape bend();");
        assert!((unbent.to_shape().unwrap().volume() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn it_has_axis_scaling() {
        run("cube() -> scale(x=2);");
        run("cube() -> scale(y=2);");
        run("cube() -> scale(z=2);");
    }

    #[test]
    fn it_can_normalize() {
        use dslcad_occt::DsShape;

        let shape = run("cube(x=2, y=4, z=6) -> normalize();");
        let shape = shape.to_shape().unwrap();
        assert!((shape.max_dimension().unwrap() - 1.0).abs() < 1e-6);
        assert!((shape.volume() - 48.0 / 216.0).abs() < 1e-6);

        let plane = run("square(x=2, y=4) -> normalize();");
        assert!((plane.to_plane().unwrap().max_dimension().unwrap() - 1.0).abs() < 1e-6);

        let line = run("line(start=point(x=0,y=0), end=point(x=3,y=4)) -> normalize();");
        assert!((line.to_line().unwrap().max_dimension().unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn it_supports_arguments() {
        let args = parse_arguments(vec!["a=\"5\""].into_iter()).unwrap();

        let ast = parse_str("var a; a;");
        let res = render(eval(ast, args).unwrap(), 0.001).unwrap();

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
        assert_eq!(Ok(5.), run("var a = { func { 5; }; }; a();").to_number());
        assert_eq!(
            Ok(5.),
            run("var a = { var b = func { 5; }; 3; }; a.b();").to_number()
        );
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
