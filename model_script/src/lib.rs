mod library;
mod parser;
mod runtime;
mod syntax;

use crate::library::Library;
use crate::parser::{Ast, ParseError};
use crate::runtime::{EvalContext, RuntimeError};
use parser::Reader;
use path_absolutize::Absolutize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use crate::syntax::Output;

pub fn parse(path: &str) -> Result<Ast, ParseError> {
    let parser = parser::Parser::new(path, &FileReader);
    parser.parse()
}

pub fn eval(ast: Ast) -> Result<Output, RuntimeError> {
    let ctx = EvalContext {
        documents: ast.documents(),
        library: Library::new(),
    };
    let main = ast.root_document();

    let output = runtime::eval(main, HashMap::new(), &ctx)?
        .value()
        .to_output()
        .map_err(|_| RuntimeError::CantWrite())?;
    Ok(output)
}

struct FileReader;
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
            library: Library::new(),
        };
        let main = documents.root_document();
        runtime::eval(main, HashMap::new(), &ctx).expect("failed to eval")
    }

    #[test]
    fn it_has_point() {
        run("point(x=10,y=10);");
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
