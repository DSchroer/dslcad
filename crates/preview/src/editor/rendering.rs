use crate::editor::stl::stl_to_triangle_mesh;
use crate::editor::Blueprint;
use bevy::prelude::*;
use bevy_points::material::PointsShaderSettings;
use bevy_points::prelude::*;

use bevy_polyline::material::PolylineMaterial;
use bevy_polyline::polyline::{Polyline, PolylineBundle};
use persistence::protocol::{Part, Point};

pub struct ModelRenderingPlugin;

impl Plugin for ModelRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PointsPlugin)
            .add_event::<RenderCommand>()
            .add_event::<RenderEvents>()
            .insert_resource(RenderState::default())
            .add_system(render_controller)
            .add_system(mesh_renderer)
            .add_system(point_renderer)
            .add_system(line_renderer);
    }
}

pub enum RenderCommand {
    Draw(Vec<Part>),
    Redraw,
}

#[derive(Resource)]
pub struct RenderState {
    model: Option<(Vec<Part>, Entity)>,
    pub show_points: bool,
    pub show_lines: bool,
    pub show_mesh: bool,
    pub part_colors: bool,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            show_points: true,
            show_lines: true,
            show_mesh: true,
            part_colors: false,
            model: None,
        }
    }
}

enum RenderEvents {
    Points,
    Lines,
    Mesh,
}

fn mesh_renderer(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut events: EventReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.iter() {
        if let RenderEvents::Mesh = event {
            if !render_state.show_mesh {
                continue;
            }

            let (parts, entity) = if let Some(e) = &render_state.model {
                e
            } else {
                return;
            };

            for (i, part) in parts.iter().enumerate() {
                if let Part::Object { mesh, .. } = part {
                    let mesh = stl_to_triangle_mesh(mesh);

                    let color = if render_state.part_colors {
                        Blueprint::part(i)
                    } else {
                        Blueprint::white()
                    };

                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(color.into()),
                            ..Default::default()
                        })
                        .set_parent(*entity);
                }
            }
        }
    }
}

fn point_renderer(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut events: EventReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut point_materials: ResMut<Assets<PointsMaterial>>,
) {
    for event in events.iter() {
        if let RenderEvents::Points = event {
            if !render_state.show_points {
                continue;
            }

            let (parts, entity) = if let Some(e) = &render_state.model {
                e
            } else {
                return;
            };

            for part in parts {
                match part {
                    Part::Empty => {}
                    Part::Planar { points, .. } => render_points(
                        &mut commands,
                        &mut meshes,
                        &mut point_materials,
                        points,
                        *entity,
                    ),
                    Part::Object { points, .. } => render_points(
                        &mut commands,
                        &mut meshes,
                        &mut point_materials,
                        points,
                        *entity,
                    ),
                }
            }
        }
    }
}

fn render_points(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    point_materials: &mut ResMut<Assets<PointsMaterial>>,
    points: &[Point],
    parent: Entity,
) {
    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(
                PointsMesh::from_iter(
                    points
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
        .set_parent(parent);
}

fn line_renderer(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut events: EventReader<RenderEvents>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    for event in events.iter() {
        if let RenderEvents::Lines = event {
            if !render_state.show_lines {
                continue;
            }

            let (parts, entity) = if let Some(e) = &render_state.model {
                e
            } else {
                return;
            };

            for part in parts {
                match part {
                    Part::Empty => {}
                    Part::Planar { lines, .. } => render_lines(
                        &mut commands,
                        &mut polylines,
                        &mut polyline_materials,
                        lines,
                        *entity,
                    ),
                    Part::Object { lines, .. } => render_lines(
                        &mut commands,
                        &mut polylines,
                        &mut polyline_materials,
                        lines,
                        *entity,
                    ),
                }
            }
        }
    }
}

fn render_lines(
    commands: &mut Commands,
    polylines: &mut ResMut<Assets<Polyline>>,
    polyline_materials: &mut ResMut<Assets<PolylineMaterial>>,
    lines: &Vec<Vec<Point>>,
    parent: Entity,
) {
    for line in lines {
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
            .set_parent(parent);
    }
}

fn render_controller(
    mut commands: Commands,
    mut events: EventReader<RenderCommand>,
    mut render_state: ResMut<RenderState>,
    mut render_events: EventWriter<RenderEvents>,
) {
    for event in events.iter() {
        match event {
            RenderCommand::Draw(render) => {
                if let Some((_, id)) = render_state.model {
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
                render_state.model = Some((render.clone(), bundle.id()));

                render_events.send(RenderEvents::Points);
                render_events.send(RenderEvents::Lines);
                render_events.send(RenderEvents::Mesh);
            }
            RenderCommand::Redraw => {
                if let Some((render, id)) = &render_state.model {
                    commands.entity(*id).despawn_recursive();

                    let bundle = commands.spawn(SpatialBundle {
                        transform: Transform::from_rotation(Quat::from_euler(
                            EulerRot::XYZ,
                            -std::f32::consts::FRAC_PI_2,
                            0.0,
                            -std::f32::consts::FRAC_PI_2,
                        )),
                        ..Default::default()
                    });
                    render_state.model = Some((render.clone(), bundle.id()));
                }

                render_events.send(RenderEvents::Points);
                render_events.send(RenderEvents::Lines);
                render_events.send(RenderEvents::Mesh);
            }
        }
    }
}
