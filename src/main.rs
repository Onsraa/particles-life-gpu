use bevy::prelude::*;
use bevy_app_compute::prelude::*;

mod components;
mod plugins;
mod resources;
mod states;

use crate::plugins::camera_plugin::CameraPlugin;
use crate::plugins::particle_life_plugin::ParticleLifePlugin;
use crate::states::game_state::GameState;

fn main() {
    println!("Starting Particle Life 3D...");
    println!("Controls:");
    println!("  ZQSD/WASD: Move camera");
    println!("  Mouse right-click + drag: Look around");
    println!("  SPACE: Pause/Resume simulation");
    println!("  R: Reset particles");
    println!("  F: Generate random forces");
    println!("  I: Apply interesting predefined forces");
    println!("  +/-: Increase/Decrease simulation speed");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppComputePlugin)
        .init_state::<GameState>()
        .add_plugins(CameraPlugin)
        .add_plugins(ParticleLifePlugin)
        .add_systems(Update, debug_states)
        .run();
}

fn debug_states(state: Res<State<GameState>>) {
    // Décommente pour debug les états
    // eprintln!("Current state: {:?}", state.get());
}