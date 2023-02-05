extern crate core;

pub mod constants;
mod library;
mod parser;
mod runtime;

use crate::parser::ParseError;
use crate::runtime::{RuntimeError, WithStack};
use library::Library;
use parser::Reader;
use path_absolutize::Absolutize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use opencascade::Mesh;
pub use runtime::Output;

#[derive(Error, Debug)]
pub enum Error {
    Parse(ParseError),
    Runtime(WithStack<RuntimeError>),
    CantWrite(),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(p) => Display::fmt(p, f),
            Error::Runtime(r) => Display::fmt(r, f),
            Error::CantWrite() => f.write_str("unable to write file"),
        }
    }
}

pub struct Dslcad<R> {
    reader: R,
    library: Library,
    paths: Vec<String>,
}

impl Default for Dslcad<FileReader> {
    fn default() -> Self {
        Dslcad::<FileReader>::new(FileReader, Library::default())
    }
}

impl<R: Reader> Dslcad<R> {
    pub fn new(reader: R, library: Library) -> Self {
        Dslcad {
            reader,
            library,
            paths: Vec::new(),
        }
    }

    pub fn render_file(&mut self, path: &str) -> Result<Vec<Output>, Error> {
        let parser = parser::Parser::new(path, &self.reader);
        let ast = parser.parse().map_err(Error::Parse)?;

        self.paths = ast.documents().keys().cloned().collect();

        let main = ast.root_document();

        let mut engine = runtime::Engine::new(&self.library, ast.documents());
        let output = engine
            .eval(main, HashMap::new())
            .map_err(Error::Runtime)?
            .value()
            .clone();

        if let Some(parts) = output.to_list() {
            let mut outputs = Vec::new();
            for part in parts {
                outputs.push(part.to_output().map_err(|_| Error::CantWrite())?);
            }
            Ok(outputs)
        } else {
            Ok(vec![output.to_output().map_err(|_| Error::CantWrite())?])
        }
    }

    pub fn documents(&self) -> impl Iterator<Item = &str> {
        self.paths.iter().map(|p| p.as_str())
    }

    pub fn cheat_sheet(&self) -> String {
        self.library.to_string()
    }
}

pub struct FileReader;
impl Reader for FileReader {
    fn read(&self, name: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(name)
    }

    fn normalize(&self, path: &Path) -> PathBuf {
        PathBuf::from(path).absolutize().unwrap().to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::tests::TestReader;

    use std::collections::HashMap;

    use super::*;
    use crate::runtime::{Engine, ScriptInstance};

    fn run(code: &str) -> ScriptInstance {
        let reader = TestReader(code);
        let parser = parser::Parser::new("test", &reader);
        let documents = parser.parse().unwrap();
        let lib = Library::new();
        let mut engine = Engine::new(&lib, documents.documents());
        let main = documents.root_document();
        engine.eval(main, HashMap::new()).expect("failed to eval")
    }

    #[test]
    fn it_has_point() {
        run("point(x=10,y=10);");
        run("point(x=10,y=10).x;");
    }

    #[test]
    fn it_has_boolean_algebra() {
        assert_eq!("true", &run("true;").to_string());
        assert_eq!("false", &run("false;").to_string());

        assert_eq!("false", &run("true and false;").to_string());
        assert_eq!("true", &run("true and true;").to_string());

        assert_eq!("true", &run("true or false;").to_string());
        assert_eq!("true", &run("true or true;").to_string());

        assert_eq!("true", &run("not false;").to_string());
        assert_eq!("true", &run("not false or false;").to_string());
    }

    #[test]
    fn it_has_math() {
        run("less_or_equal(left=10,right=10);");
        run("pi();");

        let ops = vec![
            "+", "-", "*", "/", "%", "^", ">", ">=", "==", "!=", "<", "<=",
        ];
        for op in ops {
            run(format!("1 {op} 1;").as_str());
        }
    }

    #[test]
    fn it_has_lines() {
        run("line(start=point(x=0,y=0), end=point(x=1,y=1));");
        run("arc(start=point(x=0,y=0),center=point(x=1,y=0), end=point(x=0,y=1));");
    }

    #[test]
    fn it_has_if_statements() {
        assert_eq!("10", run("if true: 10 else: 0;").to_string());
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
}
