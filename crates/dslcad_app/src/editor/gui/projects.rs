use crate::editor::gui::menu::MenuAppExt;
use crate::editor::gui::UiEvent;
use crate::reader::ProjectReader;
use bevy::prelude::*;
use bevy_egui::egui::{PointerButton, Response, Sense, Ui};
use bevy_egui::{egui, EguiContexts};
use std::io::{Read, Write};
use std::path::Path;
use vfs::{FileSystem, MemoryFS, PhysicalFS, VfsFileType};

pub struct ProjectsPlugin;

impl Plugin for ProjectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Project::default())
            .insert_resource(ProjectWindow::default())
            .insert_resource(Popup::default())
            .add_event::<ProjectUiEvent>()
            .add_event::<PopupEvent>()
            .add_res_menu_button("Window/Project", |p: &mut ProjectWindow| p.show())
            .add_system(popup_system)
            .add_system(project_window)
            .add_system(project_event_handler)
            .add_startup_system(project_test);
    }
}

#[derive(Resource, Default)]
struct ProjectWindow {
    open: bool,
}

impl ProjectWindow {
    pub fn show(&mut self) {
        self.open = true;
    }
}

#[derive(Resource, Default)]
pub struct Project {
    fs: Option<Box<dyn FileSystem>>,
    path: Option<String>,
    root: String,
    buffer: String,
}

#[derive(Resource, Default)]
struct Popup {
    message: &'static str,
    action: Option<PopupEvent>,
}

impl Popup {
    pub fn show_confirm(&mut self, message: &'static str, confirm: ProjectUiEvent) {
        self.message = message;
        self.action = Some(PopupEvent::Confirm(confirm))
    }

    pub fn prompt(&mut self, message: &'static str, confirm: ProjectUiEvent) {
        self.message = message;
        self.action = Some(PopupEvent::Prompt(confirm, String::new()))
    }
}

#[derive(Debug, Clone)]
enum PopupEvent {
    Confirm(ProjectUiEvent),
    Prompt(ProjectUiEvent, String),
}

#[derive(Debug, Clone)]
enum ProjectUiEvent {
    Open { path: String },
    NewPart { path: String },
    NewFolder { path: String },
    Delete { path: String },
    Save(),
    Render(),
}

impl Project {
    pub fn open_physical_project(&mut self, path: &Path) {
        self.fs = Some(Box::new(PhysicalFS::new(path)));
    }

    pub fn open_memory_project(&mut self) {
        let fs = MemoryFS::new();
        fs.create_dir("/").unwrap();
        self.fs = Some(Box::new(fs));
        self.root = String::from("/");
    }

    pub fn reader(&self) -> Option<ProjectReader> {
        self.fs.as_ref().map(|fs| ProjectReader::new(fs.as_ref()))
    }

    pub fn open_file(&mut self, path: String) {
        if let Some(fs) = self.fs.as_mut() {
            let mut file = fs.open_file(&path).unwrap();
            self.buffer.clear();
            file.read_to_string(&mut self.buffer)
                .expect("unable to read file");
            self.path = Some(path);
        }
    }

    pub fn new_file(&mut self, path: String) {
        if let Some(fs) = self.fs.as_mut() {
            fs.create_file(&path).unwrap();
        }
    }

    pub fn new_folder(&mut self, path: String) {
        if let Some(fs) = self.fs.as_mut() {
            fs.create_dir(&path).unwrap();
        }
    }

    pub fn delete(&mut self, path: &str) {
        if self.path.is_some() && self.path.as_ref().unwrap() == path {
            self.path = None;
            self.buffer.clear();
        }

        if let Some(fs) = self.fs.as_mut() {
            if fs.metadata(path).unwrap().file_type == VfsFileType::Directory {
                fs.remove_dir(path).expect("unable to delete folder");
            } else {
                fs.remove_file(path).expect("unable to delete file");
            }
        }
    }

    pub fn save_buffer(&mut self) {
        if let Some(fs) = self.fs.as_mut() {
            if let Some(path) = &self.path {
                let mut file = fs.create_file(path).unwrap();
                file.write_all(self.buffer.as_bytes())
                    .expect("unable to write file");
            }
        }
    }

    pub fn folders<'a>(&'a self, path: &'a str) -> Vec<String> {
        if let Some(fs) = &self.fs {
            let mut folders: Vec<String> = fs
                .read_dir(path)
                .unwrap()
                .filter(move |i| {
                    fs.metadata(&join(path, i)).unwrap().file_type == VfsFileType::Directory
                        && !i.starts_with('.')
                })
                .collect();
            folders.sort();
            folders
        } else {
            Vec::new()
        }
    }

    pub fn files<'a>(&'a self, path: &'a str) -> Vec<String> {
        if let Some(fs) = &self.fs {
            let mut files: Vec<String> = fs
                .read_dir(path)
                .unwrap()
                .filter(move |i| {
                    fs.metadata(&join(path, i)).unwrap().file_type == VfsFileType::File
                        && !i.starts_with('.')
                        && i.ends_with(".ds")
                })
                .collect();
            files.sort();
            files
        } else {
            Vec::new()
        }
    }
}

fn popup_system(mut ctx: EguiContexts, mut popup: ResMut<Popup>, events: EventWriter<PopupEvent>) {
    if popup.action.is_none() {
        return;
    }

    let mut clear = false;
    egui::Window::new("Alert")
        .vscroll(false)
        .resizable(true)
        .default_height(300.0)
        .show(ctx.ctx_mut(), |ui| {
            let message = popup.message;
            match &mut popup.action {
                None => {}
                Some(PopupEvent::Confirm(event)) => {
                    ui.label(message);
                    confirm_cancel_buttons(events, &mut clear, ui, || {
                        PopupEvent::Confirm(event.clone())
                    });
                }
                Some(PopupEvent::Prompt(event, buffer)) => {
                    ui.label(message);
                    ui.text_edit_singleline(buffer);
                    confirm_cancel_buttons(events, &mut clear, ui, || {
                        PopupEvent::Prompt(event.clone(), buffer.clone())
                    });
                }
            }
        });

    if clear {
        popup.action = None
    }
}

fn confirm_cancel_buttons(
    mut events: EventWriter<PopupEvent>,
    clear: &mut bool,
    ui: &mut Ui,
    confirm: impl FnOnce() -> PopupEvent,
) {
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            *clear = true;
        }
        if ui.button("Confirm").clicked() {
            events.send(confirm());
            *clear = true;
        }
    });
}

fn project_test(mut project: ResMut<Project>) {
    #[cfg(not(target_arch = "wasm32"))]
    project.open_physical_project(std::env::current_dir().unwrap().as_path());
    #[cfg(target_arch = "wasm32")]
    project.open_memory_project();
}

fn project_event_handler(
    mut project: ResMut<Project>,
    mut events: EventReader<ProjectUiEvent>,
    mut confirm_events: EventReader<PopupEvent>,
    mut popup: ResMut<Popup>,
    mut ui_events: EventWriter<UiEvent>,
) {
    for event in events.iter() {
        match event {
            ProjectUiEvent::Open { path } => {
                project.open_file(path.clone());
            }
            ProjectUiEvent::Save() => {
                project.save_buffer();
            }
            ProjectUiEvent::Render() => {
                project.save_buffer();
                ui_events.send(UiEvent::Render {
                    path: project.path.clone().unwrap(),
                })
            }
            ProjectUiEvent::NewPart { .. } => popup.prompt("Enter new part name:", event.clone()),
            ProjectUiEvent::NewFolder { .. } => {
                popup.prompt("Enter new folder name:", event.clone())
            }
            ProjectUiEvent::Delete { .. } => {
                popup.show_confirm("Are you sure you want to delete this file?", event.clone());
            }
        }
    }

    for event in confirm_events.iter() {
        match event {
            PopupEvent::Confirm(source) => {
                if let ProjectUiEvent::Delete { path } = source {
                    project.delete(path);
                }
            }
            PopupEvent::Prompt(source, input) => match source {
                ProjectUiEvent::NewPart { path } => {
                    project.new_file(format!("{}.ds", join(path, input)))
                }
                ProjectUiEvent::NewFolder { path } => project.new_folder(join(path, input)),
                _ => {}
            },
        }
    }
}

fn project_window(
    mut ctx: EguiContexts,
    mut project: ResMut<Project>,
    mut project_window: ResMut<ProjectWindow>,
    mut events: EventWriter<ProjectUiEvent>,
) {
    egui::Window::new("Project")
        .open(&mut project_window.open)
        .vscroll(false)
        .resizable(true)
        .default_height(300.0)
        .show(ctx.ctx_mut(), |ui| {
            let window_size = ui.available_size();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical()
                        .id_source("folders")
                        .min_scrolled_height(window_size.y)
                        .show(ui, |ui| {
                            root_folder(ui, "/", &mut events, |ui, events| {
                                draw_folder(&project, ui, &project.root, events)
                            });
                            ui.allocate_space(egui::Vec2::new(0., ui.available_height()));
                        });
                });
                ui.vertical(|ui| {
                    let interactive = project.path.is_some();

                    let bar = egui::menu::bar(ui, |ui| {
                        if interactive {
                            if ui.button("Save").clicked() {
                                events.send(ProjectUiEvent::Save {})
                            }
                            if ui.button("Render").clicked() {
                                events.send(ProjectUiEvent::Render {})
                            }
                        }
                    });

                    egui::ScrollArea::vertical()
                        .id_source("edit")
                        .min_scrolled_height(
                            window_size.y
                                - (bar.response.rect.height() + ui.spacing().item_spacing.y),
                        )
                        .show(ui, |ui| {
                            let editor = egui::TextEdit::multiline(&mut project.buffer)
                                .desired_width(f32::INFINITY)
                                .interactive(interactive)
                                .code_editor();

                            ui.add_sized(ui.available_size(), editor);
                        });
                })
            });
        });
}

fn draw_folder(
    project: &Project,
    ui: &mut Ui,
    path: &str,
    events: &mut EventWriter<ProjectUiEvent>,
) {
    for folder in project.folders(path) {
        folder_collapse(ui, path, &folder, events, |ui, events| {
            draw_folder(project, ui, &join(path, &folder), events)
        });
    }
    for file in project.files(path) {
        let full_path = join(path, &file);
        let file_label = ui.selectable_label(false, &file);
        let popup_id = ui.make_persistent_id(format!("{}_popup", &file));

        if file_label.secondary_clicked() {
            ui.memory_mut(|mem| mem.open_popup(popup_id))
        }
        if file_label.clicked() {
            events.send(ProjectUiEvent::Open {
                path: full_path.clone(),
            });
            ui.memory_mut(|mem| mem.close_popup());
        }

        egui::popup::popup_below_widget(ui, popup_id, &file_label, |ui| {
            ui.set_min_width(75.0);

            if ui.button("Delete").clicked() {
                events.send(ProjectUiEvent::Delete {
                    path: full_path.clone(),
                });
                ui.memory_mut(|mem| mem.close_popup());
            }
        });
    }
}

fn root_folder(
    ui: &mut Ui,
    folder: &str,
    events: &mut EventWriter<ProjectUiEvent>,
    body: impl FnOnce(&mut Ui, &mut EventWriter<ProjectUiEvent>),
) {
    let id = ui.make_persistent_id(folder);
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

    let header_response = ui.horizontal(|ui| {
        let prev_item_spacing = ui.spacing_mut().item_spacing;
        ui.spacing_mut().item_spacing.x = 0.0;
        let size = egui::Vec2::new(ui.spacing().indent, ui.spacing().icon_width);
        let (_id, _rect) = ui.allocate_space(size);
        ui.spacing_mut().item_spacing = prev_item_spacing;

        ui.label(folder)
    });

    right_click_menu(ui, folder, "", &header_response.response, events);

    state.show_body_indented(&header_response.response, ui, |ui| body(ui, events));
}

fn right_click_menu(
    ui: &mut Ui,
    path: &str,
    folder: &str,
    header_response: &Response,
    events: &mut EventWriter<ProjectUiEvent>,
) {
    let id = ui.make_persistent_id(format!("{}_interact", folder));
    let popup_id = ui.make_persistent_id(format!("{}_popup", folder));

    if ui
        .interact(header_response.rect, id, Sense::click())
        .clicked_by(PointerButton::Secondary)
    {
        ui.memory_mut(|mem| mem.open_popup(popup_id))
    }

    egui::popup::popup_below_widget(ui, popup_id, header_response, |ui| {
        ui.set_min_width(75.0);

        if ui.button("New Part").clicked() {
            events.send(ProjectUiEvent::NewPart {
                path: join(path, folder),
            });
            ui.memory_mut(|mem| mem.close_popup());
        }

        if ui.button("New Folder").clicked() {
            events.send(ProjectUiEvent::NewFolder {
                path: join(path, folder),
            });
            ui.memory_mut(|mem| mem.close_popup());
        }

        if ui.button("Delete").clicked() {
            events.send(ProjectUiEvent::Delete {
                path: join(path, folder),
            });
            ui.memory_mut(|mem| mem.close_popup());
        }
    });
}

fn folder_collapse(
    ui: &mut Ui,
    path: &str,
    folder: &str,
    events: &mut EventWriter<ProjectUiEvent>,
    body: impl FnOnce(&mut Ui, &mut EventWriter<ProjectUiEvent>),
) {
    let id = ui.make_persistent_id(folder);
    let state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);

    state
        .show_header(ui, |ui| {
            let response = ui.label(folder);
            right_click_menu(ui, path, folder, &response, events);
        })
        .body(|ui| body(ui, events));
}

fn join(a: &str, b: &str) -> String {
    if b.is_empty() {
        return a.to_string();
    }

    format!("{}/{}", a, b)
}
