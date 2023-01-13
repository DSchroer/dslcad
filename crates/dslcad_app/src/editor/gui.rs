use crate::editor::rendering::RenderCommand;
use crate::editor::State;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use dslcad::Dslcad;

pub struct GuiPlugin;
impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiEvent>()
            .add_plugin(EguiPlugin)
            .add_system(main_ui)
            .add_system(about)
            .add_system(cheatsheet)
            .add_system(console_panel)
            .add_system(update_ui_scale_factor)
            .add_system(keybindings);
    }
}

pub enum UiEvent {
    CreateFile(),
    OpenFile(),
    Render(),
    SaveStl(),
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

fn main_ui(
    mut egui_context: ResMut<EguiContext>,
    mut events: EventWriter<UiEvent>,
    mut render_events: EventWriter<RenderCommand>,
    mut state: ResMut<State>,
    mut exit: EventWriter<AppExit>,
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
                ui.separator();
                if ui.button("Exit").clicked() {
                    exit.send(AppExit);
                    ui.close_menu();
                }
            });
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut state.show_points, "Points").clicked() {
                    render_events.send(RenderCommand::Redraw);
                }
                if ui.checkbox(&mut state.show_lines, "Lines").clicked() {
                    render_events.send(RenderCommand::Redraw);
                }
                if ui.checkbox(&mut state.show_mesh, "Mesh").clicked() {
                    render_events.send(RenderCommand::Redraw);
                }
            });
            ui.menu_button("Rendering", |ui| {
                if ui.button("Render (F5)").clicked() {
                    events.send(UiEvent::Render());
                    ui.close_menu();
                }
                ui.separator();
                ui.checkbox(&mut state.autowatch, "Auto Render");
            });
            ui.menu_button("Export", |ui| {
                if ui.button("To folder").clicked() {
                    events.send(UiEvent::SaveStl());
                    ui.close_menu();
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Cheat Sheet").clicked() {
                    state.cheatsheet_window = true;
                    ui.close_menu();
                }
                if ui.button("About").clicked() {
                    state.about_window = true;
                    ui.close_menu();
                }
            });
        });
    });
}

fn about(mut egui_context: ResMut<EguiContext>, mut state: ResMut<State>) {
    egui::Window::new("About")
        .open(&mut state.about_window)
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(dslcad::constants::FULL_NAME);
            ui.separator();
            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            ui.label("Copyright: Dominick Schroer 2022");
        });
}

fn cheatsheet(mut egui_context: ResMut<EguiContext>, mut state: ResMut<State>) {
    egui::Window::new("Cheat Sheet")
        .open(&mut state.cheatsheet_window)
        .show(egui_context.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                .max_height(512.)
                .show(ui, |ui| ui.monospace(Dslcad::default().cheat_sheet()));
        });
}

fn console_panel(state: Res<State>, mut egui_context: ResMut<EguiContext>) {
    egui::TopBottomPanel::bottom("Console").show(egui_context.ctx_mut(), |ui| {
        ui.heading("Console:");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(256.)
            .max_width(f32::INFINITY)
            .auto_shrink([false, true])
            .show(ui, |ui| match &state.output {
                None => {}
                Some(output) => {
                    match output {
                        Ok(o) => {
                            for o in o {
                                if !o.text().is_empty() {
                                    ui.monospace(o.text());
                                }
                            }
                        }
                        Err(e) => {
                            ui.monospace(&e.to_string());
                        }
                    };
                }
            });
    });
}

fn update_ui_scale_factor(mut egui_settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.0 / window.scale_factor();
    }
}
