use crate::editor::lines::{lines_to_mesh, LineMaterial};
use crate::editor::rendering::RenderState;
use crate::editor::{Axis, Palette};
use bevy::prelude::*;
use dslcad_storage::protocol::{BoundingBox, Point};

/// Axis length used before the first model is rendered.
const DEFAULT_AXIS_LENGTH: f32 = 10.0;

/// The renderer uses reverse-z, so a negative depth bias moves a line in front
/// of the geometry it overlaps and a positive bias moves it behind.
const AXIS_DEPTH_BIAS: f32 = -0.0001;
const GRID_DEPTH_BIAS: f32 = 0.0001;

pub struct XYZPlugin;

impl Plugin for XYZPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridState>()
            .add_systems(Startup, setup)
            .add_systems(Update, update_grid);
    }
}

#[derive(Resource)]
struct AxisLines([Entity; 3]);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    let axes = Axis::ALL.map(|axis| spawn_axis(&mut commands, &mut meshes, &mut materials, axis));
    commands.insert_resource(AxisLines(axes));

    commands.insert_resource(GridMaterials {
        minor: materials.add(grid_material(Palette::grid_minor())),
        major: materials.add(grid_material(Palette::grid_major())),
    });
}

fn grid_material(color: Color) -> LineMaterial {
    LineMaterial::new(color, 1.0).with_depth_bias(GRID_DEPTH_BIAS)
}

/// Draws the positive and negative direction of a part axis through the origin.
/// The positive direction is colored red (x), green (y) or blue (z) while the
/// negative direction is dimmed. The returned root entity is scaled to fit the
/// model once it is rendered.
fn spawn_axis(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<LineMaterial>>,
    axis: Axis,
) -> Entity {
    let direction = axis.direction().as_dvec3().to_array();

    commands
        .spawn(Transform::from_scale(Vec3::splat(DEFAULT_AXIS_LENGTH)))
        .with_children(|parent| {
            for (line, color, width) in [
                (vec![[0.0, 0.0, 0.0], direction], Palette::axis(axis), 3.0),
                (
                    vec![direction, [0.0, 0.0, 0.0]],
                    Palette::negative_axis(axis),
                    2.5,
                ),
            ] {
                parent.spawn((
                    Mesh3d(meshes.add(lines_to_mesh(&[line]))),
                    MeshMaterial3d(
                        materials
                            .add(LineMaterial::new(color, width).with_depth_bias(AXIS_DEPTH_BIAS)),
                    ),
                ));
            }
        })
        .id()
}

#[derive(Resource)]
struct GridMaterials {
    minor: Handle<LineMaterial>,
    major: Handle<LineMaterial>,
}

#[derive(Resource, Default)]
struct GridState {
    bounds: Option<BoundingBox>,
    visible: bool,
    root: Option<Entity>,
}

/// Rebuilds the ground grid whenever the rendered model or its visibility
/// changes and keeps the axes scaled to the model.
fn update_grid(
    mut commands: Commands,
    render_state: Res<RenderState>,
    axes: Res<AxisLines>,
    materials: Res<GridMaterials>,
    mut state: ResMut<GridState>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let bounds = render_state.aabb();
    if bounds == state.bounds && render_state.show_grid == state.visible {
        return;
    }

    state.bounds = bounds.clone();
    state.visible = render_state.show_grid;

    if let Some(root) = state.root.take() {
        commands.entity(root).despawn_recursive();
    }

    let Some(bounds) = bounds else {
        return;
    };

    let (minor, major, extent) = grid_lines(&bounds);
    for entity in axes.0 {
        commands
            .entity(entity)
            .insert(Transform::from_scale(Vec3::splat(extent as f32)));
    }

    if !render_state.show_grid {
        return;
    }

    let root = commands
        .spawn(Transform::default())
        .with_children(|parent| {
            for (lines, material) in [
                (&minor, materials.minor.clone()),
                (&major, materials.major.clone()),
            ] {
                parent.spawn((
                    Mesh3d(meshes.add(lines_to_mesh(lines))),
                    MeshMaterial3d(material),
                ));
            }
        })
        .id();
    state.root = Some(root);
}

/// Chooses a grid spacing of 1, 2 or 5 times a power of ten that divides the
/// given reach into roughly ten cells.
fn grid_spacing(reach: f64) -> f64 {
    if !reach.is_finite() || reach <= 0.0 {
        return 1.0;
    }

    let magnitude = 10.0_f64.powf(reach.log10().floor());
    let normalized = reach / magnitude;
    let factor = if normalized < 1.5 {
        1.0
    } else if normalized < 3.5 {
        2.0
    } else if normalized < 7.5 {
        5.0
    } else {
        10.0
    };

    factor * magnitude
}

/// Builds the minor and major lines of the ground grid and returns them with
/// the half extent of the grid. The grid is centered on the origin and large
/// enough to cover the bounds of the model.
fn grid_lines(bounds: &BoundingBox) -> (Vec<Vec<Point>>, Vec<Vec<Point>>, f64) {
    let center = bounds.center();
    let scale = f64::max(bounds.max_len(), 1.0);
    let reach = center
        .iter()
        .fold(scale, |reach, value| reach.max(value.abs()))
        + scale;
    let spacing = grid_spacing(reach / 10.0);
    let count = (reach / spacing).ceil().clamp(1.0, 100.0) as i64;
    let extent = spacing * count as f64;

    let mut minor = Vec::new();
    let mut major = Vec::new();

    for index in -count..=count {
        // The axes already mark the origin
        if index == 0 {
            continue;
        }

        let offset = index as f64 * spacing;
        let lines = if index % 5 == 0 {
            &mut major
        } else {
            &mut minor
        };
        lines.push(vec![[-extent, 0.0, offset], [extent, 0.0, offset]]);
        lines.push(vec![[offset, 0.0, -extent], [offset, 0.0, extent]]);
    }

    (minor, major, extent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dslcad_storage::protocol::Part;

    fn bounds() -> BoundingBox {
        BoundingBox::from_parts([&Part::Planar {
            points: vec![[0., 0., 0.], [10., 10., 0.]],
            lines: vec![],
        }])
        .unwrap()
    }

    #[test]
    fn spacing_uses_nice_numbers() {
        assert_eq!(1.0, grid_spacing(1.0));
        assert_eq!(2.0, grid_spacing(2.0));
        assert_eq!(5.0, grid_spacing(5.0));
        assert_eq!(10.0, grid_spacing(12.0));
        assert_eq!(20.0, grid_spacing(25.0));
    }

    #[test]
    fn grid_covers_the_model_bounds() {
        let (minor, major, extent) = grid_lines(&bounds());

        assert!(extent > 10.0);
        assert!(minor.len() > major.len());
        for line in minor.iter().chain(&major) {
            // No line sits on the origin, the axes are drawn there
            assert_ne!(line[0][2], 0.0);
            assert_ne!(line[0][0], 0.0);
            // Every line stays within the extent of the grid
            assert!(line[0][0].abs() <= extent);
            assert!(line[0][2].abs() <= extent);
        }
    }
}
