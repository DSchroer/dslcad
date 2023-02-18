mod camera;
mod file_watcher;
mod gui;
mod rendering;
mod stl;
mod xyz;

use crate::editor::rendering::RenderCommand;
use bevy::prelude::*;
use bevy_polyline::prelude::*;
use dslcad::{Dslcad, Output};
use file_watcher::{FileWatcher, FileWatcherPlugin};
use gui::UiEvent;
use rfd::FileDialog;
use std::env;
use std::error::Error;
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
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Blueprint::blue()))
        .insert_resource(State::new())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: dslcad::constants::FULL_NAME.to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(PolylinePlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(gui::GuiPlugin)
        .add_plugin(xyz::XYZPlugin)
        .add_plugin(FileWatcherPlugin)
        .add_plugin(rendering::ModelRenderingPlugin)
        .add_system(controller)
        .run();
    Ok(())
}

#[derive(Resource)]
struct State {
    file: Option<PathBuf>,
    output: Option<Result<Vec<Output>, dslcad::Error>>,
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
            file: None,
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
) {
    for event in events.iter() {
        match event {
            UiEvent::CreateFile() => {
                if let Some(file) = file_dialog(&state).save_file() {
                    let file = file.with_extension(dslcad::constants::FILE_EXTENSION);
                    File::create(&file).unwrap();

                    load_file(&mut state, file);
                    render.send(RenderCommand::Redraw);
                }
            }
            UiEvent::OpenFile() => {
                let file = file_dialog(&state).pick_file();
                if let Some(file) = file {
                    load_file(&mut state, file);
                    render.send(RenderCommand::Redraw);
                }
            }
            UiEvent::Render() => {
                render_file(&mut state);
                render.send(RenderCommand::Redraw);
            }
            UiEvent::Export() => {
                if let Some(Ok(model)) = &state.output {
                    if let Some(path) = file_dialog_ext(&state, None).pick_folder() {
                        let origin = state.file.clone();
                        crate::cli::write_outputs(
                            model,
                            &path,
                            origin.unwrap().file_stem().unwrap().to_str().unwrap(),
                        )
                        .expect("unable to save stl");
                    }
                }
            }
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

fn file_dialog(state: &State) -> FileDialog {
    file_dialog_ext(state, Some(dslcad::constants::FILE_EXTENSION))
}

fn file_dialog_ext(state: &State, ext: Option<&str>) -> FileDialog {
    let dir = if let Some(file) = &state.file {
        file.parent().unwrap().to_path_buf()
    } else {
        env::current_dir().unwrap()
    };
    let dialog = FileDialog::new();
    let dialog = if let Some(ext) = ext {
        dialog.add_filter(&(dslcad::constants::NAME.to_owned() + " Script"), &[ext])
    } else {
        dialog
    };
    dialog.set_directory(dir)
}

fn render_file(state: &mut ResMut<State>) -> Option<Vec<PathBuf>> {
    let mut files = None;

    if let Some(file) = &state.file {
        let mut cad = Dslcad::default();
        let model = cad.render_file(file.to_str().unwrap());
        files = Some(cad.documents().map(PathBuf::from).collect());
        state.output = Some(model);
    }

    files
}
