use crate::editor::gui::menu::MenuAppExt;
use bevy::app::{App, Plugin};
use bevy_egui::EguiSettings;

pub struct ViewMenuPlugin;

impl Plugin for ViewMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_res_menu_button("View/Zoom In", |egui_settings: &mut EguiSettings| {
            egui_settings.scale_factor += 0.1
        })
        .add_res_menu_button("View/Zoom Out", |egui_settings: &mut EguiSettings| {
            egui_settings.scale_factor -= 0.1
        });
    }
}
