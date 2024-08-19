use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerStats {
    pub speed: f32,
    pub max_health: f32,
    pub current_health: f32,
    pub health_regen: f32,
}

impl PlayerStats {
    pub fn new(speed: f32, max_health: f32, health_regen: f32) -> Self {
        Self {
            speed,
            max_health,
            current_health: max_health,
            health_regen,
        }
    }
}
