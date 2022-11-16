use crate::editor::State;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};

pub struct GuiPlugin;
impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiEvent>()
            .add_plugin(EguiPlugin)
            .add_system(ui_example)
            .add_system(console_panel)
            .add_system(update_ui_scale_factor)
            .add_system(keybindings);
    }
}

pub enum UiEvent {
    CreateFile(),
    OpenFile(),
    Render(),
}

fn keybindings(keys: Res<Input<KeyCode>>, mut events: EventWriter<UiEvent>) {
    if keys.just_released(KeyCode::N) {
        events.send(UiEvent::CreateFile())
    }

    if keys.just_released(KeyCode::O) {
        events.send(UiEvent::OpenFile())
    }

    if keys.just_released(KeyCode::F5) {
        events.send(UiEvent::Render())
    }
}

fn ui_example(
    mut egui_context: ResMut<EguiContext>,
    mut events: EventWriter<UiEvent>,
    mut state: ResMut<State>,
) {
    egui::TopBottomPanel::top("Tools").show(egui_context.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New (n)").clicked() {
                    events.send(UiEvent::CreateFile());
                    ui.close_menu();
                }
                if ui.button("Open (o)").clicked() {
                    events.send(UiEvent::OpenFile());
                    ui.close_menu();
                }
            });
            ui.separator();
            if ui.button("Render (F5)").clicked() {
                events.send(UiEvent::Render());
            }

            ui.checkbox(&mut state.autowatch, "Auto Render");
        });
    });
}

fn console_panel(state: Res<State>, mut egui_context: ResMut<EguiContext>) {
    egui::TopBottomPanel::bottom("Console").show(egui_context.ctx_mut(), |ui| {
        ui.label("Console:");
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(256.)
            .show(ui, |ui| {
                ui.monospace(&state.output);
            });
    });
}

fn update_ui_scale_factor(mut egui_settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.0 / window.scale_factor();
    }
}
