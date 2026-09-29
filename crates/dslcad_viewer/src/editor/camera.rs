use crate::editor::rendering::{RenderCommand, RenderState};
use crate::editor::Palette;
use bevy::camera::ScalingMode;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_egui::{EguiContext, PrimaryEguiContext};
use dslcad_storage::protocol::{BoundingBox, Projection as ViewProjection, ViewAngles};
use smooth_bevy_cameras::controllers::orbit::{
    ControlMessage, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::{LookTransform, LookTransformPlugin, Smoother};

/// The isometric tilt used when a view does not specify one.
const DEFAULT_TILT: f32 = 54.736;
const DEFAULT_AZIMUTH: f32 = 45.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LookTransformPlugin)
            .add_message::<CameraCommand>()
            .insert_resource(CameraState::default())
            .add_plugins(OrbitCameraPlugin::new(true))
            .add_systems(Startup, camera_system)
            .add_systems(Update, camera_light)
            .add_systems(Update, camera_handler)
            .add_systems(Update, orthographic_zoom)
            .add_systems(Update, normalize_free_camera_up)
            .add_systems(Update, view_shortcuts)
            .add_systems(Update, input_map);
    }
}

#[derive(Default, Resource)]
struct CameraState {
    focus: Option<BoundingBox>,
}

#[derive(Message)]
pub enum CameraCommand {
    Refocus(),
    Reset(),
    Focus(BoundingBox),
    UseOrthographic(bool),
    /// Move to a named view's camera.
    View {
        angles: ViewAngles,
        projection: ViewProjection,
        zoom: Option<f32>,
        target: Option<[f64; 3]>,
    },
}

fn camera_light(
    query: Query<&Transform, With<OrbitCameraController>>,
    mut light: Query<&mut Transform, (With<DirectionalLight>, Without<OrbitCameraController>)>,
) {
    let Ok(gxf) = query.single() else {
        return;
    };
    for mut transform in light.iter_mut() {
        transform.clone_from(gxf);
        // Tilt the light off the view direction so faces shade differently
        transform.rotate_local_x(-0.35);
        transform.rotate_local_y(0.25);
    }
}

pub(crate) fn camera_system(mut commands: Commands) {
    commands
        .spawn(Camera3d::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_translate_sensitivity: Vec2::splat(0.05),
                mouse_rotate_sensitivity: Vec2::splat(0.5),
                ..Default::default()
            },
            Vec3::new(100.0, 100.0, 100.0),
            Vec3::new(0., 0., 0.),
            Vec3::new(0., 1., 0.),
        ))
        // Antialias the thin, screen-space line quads so drawing outlines do
        // not shimmer while the camera moves.
        .insert(Msaa::Sample4);

    commands.spawn((
        DirectionalLight {
            illuminance: 30000.0,
            color: Palette::light(),
            ..default()
        },
        Transform::from_translation(Vec3::splat(100.)).looking_at(Vec3::default(), Vec3::Y),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Palette::ambient(),
        brightness: 0.35,
        affects_lightmapped_meshes: true,
    });
}

fn camera_handler(
    mut camera_commands: MessageReader<CameraCommand>,
    mut projection: Query<&mut Projection>,
    mut camera: Query<&mut LookTransform, With<OrbitCameraController>>,
    mut state: ResMut<CameraState>,
) {
    for command in camera_commands.read() {
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
                    let Ok(mut transform) = camera.single_mut() else {
                        return;
                    };
                    transform.target = Vec3::default();
                    transform.eye = Vec3::splat(100.);
                }
            }
            CameraCommand::Reset() => {
                let Ok(mut transform) = camera.single_mut() else {
                    return;
                };
                transform.target = Vec3::default();
                transform.eye = Vec3::splat(100.);
            }
            CameraCommand::UseOrthographic(orthographic) => {
                let Ok(mut projection) = projection.single_mut() else {
                    return;
                };
                if *orthographic {
                    *projection = Projection::Orthographic(OrthographicProjection::default_3d());
                } else {
                    *projection = Projection::Perspective(PerspectiveProjection::default());
                }
            }
            CameraCommand::View {
                angles,
                projection: kind,
                zoom,
                target,
            } => {
                let Ok(mut current_projection) = projection.single_mut() else {
                    return;
                };
                *current_projection = match kind {
                    ViewProjection::Perspective => {
                        Projection::Perspective(PerspectiveProjection::default())
                    }
                    ViewProjection::Orthographic => {
                        Projection::Orthographic(OrthographicProjection::default_3d())
                    }
                };

                let Some(aabb) = state.focus.clone() else {
                    return;
                };

                let center = aabb.center();
                let center = Vec3::new(center[1] as f32, center[2] as f32, center[0] as f32);
                let look = match target {
                    Some([x, y, z]) => Vec3::new(*y as f32, *z as f32, *x as f32),
                    None => center,
                };

                let distance = f32::max(aabb.max_len() as f32 * 2.0, 1.0) * 3.0_f32.sqrt()
                    / zoom.unwrap_or(1.0).max(f32::EPSILON);

                let tilt = angles.x.unwrap_or(DEFAULT_TILT).to_radians();
                let azimuth = angles.y.unwrap_or(DEFAULT_AZIMUTH).to_radians();
                let roll = angles.z.unwrap_or(0.0).to_radians();

                let direction = Vec3::new(
                    tilt.sin() * azimuth.cos(),
                    tilt.sin() * azimuth.sin(),
                    tilt.cos(),
                );
                // Part space is z-up, bevy is y-up: (x, y, z) -> (y, z, x)
                let direction = Vec3::new(direction.y, direction.z, direction.x);

                let up = if direction.dot(Vec3::Y).abs() > 0.999 {
                    Vec3::X
                } else {
                    Vec3::Y
                };
                let up = Quat::from_axis_angle(direction, roll) * up;

                let Ok(mut transform) = camera.single_mut() else {
                    return;
                };
                transform.target = look;
                transform.eye = look + direction * distance;
                transform.up = up;
            }
        }
    }
}

fn orthographic_zoom(
    mut projections: Query<&mut Projection>,
    look: Query<&LookTransform>,
    smoother: Query<&Smoother>,
) {
    let Ok(transform) = look.single() else {
        return;
    };
    let Ok(smoother) = smoother.single() else {
        return;
    };
    let mut smoother = *smoother;
    let transform = smoother.smooth_transform(transform);
    let d = transform.target.distance(transform.eye);
    for mut projection in &mut projections {
        if let Projection::Orthographic(o) = projection.as_mut() {
            o.far = d + 1000.0;
            o.scaling_mode = ScalingMode::FixedVertical { viewport_height: d };
            o.scale = 1.0;
        }
    }
}

/// A named view may roll the camera (for example the top view). Once we are
/// back on the free camera, restore the world up so orbiting behaves normally.
fn normalize_free_camera_up(
    render_state: Res<RenderState>,
    mut cameras: Query<&mut LookTransform, With<OrbitCameraController>>,
) {
    if render_state.active_view_index().is_some() {
        return;
    }

    for mut look in &mut cameras {
        let direction = (look.eye - look.target).normalize_or_zero();
        // Leave the up alone while looking straight up or down, where it is
        // undefined.
        if direction.y.abs() < 0.98 {
            look.up = Vec3::Y;
        }
    }
}

/// Number keys switch to a declared view: `1` selects the first view, `2` the
/// second, and so on, while `0` returns to the free camera.
fn view_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>,
    mut state: ResMut<RenderState>,
    mut render_events: MessageWriter<RenderCommand>,
    mut camera_events: MessageWriter<CameraCommand>,
) {
    if let Ok(mut binding) = egui_ctx.single_mut() {
        if binding.get_mut().wants_keyboard_input() {
            return;
        }
    }

    let pressed = |key: KeyCode| {
        keyboard.just_pressed(key)
            && !keyboard.pressed(KeyCode::ControlLeft)
            && !keyboard.pressed(KeyCode::ControlRight)
            && !keyboard.pressed(KeyCode::AltLeft)
            && !keyboard.pressed(KeyCode::AltRight)
    };

    if pressed(KeyCode::Digit0) {
        state.set_active_view(None);
        render_events.write(RenderCommand::Redraw);
        return;
    }

    const DIGITS: [KeyCode; 9] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    for (index, key) in DIGITS.into_iter().enumerate() {
        if !pressed(key) {
            continue;
        }

        let Some(view) = state.views().get(index).cloned() else {
            return;
        };

        state.set_active_view(Some(index));
        render_events.write(RenderCommand::Redraw);
        camera_events.write(CameraCommand::View {
            angles: view.angle,
            projection: view.projection,
            zoom: view.zoom,
            target: view.target,
        });
    }
}

fn focus_on(
    camera: &mut Query<&mut LookTransform, With<OrbitCameraController>>,
    aabb: &BoundingBox,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let center = aabb.center();
    transform.target = Vec3::new(center[1] as f32, center[2] as f32, center[0] as f32);
    transform.eye = transform.target + Vec3::splat(f32::max(aabb.max_len() as f32 * 2., 1.));
}

#[allow(clippy::too_many_arguments)]
pub fn input_map(
    mut events: MessageWriter<ControlMessage>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>,
    mut render_state: ResMut<RenderState>,
    mut render_events: MessageWriter<RenderCommand>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
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

    let Ok(mut binding) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = binding.get_mut();
    if ctx.is_using_pointer() || ctx.is_pointer_over_area() {
        mouse_wheel_reader.clear();
        mouse_motion_events.clear();
        return;
    }

    // Any real camera adjustment leaves a fixed view and returns to the free
    // camera.
    let mut adjusted = false;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        adjusted |= cursor_delta != Vec2::ZERO;
        events.write(ControlMessage::Orbit(
            mouse_rotate_sensitivity * cursor_delta,
        ));
    }

    let Ok(camera_pos) = camera_pos.single() else {
        return;
    };
    let zoom_amount = camera_pos.translation.distance(Vec3::ZERO);
    if mouse_buttons.pressed(MouseButton::Right) {
        adjusted |= cursor_delta != Vec2::ZERO;
        events.write(ControlMessage::TranslateTarget(
            mouse_translate_sensitivity * cursor_delta * zoom_amount,
        ));
    }

    if keyboard.pressed(KeyCode::Equal) {
        adjusted = true;
        events.write(ControlMessage::Zoom(0.9));
    }
    if keyboard.pressed(KeyCode::Minus) {
        adjusted = true;
        events.write(ControlMessage::Zoom(1.1));
    }

    if keyboard.pressed(KeyCode::ShiftLeft) {
        if keyboard.pressed(KeyCode::ArrowLeft) {
            adjusted = true;
            events.write(ControlMessage::TranslateTarget(Vec2::new(
                1. * zoom_amount,
                0.0,
            )));
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            adjusted = true;
            events.write(ControlMessage::TranslateTarget(Vec2::new(
                -zoom_amount,
                0.0,
            )));
        }
        if keyboard.pressed(KeyCode::ArrowUp) {
            adjusted = true;
            events.write(ControlMessage::TranslateTarget(Vec2::new(
                0.0,
                1. * zoom_amount,
            )));
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            adjusted = true;
            events.write(ControlMessage::TranslateTarget(Vec2::new(
                0.0,
                -zoom_amount,
            )));
        }
    } else {
        if keyboard.pressed(KeyCode::ArrowLeft) {
            adjusted = true;
            events.write(ControlMessage::Orbit(Vec2::new(1., 0.0)));
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            adjusted = true;
            events.write(ControlMessage::Orbit(Vec2::new(-1., 0.0)));
        }
        if keyboard.pressed(KeyCode::ArrowUp) {
            adjusted = true;
            events.write(ControlMessage::Orbit(Vec2::new(0., 1.0)));
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            adjusted = true;
            events.write(ControlMessage::Orbit(Vec2::new(0., -1.0)));
        }
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.read() {
        adjusted = true;
        // scale the event magnitude per pixel or per line
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / pixels_per_line,
        };
        scalar *= 1.0 - scroll_amount * mouse_wheel_zoom_sensitivity;
    }
    events.write(ControlMessage::Zoom(scalar));

    if adjusted && render_state.active_view_index().is_some() {
        render_state.set_active_view(None);
        render_events.write(RenderCommand::Redraw);
    }
}
