use crate::editor::Blueprint;
use bevy::prelude::*;
use bevy_polyline::material::PolylineMaterial;
use bevy_polyline::polyline::{Polyline, PolylineBundle};

pub struct XYZPlugin;
impl Plugin for XYZPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(xyz_lines);
    }
}

fn xyz_lines(
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    let end = 1_000_000.0;
    let bias = 0.0001;
    let color = Blueprint::black();
    let origin = Vec3::new(0.0, 0.0, 0.0);

    commands.spawn(PolylineBundle {
        polyline: polylines.add(Polyline {
            vertices: vec![origin, Vec3::new(end, 0.0, 0.0)],
        }),
        material: polyline_materials.add(PolylineMaterial {
            width: 2.0,
            color,
            perspective: false,
            depth_bias: bias,
        }),
        ..Default::default()
    });
    commands.spawn(PolylineBundle {
        polyline: polylines.add(Polyline {
            vertices: vec![origin, Vec3::new(0.0, end, 0.0)],
        }),
        material: polyline_materials.add(PolylineMaterial {
            width: 2.0,
            color,
            perspective: false,
            depth_bias: bias,
        }),
        ..Default::default()
    });
    commands.spawn(PolylineBundle {
        polyline: polylines.add(Polyline {
            vertices: vec![origin, Vec3::new(0.0, 0.0, end)],
        }),
        material: polyline_materials.add(PolylineMaterial {
            width: 2.0,
            color,
            perspective: false,
            depth_bias: bias,
        }),
        ..Default::default()
    });
}
