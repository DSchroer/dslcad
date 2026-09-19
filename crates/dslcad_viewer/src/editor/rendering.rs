use crate::editor::lines::{lines_to_mesh, LineMaterial, LineMaterialPlugin};
use crate::editor::stl::stl_to_triangle_mesh;
use crate::editor::{model_rotation, Palette};
use bevy::prelude::*;
use bevy_points::material::PointsShaderSettings;
use bevy_points::prelude::*;

use dslcad_storage::protocol::{BoundingBox, Part, Point};

pub struct ModelRenderingPlugin;

impl Plugin for ModelRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PointsPlugin, LineMaterialPlugin))
            .add_event::<RenderCommand>()
            .add_event::<RenderEvents>()
            .insert_resource(RenderState::default())
            .add_systems(
                Update,
                (
                    render_controller,
                    mesh_renderer,
                    point_renderer,
                    line_renderer,
                ),
            );
    }
}

#[derive(Event)]
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
    pub show_grid: bool,
    pub part_colors: bool,
}

impl RenderState {
    /// The bounds of the currently rendered model, if any.
    pub fn aabb(&self) -> Option<BoundingBox> {
        let (parts, _) = self.model.as_ref()?;
        BoundingBox::from_parts(parts)
    }
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            show_points: true,
            show_lines: true,
            show_mesh: true,
            show_grid: true,
            part_colors: false,
            model: None,
        }
    }
}

#[derive(Event)]
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
    for event in events.read() {
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
                        Palette::part_color(i)
                    } else {
                        Palette::part()
                    };

                    commands
                        .spawn((
                            Mesh3d(meshes.add(mesh)),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: color,
                                cull_mode: None,
                                ..Default::default()
                            })),
                        ))
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
    for event in events.read() {
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
                let color = feature_color(&render_state, part);
                match part {
                    Part::Empty => {}
                    Part::Planar { points, .. } => render_points(
                        &mut commands,
                        &mut meshes,
                        &mut point_materials,
                        points,
                        *entity,
                        color,
                    ),
                    Part::Object { points, .. } => render_points(
                        &mut commands,
                        &mut meshes,
                        &mut point_materials,
                        points,
                        *entity,
                        color,
                    ),
                }
            }
        }
    }
}

/// Lines and points are drawn dark on top of a part's mesh, but bright when
/// they are drawn directly on the viewport background, such as for 2D parts or
/// when meshes are hidden.
fn feature_color(render_state: &RenderState, part: &Part) -> Color {
    match part {
        Part::Object { .. } if render_state.show_mesh => Palette::edge(),
        _ => Palette::wireframe(),
    }
}

fn render_points(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    point_materials: &mut ResMut<Assets<PointsMaterial>>,
    points: &[Point],
    parent: Entity,
    color: Color,
) {
    commands
        .spawn((
            Mesh3d::from(
                meshes.add(PointsMesh::from_iter(
                    points
                        .iter()
                        .map(|p| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32)),
                )),
            ),
            MeshMaterial3d(point_materials.add(PointsMaterial {
                settings: PointsShaderSettings {
                    point_size: 10.0,
                    color: color.into(),
                    ..Default::default()
                },
                perspective: false,
                circle: true,
                ..Default::default()
            })),
        ))
        .set_parent(parent);
}

fn line_renderer(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut events: EventReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    for event in events.read() {
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
                let color = feature_color(&render_state, part);
                match part {
                    Part::Empty => {}
                    Part::Planar { lines, .. } => render_lines(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        lines,
                        *entity,
                        color,
                    ),
                    Part::Object { lines, .. } => render_lines(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        lines,
                        *entity,
                        color,
                    ),
                }
            }
        }
    }
}

fn render_lines(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<LineMaterial>>,
    lines: &[Vec<Point>],
    parent: Entity,
    color: Color,
) {
    if lines.is_empty() {
        return;
    }

    commands
        .spawn((
            Mesh3d(meshes.add(lines_to_mesh(lines))),
            MeshMaterial3d(materials.add(LineMaterial::new(color, 2.0))),
        ))
        .set_parent(parent);
}

fn render_controller(
    mut commands: Commands,
    mut events: EventReader<RenderCommand>,
    mut render_state: ResMut<RenderState>,
    mut render_events: EventWriter<RenderEvents>,
) {
    for event in events.read() {
        match event {
            RenderCommand::Draw(render) => {
                if let Some((_, id)) = render_state.model {
                    commands.entity(id).despawn_recursive();
                    render_state.model = None;
                }

                let bundle = commands.spawn(Transform::from_rotation(model_rotation()));
                render_state.model = Some((render.clone(), bundle.id()));

                render_events.send(RenderEvents::Points);
                render_events.send(RenderEvents::Lines);
                render_events.send(RenderEvents::Mesh);
            }
            RenderCommand::Redraw => {
                if let Some((render, id)) = &render_state.model {
                    commands.entity(*id).despawn_recursive();

                    let bundle = commands.spawn(Transform::from_rotation(model_rotation()));
                    render_state.model = Some((render.clone(), bundle.id()));
                }

                render_events.send(RenderEvents::Points);
                render_events.send(RenderEvents::Lines);
                render_events.send(RenderEvents::Mesh);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dslcad_storage::protocol::Mesh;

    fn object() -> Part {
        Part::Object {
            points: vec![],
            lines: vec![],
            mesh: Mesh {
                vertices: vec![],
                triangles: vec![],
                normals: vec![],
            },
        }
    }

    #[test]
    fn feature_colors_follow_the_background() {
        let planar = Part::Planar {
            points: vec![],
            lines: vec![],
        };

        // Edges of a mesh stay dark on the light part
        assert_eq!(
            Palette::edge(),
            feature_color(&RenderState::default(), &object())
        );

        // Without a mesh and for 2D parts the features sit on the background
        let wireframe = RenderState {
            show_mesh: false,
            ..Default::default()
        };
        assert_eq!(Palette::wireframe(), feature_color(&wireframe, &object()));
        assert_eq!(
            Palette::wireframe(),
            feature_color(&RenderState::default(), &planar)
        );
    }
}
