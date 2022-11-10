mod input_map;

use crate::editor::input_map::input_map;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use bevy_prototype_debug_lines::*;
use model_script::{eval, parse};
use path_absolutize::Absolutize;
use rfd::FileDialog;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

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
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::new(true))
        .add_system(input_map)
        .add_plugin(bevy_stl::StlPlugin)
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
}

impl State {
    pub fn new() -> Self {
        State {
            file: None,
            model: None,
            output: String::new(),
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
    OpenFile(),
    Render(),
}

fn controller(
    mut events: EventReader<UiEvent>,
    mut commands: Commands,
    mut state: ResMut<State>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            UiEvent::OpenFile() => {
                let file = FileDialog::new()
                    .add_filter("script", &["ex"])
                    .set_directory(env::current_dir().unwrap())
                    .pick_file();

                state.file = file;

                clear_model(&mut commands, &mut state);
                display_file(&mut commands, &asset_server, &mut state, &mut materials);
            }
            UiEvent::Render() => {
                clear_model(&mut commands, &mut state);
                display_file(&mut commands, &asset_server, &mut state, &mut materials);
            }
        }
    }
}

fn keybindings(keys: Res<Input<KeyCode>>, mut events: EventWriter<UiEvent>) {
    if keys.just_released(KeyCode::O) {
        events.send(UiEvent::OpenFile())
    }

    if keys.just_released(KeyCode::F5) {
        events.send(UiEvent::Render())
    }
}

fn ui_example(mut egui_context: ResMut<EguiContext>, mut events: EventWriter<UiEvent>) {
    egui::TopBottomPanel::top("Tools").show(egui_context.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Open File").clicked() {
                events.send(UiEvent::OpenFile());
            }

            if ui.button("Render").clicked() {
                events.send(UiEvent::Render());
            }
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
    asset_server: &Res<AssetServer>,
    state: &mut ResMut<State>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let edit_file = Path::new("./a.stl");
    let edit_file = edit_file.absolutize().unwrap();
    let edit_file = edit_file.to_str().unwrap();

    if let Some(file) = &state.file {
        match parse(file.to_str().unwrap()) {
            Err(e) => state.output = format!("{}", e),
            Ok(ast) => match eval(ast) {
                Ok(mut model) => {
                    model.write_to_file(edit_file).unwrap();
                    asset_server.reload_asset(edit_file);

                    let model = commands
                        .spawn_bundle(PbrBundle {
                            mesh: asset_server.load(edit_file),
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
                Err(e) => state.output = format!("{:?}", e),
            },
        }
    }
}
