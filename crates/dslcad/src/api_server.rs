use crate::export::{export_stl, export_txt};
use crate::Dslcad;
use dslcad_api::protocol::*;
use dslcad_api::Server;
use std::error::Error;

use crate::library::Library;
use crate::parser::Reader;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

pub struct DslcadApi;
impl Server<Message> for DslcadApi {
    fn on_message(message: Message) -> Message {
        match message {
            Message::Render { path } => {
                let mut cad = Dslcad::default();
                let res = cad.render_file(&path);
                let metadata = RenderMetadata { files: cad.paths };
                match res {
                    Ok(outputs) => Message::RenderResults(Ok(Render { parts: outputs }), metadata),
                    Err(e) => Message::RenderResults(
                        Err(CadError::System {
                            error: e.to_string(),
                        }),
                        metadata,
                    ),
                }
            }
            Message::RenderString { source } => {
                let mut cad = Dslcad::new(StringReader(source), Library::new());
                let res = cad.render_file("__internal");
                let metadata = RenderMetadata { files: cad.paths };
                match res {
                    Ok(outputs) => Message::RenderResults(Ok(Render { parts: outputs }), metadata),
                    Err(e) => Message::RenderResults(
                        Err(CadError::System {
                            error: e.to_string(),
                        }),
                        metadata,
                    ),
                }
            }
            Message::Export { render, name, path } => match export(render, name, path) {
                Ok(()) => Message::ExportResults(),
                Err(e) => Message::Error(CadError::System {
                    error: e.to_string(),
                }),
            },
            _ => panic!("Unexpected message: {:#?}", message),
        }
    }
}

fn export(render: Render, name: String, path: String) -> Result<(), Box<dyn Error>> {
    for (i, part) in render.parts.iter().enumerate() {
        let name = if i == 0 {
            name.clone()
        } else {
            format!("{}_{}", &name, &i)
        };
        let path = PathBuf::from(&path).join(&name);

        let ext = match part {
            Part::Data { .. } => "txt",
            Part::Object { .. } => "stl",
            Part::Planar { .. } => todo!("2d export"),
        };

        let path = path.with_extension(ext);
        if path.exists() {
            fs::remove_file(&path)?;
        }

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        match part {
            Part::Data { text } => export_txt(text, &mut file),
            Part::Object { mesh, .. } => export_stl(mesh, &mut file),
            Part::Planar { .. } => todo!("2d export"),
        }?;
    }

    Ok(())
}

pub struct StringReader(String);
impl Reader for StringReader {
    fn read(&self, _: &Path) -> Result<String, std::io::Error> {
        Ok(self.0.to_string())
    }

    fn normalize(&self, path: &Path) -> PathBuf {
        PathBuf::from(path)
    }
}
