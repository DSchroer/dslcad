use crate::editor::rendering::{RenderCommand, RenderState};
use crate::settings::{Settings, Store};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};
use std::collections::BTreeMap;

use crate::editor::camera::CameraCommand;
use bevy_egui::egui::Id;
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .add_event::<MenuEvent>()
            .add_systems(Update, main_ui)
            .add_persistent_res_loader("points", |value, state: &mut RenderState| {
                state.show_points = value.unwrap_or("true") == "true";
            })
            .add_persistent_res_loader("lines", |value, state: &mut RenderState| {
                state.show_lines = value.unwrap_or("true") == "true";
            })
            .add_persistent_res_loader("mesh", |value, state: &mut RenderState| {
                state.show_mesh = value.unwrap_or("true") == "true";
            })
            .add_persistent_res_loader("colors", |value, state: &mut RenderState| {
                state.part_colors = value.unwrap_or("false") == "true";
            });
    }
}

pub trait MenuAppExt {
    fn add_event_menu_button<T: Event>(
        &mut self,
        path: &'static str,
        action: impl Fn(&mut EventWriter<T>) + Send + Sync + 'static,
    ) -> &mut App;

    fn add_res_menu_button<T: Resource>(
        &mut self,
        path: &'static str,
        action: impl Fn(&mut T) + Send + Sync + 'static,
    ) -> &mut App;

    fn add_persistent_res_menu_button<T: Resource>(
        &mut self,
        path: &'static str,
        key: &'static str,
        action: impl Fn(&mut T) -> String + Send + Sync + 'static,
    ) -> &mut App;

    fn add_persistent_res_loader<T: Resource>(
        &mut self,
        key: &'static str,
        action: impl Fn(Option<&str>, &mut T) + Send + Sync + 'static,
    ) -> &mut App;
}

impl MenuAppExt for App {
    fn add_event_menu_button<T: Event>(
        &mut self,
        path: &'static str,
        action: impl Fn(&mut EventWriter<T>) + Send + Sync + 'static,
    ) -> &mut App {
        let mut path = path.split('/');
        let menu_name = TopLevelMenu::from_str(path.next().expect("menu must have top level"))
            .expect("unknown top level menu");
        let action_name = path.next().expect("menu must have action");

        self.add_systems(Startup, move |mut menu: ResMut<Menu>| {
            menu.button(menu_name, action_name);
        });

        self.add_systems(
            Update,
            move |mut events: EventReader<MenuEvent>, mut event: EventWriter<T>| {
                for click in events.iter() {
                    if click.action() == action_name {
                        action(&mut event);
                    }
                }
            },
        );
        self
    }

    fn add_res_menu_button<T: Resource>(
        &mut self,
        path: &'static str,
        action: impl Fn(&mut T) + Send + Sync + 'static,
    ) -> &mut App {
        let mut path = path.split('/');
        let menu_name = TopLevelMenu::from_str(path.next().expect("menu must have top level"))
            .expect("unknown top level menu");
        let action_name = path.next().expect("menu must have action");

        self.add_systems(Startup, move |mut menu: ResMut<Menu>| {
            menu.button(menu_name, action_name);
        });

        self.add_systems(
            Update,
            move |mut events: EventReader<MenuEvent>, mut state: ResMut<T>| {
                for click in events.iter() {
                    if click.action() == action_name {
                        action(&mut state);
                    }
                }
            },
        );
        self
    }

    fn add_persistent_res_menu_button<T: Resource>(
        &mut self,
        path: &'static str,
        key: &'static str,
        action: impl Fn(&mut T) -> String + Send + Sync + 'static,
    ) -> &mut App {
        let mut path = path.split('/');
        let menu_name = TopLevelMenu::from_str(path.next().expect("menu must have top level"))
            .expect("unknown top level menu");
        let action_name = path.next().expect("menu must have action");

        self.add_systems(Startup, move |mut menu: ResMut<Menu>| {
            menu.button(menu_name, action_name);
        });

        self.add_systems(
            Update,
            move |mut events: EventReader<MenuEvent>,
                  mut state: ResMut<T>,
                  mut store: ResMut<Settings>| {
                for click in events.iter() {
                    if click.action() == action_name {
                        let new_value = action(&mut state);
                        store.store(key, &new_value);
                    }
                }
            },
        );
        self
    }

    fn add_persistent_res_loader<T: Resource>(
        &mut self,
        key: &'static str,
        action: impl Fn(Option<&str>, &mut T) + Send + Sync + 'static,
    ) -> &mut App {
        self.add_systems(
            Startup,
            move |mut state: ResMut<T>, store: Res<Settings>| {
                let value = store.load(key);
                action(value, &mut state);
            },
        )
    }
}

#[derive(Event)]
pub struct MenuEvent(&'static str);

impl MenuEvent {
    pub fn action(&self) -> &str {
        self.0
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, EnumString, Display, IntoStaticStr, Copy, Clone)]
enum TopLevelMenu {
    File,
    View,
    Camera,
    Rendering,
    Export,
    Window,
    Help,
}

#[derive(Resource, Default)]
pub struct Menu {
    tree: BTreeMap<TopLevelMenu, Vec<&'static str>>,
}

impl Menu {
    fn button(&mut self, menu: TopLevelMenu, action: &'static str) {
        self.tree.entry(menu).or_default().push(action);
    }
}

fn main_ui(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut render_events: EventWriter<RenderCommand>,
    mut menu_events: EventWriter<MenuEvent>,
    mut render_state: ResMut<RenderState>,
    mut camera_events: EventWriter<CameraCommand>,
    mut store: ResMut<Settings>,
    menu: Res<Menu>,
) {
    egui::TopBottomPanel::top("Tools").show(egui_ctx.single_mut().get_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            for (menu, actions) in &menu.tree {
                let str: &'static str = menu.into();
                ui.menu_button(str, |ui| {
                    for action in actions {
                        if ui.button(*action).clicked() {
                            menu_events.send(MenuEvent(action));
                            ui.close_menu();
                        }
                    }

                    if *menu == TopLevelMenu::Camera {
                        let id = Id::new("orthographic");
                        let mut ortho = ui.memory(|m| m.data.get_temp(id)).unwrap_or_default();

                        if ui.checkbox(&mut ortho, "Orthographic").clicked() {
                            camera_events.send(CameraCommand::UseOrthographic(ortho));
                            ui.memory_mut(|m| m.data.insert_temp(id, ortho))
                        }
                    }

                    if *menu == TopLevelMenu::View {
                        if ui
                            .checkbox(&mut render_state.show_points, "Points")
                            .clicked()
                        {
                            store.store("points", &render_state.show_points.to_string());
                            render_events.send(RenderCommand::Redraw);
                        }
                        if ui.checkbox(&mut render_state.show_lines, "Lines").clicked() {
                            store.store("lines", &render_state.show_lines.to_string());
                            render_events.send(RenderCommand::Redraw);
                        }
                        if ui.checkbox(&mut render_state.show_mesh, "Mesh").clicked() {
                            store.store("mesh", &render_state.show_mesh.to_string());
                            render_events.send(RenderCommand::Redraw);
                        }
                        if ui
                            .checkbox(&mut render_state.part_colors, "Part Colors")
                            .clicked()
                        {
                            store.store("colors", &render_state.part_colors.to_string());
                            render_events.send(RenderCommand::Redraw);
                        }
                    }
                });
            }
        });
    });
}
