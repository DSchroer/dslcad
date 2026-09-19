mod camera;
mod gizmo;
mod gui;
mod lines;
mod rendering;
mod stl;
mod xyz;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use smooth_bevy_cameras::controllers::orbit::OrbitCameraController;
use smooth_bevy_cameras::{LookTransform, Smoother};
use std::error::Error;
use std::time::Duration;

use crate::editor::camera::CameraCommand;
use crate::editor::rendering::RenderCommand;
use crate::settings::Settings;
use crate::{AxisAngles, PreviewEvent, ScreenshotOptions};
use bevy::log::LogPlugin;
use dslcad_storage::protocol::BoundingBox;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;

const SCREENSHOT_SIZE: u32 = 1024;
const SCREENSHOT_WARMUP_FRAMES: u32 = 10;
const DEFAULT_SCREENSHOT_TILT: f32 = 54.736;
const DEFAULT_SCREENSHOT_AZIMUTH: f32 = 45.0;

/// Colors used to render the preview. The palette is designed for a dark
/// viewport, similar to a CAD application.
struct Palette;

impl Palette {
    /// The viewport background.
    fn background() -> Color {
        Srgba::hex("434B57").unwrap().into()
    }

    /// The default color of a rendered part.
    fn part() -> Color {
        Srgba::hex("CED8F7").unwrap().into()
    }

    /// A distinct color for each part when part colors are enabled.
    fn part_color(index: usize) -> Color {
        Color::hsl(index as f32 * 60.0 % 360.0, 0.8, 0.55)
    }

    /// Edges and vertices drawn on top of a part's mesh.
    fn edge() -> Color {
        Srgba::hex("141A23").unwrap().into()
    }

    /// Lines and points drawn directly on the viewport background.
    fn wireframe() -> Color {
        Srgba::hex("E8EDF5").unwrap().into()
    }

    /// Minor grid lines.
    fn grid_minor() -> Color {
        Srgba::hex("4C5563").unwrap().into()
    }

    /// Major grid lines.
    fn grid_major() -> Color {
        Srgba::hex("5A6575").unwrap().into()
    }

    /// The color of a positive coordinate axis.
    fn axis(axis: Axis) -> Color {
        match axis {
            Axis::X => Srgba::hex("E5484D").unwrap().into(),
            Axis::Y => Srgba::hex("46A758").unwrap().into(),
            Axis::Z => Srgba::hex("3E8BFF").unwrap().into(),
        }
    }

    /// The color of a negative coordinate axis.
    fn negative_axis(axis: Axis) -> Color {
        match axis {
            Axis::X => Srgba::hex("8A4A4E").unwrap().into(),
            Axis::Y => Srgba::hex("416B4C").unwrap().into(),
            Axis::Z => Srgba::hex("40618C").unwrap().into(),
        }
    }

    /// The color of the directional light.
    fn light() -> Color {
        Color::WHITE
    }

    /// The color of the ambient light.
    fn ambient() -> Color {
        Srgba::hex("3A4A63").unwrap().into()
    }
}

/// A coordinate axis of a part.
///
/// Parts are modeled in a z-up space while the renderer is y-up, so every axis
/// points in a different direction once the model is rotated by
/// [`model_rotation`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    const ALL: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    /// The direction of the axis in the renderer's y-up space.
    fn direction(self) -> Vec3 {
        model_rotation()
            * match self {
                Axis::X => Vec3::X,
                Axis::Y => Vec3::Y,
                Axis::Z => Vec3::Z,
            }
    }

    /// The label shown for the axis.
    fn label(self) -> &'static str {
        match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        }
    }
}

/// The rotation that maps the z-up part space onto the y-up renderer.
fn model_rotation() -> Quat {
    Quat::from_euler(
        EulerRot::XYZ,
        -std::f32::consts::FRAC_PI_2,
        0.0,
        -std::f32::consts::FRAC_PI_2,
    )
}

pub(crate) fn main(
    cheatsheet: String,
    rx: Receiver<PreviewEvent>,
    store: Settings,
) -> Result<(), Box<dyn Error>> {
    run(cheatsheet, rx, store, None)
}

pub(crate) fn screenshot(
    rx: Receiver<PreviewEvent>,
    options: ScreenshotOptions,
) -> Result<(), Box<dyn Error>> {
    run(
        String::new(),
        rx,
        Settings::default(),
        Some(options.clone()),
    )?;

    if !options.path.exists() {
        return Err(format!("failed to save screenshot to {}", options.path.display()).into());
    }

    Ok(())
}

fn run(
    cheatsheet: String,
    rx: Receiver<PreviewEvent>,
    store: Settings,
    screenshot: Option<ScreenshotOptions>,
) -> Result<(), Box<dyn Error>> {
    let interactive = screenshot.is_none();
    let mut app = App::new();

    let rx = Mutex::new(rx);

    let window = interactive.then(|| Window {
        title: dslcad_storage::constants::FULL_NAME.to_string(),
        canvas: Some("#dslcad".to_string()),
        fit_canvas_to_parent: true,
        ..Default::default()
    });

    let mut default_plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: window,
            exit_condition: if interactive {
                ExitCondition::OnAllClosed
            } else {
                ExitCondition::DontExit
            },
            ..default()
        })
        .disable::<LogPlugin>();

    if !interactive {
        // The screenshot is rendered to an offscreen image so no window is needed.
        default_plugins = default_plugins.disable::<WinitPlugin>();
    }

    app.insert_resource(ClearColor(Palette::background()))
        .insert_resource(store)
        .add_plugins(default_plugins)
        .add_plugins((
            camera::CameraPlugin,
            xyz::XYZPlugin,
            rendering::ModelRenderingPlugin,
        ));

    if interactive {
        app.add_plugins(gui::GuiPlugin::new(cheatsheet))
            .add_systems(
                Update,
                move |mut console: ResMut<gui::Console>,
                      mut re: EventWriter<RenderCommand>,
                      mut ca: EventWriter<CameraCommand>| {
                    let rx = rx.lock().unwrap();
                    match rx.try_recv() {
                        Ok(PreviewEvent::Rendering) => {
                            console.clear();
                            console.print("Rendering...".to_string());
                        }
                        Ok(PreviewEvent::Render(render)) => {
                            if let Some(aabb) = render.aabb() {
                                ca.send(CameraCommand::Focus(aabb));
                            }
                            console.clear();
                            console.print(render.stdout);
                            re.send(RenderCommand::Draw(render.parts));
                        }
                        Ok(PreviewEvent::Error(e)) => {
                            console.print(e);
                        }
                        _ => {}
                    }
                },
            );
    } else {
        let options = screenshot.unwrap();
        app.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(16)))
            .insert_resource(ScreenshotState {
                options,
                received: false,
                aabb: None,
                frames: 0,
                captured: false,
            })
            .add_systems(Startup, setup_screenshot.after(camera::camera_system))
            .add_systems(
                Update,
                (
                    move |mut re: EventWriter<RenderCommand>,
                          mut state: ResMut<ScreenshotState>| {
                        let rx = rx.lock().unwrap();
                        match rx.try_recv() {
                            Ok(PreviewEvent::Render(render)) => {
                                state.received = true;
                                state.aabb = render.aabb();
                                re.send(RenderCommand::Draw(render.parts));
                            }
                            Ok(PreviewEvent::Error(e)) => {
                                eprintln!("{}", e);
                            }
                            _ => {}
                        }
                    },
                    capture_screenshot,
                )
                    .chain(),
            );
    }

    app.run();
    Ok(())
}

#[derive(Resource)]
struct ScreenshotState {
    options: ScreenshotOptions,
    received: bool,
    aabb: Option<BoundingBox>,
    frames: u32,
    captured: bool,
}

#[derive(Resource)]
struct ScreenshotTarget(Handle<Image>);

fn setup_screenshot(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<&mut Camera, With<OrbitCameraController>>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SCREENSHOT_SIZE,
            height: SCREENSHOT_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage = TextureUsages::COPY_SRC
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT
        | TextureUsages::TEXTURE_BINDING;

    let handle = images.add(image);
    cameras.single_mut().target = RenderTarget::Image(handle.clone());
    commands.insert_resource(ScreenshotTarget(handle));
}

fn capture_screenshot(
    mut commands: Commands,
    mut state: ResMut<ScreenshotState>,
    target: Res<ScreenshotTarget>,
    mut cameras: Query<
        (&mut LookTransform, &mut Transform, &mut Smoother),
        With<OrbitCameraController>,
    >,
) {
    if !state.received || state.captured {
        return;
    }

    if let (Some(aabb), Ok((mut look, mut transform, mut smoother))) =
        (state.aabb.as_ref(), cameras.get_single_mut())
    {
        let (target_position, eye, up) =
            screenshot_view(aabb, state.options.angle, state.options.zoom);
        look.target = target_position;
        look.eye = eye;
        look.up = up;
        *transform = (*look).into();
        *smoother = Smoother::new(0.0);
    }

    state.frames += 1;
    if state.frames < SCREENSHOT_WARMUP_FRAMES {
        return;
    }

    state.captured = true;
    let path = state.options.path.clone();
    commands
        .spawn(Screenshot::image(target.0.clone()))
        .observe(save_to_disk(path))
        .observe(
            |_: Trigger<ScreenshotCaptured>, mut exit: EventWriter<AppExit>| {
                exit.send(AppExit::Success);
            },
        );
}
/// Position the camera on a sphere around the part. `x` tilts from the top,
/// `y` is an azimuth measured from the x-axis and `z` rolls the camera. Zoom
/// scales the fit distance.
fn screenshot_view(
    aabb: &BoundingBox,
    angles: AxisAngles,
    zoom: Option<f32>,
) -> (Vec3, Vec3, Vec3) {
    let center = aabb.center();
    let target = Vec3::new(center[1] as f32, center[2] as f32, center[0] as f32);

    let distance = f32::max(aabb.max_len() as f32 * 2.0, 1.0) * 3.0_f32.sqrt()
        / zoom.unwrap_or(1.0).max(f32::EPSILON);

    let tilt = angles.x.unwrap_or(DEFAULT_SCREENSHOT_TILT).to_radians();
    let azimuth = angles.y.unwrap_or(DEFAULT_SCREENSHOT_AZIMUTH).to_radians();
    let roll = angles.z.unwrap_or(0.0).to_radians();

    let direction = Vec3::new(
        tilt.sin() * azimuth.cos(),
        tilt.sin() * azimuth.sin(),
        tilt.cos(),
    );
    // Part space is z-up, bevy is y-up: (x, y, z) -> (y, z, x)
    let direction = Vec3::new(direction.y, direction.z, direction.x);

    // Top and bottom views look along the y-axis, so they need another up vector
    let up = if direction.dot(Vec3::Y).abs() > 0.999 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let up = Quat::from_axis_angle(direction, roll) * up;

    (target, target + direction * distance, up)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dslcad_storage::protocol::{Part, Render};

    fn aabb() -> BoundingBox {
        Render {
            parts: vec![Part::Planar {
                points: vec![[0., 0., 0.], [1., 1., 1.]],
                lines: vec![],
            }],
            stdout: String::new(),
        }
        .aabb()
        .unwrap()
    }

    #[test]
    fn default_view_matches_preview_focus() {
        let (target, eye, _) = screenshot_view(&aabb(), AxisAngles::default(), None);

        assert_eq!(target, Vec3::splat(0.5));
        // Preview focuses at `Vec3::splat(max_len * 2)` from the target
        assert!((eye - target - Vec3::splat(2.0)).length() < 1e-4);
    }

    #[test]
    fn axis_directions_match_the_rendered_model() {
        // Part x points along renderer z, part y along renderer x and part z up
        assert!((Axis::X.direction() - Vec3::Z).length() < 1e-6);
        assert!((Axis::Y.direction() - Vec3::X).length() < 1e-6);
        assert!((Axis::Z.direction() - Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn angle_rotates_the_camera_around_the_part() {
        let (_, default_eye, _) = screenshot_view(&aabb(), AxisAngles::default(), None);
        let angles = AxisAngles {
            y: Some(45.0 + 90.0),
            ..Default::default()
        };
        let (_, rotated_eye, _) = screenshot_view(&aabb(), angles, None);

        // Both views keep the same distance but point from perpendicular azimuths
        assert!(
            ((default_eye - Vec3::splat(0.5)).length() - (rotated_eye - Vec3::splat(0.5)).length())
                .abs()
                < 1e-4
        );
        assert!((default_eye - rotated_eye).length() > 1.0);
    }

    #[test]
    fn zoom_scales_the_distance() {
        let (target, eye, _) = screenshot_view(&aabb(), AxisAngles::default(), Some(2.0));
        assert!(((eye - target).length() - 3.0_f32.sqrt()).abs() < 1e-4);
    }

    #[test]
    fn top_view_uses_a_valid_up_vector() {
        let (target, eye, up) = screenshot_view(
            &aabb(),
            AxisAngles {
                x: Some(0.0),
                ..Default::default()
            },
            None,
        );

        // Looking straight down the bevy y-axis from above
        let offset = eye - target;
        assert!(offset.x.abs() < 1e-4 && offset.z.abs() < 1e-4 && offset.y > 0.5);
        assert!(up.dot(Vec3::Y).abs() < 1e-4);
    }
}
