use bevy::prelude::*;

/// Composant pour une particule de vie
#[derive(Component, Debug, Clone)]
pub struct LifeParticle {
    pub particle_type: u32,
    pub index: u32, // Index dans les buffers GPU
}

impl LifeParticle {
    pub fn new(particle_type: u32, index: u32) -> Self {
        Self { particle_type, index }
    }
}

/// Marqueur pour identifier le parent des particules
#[derive(Component)]
pub struct ParticleParent;

/// Types de particules avec leurs couleurs associées
#[derive(Resource, Clone)]
pub struct ParticleTypes {
    pub colors: Vec<Color>,
    pub names: Vec<String>,
}

impl Default for ParticleTypes {
    fn default() -> Self {
        Self {
            colors: vec![
                Color::srgb(1.0, 0.2, 0.2), // Rouge
                Color::srgb(0.2, 1.0, 0.2), // Vert
                Color::srgb(0.2, 0.2, 1.0), // Bleu
                Color::srgb(1.0, 1.0, 0.2), // Jaune
                Color::srgb(1.0, 0.2, 1.0), // Magenta
                Color::srgb(0.2, 1.0, 1.0), // Cyan
            ],
            names: vec![
                "Rouge".to_string(),
                "Vert".to_string(),
                "Bleu".to_string(),
                "Jaune".to_string(),
                "Magenta".to_string(),
                "Cyan".to_string(),
            ],
        }
    }
}

impl ParticleTypes {
    pub fn get_color(&self, particle_type: u32) -> Color {
        self.colors.get(particle_type as usize)
            .copied()
            .unwrap_or(Color::WHITE)
    }

    pub fn num_types(&self) -> u32 {
        self.colors.len() as u32
    }
}