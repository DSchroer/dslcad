mod help;
mod menu;
mod view_menu;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::editor::camera::CameraCommand;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::editor::gui::help::HelpPlugin;
use crate::editor::gui::menu::{MenuAppExt, MenuPlugin};
use crate::editor::gui::view_menu::ViewMenuPlugin;

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
        app.insert_resource(CheatSheet {
            cheetsheet: self.cheetsheet.clone(),
        })
        .insert_resource(Console { text: None })
        .add_plugins(EguiPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(ViewMenuPlugin)
        .add_event_menu_button("Camera/Focus", |c: &mut EventWriter<CameraCommand>| {
            c.send(CameraCommand::Refocus());
        })
        .add_event_menu_button("Camera/Reset", |c: &mut EventWriter<CameraCommand>| {
            c.send(CameraCommand::Reset());
        })
        .add_plugins(HelpPlugin::default())
        .add_systems(Update, console_panel);
    }
}

#[derive(Resource)]
pub struct Console {
    text: Option<String>,
}

impl Console {
    pub fn print(&mut self, text: String) {
        self.text = Some(text);
    }

    pub fn clear(&mut self) {
        self.text = None;
    }
}

#[derive(Resource)]
struct CheatSheet {
    cheetsheet: String,
}

fn console_panel(
    console: Res<Console>,
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
) {
    egui::TopBottomPanel::bottom("Console").show(egui_ctx.single_mut().get_mut(), |ui| {
        ui.heading("Console:");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(256.)
            .max_width(f32::INFINITY)
            .auto_shrink([false, true])
            .show(ui, |ui| match &console.text {
                None => ui.monospace(""),
                Some(t) => ui.monospace(t),
            });
    });
}
