use dslcad::{parse, render};
use std::time::Instant;
use walkdir::WalkDir;

#[test]
fn can_run_examples() {
    for file in WalkDir::new("../../examples")
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.path().extension() == Some("ds".as_ref()))
    {
        let now = Instant::now();
        let path = file.path().to_string_lossy().to_string();

        println!("Example [{}]", &path);

        let ast = parse(path.clone()).unwrap_or_else(|_| panic!("failed to parse {}", &path));

        println!("\tParsed in {:.4}ms", now.elapsed().as_secs_f64() * 1000.0);

        render(ast).unwrap_or_else(|_| panic!("failed to render {}", &path));

        println!(
            "\tRendered in {:.4}ms",
            now.elapsed().as_secs_f64() * 1000.0
        );
    }
}