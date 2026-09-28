use crate::editor::gui::menu::MenuAppExt;
use crate::settings::{Settings, Store};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_egui::{EguiContextSettings, PrimaryEguiContext};

pub struct ViewMenuPlugin;

impl Plugin for ViewMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_stored_zoom)
            .add_persistent_res_menu_button::<
                Query<&mut EguiContextSettings, With<PrimaryEguiContext>>,
            >(
                "View/UI Zoom In",
                "zoom",
                |mut egui_settings: Query<&mut EguiContextSettings, With<PrimaryEguiContext>>| {
                    if let Ok(mut egui_settings) = egui_settings.single_mut() {
                        egui_settings.scale_factor += 0.1;
                        egui_settings.scale_factor.to_string()
                    } else {
                        String::new()
                    }
                },
            )
            .add_persistent_res_menu_button::<
                Query<&mut EguiContextSettings, With<PrimaryEguiContext>>,
            >(
                "View/UI Zoom Out",
                "zoom",
                |mut egui_settings: Query<&mut EguiContextSettings, With<PrimaryEguiContext>>| {
                    if let Ok(mut egui_settings) = egui_settings.single_mut() {
                        egui_settings.scale_factor -= 0.1;
                        egui_settings.scale_factor.to_string()
                    } else {
                        String::new()
                    }
                },
            );
    }
}

/// Applies the stored UI zoom once the primary egui context exists.
///
/// The context is created after `Startup` (when the first camera is added), so it
/// cannot be restored by the startup loaders used by the other persistent
/// settings.
fn apply_stored_zoom(
    mut applied: Local<bool>,
    store: Res<Settings>,
    mut egui_settings: Query<&mut EguiContextSettings, With<PrimaryEguiContext>>,
) {
    if *applied {
        return;
    }

    let Ok(mut egui_settings) = egui_settings.single_mut() else {
        return;
    };

    if let Some(scale) = store.load("zoom").and_then(|value| value.parse().ok()) {
        egui_settings.scale_factor = scale;
    }

    *applied = true;
}
