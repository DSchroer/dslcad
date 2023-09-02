mod reader;

use crate::reader::FsReader;
use clap::Parser;
use dslcad::export::{export_stl, export_txt};
use dslcad::library::Library;
use dslcad::runtime::Engine;
use dslcad_api::protocol::Part;
use dslcad_parser::DocId;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
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

    let root = DocId::new(args.source.clone());
    let parser = dslcad_parser::Parser::new(FsReader, root);
    let documents = parser.parse().unwrap();
    let lib = Library::new();
    let mut engine = Engine::new(&lib, documents);
    let instance = engine.eval_root(HashMap::new()).expect("failed to eval");
    let value = instance.value();

    let parts = if let Some(parts) = value.to_list() {
        let mut outputs = Vec::new();
        for part in parts {
            outputs.push(part.to_output()?);
        }
        outputs
    } else {
        vec![value.to_output()?]
    };

    let cwd = env::current_dir()?;
    let file = source.file_stem().unwrap();

    for (i, part) in parts.iter().enumerate() {
        match part {
            Part::Data { text } => {
                let outfile = fs::File::create(part_path(&cwd, file, i, "txt"))?;
                export_txt(text, outfile)?;
            }
            Part::Planar { .. } => todo!("export 2d outputs"),
            Part::Object { mesh, .. } => {
                let mut outfile = fs::File::create(part_path(&cwd, file, i, "stl"))?;
                export_stl(mesh, &mut outfile)?;
            }
        }
    }

    Ok(())
}

fn part_path(cwd: &Path, file: &OsStr, index: usize, ext: &'static str) -> PathBuf {
    if index == 0 {
        cwd.join(file).with_extension(ext)
    } else {
        cwd.join(format!("{}_{}.{}", file.to_string_lossy(), index, ext))
    }
}
