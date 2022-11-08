use clap::Parser;
use model_script::{eval, parse, FileReader, Instance};

/// model_script cad compiler
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to load
    source: String,

    /// Outfile
    #[arg(short, long)]
    out: String,
}

fn main() {
    let _args = Args::parse();
    match parse(&_args.source) {
        Err(e) => e.print(&FileReader),
        Ok(ast) => match eval(ast) {
            Ok(mut model) => {
                model.write_to_file(&_args.out);
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        },
    }
}
