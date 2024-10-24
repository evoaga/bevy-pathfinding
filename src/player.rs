use crate::obstacles::ObstaclePolygons;
use crate::pathfinding::{theta_star, NavMesh};
use crate::player_stats::PlayerStats;
use crate::utils::{does_line_intersect_polygon, Point, Polygon};
use bevy::prelude::*;
use std::time::Instant;

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct TargetPosition(pub Option<Vec<Vec3>>);

#[derive(Resource)]
pub struct GizmoPath(pub Option<Vec<Vec3>>);

#[derive(Resource, Default)]
pub struct LastTargetPosition(pub Option<Vec3>);

const SIGNIFICANT_CHANGE_THRESHOLD: f32 = 0.5;

pub fn handle_right_click_set_target_position(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<crate::Ground>>,
    mut player_query: Query<(&Transform, &PlayerStats), With<Player>>,
    mut target_position: ResMut<TargetPosition>,
    mut gizmo_path: ResMut<GizmoPath>,
    obstacle_polygons: Res<ObstaclePolygons>,
    mut nav_mesh: ResMut<NavMesh>,
    mut last_target_position: ResMut<LastTargetPosition>,
) {
    if !buttons.pressed(MouseButton::Right) {
        return;
    }

    let (camera, camera_transform) = camera_query.single();
    let ground = ground_query.single();
    let (player_transform, _player_stats) = player_query.single_mut();

    let cursor_position = match windows.single().cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    let ray = match camera.viewport_to_world(camera_transform, cursor_position) {
        Some(ray) => ray,
        None => return,
    };

    let distance =
        match ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) {
            Some(dist) => dist,
            None => return,
        };

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

    // Option to hold the first intersecting polygon
    let mut first_intersecting_polygon: Option<Polygon> = None;
    let mut multiple_intersections = false;

    for polygon in &obstacle_polygons.polygons {
        if does_line_intersect_polygon(&start_position, &goal_point, polygon) {
            if first_intersecting_polygon.is_some() {
                multiple_intersections = true;
                break;
            } else {
                first_intersecting_polygon = Some(polygon.clone());
            }
        }
    }

    if first_intersecting_polygon.is_some() || multiple_intersections {
        let significant_change = match last_target_position.0 {
            Some(last_position) => {
                goal_position.distance(last_position) > SIGNIFICANT_CHANGE_THRESHOLD
            }
            None => true,
        };

        if !significant_change {
            return;
        }

        last_target_position.0 = Some(goal_position);

        let start_time = Instant::now();

        // Decide whether to use a single polygon or all polygons
        let path = if multiple_intersections {
            theta_star(
                &mut nav_mesh,
                start_position,
                goal_point,
                &obstacle_polygons.polygons,
            )
        } else {
            theta_star(
                &mut nav_mesh,
                start_position,
                goal_point,
                &[first_intersecting_polygon.unwrap()],
            )
        };

        let duration = start_time.elapsed().as_secs_f64();
        println!("theta_star calculation took: {:?}", duration);

        if path.is_empty() {
            println!("No valid path found.");
            return;
        }

        target_position.0 = Some(path.iter().map(|p| Vec3::new(p.x, p.y, p.z)).collect());
        gizmo_path.0 = Some(path.iter().map(|p| Vec3::new(p.x, p.y, p.z)).collect());
        return;
    }

    target_position.0 = Some(vec![goal_position]);
    gizmo_path.0 = Some(vec![goal_position]);
}

pub fn move_player_towards_target(
    mut player_query: Query<(&mut Transform, &PlayerStats), With<Player>>,
    mut target_position: ResMut<TargetPosition>,
    mut gizmo_path: ResMut<GizmoPath>,
    time: Res<Time>,
) {
    let path = match &mut target_position.0 {
        Some(path) if !path.is_empty() => path,
        _ => return,
    };

    let (mut player_transform, player_stats) = player_query.single_mut();

    let target = path[0];

    let player_position_2d = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.z,
    );
    let target_position_2d = Vec2::new(target.x, target.z);

    let direction_2d = (target_position_2d - player_position_2d).normalize_or_zero();
    player_transform.translation.x += direction_2d.x * player_stats.speed * time.delta_seconds();
    player_transform.translation.z += direction_2d.y * player_stats.speed * time.delta_seconds();

    if player_position_2d.distance(target_position_2d) < 0.1 {
        path.remove(0);
        if path.is_empty() {
            gizmo_path.0 = None;
        }
    }
}
