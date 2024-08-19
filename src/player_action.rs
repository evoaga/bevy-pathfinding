use crate::cursor::CursorPosition;
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerAction {
    pub casting_q: bool,
    pub q_timer: Timer,
    pub q_direction: Option<Vec3>,
    pub cast_timer: Timer, // Add this field to handle the cast time
    pub is_casting: bool,  // Flag to indicate if the spell is in the casting phase
}

impl PlayerAction {
    pub fn new() -> Self {
        PlayerAction {
            casting_q: false,
            q_timer: Timer::from_seconds(0.3, TimerMode::Once), // Q spell lasts 0.5 seconds
            q_direction: None,
            cast_timer: Timer::from_seconds(0.2, TimerMode::Once), // Cast time of 0.3 seconds
            is_casting: false,
        }
    }
}

pub fn cast_q_spell(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor_position_res: Res<CursorPosition>,
    mut query: Query<(&mut PlayerAction, &Transform)>,
    mut gizmos: Gizmos,
) {
    for (mut player_action, player_transform) in query.iter_mut() {
        // Check if Q key is pressed and the spell is not currently casting or active
        if keyboard_input.just_pressed(KeyCode::KeyQ)
            && !player_action.casting_q
            && !player_action.is_casting
        {
            player_action.is_casting = true; // Enter the casting phase
            player_action.cast_timer.reset();

            // Calculate the direction towards the cursor and store it
            if let Some(cursor_position) = cursor_position_res.0 {
                // Project the cursor position onto the player's Y plane
                let mut direction = cursor_position - player_transform.translation;
                direction.y = 0.0; // Ensure the Y component is 0
                direction = direction.normalize();
                player_action.q_direction = Some(direction);
            }
        }

        // Handle the casting phase
        if player_action.is_casting {
            player_action.cast_timer.tick(time.delta());

            if player_action.cast_timer.finished() {
                player_action.is_casting = false; // Cast time ends
                player_action.casting_q = true; // Spell is now active
                player_action.q_timer.reset();

                // Visualize the Q spell as a line immediately after casting
                if let Some(direction) = player_action.q_direction {
                    let range = 2.0;
                    let q_endpoint = player_transform.translation + direction * range;
                    gizmos.line(
                        player_transform.translation,
                        q_endpoint,
                        Color::srgb(0.0, 0.0, 1.0),
                    );
                }
            }
        }

        // Handle the active Q spell
        if player_action.casting_q {
            player_action.q_timer.tick(time.delta());

            // Visualize the Q spell for the duration of the timer
            if player_action.q_timer.finished() {
                player_action.casting_q = false; // Q spell duration ends
                player_action.q_direction = None; // Clear the direction after the spell ends
            } else if let Some(direction) = player_action.q_direction {
                let range = 5.0;
                let q_endpoint = player_transform.translation + direction * range;

                // Keep visualizing the Q spell
                gizmos.line(
                    player_transform.translation,
                    q_endpoint,
                    Color::srgb(0.0, 0.0, 1.0),
                );
            }
        }
    }
}
