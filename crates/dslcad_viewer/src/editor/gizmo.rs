use crate::editor::{Axis, Palette};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};
use egui::{Align2, Color32, FontId, Id, Order, Sense, Stroke, Vec2};

const GIZMO_SIZE: f32 = 88.0;
const GIZMO_RADIUS: f32 = 30.0;
const GIZMO_MARGIN: Vec2 = Vec2::new(-16.0, 44.0);

/// Draws a small axis indicator in the corner of the viewport that rotates with
/// the camera, like the origin gizmo of a CAD application.
pub struct AxisGizmoPlugin;

impl Plugin for AxisGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, axis_gizmo);
    }
}

fn axis_gizmo(
    camera: Query<&GlobalTransform, With<Camera3d>>,
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
) {
    let Ok(camera) = camera.get_single() else {
        return;
    };
    let Ok(mut context) = egui_ctx.get_single_mut() else {
        return;
    };

    let axes = projected_axes(camera.rotation().inverse());

    egui::Area::new(Id::new("axis_gizmo"))
        .anchor(Align2::RIGHT_TOP, GIZMO_MARGIN)
        .order(Order::Foreground)
        .interactable(false)
        .show(context.get_mut(), |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(GIZMO_SIZE), Sense::hover());
            let center = rect.center();
            let painter = ui.painter();

            for (axis, direction) in axes {
                let tip = center + Vec2::new(direction.x, -direction.y) * GIZMO_RADIUS;
                let color = egui_color(Palette::axis(axis));

                painter.line_segment([center, tip], Stroke::new(2.0_f32, color));
                painter.circle_filled(tip, 8.0, color);
                painter.text(
                    tip,
                    Align2::CENTER_CENTER,
                    axis.label(),
                    FontId::proportional(11.0),
                    Color32::WHITE,
                );
            }
        });
}

/// Projects every axis into view space. The axes pointing away from the camera
/// come first so that they are drawn behind the others.
fn projected_axes(view: Quat) -> [(Axis, Vec3); 3] {
    let mut axes = Axis::ALL.map(|axis| (axis, view * axis.direction()));
    axes.sort_by(|a, b| b.1.z.total_cmp(&a.1.z));
    axes
}

fn egui_color(color: Color) -> Color32 {
    let color = color.to_srgba();
    Color32::from_rgba_unmultiplied(
        (color.red * 255.0).round() as u8,
        (color.green * 255.0).round() as u8,
        (color.blue * 255.0).round() as u8,
        (color.alpha * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen_direction(axes: &[(Axis, Vec3); 3], axis: Axis) -> Vec2 {
        let (_, direction) = axes.iter().find(|(a, _)| *a == axis).unwrap();
        Vec2::new(direction.x, direction.y)
    }

    #[test]
    fn axes_are_sorted_back_to_front() {
        let axes = projected_axes(Quat::from_rotation_y(0.7));

        assert!(axes.windows(2).all(|window| window[0].1.z >= window[1].1.z));
    }

    #[test]
    fn axes_are_projected_onto_the_screen() {
        let axes = projected_axes(Quat::IDENTITY);

        // Looking down the renderer's z axis, part x points at the camera
        assert!(screen_direction(&axes, Axis::X).length() < 1e-6);
        assert!((screen_direction(&axes, Axis::Y) - Vec2::X).length() < 1e-6);
        assert!((screen_direction(&axes, Axis::Z) - Vec2::Y).length() < 1e-6);
    }
}
