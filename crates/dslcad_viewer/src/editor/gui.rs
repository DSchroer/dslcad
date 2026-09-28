mod help;
mod menu;
mod view_menu;
mod views_panel;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::editor::camera::CameraCommand;
use crate::editor::gizmo::AxisGizmoPlugin;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::editor::gui::help::HelpPlugin;
use crate::editor::gui::menu::{MenuAppExt, MenuPlugin};
use crate::editor::gui::view_menu::ViewMenuPlugin;
use crate::editor::gui::views_panel::{ViewsPanel, ViewsPanelPlugin};
use crate::settings::{Settings, Store};

/// Every egui panel is added in this set, so the axis gizmo can run after all
/// of them and use the remaining central rectangle.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GuiSet;

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
        .insert_resource(Console {
            text: None,
            open: true,
        })
        .add_plugins(EguiPlugin)
        .add_plugins(AxisGizmoPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(ViewMenuPlugin)
        .add_plugins(ViewsPanelPlugin)
        .add_event_menu_button("Camera/Focus", |c: &mut EventWriter<CameraCommand>| {
            c.send(CameraCommand::Refocus());
        })
        .add_event_menu_button("Camera/Reset", |c: &mut EventWriter<CameraCommand>| {
            c.send(CameraCommand::Reset());
        })
        .add_persistent_res_menu_button::<ResMut<ViewsPanel>>(
            "View/Views Panel",
            "views",
            |mut panel: ResMut<ViewsPanel>| {
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
        .add_persistent_res_loader::<ResMut<Console>>(
            "console",
            |value, mut console: ResMut<Console>| {
                console.open = value.unwrap_or("true") == "true";
            },
        )
        .add_plugins(HelpPlugin::default())
        .configure_sets(Update, GuiSet)
        .add_systems(Startup, dark_theme)
        .add_systems(Update, (toolbar, console_panel).chain().in_set(GuiSet));
    }
}

/// Matches the egui panels to the dark CAD viewport.
fn dark_theme(mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>) {
    if let Ok(mut context) = egui_ctx.get_single_mut() {
        context.get_mut().set_visuals(egui::Visuals::dark());
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
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
) {
    egui::TopBottomPanel::bottom("Console")
        .resizable(true)
        .default_height(180.0)
        .show_animated(egui_ctx.single_mut().get_mut(), console.open, |ui| {
            ui.heading("Console");
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

/// A quick access toolbar at the very bottom of the window that toggles the
/// auxiliary panels. It is added before the console so it sits below it.
fn toolbar(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut views: ResMut<ViewsPanel>,
    mut console: ResMut<Console>,
    mut store: ResMut<Settings>,
) {
    egui::TopBottomPanel::bottom("Toolbar").show(egui_ctx.single_mut().get_mut(), |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                let icon = |ui: &mut egui::Ui, open: bool, glyph: &str| {
                    ui.add_sized(
                        egui::vec2(20.0, 18.0),
                        egui::SelectableLabel::new(open, egui::RichText::new(glyph).size(12.0)),
                    )
                };

                if icon(ui, views.open, "👁")
                    .on_hover_text("Toggle the views panel")
                    .clicked()
                {
                    views.open = !views.open;
                    store.store("views", &views.open.to_string());
                }

                if icon(ui, console.open, ">_")
                    .on_hover_text("Toggle the console panel")
                    .clicked()
                {
                    console.open = !console.open;
                    store.store("console", &console.open.to_string());
                }
            });
        });
    });
}
