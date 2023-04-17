mod camera;
mod file_watcher;
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
use file_watcher::{FileWatcher, FileWatcherPlugin};
use gui::UiEvent;
use std::error::Error;

#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use dslcad_parser::{DocId, ParseError};
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
        .insert_resource(State::new())
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

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugin(FileWatcherPlugin);

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

#[derive(Resource)]
struct State {
    output: Option<Result<Render, CadError>>,
    autowatch: bool,
    watcher: Option<FileWatcher>,

    show_points: bool,
    show_lines: bool,
    show_mesh: bool,

    about_window: bool,
    cheatsheet_window: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            output: None,
            autowatch: true,
            watcher: None,
            show_points: true,
            show_lines: true,
            show_mesh: true,
            about_window: false,
            cheatsheet_window: false,
        }
    }
}

fn controller(
    mut events: EventReader<UiEvent>,
    mut render: EventWriter<RenderCommand>,
    mut state: ResMut<State>,
    project: Res<Project>,
) {
    for event in events.iter() {
        match event {
            #[cfg(not(target_arch = "wasm32"))]
            UiEvent::Render { path } => {
                render_file(&mut state, path, &project);
                render.send(RenderCommand::Redraw);
            }
            // #[cfg(not(target_arch = "wasm32"))]
            // UiEvent::Export() => {
            //     if let Some(Ok(render)) = &state.output {
            //         if let Some(path) = file_dialog_ext(&state, None).pick_folder() {
            //             let origin = state.file.clone();
            //             let client: Client<Message> = Client::new(server);
            //             let result = client
            //                 .send(Message::Export {
            //                     render: render.clone(),
            //                     name: origin
            //                         .unwrap()
            //                         .file_stem()
            //                         .unwrap()
            //                         .to_str()
            //                         .unwrap()
            //                         .to_string(),
            //                     path: format!("{}", path.display()),
            //                 })
            //                 .busy_loop();
            //
            //             match result {
            //                 Message::ExportResults() => {}
            //                 Message::Error(e) => {
            //                     panic!("{}", e);
            //                 }
            //                 _ => panic!("unexpected message {:?}", result),
            //             };
            //         }
            //     }
            // }
            _ => todo!(),
        }
    }
}

fn render_file(state: &mut ResMut<State>, path: &str, project: &Project) -> Option<Vec<PathBuf>> {
    let mut paths = Vec::new();

    let reader = project.reader().expect("cant render unopened project");
    let parser = dslcad_parser::Parser::new(reader, DocId::new(path.to_string()));
    let ast = parser.parse();
    state.output = Some(match ast {
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
    });

    Some(paths)
}
