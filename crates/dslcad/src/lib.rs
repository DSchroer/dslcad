use crate::library::Library;
use crate::parser::{Ast, DocId, ParseError};
use crate::reader::FsReader;
use crate::runtime::{Engine, RuntimeError, WithStack};
use persistence::protocol::Render;
use std::collections::HashMap;

pub mod export;
pub mod library;
pub mod parser;
pub mod reader;
pub mod runtime;

pub fn parse(source: String) -> Result<Ast, ParseError> {
    let parser = parser::Parser::new(FsReader, DocId::new(source));
    parser.parse()
}

pub fn render(documents: Ast) -> Result<Render, WithStack<RuntimeError>> {
    let lib = Library::new();

    let mut engine = Engine::new(&lib, documents);
    let instance = engine.eval_root(HashMap::new())?;
    let value = instance.value();

    let render = if let Some(parts) = value.to_list() {
        let mut outputs = Vec::new();
        for part in parts {
            outputs.push(part.to_output().unwrap());
        }
        Render { parts: outputs }
    } else {
        Render {
            parts: vec![value.to_output().unwrap()],
        }
    };

    Ok(render)
}

#[cfg(test)]
mod tests {
    use crate::library::Library;
    use crate::parser::{DocId, Reader};
    use crate::runtime::{Engine, ScriptInstance};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    fn run(code: &'static str) -> ScriptInstance {
        let reader = TestReader(code);
        let root = DocId::new("test".to_string());
        let parser = crate::parser::Parser::new(reader, root);
        let documents = parser.parse().unwrap();
        let lib = Library::new();
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
