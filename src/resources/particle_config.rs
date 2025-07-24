use bevy::prelude::*;
use rand::Rng;
use crate::globals::*;

#[derive(Resource)]
pub struct ParticleConfig {
    pub num_particles: u32,
    pub world_size: f32,
    pub num_types: u32,
    pub particle_size: f32,
    pub force_matrix: Vec<f32>,
    pub update_timer: Timer,
}

impl Default for ParticleConfig {
    fn default() -> Self {
        let num_types = DEFAULT_PARTICLE_TYPES as u32;
        let mut config = Self {
            num_particles: DEFAULT_PARTICLE_COUNT as u32,
            world_size: DEFAULT_WORLD_SIZE,
            num_types,
            particle_size: DEFAULT_PARTICLE_SIZE,
            force_matrix: vec![0.0; (num_types * num_types) as usize],
            update_timer: Timer::from_seconds(PHYSICS_TIMESTEP, TimerMode::Repeating),
        };

        config.generate_random_forces();
        config
    }
}

impl ParticleConfig {
    /// Génère des forces d'interaction aléatoires entre les types
    pub fn generate_random_forces(&mut self) {
        let mut rng = rand::rng();

        // Remplit la matrice avec des valeurs équilibrées
        for i in 0..self.num_types {
            for j in 0..self.num_types {
                let force = if i == j {
                    // Auto-répulsion pour éviter l'agglomération
                    rng.random::<f32>() * -1.0 - 0.5 // Entre -1.5 et -0.5
                } else {
                    // Forces variées entre types différents (équilibrées)
                    rng.random::<f32>() * 4.0 - 2.0 // Entre -2.0 et 2.0
                };

                let index = (i * self.num_types + j) as usize;
                self.force_matrix[index] = force;
            }
        }

        println!("Generated balanced force matrix:");
        self.print_force_matrix();
    }

    /// Définit la force entre deux types
    pub fn set_force(&mut self, type_a: u32, type_b: u32, force: f32) {
        let index = (type_a * self.num_types + type_b) as usize;
        if index < self.force_matrix.len() {
            self.force_matrix[index] = force;
        }
    }

    /// Récupère la force entre deux types
    pub fn get_force(&self, type_a: u32, type_b: u32) -> f32 {
        let index = (type_a * self.num_types + type_b) as usize;
        self.force_matrix.get(index).copied().unwrap_or(0.0)
    }

    /// Affiche la matrice des forces
    pub fn print_force_matrix(&self) {
        for i in 0..self.num_types {
            for j in 0..self.num_types {
                let force = self.get_force(i, j);
                print!("{:6.2} ", force);
            }
            println!();
        }
    }

    /// Configure des forces intéressantes prédéfinies
    pub fn set_interesting_forces(&mut self) {
        self.force_matrix.fill(0.0);

        match self.num_types {
            3 => {
                // Configuration rock-paper-scissors
                self.set_force(0, 1, 1.0);   // Rouge attire Vert
                self.set_force(1, 2, 1.0);   // Vert attire Bleu
                self.set_force(2, 0, 1.0);   // Bleu attire Rouge
                self.set_force(1, 0, -0.5);  // Vert repousse Rouge
                self.set_force(2, 1, -0.5);  // Bleu repousse Vert
                self.set_force(0, 2, -0.5);  // Rouge repousse Bleu

                // Auto-répulsion légère
                for i in 0..3 {
                    self.set_force(i, i, -0.3);
                }
            },
            6 => {
                // Configuration plus complexe pour 6 types
                // Créer des cycles d'attraction et de répulsion
                for i in 0..6 {
                    let next = (i + 1) % 6;
                    let prev = (i + 5) % 6;

                    self.set_force(i, next, 1.2);      // Attire le suivant
                    self.set_force(i, prev, -0.8);     // Repousse le précédent
                    self.set_force(i, i, -0.4);        // Auto-répulsion

                    // Forces moyennes avec les autres
                    for j in 0..6 {
                        if j != i && j != next && j != prev {
                            self.set_force(i, j, (rand::rng().random::<f32>() - 0.5) * 0.6);
                        }
                    }
                }
            },
            _ => {
                // Configuration générique aléatoire
                self.generate_random_forces();
            }
        }

        println!("Set interesting forces:");
        self.print_force_matrix();
    }
}