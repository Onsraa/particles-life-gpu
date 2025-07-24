use bevy::prelude::*;
use bevy_app_compute::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

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
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .init_resource::<MenuConfig>()
        .init_resource::<SimulationUI>()
        .add_plugins(ParticleLifePlugin)
        .add_systems(
            EguiPrimaryContextPass,
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
