mod help;
mod menu;
mod projects;
mod view_menu;

use crate::editor::State;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::editor::gui::help::HelpPlugin;
use crate::editor::gui::menu::MenuPlugin;
pub use crate::editor::gui::projects::Project;
use crate::editor::gui::projects::ProjectsPlugin;
use crate::editor::gui::view_menu::ViewMenuPlugin;
use dslcad_api::protocol::Part;

pub struct GuiPlugin {
    cheetsheet: String,
}

impl GuiPlugin {
    pub fn new(cheetsheet: String) -> Self {
        Self { cheetsheet }
    }
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiEvent>()
            .insert_resource(CheatSheet {
                cheetsheet: self.cheetsheet.clone(),
            })
            .add_plugin(EguiPlugin)
            .add_plugin(ProjectsPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(ViewMenuPlugin)
            .add_plugin(HelpPlugin::default())
            .add_system(console_panel);
    }
}

#[derive(Resource)]
struct CheatSheet {
    cheetsheet: String,
}

pub enum UiEvent {
    Render { path: String },
    ReRender(),
    Export(),
}

fn console_panel(state: Res<State>, mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>) {
    egui::TopBottomPanel::bottom("Console").show(egui_ctx.single_mut().get_mut(), |ui| {
        ui.heading("Console:");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(256.)
            .max_width(f32::INFINITY)
            .auto_shrink([false, true])
            .show(ui, |ui| match state.as_ref() {
                State::Empty => {}
                State::Rendered { output, .. } => {
                    match output {
                        Ok(o) => {
                            for part in &o.parts {
                                if let Part::Data { text } = part {
                                    ui.monospace(text);
                                }
                            }
                        }
                        Err(e) => {
                            ui.monospace(&e.to_string());
                        }
                    };
                }
            });
    });
}
