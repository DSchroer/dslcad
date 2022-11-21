use clap::Parser;
use model_script::DSLCAD;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;
use stl_io::Triangle;

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
    println!("{}", &model);

    let mut triangles = Vec::new();
    let mesh = model.mesh();
    for face in &mesh.faces {
        triangles.push(Triangle {
            normal: face.normal,
            vertices: [
                mesh.vertices[face.vertices[0]],
                mesh.vertices[face.vertices[1]],
                mesh.vertices[face.vertices[2]],
            ],
        })
    }

    let outpath = Path::new(&args.out).with_extension("stl");
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(outpath)?;
    stl_io::write_stl(&mut file, triangles.iter())?;
    Ok(())
}
