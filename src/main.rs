use bevy::prelude::*;
use bevy_app_compute::prelude::*;

mod components;
mod plugins;
mod resources;
mod states;
mod systems; // Nouveau

use crate::plugins::particle_life_plugin::ParticleLifePlugin;
use crate::states::game_state::GameState;

fn main() {
    println!("Starting Particle Life 3D - 8 Simulations...");
    println!("Controls:");
    println!("  SPACE: Pause/Resume simulation");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppComputePlugin)
        .init_state::<GameState>()
        .add_plugins(ParticleLifePlugin)
        .run();
}