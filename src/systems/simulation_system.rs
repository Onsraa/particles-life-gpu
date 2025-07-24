use bevy::prelude::*;
use bevy::render::view::{Layer, RenderLayers};
use rand::Rng;

use crate::components::{particle::*, food::*, simulation::*};
use crate::resources::{particle_config::*, simulation_config::*};

pub fn setup_simulations(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sim_config: Res<SimulationConfig>,
    particle_config: Res<ParticleConfig>,
    particle_types: Res<ParticleTypes>,
) {
    let mut rng = rand::rng();

    // Meshes partagés
    let particle_mesh = meshes.add(Mesh::from(Sphere::new(particle_config.particle_size)));
    let food_mesh = meshes.add(Mesh::from(Sphere::new(0.2)));

    // Matériaux pour particules
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

    // Matériau pour nourriture
    let food_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::WHITE,
        ..default()
    });

    // Créer chaque simulation
    for sim_id in 0..sim_config.num_simulations {
        let render_layer = sim_id + 1;

        // Entité simulation parente
        let simulation_entity = commands.spawn((
            Simulation::new(sim_id),
            Transform::default(),
            Visibility::Visible,
            RenderLayers::layer(render_layer as Layer),
        )).id();

        // Générer matrice de forces unique pour cette simulation
        let mut force_matrix = vec![0.0; (particle_config.num_types * particle_config.num_types) as usize];
        for i in 0..particle_config.num_types {
            for j in 0..particle_config.num_types {
                let force = if i == j {
                    rng.random::<f32>() * -1.0 - 0.5 // Auto-répulsion
                } else {
                    rng.random::<f32>() * 4.0 - 2.0 // Forces variées
                };
                let index = (i * particle_config.num_types + j) as usize;
                force_matrix[index] = force;
            }
        }

        // Spawn particules pour cette simulation
        commands.entity(simulation_entity).with_children(|parent| {
            for i in 0..sim_config.particles_per_simulation {
                let particle_type = (i % particle_config.num_types) as u32;
                let x = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let y = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let z = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;

                let material = particle_materials.get(particle_type as usize)
                    .cloned()
                    .unwrap_or_else(|| particle_materials[0].clone());

                parent.spawn((
                    LifeParticle::new(particle_type, i),
                    Mesh3d(particle_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(x, y, z)),
                    Visibility::Inherited,
                    RenderLayers::layer(render_layer as Layer),
                ));
            }

            // Spawn nourriture pour cette simulation
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
}