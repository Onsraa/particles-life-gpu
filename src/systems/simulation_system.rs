use bevy::prelude::*;
use bevy::render::view::{Layer, RenderLayers};
use bevy_app_compute::prelude::*;
use rand::Rng;

use crate::components::{particle::*, food::*, simulation::*};
use crate::plugins::particle_life_plugin::ParticleComputeWorker;
use crate::resources::{particle_config::*, simulation_config::*};
use crate::states::game_state::GameState;

pub fn initialize_gpu_data(
    mut compute_worker: ResMut<AppComputeWorker<ParticleComputeWorker>>,
    particles: Query<(&Transform, &LifeParticle)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut initialized: Local<bool>,
    config: Res<ParticleConfig>,
) {
    if *initialized || !compute_worker.ready() {
        return;
    }

    let particle_count = particles.iter().count();
    println!("Found {} particles, expected {}", particle_count, config.num_particles);

    if particle_count != config.num_particles as usize {
        println!("❌ Particle count mismatch! Adjusting...");
    }

    let mut positions = Vec::new();
    let mut velocities = Vec::new();

    // Limiter au nombre attendu
    for (transform, particle) in particles.iter().take(config.num_particles as usize) {
        let pos = transform.translation;
        positions.push([pos.x, pos.y, pos.z, particle.particle_type as f32]);
        velocities.push([0.0, 0.0, 0.0, 0.0]);
    }

    println!("💾 Writing {} particles to GPU", positions.len());

    compute_worker.write_slice("positions", &positions);
    compute_worker.write_slice("velocities", &velocities);

    *initialized = true;
    next_state.set(GameState::Running);
    println!("✅ GPU initialized");
}

pub fn setup_simulations(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sim_config: Res<SimulationConfig>,
    particle_config: Res<ParticleConfig>,
    particle_types: Res<ParticleTypes>,
) {
    let mut rng = rand::rng();
    let mut global_particle_index = 0u32;

    // Génération des positions CPU (pour sync avec GPU)
    let mut all_positions = Vec::new();
    let mut all_velocities = Vec::new();

    for _ in 0..particle_config.num_particles {
        let x = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
        let y = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
        let z = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
        let particle_type = (global_particle_index % particle_config.num_types) as f32;

        all_positions.push([x, y, z, particle_type]);

        let vx = (rng.random::<f32>() - 0.5) * 2.0;
        let vy = (rng.random::<f32>() - 0.5) * 2.0;
        let vz = (rng.random::<f32>() - 0.5) * 2.0;
        all_velocities.push([vx, vy, vz, 0.0]);

        global_particle_index += 1;
    }

    // Meshes
    let particle_mesh = meshes.add(Mesh::from(Sphere::new(particle_config.particle_size)));
    let food_mesh = meshes.add(Mesh::from(Sphere::new(0.2)));

    // Matériaux particules
    let mut particle_materials = Vec::new();
    for i in 0..particle_config.num_types {
        let color = particle_types.get_color(i);
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: color.into(),
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

    // Spawn simulations avec positions CPU synchronisées
    let mut particle_idx = 0;
    for sim_id in 0..sim_config.num_simulations {
        let render_layer = sim_id + 1;

        let simulation_entity = commands.spawn((
            Simulation::new(sim_id),
            Transform::default(),
            Visibility::Visible,
            RenderLayers::layer(render_layer as Layer),
        )).id();

        commands.entity(simulation_entity).with_children(|parent| {
            // Particules avec positions du GPU
            for _ in 0..sim_config.particles_per_simulation {
                let pos_data = all_positions[particle_idx];
                let particle_type = pos_data[3] as u32;
                let position = Vec3::new(pos_data[0], pos_data[1], pos_data[2]);

                let material = particle_materials.get(particle_type as usize)
                    .cloned()
                    .unwrap_or_else(|| particle_materials[0].clone());

                parent.spawn((
                    LifeParticle::new(particle_type, particle_idx as u32),
                    Mesh3d(particle_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(position), // Position synchronisée
                    Visibility::Inherited,
                    RenderLayers::layer(render_layer as Layer),
                ));

                particle_idx += 1;
            }

            // Nourriture
            for _ in 0..sim_config.food_per_simulation {
                let x = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let y = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let z = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;

                parent.spawn((
                    Food { value: 1.0, simulation_id: sim_id },
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

    println!("Setup {} simulations with synchronized positions", sim_config.num_simulations);
}

pub fn finished_loading(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Running);
}