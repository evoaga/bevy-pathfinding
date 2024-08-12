use crate::player::Player;
use bevy::prelude::*;

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
