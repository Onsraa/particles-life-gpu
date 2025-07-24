use bevy::prelude::*;
use crate::globals::*;

#[derive(Resource, Clone)]
pub struct SimulationConfig {
    // Paramètres de grille
    pub world_size: f32,

    // Paramètres de simulation
    pub simulation_count: usize,
    pub particle_count: usize,
    pub particle_types: usize,
    pub particles_per_simulation: usize,

    // Paramètres des forces
    pub max_force_range: f32,
    pub velocity_half_life: f32,

    // Paramètres de nourriture
    pub food_count: usize,
    pub food_respawn_enabled: bool,
    pub food_respawn_time: f32,
    pub food_value: f32,

    // Paramètres génétiques
    pub elite_ratio: f32,
    pub mutation_rate: f32,
    pub crossover_rate: f32,

    // Paramètres de viewport
    pub viewport_rows: u32,
    pub viewport_cols: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            world_size: DEFAULT_WORLD_SIZE,
            simulation_count: DEFAULT_SIMULATION_COUNT,
            particle_count: DEFAULT_PARTICLE_COUNT,
            particle_types: DEFAULT_PARTICLE_TYPES,
            particles_per_simulation: DEFAULT_PARTICLE_COUNT / DEFAULT_SIMULATION_COUNT,
            max_force_range: DEFAULT_MAX_FORCE_RANGE,
            velocity_half_life: VELOCITY_HALF_LIFE,
            food_count: DEFAULT_FOOD_COUNT,
            food_respawn_enabled: true,
            food_respawn_time: DEFAULT_FOOD_RESPAWN_TIME,
            food_value: DEFAULT_FOOD_VALUE,
            elite_ratio: DEFAULT_ELITE_RATIO,
            mutation_rate: DEFAULT_MUTATION_RATE,
            crossover_rate: DEFAULT_CROSSOVER_RATE,
            viewport_rows: 2,
            viewport_cols: 4,
        }
    }
}

#[derive(Resource)]
pub struct ParticleTypesConfig {
    pub colors: Vec<(Color, LinearRgba)>,
}

impl Default for ParticleTypesConfig {
    fn default() -> Self {
        Self {
            colors: Self::generate_colors(DEFAULT_PARTICLE_TYPES),
        }
    }
}

impl ParticleTypesConfig {
    pub fn new(type_count: usize) -> Self {
        Self {
            colors: Self::generate_colors(type_count),
        }
    }

    fn generate_colors(count: usize) -> Vec<(Color, LinearRgba)> {
        (0..count)
            .map(|i| {
                let hue = (i as f32 / count as f32) * 360.0;
                let base_color = Color::hsl(hue, 0.8, 0.6);
                let emissive = base_color.to_linear() * 0.5;
                (base_color, emissive)
            })
            .collect()
    }

    pub fn get_color_for_type(&self, type_index: usize) -> (Color, LinearRgba) {
        self.colors[type_index % self.colors.len()]
    }

    pub fn num_types(&self) -> usize {
        self.colors.len()
    }
}