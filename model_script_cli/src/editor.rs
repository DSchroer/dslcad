mod camera;
mod file_watcher;
mod gui;
mod stl;
mod xyz;

use bevy::prelude::*;
use bevy_polyline::prelude::*;
use file_watcher::{FileWatcher, FileWatcherPlugin};
use gui::UiEvent;
use model_script::{Output, DSLCAD};
use rfd::FileDialog;
use std::env;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use stl::stl_to_triangle_mesh;

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
                title: model_script::constants::FULL_NAME.to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(PolylinePlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(gui::GuiPlugin)
        .add_plugin(xyz::XYZPlugin)
        .add_plugin(FileWatcherPlugin)
        .add_system(controller)
        .run();
    Ok(())
}

#[derive(Resource)]
struct State {
    file: Option<PathBuf>,
    model: Option<Entity>,
    output: String,
    autowatch: bool,
    watcher: Option<FileWatcher>,
}

impl State {
    pub fn new() -> Self {
        State {
            file: None,
            model: None,
            output: String::new(),
            autowatch: true,
            watcher: None,
        }
    }
}

fn controller(
    mut events: EventReader<UiEvent>,
    mut commands: Commands,
    mut state: ResMut<State>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    for event in events.iter() {
        match event {
            UiEvent::CreateFile() => {
                let file = file_dialog(&state).save_file();

                if let Some(file) = file {
                    let file = file.with_extension(model_script::constants::FILE_EXTENSION);
                    File::create(&file).unwrap();

                    load_file(
                        &mut commands,
                        &mut state,
                        &mut meshes,
                        &mut materials,
                        &mut polyline_materials,
                        &mut polylines,
                        file,
                    );
                }
            }
            UiEvent::OpenFile() => {
                let file = file_dialog(&state).pick_file();
                if let Some(file) = file {
                    load_file(
                        &mut commands,
                        &mut state,
                        &mut meshes,
                        &mut materials,
                        &mut polyline_materials,
                        &mut polylines,
                        file,
                    );
                }
            }
            UiEvent::Render() => {
                clear_model(&mut commands, &mut state);
                display_file(
                    &mut commands,
                    &mut state,
                    &mut meshes,
                    &mut materials,
                    &mut polyline_materials,
                    &mut polylines,
                );
            }
        }
    }
}

fn load_file(
    commands: &mut Commands,
    state: &mut ResMut<State>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    polyline_materials: &mut ResMut<Assets<PolylineMaterial>>,
    polylines: &mut ResMut<Assets<Polyline>>,
    file: PathBuf,
) {
    state
        .watcher
        .as_mut()
        .unwrap()
        .clear()
        .expect("failed to clear watcher");
    state.file = Some(file);

    clear_model(commands, state);
    let files = display_file(
        commands,
        state,
        meshes,
        materials,
        polyline_materials,
        polylines,
    );
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
    let dir = if let Some(file) = &state.file {
        file.parent().unwrap().to_path_buf()
    } else {
        env::current_dir().unwrap()
    };

    FileDialog::new()
        .add_filter(
            &(model_script::constants::NAME.to_owned() + " Script"),
            &[model_script::constants::FILE_EXTENSION],
        )
        .set_directory(dir)
}

fn clear_model(commands: &mut Commands, state: &mut ResMut<State>) {
    if let Some(id) = state.model {
        commands.entity(id).despawn_recursive();
        state.model = None;
    }
}

fn display_file(
    commands: &mut Commands,
    state: &mut ResMut<State>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    polyline_materials: &mut ResMut<Assets<PolylineMaterial>>,
    polylines: &mut ResMut<Assets<Polyline>>,
) -> Option<Vec<PathBuf>> {
    let mut files = None;

    if let Some(file) = &state.file {
        let mut cad = DSLCAD::default();
        let model = cad.render_file(file.to_str().unwrap());
        files = Some(cad.documents().map(PathBuf::from).collect());

        match model {
            Ok(model) => match model {
                Output::Value(s) => state.output = s,
                Output::Figure(lines) => {
                    let mut model = commands.spawn(SpatialBundle::default());
                    model.add_children(|builder| {
                        for line in lines {
                            builder.spawn(PolylineBundle {
                                polyline: polylines.add(Polyline {
                                    vertices: line
                                        .iter()
                                        .map(|p| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32))
                                        .collect(),
                                }),
                                material: polyline_materials.add(PolylineMaterial {
                                    width: 2.0,
                                    color: Blueprint::white(),
                                    perspective: false,
                                    ..Default::default()
                                }),
                                transform: Transform::from_rotation(Quat::from_euler(
                                    EulerRot::XYZ,
                                    -std::f32::consts::PI / 2.,
                                    0.0,
                                    -std::f32::consts::PI / 2.,
                                )),
                                ..Default::default()
                            });
                        }
                    });
                    state.model = Some(model.id());
                    state.output.clear();
                }
                Output::Shape(mesh) => {
                    let mesh = stl_to_triangle_mesh(&mesh);

                    let model = commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(Blueprint::white().into()),
                            transform: Transform::from_rotation(Quat::from_euler(
                                EulerRot::XYZ,
                                -std::f32::consts::PI / 2.,
                                0.0,
                                -std::f32::consts::PI / 2.,
                            )),
                            ..Default::default()
                        })
                        .id();
                    state.model = Some(model);
                    state.output.clear();
                }
            },
            Err(e) => state.output = format!("{:?}", e),
        }
    }

    files
}
