use crate::editor::gui::menu::MenuAppExt;
use bevy::app::{App, Plugin};
use bevy_egui::EguiSettings;

pub struct ViewMenuPlugin;

impl Plugin for ViewMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_persistent_res_loader("zoom", |value, egui_settings: &mut EguiSettings| {
            if let Some(scale) = value.and_then(|v| v.parse().ok()) {
                egui_settings.scale_factor = scale
            }
        });
        app.add_persistent_res_menu_button(
            "View/UI Zoom In",
            "zoom",
            |egui_settings: &mut EguiSettings| {
                egui_settings.scale_factor += 0.1;
                egui_settings.scale_factor.to_string()
            },
        )
        .add_persistent_res_menu_button(
            "View/UI Zoom Out",
            "zoom",
            |egui_settings: &mut EguiSettings| {
                egui_settings.scale_factor -= 0.1;
                egui_settings.scale_factor.to_string()
            },
        );
    }
}
