use clap::Parser;
use dslcad::{Dslcad, Output};
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use stl_io::{Normal, Triangle, Vector};

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
        let file = dir.join(Path::new(&format!("{full_name}.stl")));
        write_stl_to_file(model, &file)?;
    }
    Ok(())
}

fn write_stl_to_file(output: &Output, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut triangles = Vec::new();
    let mesh = output.mesh();
    for (face, normal) in mesh.triangles_with_normals() {
        triangles.push(Triangle {
            vertices: [
                Vector::new(mesh.vertex_f32(face[0])),
                Vector::new(mesh.vertex_f32(face[1])),
                Vector::new(mesh.vertex_f32(face[2])),
            ],
            normal: Normal::new(normal.map(|n| n as f32)),
        })
    }

    let outpath = Path::new(path).with_extension("stl");
    if outpath.exists() {
        fs::remove_file(&outpath)?;
    }

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(outpath)?;
    stl_io::write_stl(&mut file, triangles.iter())?;
    Ok(())
}
