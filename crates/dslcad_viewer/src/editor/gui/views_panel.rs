//! A collapsible panel on the right of the viewport that lists the views a
//! model declares, plus a `Default` entry for the free camera.

use crate::editor::camera::CameraCommand;
use crate::editor::rendering::{RenderCommand, RenderState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};

pub struct ViewsPanelPlugin;

impl Plugin for ViewsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewsPanel>()
            .add_systems(Update, views_panel.in_set(super::GuiSet));
    }
}

/// Whether the views panel is open.
#[derive(Resource)]
pub struct ViewsPanel {
    pub open: bool,
}

impl Default for ViewsPanel {
    fn default() -> Self {
        ViewsPanel { open: true }
    }
}

fn views_panel(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut state: ResMut<RenderState>,
    panel: Res<ViewsPanel>,
    mut render_events: EventWriter<RenderCommand>,
    mut camera_events: EventWriter<CameraCommand>,
) {
    let views: Vec<String> = state
        .views()
        .iter()
        .enumerate()
        .map(|(index, view)| {
            view.name
                .clone()
                .unwrap_or_else(|| format!("View {}", index + 1))
        })
        .collect();

    if views.is_empty() {
        return;
    }

    egui::SidePanel::right("Views")
        .resizable(true)
        .default_width(160.0)
        .show_animated(egui_ctx.single_mut().get_mut(), panel.open, |ui| {
            ui.label(
                egui::RichText::new("Views")
                    .heading()
                    .color(super::theme::heading_color()),
            );
            ui.separator();

            let active = state.active_view_index();

            if view_entry(ui, active.is_none(), "Default").clicked() {
                state.set_active_view(None);
                render_events.send(RenderCommand::Redraw);
            }

            for (index, name) in views.iter().enumerate() {
                if view_entry(ui, active == Some(index), name).clicked() {
                    let view = state.views()[index].clone();
                    state.set_active_view(Some(index));
                    render_events.send(RenderCommand::Redraw);
                    camera_events.send(CameraCommand::View {
                        angles: view.angle,
                        projection: view.projection,
                        zoom: view.zoom,
                        target: view.target,
                    });
                }
            }
        });
}

/// A full width, left aligned entry in the views list.
fn view_entry(ui: &mut egui::Ui, selected: bool, label: &str) -> egui::Response {
    let size = egui::vec2(ui.available_width(), 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let visuals = ui.style().interact_selectable(&response, selected);

    if selected || response.hovered() {
        ui.painter().rect_filled(rect, 2.0, visuals.bg_fill);
    }

    ui.painter().text(
        rect.left_center() + egui::vec2(6.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.0),
        visuals.text_color(),
    );

    response
}
