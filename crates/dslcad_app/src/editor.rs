mod camera;
mod gui;
mod rendering;
mod stl;
mod xyz;

use crate::dslcad::server;
use crate::editor::rendering::RenderCommand;
use bevy::prelude::*;
use bevy_polyline::prelude::*;
use dslcad_api::protocol::{CadError, Message, Render};
use dslcad_api::Client;
use gui::UiEvent;
use std::error::Error;

use dslcad_parser::DocId;
pub use gui::Project;
use std::path::PathBuf;

struct Blueprint;
impl Blueprint {
    fn white() -> Color {
        Color::hex("CED8F7").unwrap()
    }

    fn blue() -> Color {
        Color::hex("3057E1").unwrap()
    }

    fn black() -> Color {
        Color::hex("002082").unwrap()
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    let cheatsheet = load_cheetsheet();

    app.insert_resource(Msaa::default())
        .insert_resource(ClearColor(Blueprint::blue()))
        .insert_resource(State::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: dslcad_api::constants::FULL_NAME.to_string(),
                canvas: Some("#dslcad".to_string()),
                fit_canvas_to_parent: true,
                ..Default::default()
            }),
            ..default()
        }))
        .add_plugin(PolylinePlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(gui::GuiPlugin::new(cheatsheet))
        .add_plugin(xyz::XYZPlugin)
        .add_plugin(rendering::ModelRenderingPlugin)
        .add_system(controller);

    app.run();
    Ok(())
}

fn load_cheetsheet() -> String {
    let client: Client<Message> = Client::new(server);
    let result = client.send(Message::CheatSheet()).busy_loop();

    match result {
        Message::CheatSheetResults { cheatsheet } => cheatsheet,
        _ => panic!("unexpected message {:?}", result),
    }
}

#[derive(Resource, Default, Debug)]
enum State {
    #[default]
    Empty,
    Rendered {
        document: DocId,
        output: Result<Render, CadError>,
    },
}

fn controller(
    mut events: EventReader<UiEvent>,
    mut render: EventWriter<RenderCommand>,
    state: ResMut<State>,
    project: Res<Project>,
) {
    if let Some(event) = events.iter().next() {
        match event {
            UiEvent::Render { path } => {
                render_file(state, DocId::new(path.to_string()), &project);
                render.send(RenderCommand::Redraw);
            }
            UiEvent::ReRender() => {
                let path = if let State::Rendered { document, .. } = &state.as_ref() {
                    Some(document.clone())
                } else {
                    None
                };

                if let Some(path) = path {
                    render_file(state, path, &project);
                    render.send(RenderCommand::Redraw);
                }
            }
            UiEvent::Export() => todo!(),
        }
    }
}

fn render_file(mut state: ResMut<State>, path: DocId, project: &Project) -> Option<Vec<PathBuf>> {
    let mut paths = Vec::new();

    let reader = project.reader().expect("cant render unopened project");
    let parser = dslcad_parser::Parser::new(reader, path.clone());
    let ast = parser.parse();
    let render = State::Rendered {
        document: path,
        output: match ast {
            Ok(ast) => {
                paths = ast
                    .documents
                    .keys()
                    .map(|d| d.to_path().to_path_buf())
                    .collect();

                let client: Client<Message> = Client::new(server);
                let result = client.send(Message::Render { ast }).busy_loop();
                match result {
                    Message::RenderResults(result, _) => result,
                    _ => panic!("unexpected message {:?}", result),
                }
            }
            Err(e) => Err(CadError {
                error: e.to_string(),
            }),
        },
    };
    *state = render;
    Some(paths)
}
