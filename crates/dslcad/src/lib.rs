use crate::library::Library;
use crate::parser::{Ast, DocId, ParseError};
use crate::reader::FsReader;
use crate::runtime::{Engine, RuntimeError, WithStack};
use persistence::protocol::Render;
use std::collections::HashMap;

pub mod library;
pub mod parser;
pub mod reader;
pub mod runtime;

pub fn parse(source: String) -> Result<Ast, ParseError> {
    let parser = parser::Parser::new(FsReader, DocId::new(source));
    parser.parse()
}

pub fn render(documents: Ast) -> Result<Render, WithStack<RuntimeError>> {
    let lib = Library::default();

    let mut engine = Engine::new(&lib, documents);
    let instance = engine.eval_root(HashMap::new())?;

    Ok(Render {
        parts: instance
            .value()
            .to_output()
            .map_err(|e| WithStack::from_err(e, &vec![]))?,
    })
}

#[cfg(test)]
mod tests {
    use crate::library::Library;
    use crate::parser::{DocId, Reader};
    use crate::runtime::{Engine, ScriptInstance, Value};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    fn run(code: &'static str) -> ScriptInstance {
        let reader = TestReader(code);
        let root = DocId::new("test".to_string());
        let parser = crate::parser::Parser::new(reader, root);
        let documents = parser.parse().unwrap();
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
        assert_eq!(Ok(6.), run("5 / 5 + 5;").value().to_number());
    }

    #[test]
    fn it_has_boolean_algebra() {
        assert_eq!(Ok(true), run("true;").value().to_bool());
        assert_eq!(Ok(false), run("false;").value().to_bool());

        assert_eq!(Ok(false), run("true and false;").value().to_bool());
        assert_eq!(Ok(true), run("true and true;").value().to_bool());

        assert_eq!(Ok(true), run("true or false;").value().to_bool());
        assert_eq!(Ok(true), run("true or true;").value().to_bool());

        assert_eq!(Ok(true), run("not false;").value().to_bool());
        assert_eq!(Ok(true), run("not false or false;").value().to_bool());
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
        assert_eq!(Ok(10.), run("if true: 10 else: 0;").value().to_number());
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
        let i: Value = run(r"
[[cube(), cube()], [cube(), cube()]];
        ")
        .into();
        assert_eq!(4, i.to_output().unwrap().len());
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
