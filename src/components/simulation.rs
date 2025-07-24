use bevy::prelude::*;

#[derive(Component)]
pub struct Simulation {
    pub id: u32,
    pub score: f32,
}

impl Simulation {
    pub fn new(id: u32) -> Self {
        Self { id, score: 0.0 }
    }

    pub fn add_score(&mut self, points: f32) {
        self.score += points;
    }
}