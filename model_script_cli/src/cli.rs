use clap::Parser;
use model_script::{eval, parse};
use std::error::Error;

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

pub(crate) fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let ast = parse(&args.source)?;
    let mut model = eval(ast)?;
    model.write_to_file(&args.out)?;
    Ok(())
}
