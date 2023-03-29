mod api_server;
mod export;
mod library;
mod parser;
mod runtime;

use crate::parser::{ParseError, SourceStore};
use crate::runtime::{RuntimeError, WithStack};
use dslcad_api::Server;
use library::Library;
use parser::Reader;
use path_absolutize::Absolutize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use dslcad_api::protocol::Part;

/// # Safety
/// user must ensure that length & message are valid and accessible within the memory of the app
#[no_mangle]
pub unsafe extern "C" fn server(
    length: usize,
    message: *const u8,
    cb: unsafe extern "C" fn(usize, *const u8),
) {
    api_server::DslcadApi::receive(length, message, cb)
}

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

pub struct Dslcad {
    store: SourceStore,
    library: Library,
    paths: Vec<String>,
}

impl Default for Dslcad {
    fn default() -> Self {
        Dslcad::new(FileReader, Library::default())
    }
}

impl Dslcad {
    pub fn new<R: Reader + 'static>(reader: R, library: Library) -> Self {
        Dslcad {
            store: SourceStore::new(Box::new(reader)),
            library,
            paths: Vec::new(),
        }
    }

    pub fn render_file(&mut self, path: &str) -> Result<Vec<Part>, Error> {
        let id = self
            .store
            .forge_id(path.to_string())
            .map_err(Error::Parse)?;
        let parser = parser::Parser::new(id, &self.store);
        let ast = parser.parse().map_err(Error::Parse)?;

        self.paths = ast
            .documents()
            .keys()
            .cloned()
            .map(|id| id.to_str().to_owned())
            .collect();

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

    pub fn cheat_sheet(&self) -> String {
        self.library.to_string()
    }
}

pub(crate) struct FileReader;
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

    use super::*;
    use crate::runtime::{Engine, ScriptInstance};
    use std::collections::HashMap;

    fn run(code: &'static str) -> ScriptInstance {
        let reader = TestReader(code);
        let store = SourceStore::new(Box::new(reader));
        let root = store.forge_id("test".to_string()).unwrap();
        let parser = parser::Parser::new(root, &store);
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
    fn it_supports_order_of_operations() {
        assert_eq!("6", &run("5 / 5 + 5;").to_string());
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
