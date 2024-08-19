use crate::player::GizmoPath;
use crate::Ground;
use bevy::prelude::*;

pub fn draw_path_gizmos(
    gizmo_path: Res<GizmoPath>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    mut gizmos: Gizmos,
) {
    let ground = ground_query.single();

    if let Some(path) = &gizmo_path.0 {
        // Draw circles at each waypoint
        for target in path {
            gizmos.circle(*target + Vec3::Y * 0.01, ground.up(), 0.2, Color::WHITE);
        }

        // Draw lines connecting each waypoint
        for window in path.windows(2) {
            if let [start, end] = window {
                gizmos.line(
                    *start + Vec3::Y * 0.01,
                    *end + Vec3::Y * 0.01,
                    Color::srgb(0.5, 0.0, 0.5),
                );
            }
        }
    }
}
