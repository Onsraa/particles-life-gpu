use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_app_compute::prelude::*;
use rand::Rng;

use crate::components::particle::*;
use crate::resources::{particle_config::*, simulation_config::*};
use crate::states::game_state::*;
use crate::systems::{simulation_system::*, food_system::*, viewport_system::*};

pub struct ParticleLifePlugin;

impl Plugin for ParticleLifePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ParticleConfig>()
            .init_resource::<SimulationConfig>()
            .init_resource::<ParticleTypes>()
            .add_plugins(AppComputeWorkerPlugin::<ParticleComputeWorker>::default())
            .add_systems(OnEnter(GameState::Loading), (
                setup_simulations,
                setup_viewports,
                setup_lighting,
            ))
            .add_systems(Update, (
                update_particle_simulation,
                update_particle_visualization.after(update_particle_simulation),
                handle_food_interactions,
                display_scores,
                update_viewports_on_resize,
                handle_input,
            ).run_if(in_state(GameState::Running)))
            .add_systems(OnExit(GameState::Loading), transition_to_running);
    }
}

#[derive(TypePath)]
struct ParticleComputeShader;

impl ComputeShader for ParticleComputeShader {
    fn shader() -> ShaderRef {
        "shaders/particle_compute.wgsl".into()
    }
}

#[derive(Resource)]
pub struct ParticleComputeWorker;

impl ComputeWorker for ParticleComputeWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let config = world.resource::<ParticleConfig>();
        let num_particles = config.num_particles;
        let world_size = config.world_size;
        let num_types = config.num_types;
        let force_matrix = config.force_matrix.clone();

        // Buffers vides, remplis par setup_simulations
        let positions = vec![[0.0f32; 4]; num_particles as usize];
        let velocities = vec![[0.0f32; 4]; num_particles as usize];

        println!("Initializing {} particles with {} types", num_particles, num_types);

        AppComputeWorkerBuilder::new(world)
            .add_uniform("num_particles", &num_particles)
            .add_uniform("dt", &(1.0f32 / 60.0))
            .add_uniform("world_size", &world_size)
            .add_uniform("num_types", &num_types)
            .add_staging("positions", &positions)
            .add_staging("velocities", &velocities)
            .add_staging("new_positions", &positions)
            .add_staging("new_velocities", &velocities)
            .add_staging("force_matrix", &force_matrix)
            .add_pass::<ParticleComputeShader>(
                [((num_particles + 63) / 64) as u32, 1, 1],
                &["num_particles", "dt", "world_size", "num_types",
                    "positions", "velocities", "new_positions", "new_velocities", "force_matrix"]
            )
            .add_swap("positions", "new_positions")
            .add_swap("velocities", "new_velocities")
            .build()
    }
}

fn update_particle_simulation(
    mut compute_worker: ResMut<AppComputeWorker<ParticleComputeWorker>>,
    mut config: ResMut<ParticleConfig>,
    time: Res<Time>,
) {
    config.update_timer.tick(time.delta());

    if !config.update_timer.just_finished() {
        return;
    }

    if !compute_worker.ready() {
        return;
    }

    // Juste exécuter - les swaps sont automatiques
    compute_worker.execute();
}

fn update_particle_visualization(
    compute_worker: Res<AppComputeWorker<ParticleComputeWorker>>,
    mut query: Query<(&LifeParticle, &mut Transform)>,
) {
    if !compute_worker.ready() {
        return;
    }

    // Lire depuis "new_positions" (destination après swap)
    let positions: Vec<[f32; 4]> = compute_worker.read_vec("new_positions");

    for (particle, mut transform) in query.iter_mut() {
        if let Some(pos) = positions.get(particle.index as usize) {
            transform.translation = Vec3::new(pos[0], pos[1], pos[2]);
        }
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, -0.7, 0.0)),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: false,
    });
}

fn transition_to_running(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Running);
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(match current_state.get() {
            GameState::Paused => GameState::Running,
            _ => GameState::Paused,
        });
    }
}