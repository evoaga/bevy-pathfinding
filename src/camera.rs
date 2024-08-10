use crate::Player;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraFollowToggle(pub bool);

#[derive(Resource)]
pub struct CameraZoom(pub f32);

pub fn toggle_camera_follow(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_follow_toggle: ResMut<CameraFollowToggle>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyY) {
        camera_follow_toggle.0 = !camera_follow_toggle.0;
    }
}

pub fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    camera_follow_toggle: Res<CameraFollowToggle>,
) {
    if camera_follow_toggle.0 {
        let player_transform = player_query.single();
        let mut camera_transform = camera_query.single_mut();

        // Set the camera position to be at a fixed offset from the player
        let offset = Vec3::new(16.875, 16.875, 0.0);
        camera_transform.translation = player_transform.translation + offset;
    }
}

pub fn camera_zoom(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut camera_zoom: ResMut<CameraZoom>,
) {
    let zoom_speed: f32 = 2.0;
    for ev in evr_scroll.read() {
        camera_zoom.0 -= ev.y * zoom_speed;
        camera_zoom.0 = camera_zoom.0.clamp(5.0, 40.0);

        let mut camera_transform = camera_query.single_mut();
        let forward = camera_transform.forward();
        camera_transform.translation += forward * ev.y * zoom_speed;
    }
}

pub fn camera_edge_pan(
    windows: Query<&Window>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    camera_follow_toggle: Res<CameraFollowToggle>,
    time: Res<Time>,
) {
    if camera_follow_toggle.0 {
        return; // Don't pan if camera is following the player
    }

    let window = windows.single();
    let mut camera_transform = camera_query.single_mut();

    if let Some(cursor_position) = window.cursor_position() {
        let window_size = Vec2::new(window.width(), window.height());
        let edge_size = 200.0;
        let pan_speed = 20.0;

        let mut pan_direction = Vec3::ZERO;

        if cursor_position.x < edge_size {
            pan_direction += Vec3::Z;
        }
        if cursor_position.x > window_size.x - edge_size {
            pan_direction -= Vec3::Z;
        }
        if cursor_position.y < edge_size {
            pan_direction -= Vec3::X;
        }
        if cursor_position.y > window_size.y - edge_size {
            pan_direction += Vec3::X;
        }

        if pan_direction != Vec3::ZERO {
            camera_transform.translation +=
                pan_direction.normalize() * pan_speed * time.delta_seconds();
        }
    }
}
