use bevy::prelude::*;
use bevy_app_compute::prelude::*;
use bevy_egui::{EguiContextPass, EguiPlugin};

mod components;
mod globals;
mod plugins;
mod resources;
mod states;
mod systems;
mod ui;

use crate::plugins::particle_life_plugin::ParticleLifePlugin;
use crate::states::app_state::AppState;
use crate::ui::main_menu::{MenuConfig, main_menu_ui};
use crate::ui::simulation_ui::{SimulationUI, force_matrix_window, simulations_list_ui};

fn main() {
    println!("Starting Particle Life 3D - Enhanced Menu...");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppComputePlugin)
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .init_state::<AppState>()
        .init_resource::<MenuConfig>()
        .init_resource::<SimulationUI>()
        .add_plugins(ParticleLifePlugin)
        .add_systems(
            EguiContextPass,
            (main_menu_ui, simulations_list_ui).run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(
            Update,
            ((
                simulations_list_ui,
                force_matrix_window,
                handle_simulation_input,
            )
                .run_if(in_state(AppState::Simulation)),),
        )
        .add_systems(Update, check_state)
        .run();
}

fn handle_simulation_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

fn check_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        info!("State: {:?}", state);
    }
}