mod camera;
mod gui;
mod rendering;
mod stl;
mod xyz;

use bevy::prelude::*;
use bevy_polyline::prelude::*;
use std::error::Error;

use crate::editor::camera::CameraCommand;
use crate::editor::rendering::RenderCommand;
use crate::settings::Settings;
use crate::PreviewEvent;
use bevy::log::LogPlugin;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;

struct Blueprint;
impl Blueprint {
    fn white() -> Color {
        Srgba::hex("CED8F7").unwrap().into()
    }

    fn blue() -> Color {
        Srgba::hex("3057E1").unwrap().into()
    }

    fn black() -> Color {
        Srgba::hex("002082").unwrap().into()
    }

    fn part(index: usize) -> Color {
        Color::hsl(index as f32 * 60.0 % 360.0, 1.0, 0.5)
    }
}

pub(crate) fn main(
    cheatsheet: String,
    rx: Receiver<PreviewEvent>,
    store: Settings,
) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    let rx = Mutex::new(rx);

    app
        //.insert_resource(Msaa::default())
        .insert_resource(ClearColor(Blueprint::blue()))
        .insert_resource(store)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: dslcad_storage::constants::FULL_NAME.to_string(),
                        canvas: Some("#dslcad".to_string()),
                        fit_canvas_to_parent: true,
                        ..Default::default()
                    }),
                    ..default()
                })
                .disable::<LogPlugin>(),
        )
        .add_plugins((
            PolylinePlugin,
            camera::CameraPlugin,
            gui::GuiPlugin::new(cheatsheet),
            xyz::XYZPlugin,
            rendering::ModelRenderingPlugin,
        ))
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

    app.run();
    Ok(())
}
