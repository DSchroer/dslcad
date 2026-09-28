use crate::editor::lines::{
    lines_to_mesh, AnnotationLineMaterial, LineMaterial, LineMaterialPlugin,
};
use crate::editor::stl::stl_to_triangle_mesh;
use crate::editor::{model_rotation, Palette};
use bevy::prelude::*;
use bevy_points::material::PointsShaderSettings;
use bevy_points::prelude::*;
use smooth_bevy_cameras::controllers::orbit::OrbitCameraController;

use dslcad_storage::protocol::{
    Annotation, BoundingBox, Parameter, Part, Point, TextBlock, TextPlane, ViewDef,
};

pub struct ModelRenderingPlugin;

impl Plugin for ModelRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PointsPlugin, LineMaterialPlugin))
            .add_message::<RenderCommand>()
            .add_message::<RenderEvents>()
            .insert_resource(RenderState::default())
            .add_systems(
                Update,
                (
                    render_controller,
                    mesh_renderer,
                    point_renderer,
                    line_renderer,
                    annotation_renderer,
                    billboard_text,
                ),
            );
    }
}

/// Marks a text annotation that should always face the camera.
#[derive(Component)]
struct BillboardText;

#[derive(Message)]
pub enum RenderCommand {
    Draw(Vec<Part>, Vec<Annotation>, Vec<ViewDef>, Vec<Parameter>),
    Redraw,
}

/// The scene currently on screen: shared geometry, global annotations, the
/// declared views, the declared parameters and the entity that carries the
/// model rotation.
struct Model {
    parts: Vec<Part>,
    annotations: Vec<Annotation>,
    views: Vec<ViewDef>,
    parameters: Vec<Parameter>,
    entity: Entity,
}

#[derive(Resource)]
pub struct RenderState {
    model: Option<Model>,
    active_view: Option<usize>,
    pub show_points: bool,
    pub show_lines: bool,
    pub show_mesh: bool,
    pub show_grid: bool,
    pub show_annotations: bool,
    pub part_colors: bool,
}

impl RenderState {
    /// The bounds of the currently rendered model, if any. Annotations are
    /// overlays and never affect the framing.
    pub fn aabb(&self) -> Option<BoundingBox> {
        BoundingBox::from_parts(&self.model.as_ref()?.parts)
    }

    /// The available views, in declaration order.
    pub fn views(&self) -> &[ViewDef] {
        self.model
            .as_ref()
            .map(|m| m.views.as_slice())
            .unwrap_or(&[])
    }

    /// The declared parameters, in declaration order.
    pub fn parameters(&self) -> &[Parameter] {
        self.model
            .as_ref()
            .map(|m| m.parameters.as_slice())
            .unwrap_or(&[])
    }

    pub fn set_active_view(&mut self, index: Option<usize>) {
        self.active_view = index;
    }

    /// The index of the active view, or `None` for the free camera.
    pub fn active_view_index(&self) -> Option<usize> {
        self.active_view
    }

    /// Global annotations plus the active view's annotations.
    pub fn annotations(&self) -> Vec<Annotation> {
        let mut annotations = Vec::new();
        if let Some(model) = &self.model {
            annotations.extend(model.annotations.iter().cloned());
            if let Some(view) = self.active_view.and_then(|index| model.views.get(index)) {
                annotations.extend(view.annotations.iter().cloned());
            }
        }
        annotations
    }
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            show_points: true,
            show_lines: true,
            show_mesh: true,
            show_grid: true,
            show_annotations: true,
            part_colors: false,
            model: None,
            active_view: None,
        }
    }
}

#[derive(Message)]
enum RenderEvents {
    Points,
    Lines,
    Mesh,
    Annotations,
}

fn mesh_renderer(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut events: MessageReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        if let RenderEvents::Mesh = event {
            if !render_state.show_mesh {
                continue;
            }

            let Some(Model { parts, entity, .. }) = &render_state.model else {
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
                        .insert(ChildOf(*entity));
                }
            }
        }
    }
}

fn point_renderer(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut events: MessageReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut point_materials: ResMut<Assets<PointsMaterial>>,
) {
    for event in events.read() {
        if let RenderEvents::Points = event {
            if !render_state.show_points {
                continue;
            }

            let Some(Model { parts, entity, .. }) = &render_state.model else {
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
        .insert(ChildOf(parent));
}

fn line_renderer(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut events: MessageReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    for event in events.read() {
        if let RenderEvents::Lines = event {
            if !render_state.show_lines {
                continue;
            }

            let Some(Model { parts, entity, .. }) = &render_state.model else {
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
        .insert(ChildOf(parent));
}

fn render_controller(
    mut commands: Commands,
    mut events: MessageReader<RenderCommand>,
    mut render_state: ResMut<RenderState>,
    mut render_events: MessageWriter<RenderEvents>,
) {
    for event in events.read() {
        match event {
            RenderCommand::Draw(parts, annotations, views, parameters) => {
                // Keep the selected view across edits, but start on the free
                // camera ("default") when the preview first opens.
                let previous = render_state
                    .active_view
                    .and_then(|index| render_state.model.as_ref()?.views.get(index))
                    .and_then(|view| view.name.clone());

                if let Some(model) = &render_state.model {
                    commands.entity(model.entity).despawn();
                    render_state.model = None;
                }

                render_state.active_view = previous.and_then(|name| {
                    views
                        .iter()
                        .position(|view| view.name.as_deref() == Some(name.as_str()))
                });

                let bundle = commands.spawn(Transform::from_rotation(model_rotation()));
                render_state.model = Some(Model {
                    parts: parts.clone(),
                    annotations: annotations.clone(),
                    views: views.clone(),
                    parameters: parameters.clone(),
                    entity: bundle.id(),
                });
            }
            RenderCommand::Redraw => {
                if let Some(model) = &render_state.model {
                    commands.entity(model.entity).despawn();

                    let bundle = commands.spawn(Transform::from_rotation(model_rotation()));
                    render_state.model = Some(Model {
                        parts: model.parts.clone(),
                        annotations: model.annotations.clone(),
                        views: model.views.clone(),
                        parameters: model.parameters.clone(),
                        entity: bundle.id(),
                    });
                }
            }
        }

        render_events.write(RenderEvents::Annotations);
        render_events.write(RenderEvents::Points);
        render_events.write(RenderEvents::Lines);
        render_events.write(RenderEvents::Mesh);
    }
}

fn annotation_renderer(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut events: MessageReader<RenderEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<AnnotationLineMaterial>>,
    mut point_materials: ResMut<Assets<PointsMaterial>>,
) {
    for event in events.read() {
        if let RenderEvents::Annotations = event {
            if !render_state.show_annotations {
                continue;
            }

            let Some(Model { entity, .. }) = &render_state.model else {
                return;
            };

            for annotation in render_state.annotations() {
                if !annotation.lines.is_empty() {
                    spawn_annotation_lines(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &annotation.lines,
                        *entity,
                    );
                }

                if !annotation.points.is_empty() {
                    render_points(
                        &mut commands,
                        &mut meshes,
                        &mut point_materials,
                        &annotation.points,
                        *entity,
                        Palette::annotation(),
                    );
                }

                for text in &annotation.texts {
                    if text.lines.is_empty() && text.outline.is_empty() {
                        continue;
                    }

                    let text_entity =
                        spawn_text_block(&mut commands, &mut meshes, &mut materials, text, *entity);

                    if let TextPlane::Billboard = text.plane {
                        commands.entity(text_entity).insert(BillboardText);
                    }
                }
            }
        }
    }
}

/// Draw annotation strokes with a dark border baked into the same material, so
/// they stay readable over the light part, dark background and colored parts.
/// Returns the entity that carries the stroke transform so callers can place
/// text.
fn spawn_annotation_lines(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<AnnotationLineMaterial>>,
    lines: &[Vec<Point>],
    parent: Entity,
) -> Entity {
    let wrapper = commands
        .spawn(Transform::default())
        .insert(ChildOf(parent))
        .id();

    commands
        .spawn((
            Mesh3d(meshes.add(lines_to_mesh(lines))),
            MeshMaterial3d(materials.add(AnnotationLineMaterial::new(
                Palette::annotation(),
                Palette::edge(),
                2.2,
            ))),
        ))
        .insert(ChildOf(wrapper));

    wrapper
}

/// Draw a text block as a dark glyph outline under a solid colored fill. The
/// two do not overlap much, so the order they draw in does not matter and the
/// text stays readable over any part color.
fn spawn_text_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<AnnotationLineMaterial>>,
    text: &TextBlock,
    parent: Entity,
) -> Entity {
    let wrapper = commands
        .spawn(
            Transform::from_translation(Vec3::new(
                text.origin[0] as f32,
                text.origin[1] as f32,
                text.origin[2] as f32,
            ))
            .with_rotation(plane_rotation(text.plane)),
        )
        .insert(ChildOf(parent))
        .id();

    if !text.outline.is_empty() {
        commands
            .spawn((
                Mesh3d(meshes.add(lines_to_mesh(&text.outline))),
                MeshMaterial3d(materials.add(AnnotationLineMaterial::new(
                    Palette::edge(),
                    Palette::edge(),
                    2.4,
                ))),
            ))
            .insert(ChildOf(wrapper));
    }

    if !text.lines.is_empty() {
        commands
            .spawn((
                Mesh3d(meshes.add(lines_to_mesh(&text.lines))),
                MeshMaterial3d(materials.add(AnnotationLineMaterial::new(
                    Palette::annotation(),
                    Palette::annotation(),
                    1.8,
                ))),
            ))
            .insert(ChildOf(wrapper));
    }

    wrapper
}

/// The local rotation that lays text out on a fixed drawing plane. Text is
/// outlined in the XY plane, so the plane selects the local x and y axes.
fn plane_rotation(plane: TextPlane) -> Quat {
    let (right, up) = match plane {
        TextPlane::Xy | TextPlane::Billboard => (Vec3::X, Vec3::Y),
        TextPlane::Yz => (Vec3::Y, Vec3::Z),
        TextPlane::Xz => (Vec3::X, Vec3::Z),
    };

    Quat::from_mat3(&Mat3::from_cols(right, up, right.cross(up)))
}

/// Rotate billboarded text so it faces the camera. Text is a child of the model
/// root, which is rotated by [`model_rotation`], so the desired world rotation
/// is converted back into the root's local space.
fn billboard_text(
    camera: Query<&Transform, With<OrbitCameraController>>,
    mut texts: Query<&mut Transform, (With<BillboardText>, Without<OrbitCameraController>)>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };

    let right = *camera.right();
    let up = *camera.up();
    let world = Quat::from_mat3(&Mat3::from_cols(right, up, right.cross(up)));
    let local = model_rotation().inverse() * world;

    for mut transform in &mut texts {
        transform.rotation = local;
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
