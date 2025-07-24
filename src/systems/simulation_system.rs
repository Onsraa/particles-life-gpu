use bevy::prelude::*;
use bevy::render::view::{Layer, RenderLayers};
use bevy_app_compute::prelude::*;
use rand::Rng;

use crate::components::{particle::*, food::*, simulation::*};
use crate::plugins::particle_life_plugin::ParticleComputeWorker;
use crate::resources::{particle_config::*, simulation_config::*};
use crate::states::app_state::AppState;

pub fn initialize_gpu_data(
    mut compute_worker: ResMut<AppComputeWorker<ParticleComputeWorker>>,
    particles: Query<(&Transform, &LifeParticle)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut initialized: Local<bool>,
    config: Res<ParticleConfig>,
) {
    if *initialized || !compute_worker.ready() {
        return;
    }

    let particle_count = particles.iter().count();
    println!("Found {} particles, expected {}", particle_count, config.num_particles);

    let mut positions = Vec::new();
    let mut velocities = Vec::new();

    for (transform, particle) in particles.iter().take(config.num_particles as usize) {
        let pos = transform.translation;
        positions.push([pos.x, pos.y, pos.z, particle.particle_type as f32]);
        velocities.push([0.0, 0.0, 0.0, 0.0]);
    }

    if !positions.is_empty() {
        compute_worker.write_slice("positions", &positions);
        compute_worker.write_slice("velocities", &velocities);
        println!("✅ GPU initialized with {} particles", positions.len());
    }

    *initialized = true;
}

pub fn setup_simulations_from_config(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sim_config: Res<SimulationConfig>,
    particle_types: Res<ParticleTypesConfig>,
    mut particle_config: ResMut<ParticleConfig>,
) {
    // Mettre à jour la config des particules
    particle_config.num_particles = sim_config.particle_count as u32;
    particle_config.world_size = sim_config.world_size;
    particle_config.num_types = sim_config.particle_types as u32;
    particle_config.generate_random_forces();

    let mut rng = rand::rng();
    let mut global_particle_index = 0u32;

    // Mesh et matériaux
    let particle_mesh = meshes.add(Mesh::from(Sphere::new(crate::globals::DEFAULT_PARTICLE_SIZE)));
    let food_mesh = meshes.add(Mesh::from(Sphere::new(0.2)));

    let mut particle_materials = Vec::new();
    for i in 0..sim_config.particle_types {
        let (color, emissive) = particle_types.get_color_for_type(i);
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive,
            metallic: 0.1,
            ..default()
        });
        particle_materials.push(material);
    }

    let food_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::WHITE,
        ..default()
    });

    // Spawn simulations
    for sim_id in 0..sim_config.simulation_count {
        let render_layer = sim_id + 1;

        let simulation_entity = commands.spawn((
            Simulation::new(sim_id as u32),
            Transform::default(),
            Visibility::Visible,
            RenderLayers::layer(render_layer as Layer),
        )).id();

        commands.entity(simulation_entity).with_children(|parent| {
            // Particules
            for _ in 0..sim_config.particles_per_simulation {
                let particle_type = (global_particle_index % sim_config.particle_types as u32) as u32;

                let x = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let y = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let z = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let position = Vec3::new(x, y, z);

                let material = particle_materials.get(particle_type as usize)
                    .cloned()
                    .unwrap_or_else(|| particle_materials[0].clone());

                parent.spawn((
                    LifeParticle::new(particle_type, global_particle_index),
                    Mesh3d(particle_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(position),
                    Visibility::Inherited,
                    RenderLayers::layer(render_layer as Layer),
                ));

                global_particle_index += 1;
            }

            // Nourriture
            for _ in 0..sim_config.food_count {
                let x = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let y = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let z = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;

                parent.spawn((
                    Food {
                        value: sim_config.food_value,
                        simulation_id: sim_id as u32
                    },
                    FoodRespawnTimer::default(),
                    Mesh3d(food_mesh.clone()),
                    MeshMaterial3d(food_material.clone()),
                    Transform::from_translation(Vec3::new(x, y, z)),
                    Visibility::Inherited,
                    RenderLayers::layer(render_layer as Layer),
                ));
            }
        });
    }

    println!("Setup {} simulations with configuration", sim_config.simulation_count);
}