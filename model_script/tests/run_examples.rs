use std::io;
use std::fs::{self, DirEntry};
use model_script::{eval, parse};
use std::path::Path;

#[test]
fn test_can_run_examples() {
    let mut examples = Vec::new();
    visit_dirs(Path::new("../examples"), &mut examples).expect("cant read examples");

    assert_ne!(0, examples.len());
    for example in examples {
        let ast = parse(example.path().to_str().unwrap()).expect(&format!("cant parse {:?}", example));
        eval(ast).expect(&format!("cant run {:?}", example));
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