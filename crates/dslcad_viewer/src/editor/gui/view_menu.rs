use crate::editor::gui::menu::MenuAppExt;
use bevy::app::{App, Plugin};
use bevy::prelude::{Query, With};
use bevy::window::PrimaryWindow;
use bevy_egui::EguiSettings;

pub struct ViewMenuPlugin;

impl Plugin for ViewMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_persistent_res_loader::<Query<&mut EguiSettings, With<PrimaryWindow>>>(
            "zoom",
            |value: Option<&str>,
             mut egui_settings: Query<&mut EguiSettings, With<PrimaryWindow>>| {
                if let Some(scale) = value.and_then(|v| v.parse().ok()) {
                    if let Ok(mut egui_settings) = egui_settings.get_single_mut() {
                        egui_settings.scale_factor = scale
                    }
                }
            },
        );
        app.add_persistent_res_menu_button::<Query<&mut EguiSettings, With<PrimaryWindow>>>(
            "View/UI Zoom In",
            "zoom",
            |mut egui_settings: Query<&mut EguiSettings, With<PrimaryWindow>>| {
                if let Ok(mut egui_settings) = egui_settings.get_single_mut() {
                    egui_settings.scale_factor += 0.1;
                    egui_settings.scale_factor.to_string()
                } else {
                    String::new()
                }
            },
        )
        .add_persistent_res_menu_button::<Query<&mut EguiSettings, With<PrimaryWindow>>>(
            "View/UI Zoom Out",
            "zoom",
            |mut egui_settings: Query<&mut EguiSettings, With<PrimaryWindow>>| {
                if let Ok(mut egui_settings) = egui_settings.get_single_mut() {
                    egui_settings.scale_factor -= 0.1;
                    egui_settings.scale_factor.to_string()
                } else {
                    String::new()
                }
            },
        );
    }
}
