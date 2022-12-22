use clap::Parser;
use dslcad::DSLCAD;
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

    /// Outfile
    #[arg(short, long)]
    out: String,
}

pub(crate) fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut cad = DSLCAD::default();
    let model = cad.render_file(&args.source)?;

    let mut triangles = Vec::new();
    let mesh = model.mesh();
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

    let outpath = Path::new(&args.out).with_extension("stl");
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
