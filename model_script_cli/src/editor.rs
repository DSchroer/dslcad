mod file_watcher;
mod input_map;
mod stl;

use crate::editor::input_map::input_map;
use crate::editor::stl::stl_to_triangle_mesh;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use bevy_prototype_debug_lines::*;
use file_watcher::{FileWatcher, FileWatcherPlugin};
use model_script::{eval, parse, Output};
use rfd::FileDialog;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
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
        .add_event::<UiEvent>()
        .add_plugin(FileWatcherPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::new(true))
        .add_system(input_map)
        .add_plugin(EguiPlugin)
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_startup_system(setup)
        .add_system(ui_example)
        .add_system(console_panel)
        .add_system(camera_light)
        .add_system(xyz_lines)
        .add_system(controller)
        .add_system(update_ui_scale_factor)
        .add_system(keybindings)
        .run();
    Ok(())
}

fn update_ui_scale_factor(mut egui_settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.0 / window.scale_factor();
    }
}

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

fn xyz_lines(mut lines: ResMut<DebugLines>) {
    let end = 1_000_000.0;
    let color = Blueprint::black();
    lines.line_colored(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(end, 0.0, 0.0),
        0.0,
        color,
    );
    lines.line_colored(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, end, 0.0),
        0.0,
        color,
    );
    lines.line_colored(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, end),
        0.0,
        color,
    );
}

fn camera_light(
    query: Query<&Transform, With<OrbitCameraController>>,
    mut light: Query<&mut Transform, (With<DirectionalLight>, Without<OrbitCameraController>)>,
) {
    let gxf = query.single();
    for mut transform in light.iter_mut() {
        transform.clone_from(gxf);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert_bundle(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_translate_sensitivity: Vec2::splat(0.001),
                ..Default::default()
            },
            Vec3::new(100.0, 100.0, 100.0),
            Vec3::new(0., 0., 0.),
        ));

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 100000.0,
            color: Blueprint::white(),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(100.0, 100.0, 100.0))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Blueprint::blue(),
        brightness: 0.2,
    });
}

pub enum UiEvent {
    CreateFile(),
    OpenFile(),
    Render(),
}

fn controller(
    mut events: EventReader<UiEvent>,
    mut commands: Commands,
    mut state: ResMut<State>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            UiEvent::CreateFile() => {
                let file = file_dialog(&state).save_file();

                if let Some(file) = file {
                    let file = file.with_extension("ex");
                    File::create(&file).unwrap();

                    load_file(&mut commands, &mut state, &mut meshes, &mut materials, file);
                }
            }
            UiEvent::OpenFile() => {
                let file = file_dialog(&state).pick_file();
                if let Some(file) = file {
                    load_file(&mut commands, &mut state, &mut meshes, &mut materials, file);
                }
            }
            UiEvent::Render() => {
                clear_model(&mut commands, &mut state);
                display_file(&mut commands, &mut state, &mut meshes, &mut materials);
            }
        }
    }
}

fn load_file(
    commands: &mut Commands,
    state: &mut ResMut<State>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
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
    let files = display_file(commands, state, meshes, materials);
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
        .add_filter("script", &["ex"])
        .set_directory(dir)
}

fn keybindings(keys: Res<Input<KeyCode>>, mut events: EventWriter<UiEvent>) {
    if keys.just_released(KeyCode::N) {
        events.send(UiEvent::CreateFile())
    }

    if keys.just_released(KeyCode::O) {
        events.send(UiEvent::OpenFile())
    }

    if keys.just_released(KeyCode::F5) {
        events.send(UiEvent::Render())
    }
}

fn ui_example(
    mut egui_context: ResMut<EguiContext>,
    mut events: EventWriter<UiEvent>,
    mut state: ResMut<State>,
) {
    egui::TopBottomPanel::top("Tools").show(egui_context.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New (n)").clicked() {
                    events.send(UiEvent::CreateFile());
                    ui.close_menu();
                }
                if ui.button("Open (o)").clicked() {
                    events.send(UiEvent::OpenFile());
                    ui.close_menu();
                }
            });
            ui.separator();
            if ui.button("Render (F5)").clicked() {
                events.send(UiEvent::Render());
            }

            ui.checkbox(&mut state.autowatch, "Auto Render");
        });
    });
}

fn console_panel(state: Res<State>, mut egui_context: ResMut<EguiContext>) {
    egui::TopBottomPanel::bottom("Console").show(egui_context.ctx_mut(), |ui| {
        ui.label("Console:");
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(256.)
            .show(ui, |ui| {
                ui.monospace(&state.output);
            });
    });
}

fn clear_model(commands: &mut Commands, state: &mut ResMut<State>) {
    if let Some(id) = state.model {
        commands.entity(id).despawn();
        state.model = None;
    }
}

fn display_file(
    commands: &mut Commands,
    state: &mut ResMut<State>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Option<Vec<PathBuf>> {
    let mut files = None;

    if let Some(file) = &state.file {
        match parse(file.to_str().unwrap()) {
            Err(e) => state.output = e.to_string(),
            Ok(ast) => {
                files = Some(ast.documents().keys().map(PathBuf::from).collect());

                match eval(ast) {
                    Ok(model) => match model {
                        Output::Value(s) => state.output = s,
                        Output::Figure() => state.output = String::from("TODO display 2D!"),
                        Output::Shape(mesh) => {
                            let mesh = stl_to_triangle_mesh(&mesh);

                            let model = commands
                                .spawn_bundle(PbrBundle {
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
        }
    }

    files
}
