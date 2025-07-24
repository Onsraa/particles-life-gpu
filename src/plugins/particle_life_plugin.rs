use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_app_compute::prelude::*;
use rand::Rng;

use crate::components::particle::*;
use crate::resources::{particle_config::*, simulation_config::*};
use crate::states::app_state::AppState;
use crate::systems::{simulation_system::*, food_system::*, viewport_system::*};

pub struct ParticleLifePlugin;

impl Plugin for ParticleLifePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ParticleConfig>()
            .add_plugins(AppComputeWorkerPlugin::<ParticleComputeWorker>::default())
            .add_systems(OnEnter(AppState::Simulation), (
                setup_simulations_from_config,
                setup_viewports,
                setup_lighting,
            ))
            .add_systems(Update,
                         initialize_gpu_data.run_if(in_state(AppState::Simulation))
            )
            .add_systems(Update, (
                update_particle_simulation,
                update_particle_visualization.after(update_particle_simulation),
                handle_food_interactions,
                display_scores,
                update_viewports_on_resize,
            ).run_if(in_state(AppState::Simulation)));
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
        // Utiliser la configuration du menu si disponible
        let (num_particles, world_size, num_types, force_matrix) = if let Some(sim_config) = world.get_resource::<SimulationConfig>() {
            (
                sim_config.particle_count as u32,
                sim_config.world_size,
                sim_config.particle_types as u32,
                generate_random_force_matrix(sim_config.particle_types),
            )
        } else {
            // Valeurs par défaut
            (
                crate::globals::DEFAULT_PARTICLE_COUNT as u32,
                crate::globals::DEFAULT_WORLD_SIZE,
                crate::globals::DEFAULT_PARTICLE_TYPES as u32,
                generate_random_force_matrix(crate::globals::DEFAULT_PARTICLE_TYPES),
            )
        };

        let positions = vec![[0.0f32; 4]; num_particles as usize];
        let velocities = vec![[0.0f32; 4]; num_particles as usize];

        println!("Initializing {} particles with {} types", num_particles, num_types);

        AppComputeWorkerBuilder::new(world)
            .add_uniform("num_particles", &num_particles)
            .add_uniform("dt", &(crate::globals::PHYSICS_TIMESTEP))
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

fn generate_random_force_matrix(num_types: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    let matrix_size = num_types * num_types;

    (0..matrix_size)
        .map(|i| {
            let type_a = i / num_types;
            let type_b = i % num_types;

            if type_a == type_b {
                // Auto-répulsion pour éviter l'agglomération
                rng.random_range(-1.0..=-0.1)
            } else {
                // Forces variées entre types différents
                rng.random_range(-2.0..=2.0)
            }
        })
        .collect()
}

// Reste du code identique...
fn update_particle_simulation(
    mut compute_worker: ResMut<AppComputeWorker<ParticleComputeWorker>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(crate::globals::PHYSICS_TIMESTEP, TimerMode::Repeating);
    }

    timer.tick(time.delta());

    if !timer.just_finished() {
        return;
    }

    if !compute_worker.ready() {
        return;
    }

    compute_worker.execute();
}

fn update_particle_visualization(
    compute_worker: Res<AppComputeWorker<ParticleComputeWorker>>,
    mut query: Query<(&LifeParticle, &mut Transform)>,
) {
    if !compute_worker.ready() {
        return;
    }

    let positions: Vec<[f32; 4]> = compute_worker.read_vec("positions");

    for (particle, mut transform) in query.iter_mut() {
        if let Some(pos) = positions.get(particle.index as usize) {
            let new_pos = Vec3::new(pos[0], pos[1], pos[2]);
            transform.translation = new_pos;
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