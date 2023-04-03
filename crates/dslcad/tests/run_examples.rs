use std::fs::{self, DirEntry};
use std::io;
use std::io::Error;
use std::path::{Path, PathBuf};

use dslcad_api::protocol::Message;
use dslcad_api::Client;
use dslcad_parser::{DocId, Parser, Reader};

#[test]
fn test_can_run_examples() {
    let mut examples = Vec::new();
    visit_dirs(Path::new("../../examples"), &mut examples).expect("cant read examples");
    let client = Client::new(dslcad::server);

    assert_ne!(0, examples.len());
    for example in examples {
        println!("rendering {}", example.path().to_str().unwrap());

        let parser = Parser::new(
            FsReader,
            DocId::new(example.path().to_str().unwrap().to_string()),
        );
        let ast = parser.parse().expect("failed to parse");

        let result = client.send(Message::Render { ast }).busy_loop();
        if let Message::Error(e) = result {
            panic!("{}", e)
        }
    }
}

fn visit_dirs(dir: &Path, vec: &mut Vec<DirEntry>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, vec)?;
            } else {
                vec.push(entry);
            }
        }
    }
    Ok(())
}

struct FsReader;
impl Reader for FsReader {
    fn read(&self, path: &Path) -> Result<String, Error> {
        fs::read_to_string(path)
    }

    fn normalize(&self, _path: &Path) -> PathBuf {
        todo!()
    }
}
