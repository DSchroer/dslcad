extern crate core;

mod library;
mod parser;
mod runtime;
mod syntax;

use clap::Parser;
use library::Library;
use parser::{ParseResult, Reader};
use path_absolutize::Absolutize;
use runtime::{eval, EvalContext};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use syntax::Instance;

/// model_script cad compiler
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to load
    source: String,

    /// Variable definition
    #[arg(short, long)]
    variable: Vec<String>,

    /// Variable definition
    #[arg(short, long)]
    out: String,

    /// Debug mode
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let parser = parser::Parser::new(&args.source, &FileReader);
    match parser.parse() {
        ParseResult::Failure(errors) => {
            for err in errors {
                err.print(&FileReader);
            }
        }
        ParseResult::Success(documents) => {
            if args.debug {
                println!("{:#?}", documents)
            }

            let path = Path::new(&args.source).absolutize().unwrap();

            let ctx = EvalContext {
                documents,
                library: Library,
            };
            let main = ctx.documents.get(path.to_str().unwrap()).unwrap();

            let mut res = eval(main, HashMap::new(), &ctx)?;

            res.write_to_file(&args.out)?;
        }
    }

    Ok(())
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
