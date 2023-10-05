use crate::editor::rendering::{RenderCommand, RenderState};
use crate::settings::{Settings, Store};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};
use std::collections::BTreeMap;
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .add_event::<MenuEvent>()
            .add_system(main_ui);
    }
}

pub trait MenuAppExt {
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
    fn add_res_menu_button<T: Resource>(
        &mut self,
        path: &'static str,
        action: impl Fn(&mut T) + Send + Sync + 'static,
    ) -> &mut App {
        let mut path = path.split('/');
        let menu_name = TopLevelMenu::from_str(path.next().expect("menu must have top level"))
            .expect("unknown top level menu");
        let action_name = path.next().expect("menu must have action");

        self.add_startup_system(move |mut menu: ResMut<Menu>| {
            menu.button(menu_name, action_name);
        });

        self.add_system(
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

        self.add_startup_system(move |mut menu: ResMut<Menu>| {
            menu.button(menu_name, action_name);
        });

        self.add_system(
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
        self.add_startup_system(move |mut state: ResMut<T>, store: Res<Settings>| {
            let value = store.load(key);
            action(value, &mut state);
        })
    }
}

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

                    if *menu == TopLevelMenu::View {
                        if ui
                            .checkbox(&mut render_state.show_points, "Points")
                            .clicked()
                        {
                            render_events.send(RenderCommand::Redraw);
                        }
                        if ui.checkbox(&mut render_state.show_lines, "Lines").clicked() {
                            render_events.send(RenderCommand::Redraw);
                        }
                        if ui.checkbox(&mut render_state.show_mesh, "Mesh").clicked() {
                            render_events.send(RenderCommand::Redraw);
                        }
                    }
                });
            }
        });
    });
}
