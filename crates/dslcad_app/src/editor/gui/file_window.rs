use crate::editor::gui::UiEvent;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};

pub struct FileWindowPlugin;
impl Plugin for FileWindowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FileWindowState { open: false })
            .insert_resource(FileWindowBuffer {
                buffer: String::new(),
            })
            .add_system(file_window);
    }
}

#[derive(Resource)]
pub struct FileWindowBuffer {
    buffer: String,
}

#[derive(Resource)]
pub struct FileWindowState {
    open: bool,
}

impl FileWindowState {
    pub fn open(&mut self) {
        self.open = true;
    }
}

pub fn file_window(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut state: ResMut<FileWindowState>,
    mut buffer: ResMut<FileWindowBuffer>,
    mut _events: EventWriter<UiEvent>,
) {
    egui::Window::new("Editor")
        .open(&mut state.open)
        .show(egui_ctx.single_mut().get_mut(), |ui| {
            ui.text_edit_multiline(&mut buffer.buffer);
            if ui.button("Save").clicked() {
                todo!()
                // events.send(UiEvent::RenderString(buffer.buffer.clone()));
            }
        });
}
