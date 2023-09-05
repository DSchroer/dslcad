use crate::editor::gui::menu::MenuAppExt;
use crate::editor::gui::CheatSheet;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};

#[derive(Resource, Clone, Default)]
pub struct HelpPlugin {
    about_window: bool,
    cheatsheet_window: bool,
}

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_res_menu_button("Help/Cheat Sheet", |state: &mut HelpPlugin| {
                state.cheatsheet_window = true
            })
            .add_res_menu_button("Help/About", |state: &mut HelpPlugin| {
                state.about_window = true
            })
            .add_system(about)
            .add_system(cheatsheet);
    }
}

fn about(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut state: ResMut<HelpPlugin>,
) {
    egui::Window::new("About")
        .open(&mut state.about_window)
        .show(egui_ctx.single_mut().get_mut(), |ui| {
            ui.label(threemf::constants::FULL_NAME);
            ui.separator();
            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            ui.label("Copyright: Dominick Schroer 2022");
        });
}

fn cheatsheet(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut state: ResMut<HelpPlugin>,
    cheetsheet: Res<CheatSheet>,
) {
    egui::Window::new("Cheat Sheet")
        .open(&mut state.cheatsheet_window)
        .show(egui_ctx.single_mut().get_mut(), |ui| {
            egui::ScrollArea::vertical()
                .max_height(512.)
                .show(ui, |ui| ui.monospace(&cheetsheet.cheetsheet));
        });
}
