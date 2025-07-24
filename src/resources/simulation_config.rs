use bevy::prelude::*;

#[derive(Resource)]
pub struct SimulationConfig {
    pub num_simulations: u32,
    pub particles_per_simulation: u32,
    pub food_per_simulation: u32,
    pub world_size: f32,
    pub viewport_rows: u32,
    pub viewport_cols: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            num_simulations: 8,
            particles_per_simulation: 200,
            food_per_simulation: 200,
            world_size: 50.0,
            viewport_rows: 2,
            viewport_cols: 4,
        }
    }
}