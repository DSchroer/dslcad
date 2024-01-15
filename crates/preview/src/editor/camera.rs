use crate::editor::Blueprint;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use persistence::protocol::BoundingBox;
use smooth_bevy_cameras::controllers::orbit::{
    ControlEvent, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::{LookTransform, LookTransformPlugin};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LookTransformPlugin)
            .add_event::<CameraCommand>()
            .insert_resource(CameraState::default())
            .add_plugins(OrbitCameraPlugin::new(true))
            .add_systems(Startup, camera_system)
            .add_systems(Update, camera_light)
            .add_systems(Update, camera_handler)
            .add_systems(Update, input_map);
    }
}

#[derive(Default, Resource)]
struct CameraState {
    focus: Option<BoundingBox>,
}

#[derive(Event)]
pub enum CameraCommand {
    Refocus(),
    Focus(BoundingBox),
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

fn camera_system(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_translate_sensitivity: Vec2::splat(0.05),
                mouse_rotate_sensitivity: Vec2::splat(0.5),
                ..Default::default()
            },
            Vec3::new(100.0, 100.0, 100.0),
            Vec3::new(0., 0., 0.),
            Vec3::new(0., 1., 0.),
        ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 100000.0,
            color: Blueprint::white(),
            ..default()
        },
        transform: Transform::from_translation(Vec3::splat(100.))
            .looking_at(Vec3::default(), Vec3::Y),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Blueprint::blue(),
        brightness: 0.2,
    });
}

fn camera_handler(
    mut camera_commands: EventReader<CameraCommand>,
    mut camera: Query<&mut LookTransform, With<OrbitCameraController>>,
    mut state: ResMut<CameraState>,
) {
    for command in camera_commands.iter() {
        match command {
            CameraCommand::Focus(aabb) => {
                if state.focus.is_none() {
                    focus_on(&mut camera, aabb);
                }

                state.focus = Some(aabb.clone());
            }
            CameraCommand::Refocus() => {
                if let Some(aabb) = &state.focus {
                    focus_on(&mut camera, aabb);
                } else {
                    let mut transform = camera.single_mut();
                    transform.target = Vec3::default();
                    transform.eye = Vec3::splat(100.);
                }
            }
        }
    }
}

fn focus_on(
    camera: &mut Query<&mut LookTransform, With<OrbitCameraController>>,
    aabb: &BoundingBox,
) {
    let mut transform = camera.single_mut();
    let center = aabb.center();
    transform.target = Vec3::new(center[1] as f32, center[2] as f32, center[0] as f32);
    transform.eye = transform.target + Vec3::splat(f32::max(aabb.max_len() as f32 * 2., 1.));
}

#[allow(clippy::too_many_arguments)]
pub fn input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    keyboard: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    controllers: Query<&OrbitCameraController>,
    camera_pos: Query<&Transform, With<OrbitCameraController>>,
) {
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        mouse_wheel_zoom_sensitivity,
        pixels_per_line,
        ..
    } = *controller;

    let mut binding = egui_ctx.single_mut();
    let ctx = binding.get_mut();
    if ctx.is_using_pointer() || ctx.is_pointer_over_area() {
        mouse_wheel_reader.clear();
        mouse_motion_events.clear();
        return;
    }

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
    }

    let camera_pos = camera_pos.single();
    let zoom_amount = camera_pos.translation.distance(Vec3::ZERO);
    if mouse_buttons.pressed(MouseButton::Right) {
        events.send(ControlEvent::TranslateTarget(
            mouse_translate_sensitivity * cursor_delta * zoom_amount,
        ));
    }

    if keyboard.pressed(KeyCode::Minus) {
        events.send(ControlEvent::Zoom(1.1));
    }
    if keyboard.pressed(KeyCode::Plus) || keyboard.pressed(KeyCode::Equals) {
        events.send(ControlEvent::Zoom(0.9));
    }

    if keyboard.pressed(KeyCode::ShiftLeft) {
        if keyboard.pressed(KeyCode::Left) {
            events.send(ControlEvent::TranslateTarget(Vec2::new(
                1. * zoom_amount,
                0.0,
            )));
        }
        if keyboard.pressed(KeyCode::Right) {
            events.send(ControlEvent::TranslateTarget(Vec2::new(
                -1. * zoom_amount,
                0.0,
            )));
        }
        if keyboard.pressed(KeyCode::Up) {
            events.send(ControlEvent::TranslateTarget(Vec2::new(
                0.0,
                1. * zoom_amount,
            )));
        }
        if keyboard.pressed(KeyCode::Down) {
            events.send(ControlEvent::TranslateTarget(Vec2::new(
                0.0,
                -1. * zoom_amount,
            )));
        }
    } else {
        if keyboard.pressed(KeyCode::Left) {
            events.send(ControlEvent::Orbit(Vec2::new(1., 0.0)));
        }
        if keyboard.pressed(KeyCode::Right) {
            events.send(ControlEvent::Orbit(Vec2::new(-1., 0.0)));
        }
        if keyboard.pressed(KeyCode::Up) {
            events.send(ControlEvent::Orbit(Vec2::new(0., 1.0)));
        }
        if keyboard.pressed(KeyCode::Down) {
            events.send(ControlEvent::Orbit(Vec2::new(0., -1.0)));
        }
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.iter() {
        // scale the event magnitude per pixel or per line
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / pixels_per_line,
        };
        scalar *= 1.0 - scroll_amount * mouse_wheel_zoom_sensitivity;
    }
    events.send(ControlEvent::Zoom(scalar));
}
