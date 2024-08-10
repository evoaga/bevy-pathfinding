use crate::obstacles::ObstaclePolygons;
use crate::pathfinding::{theta_star, NavMesh};
use crate::utils::{does_line_intersect_polygon, Point};
use bevy::prelude::*;
use std::time::Instant;

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct TargetPosition(pub Option<Vec<Vec3>>);

#[derive(Resource)]
pub struct GizmoPath(pub Option<Vec<Vec3>>);

pub fn move_player(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<crate::Ground>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut target_position: ResMut<TargetPosition>,
    mut gizmo_path: ResMut<GizmoPath>,
    time: Res<Time>,
    obstacle_polygons: Res<ObstaclePolygons>,
    mut nav_mesh: ResMut<NavMesh>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        let (camera, camera_transform) = camera_query.single();
        let ground = ground_query.single();
        let player_transform = player_query.single_mut();

        if let Some(cursor_position) = windows.single().cursor_position() {
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                if let Some(distance) =
                    ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
                {
                    let goal_position = ray.get_point(distance);

                    let start_position = Point {
                        x: player_transform.translation.x,
                        y: player_transform.translation.y,
                        z: player_transform.translation.z,
                    };

                    let goal_point = Point {
                        x: goal_position.x,
                        y: goal_position.y,
                        z: goal_position.z,
                    };

                    let mut intersects = false;
                    for polygon in &obstacle_polygons.polygons {
                        if does_line_intersect_polygon(&start_position, &goal_point, polygon) {
                            intersects = true;
                            break;
                        }
                    }

                    if !intersects {
                        target_position.0 = Some(vec![goal_position]);
                        gizmo_path.0 = Some(vec![goal_position]);
                    } else {
                        // Start timing the theta_star calculation
                        let start_time = Instant::now();

                        let path = theta_star(
                            &mut nav_mesh,
                            start_position,
                            goal_point,
                            &obstacle_polygons.polygons,
                        );

                        // Calculate the duration and print it
                        let duration = start_time.elapsed().as_secs_f64();
                        println!("theta_star calculation took: {:?}", duration);

                        if !path.is_empty() {
                            target_position.0 =
                                Some(path.iter().map(|p| Vec3::new(p.x, p.y, p.z)).collect());
                            gizmo_path.0 =
                                Some(path.iter().map(|p| Vec3::new(p.x, p.y, p.z)).collect());
                        } else {
                            println!("No valid path found.");
                        }
                    }
                }
            }
        }
    }

    if let Some(path) = &mut target_position.0 {
        if !path.is_empty() {
            let mut player_transform = player_query.single_mut();
            let target = path[0];

            let player_position_2d = Vec2::new(
                player_transform.translation.x,
                player_transform.translation.z,
            );
            let target_position_2d = Vec2::new(target.x, target.z);

            let direction_2d = (target_position_2d - player_position_2d).normalize_or_zero();
            player_transform.translation.x += direction_2d.x * 5.0 * time.delta_seconds();
            player_transform.translation.z += direction_2d.y * 5.0 * time.delta_seconds();

            if player_position_2d.distance(target_position_2d) < 0.1 {
                path.remove(0);
                if path.is_empty() {
                    gizmo_path.0 = None;
                }
            }
        }
    }
}

pub fn draw_path_gizmos(
    gizmo_path: Res<GizmoPath>,
    ground_query: Query<&GlobalTransform, With<crate::Ground>>,
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

pub fn move_player_with_wasd(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.z += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.z -= 1.0;
    }

    if direction != Vec3::ZERO {
        direction = direction.normalize();
        player_transform.translation += direction * 5.0 * time.delta_seconds();
    }
}
