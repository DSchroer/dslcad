pub mod constants;
mod library;
mod parser;
mod runtime;
mod syntax;

use crate::parser::ParseError;
use crate::runtime::{EvalContext, RuntimeError};
use library::Library;
use parser::Reader;
use path_absolutize::Absolutize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use crate::syntax::Output;

#[derive(Error, Debug)]
pub enum Error {
    Parse(ParseError),
    Runtime(RuntimeError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(p) => Display::fmt(p, f),
            Error::Runtime(r) => Display::fmt(r, f),
        }
    }
}

pub struct DSLCAD<R> {
    reader: R,
    library: Library,
    paths: Vec<String>,
}

impl Default for DSLCAD<FileReader> {
    fn default() -> Self {
        DSLCAD::<FileReader>::new(FileReader, Library::default())
    }
}

impl<R: Reader> DSLCAD<R> {
    pub fn new(reader: R, library: Library) -> Self {
        DSLCAD {
            reader,
            library,
            paths: Vec::new(),
        }
    }

    pub fn render_file(&mut self, path: &str) -> Result<Output, Error> {
        let parser = parser::Parser::new(path, &self.reader);
        let ast = parser.parse().map_err(Error::Parse)?;

        self.paths = ast.documents().keys().cloned().collect();

        let ctx = EvalContext {
            documents: ast.documents(),
            library: &self.library,
        };
        let main = ast.root_document();

        let output = runtime::eval(main, HashMap::new(), &ctx)
            .map_err(Error::Runtime)?
            .value()
            .to_output()
            .map_err(|_| Error::Runtime(RuntimeError::CantWrite()))?;
        Ok(output)
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
    use crate::runtime::ScriptInstance;

    fn run(code: &str) -> ScriptInstance {
        let reader = TestReader(code);
        let parser = parser::Parser::new("test", &reader);
        let documents = parser.parse().unwrap();
        let ctx = EvalContext {
            documents: documents.documents(),
            library: &Library::new(),
        };
        let main = documents.root_document();
        runtime::eval(main, HashMap::new(), &ctx).expect("failed to eval")
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
            run(format!("1 {} 1;", op).as_str());
        }
    }

    #[test]
    fn it_has_lines() {
        run("line(start=point(x=0,y=0), end=point(x=1,y=1));");
        run("arc(start=point(x=0,y=0),center=point(x=1,y=0), end=point(x=0,y=1));");
    }

    #[test]
    fn it_can_join_lines() {
        run(r"
line(start=point(x=0,y=0), end=point(x=1,y=1))
    ->left union(right=line(start=point(x=0,y=0), end=point(x=1,y=1)));
        ");
    }
}
