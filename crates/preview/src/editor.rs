mod camera;
mod gui;
mod rendering;
mod stl;
mod xyz;

use bevy::prelude::*;
use bevy_polyline::prelude::*;
use std::error::Error;

use crate::editor::rendering::RenderCommand;
use crate::settings::Settings;
use crate::PreviewEvent;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;

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

pub(crate) fn main(
    cheatsheet: String,
    rx: Receiver<PreviewEvent>,
    store: Settings,
) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    let rx = Mutex::new(rx);

    app.insert_resource(Msaa::default())
        .insert_resource(ClearColor(Blueprint::blue()))
        .insert_resource(store)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: persistence::constants::FULL_NAME.to_string(),
                canvas: Some("#dslcad".to_string()),
                fit_canvas_to_parent: true,
                ..Default::default()
            }),
            ..default()
        }))
        .add_plugin(PolylinePlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(gui::GuiPlugin::new(cheatsheet))
        .add_plugin(xyz::XYZPlugin)
        .add_plugin(rendering::ModelRenderingPlugin)
        .add_system(
            move |mut console: ResMut<gui::Console>, mut re: EventWriter<RenderCommand>| {
                let rx = rx.lock().unwrap();
                match rx.try_recv() {
                    Ok(PreviewEvent::Render(render)) => {
                        re.send(RenderCommand::Draw(render));
                        console.clear();
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
