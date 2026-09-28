//! A collapsible panel on the left of the viewport that lists the parameters a
//! model declares and lets the user edit them. Edits are sent back to the
//! evaluator as script argument overrides, which triggers a re-render.

use crate::editor::rendering::RenderState;
use crate::ParameterHandle;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};
use dslcad_storage::protocol::{Parameter, ParameterType, ParameterValue};
use std::collections::HashMap;

pub struct ParametersPanelPlugin;

impl Plugin for ParametersPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParametersPanel>()
            .init_resource::<ParameterEditor>()
            .add_systems(Update, parameters_panel.in_set(super::GuiSet));
    }
}

/// Whether the parameters panel is open.
#[derive(Resource)]
pub struct ParametersPanel {
    pub open: bool,
}

impl Default for ParametersPanel {
    fn default() -> Self {
        ParametersPanel { open: true }
    }
}

/// Values being edited in the panel, shown immediately while the evaluator
/// catches up with the re-render.
#[derive(Resource, Default)]
struct ParameterEditor {
    editing: HashMap<String, ParameterValue>,
}

fn parameters_panel(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    state: Res<RenderState>,
    panel: Res<ParametersPanel>,
    handle: Res<ParameterHandle>,
    mut editor: ResMut<ParameterEditor>,
) {
    let parameters = state.parameters().to_vec();
    if parameters.is_empty() {
        return;
    }

    egui::SidePanel::left("Parameters")
        .resizable(true)
        .default_width(220.0)
        .show_animated(egui_ctx.single_mut().get_mut(), panel.open, |ui| {
            ui.heading("Parameters");
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for parameter in &parameters {
                        parameter_widget(ui, parameter, &mut editor, &handle);
                        ui.separator();
                    }
                });
        });
}

fn parameter_widget(
    ui: &mut egui::Ui,
    parameter: &Parameter,
    editor: &mut ParameterEditor,
    handle: &ParameterHandle,
) {
    let name = &parameter.name;
    let current = editor
        .editing
        .get(name)
        .cloned()
        .unwrap_or_else(|| parameter.value.clone());

    ui.label(egui::RichText::new(name).strong());

    match (parameter.kind, current) {
        (ParameterType::Bool, ParameterValue::Bool(mut value)) => {
            if ui.checkbox(&mut value, "").changed() {
                commit(handle, name, parameter.kind, &ParameterValue::Bool(value));
                editor.editing.remove(name);
            }
        }
        (ParameterType::Text, ParameterValue::Text(mut value)) => {
            let response =
                ui.add(egui::TextEdit::singleline(&mut value).desired_width(f32::INFINITY));
            if response.changed() {
                editor
                    .editing
                    .insert(name.clone(), ParameterValue::Text(value.clone()));
            }
            if response.lost_focus() {
                commit(handle, name, parameter.kind, &ParameterValue::Text(value));
                editor.editing.remove(name);
            }
        }
        (ParameterType::Number, ParameterValue::Number(value))
        | (ParameterType::Integer, ParameterValue::Number(value)) => {
            number_widget(ui, parameter, name, value, editor, handle);
        }
        // The declared type and the current value disagree; fall back to text.
        (_, other) => {
            let mut value = format_value(&other);
            let response =
                ui.add(egui::TextEdit::singleline(&mut value).desired_width(f32::INFINITY));
            if response.lost_focus() {
                handle.set(name, &quote(&value));
            }
        }
    }
}

fn number_widget(
    ui: &mut egui::Ui,
    parameter: &Parameter,
    name: &str,
    mut value: f64,
    editor: &mut ParameterEditor,
    handle: &ParameterHandle,
) {
    let integer = parameter.kind == ParameterType::Integer;

    let response = match (parameter.min, parameter.max) {
        (Some(min), Some(max)) => {
            let mut slider = egui::Slider::new(&mut value, min..=max);
            if let Some(step) = parameter.step {
                slider = slider.step_by(step);
            }
            if integer {
                slider = slider.fixed_decimals(0);
            }
            ui.add(slider)
        }
        _ => {
            let mut drag = egui::DragValue::new(&mut value);
            if let Some(step) = parameter.step {
                drag = drag.speed(step);
            }
            if integer {
                drag = drag.fixed_decimals(0);
            }
            ui.add(drag)
        }
    };

    if response.changed() {
        editor
            .editing
            .insert(name.to_string(), ParameterValue::Number(value));
    }

    let commit_now = response.drag_stopped() || (response.changed() && !response.dragged());
    if commit_now {
        commit(handle, name, parameter.kind, &ParameterValue::Number(value));
        editor.editing.remove(name);
    }
}

/// Sends a value to the evaluator using `--argument` syntax.
fn commit(handle: &ParameterHandle, name: &str, kind: ParameterType, value: &ParameterValue) {
    let value = match (kind, value) {
        (ParameterType::Integer, ParameterValue::Number(number)) => {
            format!("{}", number.round() as i64)
        }
        (ParameterType::Text, ParameterValue::Text(text)) => quote(text),
        (_, value) => format_value(value),
    };
    handle.set(name, &value);
}

fn format_value(value: &ParameterValue) -> String {
    match value {
        ParameterValue::Number(number) => format!("{number}"),
        ParameterValue::Bool(value) => value.to_string(),
        ParameterValue::Text(value) => quote(value),
    }
}

/// Encodes text as a DSLCAD string literal.
fn quote(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str(r"\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str(r"\n"),
            '\r' => output.push_str(r"\r"),
            '\t' => output.push_str(r"\t"),
            other => output.push(other),
        }
    }
    output.push('"');
    output
}
