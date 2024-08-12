use crate::obstacles::ObstaclePolygons;
use crate::pathfinding::{theta_star, NavMesh};
use crate::player_action::PlayerAction;
use crate::player_stats::PlayerStats;
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
    mut player_query: Query<(&mut Transform, &PlayerAction, &PlayerStats), With<Player>>,
    mut target_position: ResMut<TargetPosition>,
    mut gizmo_path: ResMut<GizmoPath>,
    time: Res<Time>,
    obstacle_polygons: Res<ObstaclePolygons>,
    mut nav_mesh: ResMut<NavMesh>,
) {
    // Handle right-click for setting the movement target
    if buttons.just_pressed(MouseButton::Right) {
        let (camera, camera_transform) = camera_query.single();
        let ground = ground_query.single();
        let (player_transform, player_action, player_stats) = player_query.single_mut();

        // Skip movement if the player is casting or has an active Q spell
        if player_action.is_casting || player_action.casting_q {
            return;
        }

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

    // Handle movement towards the target
    if let Some(path) = &mut target_position.0 {
        if !path.is_empty() {
            let (mut player_transform, player_action, player_stats) = player_query.single_mut();

            // Skip movement if the player is casting or has an active Q spell
            if player_action.is_casting || player_action.casting_q {
                return;
            }

            let target = path[0];

            let player_position_2d = Vec2::new(
                player_transform.translation.x,
                player_transform.translation.z,
            );
            let target_position_2d = Vec2::new(target.x, target.z);

            let direction_2d = (target_position_2d - player_position_2d).normalize_or_zero();
            player_transform.translation.x +=
                direction_2d.x * player_stats.speed * time.delta_seconds();
            player_transform.translation.z +=
                direction_2d.y * player_stats.speed * time.delta_seconds();

            if player_position_2d.distance(target_position_2d) < 0.1 {
                path.remove(0);
                if path.is_empty() {
                    gizmo_path.0 = None;
                }
            }
        }
    }
}
