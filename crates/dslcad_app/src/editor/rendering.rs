use crate::editor::stl::stl_to_triangle_mesh;
use crate::editor::Blueprint;
use bevy::prelude::*;
use bevy_points::material::PointsShaderSettings;
use bevy_points::prelude::*;

use bevy_polyline::material::PolylineMaterial;
use bevy_polyline::polyline::{Polyline, PolylineBundle};

pub struct ModelRenderingPlugin;

impl Plugin for ModelRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PointsPlugin)
            .add_event::<RenderCommand>()
            .add_event::<RenderEvents>()
            .insert_resource(RenderState { model: None })
            .add_system(render_controller)
            .add_system(mesh_renderer)
            .add_system(point_renderer)
            .add_system(line_renderer);
    }
}

pub enum RenderCommand {
    Redraw,
}

#[derive(Resource)]
pub struct RenderState {
    model: Option<Entity>,
}

enum RenderEvents {
    Points,
    Lines,
    Mesh,
}

fn mesh_renderer(
    mut commands: Commands,
    state: Res<super::State>,
    render_state: Res<RenderState>,
    mut events: EventReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.iter() {
        if let RenderEvents::Mesh = event {
            if !state.show_mesh {
                continue;
            }

            let entity = if let Some(e) = render_state.model {
                e
            } else {
                return;
            };

            if let Some(Ok(models)) = &state.output {
                for model in models {
                    let mesh = stl_to_triangle_mesh(model.mesh());

                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(Blueprint::white().into()),
                            ..Default::default()
                        })
                        .set_parent(entity);
                }
            }
        }
    }
}

fn point_renderer(
    mut commands: Commands,
    state: Res<super::State>,
    render_state: Res<RenderState>,
    mut events: EventReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut point_materials: ResMut<Assets<PointsMaterial>>,
) {
    for event in events.iter() {
        if let RenderEvents::Points = event {
            if !state.show_points {
                continue;
            }

            let entity = if let Some(e) = render_state.model {
                e
            } else {
                return;
            };

            if let Some(Ok(models)) = &state.output {
                for model in models {
                    commands
                        .spawn(MaterialMeshBundle {
                            mesh: meshes.add(
                                PointsMesh::from_iter(
                                    model
                                        .points()
                                        .iter()
                                        .map(|p| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32)),
                                )
                                .into(),
                            ),
                            material: point_materials.add(PointsMaterial {
                                settings: PointsShaderSettings {
                                    point_size: 10.0,
                                    color: Blueprint::black(),
                                    ..Default::default()
                                },
                                perspective: false,
                                circle: true,
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .set_parent(entity);
                }
            }
        }
    }
}

fn line_renderer(
    mut commands: Commands,
    state: Res<super::State>,
    render_state: Res<RenderState>,
    mut events: EventReader<RenderEvents>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    for event in events.iter() {
        if let RenderEvents::Lines = event {
            if !state.show_lines {
                continue;
            }

            let entity = if let Some(e) = render_state.model {
                e
            } else {
                return;
            };

            if let Some(Ok(models)) = &state.output {
                for model in models {
                    for line in model.lines() {
                        commands
                            .spawn(PolylineBundle {
                                polyline: polylines.add(Polyline {
                                    vertices: line
                                        .iter()
                                        .map(|p| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32))
                                        .collect(),
                                }),
                                material: polyline_materials.add(PolylineMaterial {
                                    width: 2.0,
                                    color: Blueprint::black(),
                                    perspective: false,
                                    ..Default::default()
                                }),
                                ..Default::default()
                            })
                            .set_parent(entity);
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_controller(
    mut commands: Commands,
    mut events: EventReader<RenderCommand>,
    mut render_state: ResMut<RenderState>,
    mut render_events: EventWriter<RenderEvents>,
) {
    for event in events.iter() {
        match event {
            RenderCommand::Redraw => {
                if let Some(id) = render_state.model {
                    commands.entity(id).despawn_recursive();
                    render_state.model = None;
                }
                let bundle = commands.spawn(SpatialBundle {
                    transform: Transform::from_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        -std::f32::consts::FRAC_PI_2,
                        0.0,
                        -std::f32::consts::FRAC_PI_2,
                    )),
                    ..Default::default()
                });
                render_state.model = Some(bundle.id());

                render_events.send(RenderEvents::Points);
                render_events.send(RenderEvents::Lines);
                render_events.send(RenderEvents::Mesh);
            }
        }
    }
}
