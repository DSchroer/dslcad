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
use std::env;
use std::error::Error;

#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use std::fs::File;

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
    let cheatsheet = match result {
        Message::CheatSheetResults { cheatsheet } => cheatsheet,
        _ => panic!("unexpected message {:?}", result),
    };
    cheatsheet
}

#[derive(Resource)]
struct State {
    file: Option<PathBuf>,
    output: Option<Result<Render, CadError>>,
    autowatch: bool,
    watcher: Option<FileWatcher>,

    show_points: bool,
    show_lines: bool,
    show_mesh: bool,

    about_window: bool,
    cheatsheet_window: bool,
    editor_window: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            file: None,
            output: None,
            autowatch: true,
            watcher: None,
            show_points: true,
            show_lines: true,
            show_mesh: true,
            about_window: false,
            cheatsheet_window: false,
            editor_window: true,
        }
    }
}

fn controller(
    mut events: EventReader<UiEvent>,
    mut render: EventWriter<RenderCommand>,
    mut state: ResMut<State>,
) {
    for event in events.iter() {
        match event {
            UiEvent::RenderString(string) => {
                let client: Client<Message> = Client::new(server);
                let result = client
                    .send(Message::RenderString {
                        source: string.clone(),
                    })
                    .busy_loop();
                state.output = Some(match result {
                    Message::RenderResults(result, _) => result,
                    _ => panic!("unexpected message {:?}", result),
                });
                render.send(RenderCommand::Redraw);
            }
            #[cfg(not(target_arch = "wasm32"))]
            UiEvent::CreateFile() => {
                if let Some(file) = file_dialog(&state).save_file() {
                    let file = file.with_extension(dslcad_api::constants::FILE_EXTENSION);
                    File::create(&file).unwrap();

                    load_file(&mut state, file);
                    render.send(RenderCommand::Redraw);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            UiEvent::OpenFile() => {
                let file = file_dialog(&state).pick_file();
                if let Some(file) = file {
                    load_file(&mut state, file);
                    render.send(RenderCommand::Redraw);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            UiEvent::Render() => {
                render_file(&mut state);
                render.send(RenderCommand::Redraw);
            }
            #[cfg(not(target_arch = "wasm32"))]
            UiEvent::Export() => {
                if let Some(Ok(render)) = &state.output {
                    if let Some(path) = file_dialog_ext(&state, None).pick_folder() {
                        let origin = state.file.clone();
                        let client: Client<Message> = Client::new(server);
                        let result = client
                            .send(Message::Export {
                                render: render.clone(),
                                name: origin
                                    .unwrap()
                                    .file_stem()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string(),
                                path: format!("{}", path.display()),
                            })
                            .busy_loop();

                        match result {
                            Message::ExportResults() => {}
                            Message::Error(e) => {
                                panic!("{}", e);
                            }
                            _ => panic!("unexpected message {:?}", result),
                        };
                    }
                }
            }
            _ => todo!(),
        }
    }
}

fn load_file(state: &mut ResMut<State>, file: PathBuf) {
    state
        .watcher
        .as_mut()
        .unwrap()
        .clear()
        .expect("failed to clear watcher");
    state.file = Some(file);

    let files = render_file(state);
    if let Some(files) = files {
        for file in files {
            state
                .watcher
                .as_mut()
                .unwrap()
                .add(file)
                .expect("failed to watch file");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn file_dialog(state: &State) -> FileDialog {
    file_dialog_ext(state, Some(dslcad_api::constants::FILE_EXTENSION))
}

#[cfg(not(target_arch = "wasm32"))]
fn file_dialog_ext(state: &State, ext: Option<&str>) -> FileDialog {
    let dir = if let Some(file) = &state.file {
        file.parent().unwrap().to_path_buf()
    } else {
        env::current_dir().unwrap()
    };
    let dialog = FileDialog::new();
    let dialog = if let Some(ext) = ext {
        dialog.add_filter(
            &(dslcad_api::constants::NAME.to_owned() + " Script"),
            &[ext],
        )
    } else {
        dialog
    };
    dialog.set_directory(dir)
}

fn render_file(state: &mut ResMut<State>) -> Option<Vec<PathBuf>> {
    let mut paths = Vec::new();

    if let Some(file) = &state.file {
        let client: Client<Message> = Client::new(server);
        let result = client
            .send(Message::Render {
                path: format!("{}", file.display()),
            })
            .busy_loop();
        state.output = Some(match result {
            Message::RenderResults(result, meta) => {
                paths = meta.files.iter().map(PathBuf::from).collect();
                result
            }
            _ => panic!("unexpected message {:?}", result),
        });
    }

    Some(paths)
}
