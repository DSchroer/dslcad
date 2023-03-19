use crate::editor::Blueprint;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use smooth_bevy_cameras::controllers::orbit::{
    ControlEvent, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LookTransformPlugin)
            .add_plugin(OrbitCameraPlugin::new(true))
            .add_startup_system(camera_system)
            .add_system(camera_light)
            .add_system(input_map);
    }
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
        transform: Transform::from_translation(Vec3::new(100.0, 100.0, 100.0))
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Blueprint::blue(),
        brightness: 0.2,
    });
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

    if keyboard.pressed(KeyCode::LShift) {
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
