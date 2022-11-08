use std::env;
use std::path::{Path, PathBuf};
use clap::Parser;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use rfd::FileDialog;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use path_absolutize::Absolutize;
use model_script::{eval, parse, FileReader, Instance};

/// model_script cad compiler
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to load
    source: String,

    /// Outfile
    #[arg(short, long)]
    out: String,
}

fn main() {
    if env::args().len() > 1 {
        run_cli(Args::parse());
    } else {
        App::new()
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(ClearColor(Color::hex("3057E1").unwrap()))
            .insert_resource(State::new())
            .add_plugins(DefaultPlugins)
            .add_plugin(LookTransformPlugin)
            .add_plugin(OrbitCameraPlugin::default())
            .add_plugin(bevy_stl::StlPlugin)
            .add_plugin(EguiPlugin)
            .add_startup_system(setup)
            .add_system(ui_example)
            .run();
    }
}

struct State {
    file: Option<PathBuf>,
    model: Option<Entity>
}

impl State {
    pub fn new() -> Self{
        State{
            file: None,
            model: None
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle::default())
        .insert_bundle(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(-40.0, 40.0, 0.0),
            Vec3::new(0., 0., 0.),
        ));

    commands.insert_resource(AmbientLight {
        color: Color::hex("CED8F7").unwrap(),
        brightness: 0.4,
    });
}

fn ui_example(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<State>, materials: ResMut<Assets<StandardMaterial>>, mut egui_context: ResMut<EguiContext>) {
    egui::TopBottomPanel::top("Tools").show(egui_context.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Open File").clicked() {
                let file = FileDialog::new()
                    .add_filter("script", &["ex"])
                    .set_directory(env::current_dir().unwrap())
                    .pick_file();

                state.file = file;
                try_clear(&mut commands, &mut state);
                display_file(commands, asset_server, state, materials);
            } else if ui.button("Render").clicked() {
                try_clear(&mut commands, &mut state);
                display_file(commands, asset_server, state, materials);
            }
        });
    });
}

fn try_clear(commands: &mut Commands, state: &mut ResMut<State>) {
    if let Some(id) = state.model {
        commands.entity(id).despawn();
        state.model = None;
    }
}

fn display_file(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<State>, mut materials: ResMut<Assets<StandardMaterial>>){
    let edit_file = Path::new("./a.stl");
    let edit_file = edit_file.absolutize().unwrap();
    let edit_file = edit_file.to_str().unwrap();

    if let Some(file) = &state.file {
        match parse(&file.to_str().unwrap()) {
            Err(e) => e.print(&FileReader),
            Ok(ast) => match eval(ast) {
                Ok(mut model) => {
                    model.write_to_file(edit_file).unwrap();
                    asset_server.reload_asset(edit_file);

                    let model = commands
                        .spawn_bundle(PbrBundle {
                            mesh: asset_server.load(edit_file),
                            material: materials.add(Color::WHITE.into()),
                            transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.)),
                            ..Default::default()
                        }).id();
                    state.model = Some(model);
                }
                Err(e) => {
                    eprintln!("{:?}", e)
                }
            },
        }
    }
}

fn run_cli(args: Args) {
    match parse(&args.source) {
        Err(e) => e.print(&FileReader),
        Ok(ast) => match eval(ast) {
            Ok(mut model) => {
                model.write_to_file(&args.out).unwrap();
            }
            Err(e) => {
                eprintln!("{:?}", e)
            }
        },
    }
}
