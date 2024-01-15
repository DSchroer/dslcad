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
use bevy::log::LogPlugin;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
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

    fn part(index: usize) -> Color {
        let mut rng = StdRng::seed_from_u64(index as u64);

        let h = rng.gen_range(0..360) as f32;
        let s = rng.gen_range(20..50) as f32 / 100.;
        let l = rng.gen_range(40..90) as f32 / 100.;

        Color::hsl(h, s, l)
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
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: persistence::constants::FULL_NAME.to_string(),
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
            move |mut console: ResMut<gui::Console>, mut re: EventWriter<RenderCommand>| {
                let rx = rx.lock().unwrap();
                match rx.try_recv() {
                    Ok(PreviewEvent::Rendering) => {
                        console.clear();
                        console.print("Rendering...".to_string());
                    }
                    Ok(PreviewEvent::Render(render)) => {
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
