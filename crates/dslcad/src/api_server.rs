use crate::export::{export_stl, export_txt};
use dslcad_api::protocol::*;
use dslcad_api::Server;
use std::collections::HashMap;
use std::error::Error;

use crate::library::Library;
use crate::runtime::Engine;
use dslcad_parser::Ast;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;

pub struct DslcadApi;
impl Server<Message> for DslcadApi {
    fn on_message(message: Message) -> Message {
        match message {
            Message::Render { ast } => {
                let res = render_ast(ast);
                let metadata = RenderMetadata {};
                match res {
                    Ok(outputs) => Message::RenderResults(Ok(Render { parts: outputs }), metadata),
                    Err(e) => Message::RenderResults(
                        Err(CadError {
                            error: e.to_string(),
                        }),
                        metadata,
                    ),
                }
            }
            Message::Export { render, name, path } => match export(render, name, path) {
                Ok(()) => Message::ExportResults(),
                Err(e) => Message::Error(CadError {
                    error: e.to_string(),
                }),
            },
            Message::CheatSheet() => {
                let cad = Library::default();
                let cheatsheet = cad.to_string();
                Message::CheatSheetResults { cheatsheet }
            }
            _ => panic!("Unexpected message: {:#?}", message),
        }
    }
}

fn render_ast(ast: Ast) -> Result<Vec<Part>, Box<dyn Error>> {
    let library = Library::default();
    let mut engine = Engine::new(&library, ast);
    let instance = engine.eval_root(HashMap::new())?;
    let value = instance.value();

    if let Some(parts) = value.to_list() {
        let mut outputs = Vec::new();
        for part in parts {
            outputs.push(part.to_output()?);
        }
        Ok(outputs)
    } else {
        Ok(vec![value.to_output()?])
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
