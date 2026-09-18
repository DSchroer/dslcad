use crate::editor::lines::{lines_to_mesh, LineMaterial};
use crate::editor::Blueprint;
use bevy::prelude::*;

pub struct XYZPlugin;
impl Plugin for XYZPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, xyz_lines);
    }
}

fn xyz_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    let end = 1_000_000.0;
    let lines = vec![
        vec![[0.0, 0.0, 0.0], [end, 0.0, 0.0]],
        vec![[0.0, 0.0, 0.0], [0.0, end, 0.0]],
        vec![[0.0, 0.0, 0.0], [0.0, 0.0, end]],
    ];

    commands.spawn((
        Mesh3d(meshes.add(lines_to_mesh(&lines))),
        MeshMaterial3d(
            materials.add(LineMaterial::new(Blueprint::black(), 2.0).with_depth_bias(0.0001)),
        ),
    ));
}
