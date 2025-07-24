use bevy::prelude::*;
use rand::Rng;

use crate::components::{particle::*, food::*, simulation::*};
use crate::resources::simulation_config::*;

pub fn handle_food_interactions(
    mut commands: Commands,
    time: Res<Time>,
    sim_config: Res<SimulationConfig>,
    mut simulations: Query<&mut Simulation>,
    particles: Query<(&Transform, &ChildOf), With<LifeParticle>>,
    mut food_query: Query<(Entity, &mut Transform, &Food, &mut FoodRespawnTimer, &mut Visibility, &ChildOf), (With<Food>, Without<LifeParticle>)>,
) {
    let mut rng = rand::rng();

    // Traiter chaque nourriture
    for (food_entity, mut food_transform, food, mut respawn_timer, mut visibility, parent) in food_query.iter_mut() {
        // Gérer le respawn timer
        if !matches!(*visibility, Visibility::Visible) {
            respawn_timer.timer.tick(time.delta());
            if respawn_timer.timer.just_finished() {
                // Respawn la nourriture à une nouvelle position
                let x = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let y = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                let z = rng.random::<f32>() * sim_config.world_size - sim_config.world_size * 0.5;
                food_transform.translation = Vec3::new(x, y, z);
                *visibility = Visibility::Visible;
                respawn_timer.timer.reset();
            }
            continue;
        }

        // Vérifier collisions avec particules de la même simulation
        for (particle_transform, particle_parent) in particles.iter() {
            if particle_parent.parent() != parent.parent() {
                continue; // Pas la même simulation
            }

            let distance = food_transform.translation.distance(particle_transform.translation);
            if distance < 1.0 { // Distance de collision
                // Collision ! Ajouter score et cacher nourriture
                if let Ok(mut simulation) = simulations.get_mut(parent.parent()) {
                    simulation.add_score(food.value);
                }

                *visibility = Visibility::Hidden;
                respawn_timer.timer.reset();
                break;
            }
        }
    }
}

pub fn display_scores(
    simulations: Query<&Simulation>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(3.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());
    if timer.just_finished() {
        println!("=== SCORES ===");
        for sim in simulations.iter() {
            println!("Simulation {}: {:.1} points", sim.id, sim.score);
        }
    }
}