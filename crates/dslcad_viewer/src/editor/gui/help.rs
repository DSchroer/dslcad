use crate::editor::gui::menu::MenuAppExt;
use crate::editor::gui::CheatSheet;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPrimaryContextPass, PrimaryEguiContext};

#[derive(Resource, Clone, Default)]
pub struct HelpPlugin {
    about_window: bool,
    cheatsheet_window: bool,
    /// The filter typed into the cheat sheet search box.
    search: String,
}

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_res_menu_button("Help/Cheat Sheet", |state: &mut HelpPlugin| {
                state.cheatsheet_window = true
            })
            .add_res_menu_button("Help/About", |state: &mut HelpPlugin| {
                state.about_window = true
            })
            .add_systems(EguiPrimaryContextPass, (about, cheatsheet));
    }
}

fn about(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>,
    mut state: ResMut<HelpPlugin>,
) {
    egui::Window::new("About")
        .open(&mut state.about_window)
        .show(egui_ctx.single_mut().unwrap().get_mut(), |ui| {
            ui.label(dslcad_storage::constants::FULL_NAME);
            ui.separator();
            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            ui.label("Copyright: Dominick Schroer 2022");
        });
}

/// A named group of reference entries, split out of the cheat sheet markdown.
struct Section {
    title: String,
    lines: Vec<String>,
}

/// Splits the cheat sheet into its `##` sections. The leading `#` title and
/// blank lines are dropped, since the window already supplies the title.
fn parse_sections(text: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();

    for line in text.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            sections.push(Section {
                title: title.trim().to_string(),
                lines: Vec::new(),
            });
        } else if line.starts_with('#') || line.trim().is_empty() {
            continue;
        } else if let Some(section) = sections.last_mut() {
            section.lines.push(line.to_string());
        }
    }

    sections
}

/// Draws one reference entry, rendering the leading code span in the accent
/// color so signatures stand out from their descriptions.
fn reference_line(ui: &mut egui::Ui, line: &str) {
    let Some(item) = line.trim_start().strip_prefix("- ") else {
        ui.label(line.trim());
        return;
    };

    let code_color = ui.visuals().hyperlink_color;
    ui.horizontal_wrapped(|ui| {
        match item.strip_prefix('`').and_then(|rest| rest.split_once('`')) {
            Some((code, description)) => {
                ui.label(
                    egui::RichText::new(format!("`{code}`"))
                        .monospace()
                        .color(code_color),
                );
                ui.label(description.trim_start());
            }
            None => {
                ui.label(item);
            }
        }
    });
}

fn cheatsheet(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>,
    mut state: ResMut<HelpPlugin>,
    cheatsheet: Res<CheatSheet>,
) {
    if cheatsheet.cheatsheet.is_empty() || !state.cheatsheet_window {
        return;
    }

    let sections = parse_sections(&cheatsheet.cheatsheet);
    let mut open = state.cheatsheet_window;
    let mut search = std::mem::take(&mut state.search);
    let query = search.trim().to_lowercase();

    egui::Window::new("Cheat Sheet")
        .open(&mut open)
        .default_size([640.0, 512.0])
        .resizable(true)
        .show(egui_ctx.single_mut().unwrap().get_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Search");
                ui.add(
                    egui::TextEdit::singleline(&mut search)
                        .hint_text("filter operators and syntax")
                        .desired_width(f32::INFINITY),
                );
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut shown = 0;

                    for (index, section) in sections.iter().enumerate() {
                        let lines: Vec<&String> = if query.is_empty() {
                            section.lines.iter().collect()
                        } else {
                            section
                                .lines
                                .iter()
                                .filter(|line| line.to_lowercase().contains(&query))
                                .collect()
                        };

                        if !query.is_empty() && lines.is_empty() {
                            continue;
                        }

                        shown += 1;
                        egui::CollapsingHeader::new(&section.title)
                            .id_salt(index)
                            .default_open(true)
                            .show(ui, |ui| {
                                for line in lines {
                                    reference_line(ui, line);
                                }
                            });
                    }

                    if shown == 0 {
                        ui.label("No matching entries");
                    }
                });
        });

    state.cheatsheet_window = open;
    state.search = search;
}
