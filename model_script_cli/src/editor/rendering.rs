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
            .insert_resource(RenderState { model: None })
            .add_system(render_controller);
    }
}

pub enum RenderCommand {
    Redraw,
}

#[derive(Resource)]
pub struct RenderState {
    model: Option<Entity>,
}

#[allow(clippy::too_many_arguments)]
fn render_controller(
    mut commands: Commands,
    mut events: EventReader<RenderCommand>,
    state: Res<super::State>,
    mut render_state: ResMut<RenderState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut point_materials: ResMut<Assets<PointMaterial>>,
) {
    for event in events.iter() {
        match event {
            RenderCommand::Redraw => {
                if let Some(id) = render_state.model {
                    commands.entity(id).despawn_recursive();
                    render_state.model = None;
                }

                if let Some(Ok(model)) = &state.output {
                    let mut bundle = commands.spawn(SpatialBundle {
                        transform: Transform::from_rotation(Quat::from_euler(
                            EulerRot::XYZ,
                            -std::f32::consts::FRAC_PI_2,
                            0.0,
                            -std::f32::consts::FRAC_PI_2,
                        )),
                        ..Default::default()
                    });
                    render_state.model = Some(bundle.id());
                    bundle.add_children(|builder| {
                        if state.show_points {
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
                        }

                        if state.show_lines {
                            for line in model.lines() {
                                builder.spawn(PolylineBundle {
                                    polyline: polylines.add(Polyline {
                                        vertices: line
                                            .iter()
                                            .map(|p| {
                                                Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32)
                                            })
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
                            }
                        }

                        if state.show_mesh {
                            let mesh = stl_to_triangle_mesh(model.mesh());

                            builder.spawn(PbrBundle {
                                mesh: meshes.add(mesh),
                                material: materials.add(Blueprint::white().into()),
                                ..Default::default()
                            });
                        }
                    });
                }
            }
        }
    }
}
