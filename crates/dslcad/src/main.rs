use clap::Parser;
use dslcad::library::Library;
use dslcad::parser::DocId;
use dslcad::reader::FsReader;
use dslcad::runtime::Engine;
use dslcad_api::protocol::Render;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{env, fs};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to load
    source: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let source = Path::new(&args.source);

    let parser = dslcad::parser::Parser::new(FsReader, DocId::new(args.source.clone()));
    let lib = Library::new();

    let documents = parser.parse()?;
    let mut engine = Engine::new(&lib, documents);
    let instance = engine.eval_root(HashMap::new()).expect("failed to eval");
    let value = instance.value();

    let parts = if let Some(parts) = value.to_list() {
        let mut outputs = Vec::new();
        for part in parts {
            outputs.push(part.to_output()?);
        }
        Render { parts: outputs }
    } else {
        Render {
            parts: vec![value.to_output()?],
        }
    };

    let cwd = env::current_dir()?;
    let file = source.file_stem().unwrap();
    let outpath = cwd.join(format!("{}.parts", file.to_string_lossy()));

    fs::write(outpath, parts.to_bytes())?;

    Ok(())
}
