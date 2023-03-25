use crate::export::{export_stl, export_txt};
use crate::Dslcad;
use dslcad_api::protocol::*;
use dslcad_api::{server_fn, Server};
use std::error::Error;
use std::fmt::format;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;

server_fn!(DslcadApi);

struct DslcadApi;
impl Server<Message> for DslcadApi {
    fn on_message(message: Message) -> Message {
        match message {
            Message::Render { path } => {
                let mut cad = Dslcad::default();
                let res = cad.render_file(&path);
                match res {
                    Ok(outputs) => Message::RenderResults(Render { parts: outputs }),
                    Err(e) => Message::Error(CadError::System {
                        error: e.to_string(),
                    }),
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
