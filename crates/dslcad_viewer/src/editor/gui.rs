mod help;
mod menu;
mod parameters_panel;
mod theme;
mod view_menu;
mod views_panel;

use bevy::prelude::*;

use crate::editor::camera::CameraCommand;
use crate::editor::gizmo::AxisGizmoPlugin;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};

use crate::editor::gui::help::HelpPlugin;
use crate::editor::gui::menu::{MenuAppExt, MenuPlugin};
use crate::editor::gui::parameters_panel::{ParametersPanel, ParametersPanelPlugin};
use crate::editor::gui::view_menu::ViewMenuPlugin;
use crate::editor::gui::views_panel::{ViewsPanel, ViewsPanelPlugin};
use crate::settings::{Settings, Store};
use crate::ParameterHandle;

/// Every egui panel is added in this set, so the axis gizmo can run after all
/// of them and use the remaining central rectangle.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GuiSet;

pub struct GuiPlugin {
    cheatsheet: String,
    parameters: ParameterHandle,
}

impl GuiPlugin {
    pub fn new(cheatsheet: String, parameters: ParameterHandle) -> Self {
        Self {
            cheatsheet,
            parameters,
        }
    }
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CheatSheet {
            cheetsheet: self.cheatsheet.clone(),
        })
        .insert_resource(self.parameters.clone())
        .insert_resource(Console {
            text: None,
            open: true,
        })
        .add_plugins(EguiPlugin::default())
        .add_plugins(AxisGizmoPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(ViewMenuPlugin)
        .add_plugins(ViewsPanelPlugin)
        .add_plugins(ParametersPanelPlugin)
        .add_event_menu_button("Camera/Focus", |c: &mut MessageWriter<CameraCommand>| {
            c.write(CameraCommand::Refocus());
        })
        .add_event_menu_button("Camera/Reset", |c: &mut MessageWriter<CameraCommand>| {
            c.write(CameraCommand::Reset());
        })
        .add_persistent_res_menu_button::<ResMut<ViewsPanel>>(
            "View/Views Panel",
            "views",
            |mut panel: ResMut<ViewsPanel>| {
                panel.open = !panel.open;
                panel.open.to_string()
            },
        )
        .add_persistent_res_menu_button::<ResMut<ParametersPanel>>(
            "View/Parameters Panel",
            "parameters",
            |mut panel: ResMut<ParametersPanel>| {
                panel.open = !panel.open;
                panel.open.to_string()
            },
        )
        .add_persistent_res_menu_button::<ResMut<Console>>(
            "View/Console",
            "console",
            |mut console: ResMut<Console>| {
                console.open = !console.open;
                console.open.to_string()
            },
        )
        .add_persistent_res_loader::<ResMut<ViewsPanel>>(
            "views",
            |value, mut panel: ResMut<ViewsPanel>| {
                panel.open = value.unwrap_or("true") == "true";
            },
        )
        .add_persistent_res_loader::<ResMut<ParametersPanel>>(
            "parameters",
            |value, mut panel: ResMut<ParametersPanel>| {
                panel.open = value.unwrap_or("true") == "true";
            },
        )
        .add_persistent_res_loader::<ResMut<Console>>(
            "console",
            |value, mut console: ResMut<Console>| {
                console.open = value.unwrap_or("true") == "true";
            },
        )
        .add_plugins(HelpPlugin::default())
        .configure_sets(EguiPrimaryContextPass, GuiSet)
        .add_systems(
            EguiPrimaryContextPass,
            (dark_theme, (toolbar, console_panel).chain())
                .chain()
                .in_set(GuiSet),
        );
    }
}

/// Matches the egui panels to the dark CAD viewport.
fn dark_theme(mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>) {
    if let Ok(mut context) = egui_ctx.single_mut() {
        theme::apply(context.get_mut());
    }
}

#[derive(Resource)]
pub struct Console {
    text: Option<String>,
    /// Whether the console panel is open.
    pub open: bool,
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
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>,
) {
    egui::TopBottomPanel::bottom("Console")
        .resizable(true)
        .default_height(180.0)
        .show_animated(
            egui_ctx.single_mut().unwrap().get_mut(),
            console.open,
            |ui| {
                ui.label(
                    egui::RichText::new("Console")
                        .heading()
                        .color(theme::heading_color()),
                );
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(256.)
                    .max_width(f32::INFINITY)
                    .auto_shrink([false, true])
                    .show(ui, |ui| match &console.text {
                        None => ui.monospace(""),
                        Some(t) => ui.monospace(t),
                    });
            },
        );
}

/// A quick access toolbar at the very bottom of the window that toggles the
/// auxiliary panels. It is added before the console so it sits below it.
fn toolbar(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>,
    mut views: ResMut<ViewsPanel>,
    mut parameters: ResMut<ParametersPanel>,
    mut console: ResMut<Console>,
    mut store: ResMut<Settings>,
) {
    egui::TopBottomPanel::bottom("Toolbar").show(egui_ctx.single_mut().unwrap().get_mut(), |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                let icon = |ui: &mut egui::Ui, open: bool, glyph: &str| {
                    ui.add(
                        egui::Button::selectable(open, egui::RichText::new(glyph).size(12.0))
                            .min_size(egui::vec2(28.0, 20.0)),
                    )
                };

                if icon(ui, parameters.open, "⚙")
                    .on_hover_text("Toggle the parameters panel")
                    .clicked()
                {
                    parameters.open = !parameters.open;
                    store.store("parameters", &parameters.open.to_string());
                }

                if icon(ui, console.open, ">_")
                    .on_hover_text("Toggle the console panel")
                    .clicked()
                {
                    console.open = !console.open;
                    store.store("console", &console.open.to_string());
                }

                if icon(ui, views.open, "👁")
                    .on_hover_text("Toggle the views panel")
                    .clicked()
                {
                    views.open = !views.open;
                    store.store("views", &views.open.to_string());
                }
            });
        });
    });
}
