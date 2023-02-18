use clap::Parser;
use dslcad::export;
use dslcad::{Dslcad, Output};
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;

/// model_script cad compiler
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to load
    source: String,

    /// Outdir
    #[arg(short, long, default_value = ".")]
    out: String,
}

pub(crate) fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut cad = Dslcad::default();
    let source = Path::new(&args.source);
    let root = Path::new(&args.out);

    write_outputs(
        &cad.render_file(source.to_str().unwrap())?,
        root,
        source.file_stem().unwrap().to_str().unwrap(),
    )?;

    Ok(())
}

pub fn write_outputs(outputs: &[Output], dir: &Path, name: &str) -> Result<(), Box<dyn Error>> {
    for (index, model) in outputs.iter().enumerate() {
        let full_name = if index == 0 {
            name.to_string()
        } else {
            format!("{name}_{index}")
        };
        let file = dir.join(Path::new(&full_name));

        write_txt_to_file(model, &file)?;
        write_stl_to_file(model, &file)?;
    }
    Ok(())
}

fn write_txt_to_file(output: &Output, path: &Path) -> Result<(), Box<dyn Error>> {
    if output.text().is_empty() {
        return Ok(());
    }

    let path = Path::new(path).with_extension("txt");
    if path.exists() {
        fs::remove_file(&path)?;
    }

    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;

    export::export_txt(output, &mut file)?;
    Ok(())
}

fn write_stl_to_file(output: &Output, path: &Path) -> Result<(), Box<dyn Error>> {
    if output.mesh().triangles.is_empty() {
        return Ok(());
    }

    let path = Path::new(path).with_extension("stl");
    if path.exists() {
        fs::remove_file(&path)?;
    }

    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;

    export::export_stl(output, &mut file)?;
    Ok(())
}
