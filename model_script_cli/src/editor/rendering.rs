use crate::editor::point_render::PointMaterial;
use crate::editor::stl::stl_to_triangle_mesh;
use crate::editor::Blueprint;
use bevy::prelude::*;
use bevy_polyline::material::PolylineMaterial;
use bevy_polyline::polyline::{Polyline, PolylineBundle};

pub struct ModelRenderingPlugin;

impl Plugin for ModelRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RenderCommand>()
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

            let mut entity = if let Some(e) = render_state.model {
                commands.get_entity(e).unwrap()
            } else {
                return;
            };

            if let Some(Ok(model)) = &state.output {
                let mesh = stl_to_triangle_mesh(model.mesh());

                entity.add_children(|builder| {
                    builder.spawn(PbrBundle {
                        mesh: meshes.add(mesh),
                        material: materials.add(Blueprint::white().into()),
                        ..Default::default()
                    });
                });
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
    mut point_materials: ResMut<Assets<PointMaterial>>,
) {
    for event in events.iter() {
        if let RenderEvents::Points = event {
            if !state.show_points {
                continue;
            }

            let mut entity = if let Some(e) = render_state.model {
                commands.get_entity(e).unwrap()
            } else {
                return;
            };

            if let Some(Ok(model)) = &state.output {
                entity.add_children(|builder| {
                    for point in model.points() {
                        builder.spawn(MaterialMeshBundle {
                            mesh: meshes.add(
                                shape::UVSphere {
                                    radius: 1.0,
                                    sectors: 3,
                                    stacks: 3,
                                }
                                .into(),
                            ),
                            material: point_materials.add(PointMaterial {
                                color: Blueprint::black(),
                            }),
                            transform: Transform::from_translation(Vec3::new(
                                point[0] as f32,
                                point[1] as f32,
                                point[2] as f32,
                            )),
                            ..Default::default()
                        });
                    }
                });
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

            let mut entity = if let Some(e) = render_state.model {
                commands.get_entity(e).unwrap()
            } else {
                return;
            };

            if let Some(Ok(model)) = &state.output {
                for line in model.lines() {
                    entity.add_children(|builder| {
                        builder.spawn(PolylineBundle {
                            polyline: polylines.add(Polyline {
                                vertices: line
                                    .iter()
                                    .map(|p| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32))
                                    .collect(),
                            }),
                            material: polyline_materials.add(PolylineMaterial {
                                width: 2.0,
                                color: Blueprint::white(),
                                perspective: false,
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    });
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
