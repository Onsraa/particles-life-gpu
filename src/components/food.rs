use bevy::prelude::*;

#[derive(Component)]
pub struct Food {
    pub value: f32,
    pub simulation_id: u32,
}

#[derive(Component)]
pub struct FoodRespawnTimer {
    pub timer: Timer,
}

impl Default for FoodRespawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}