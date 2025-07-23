use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_app_compute::prelude::*;
use rand::Rng;

use crate::components::particle::*;
use crate::resources::particle_config::*;
use crate::states::game_state::*;

pub struct ParticleLifePlugin;

#[derive(Resource)]
struct DebugTimer(Timer);

impl Default for DebugTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

impl Plugin for ParticleLifePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ParticleConfig>()
            .init_resource::<ParticleTypes>()
            .init_resource::<DebugTimer>()
            .add_plugins(AppComputeWorkerPlugin::<ParticleComputeWorker>::default())
            .add_systems(Startup, initialize_particle_system)
            .add_systems(Update, (
                update_particle_simulation,
                update_particle_visualization.after(update_particle_simulation),
                handle_input,
                debug_system,
            ).run_if(in_state(GameState::Running)))
            .add_systems(OnEnter(GameState::Loading), setup_particle_world)
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
struct ParticleComputeWorker;

impl ComputeWorker for ParticleComputeWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let config = world.resource::<ParticleConfig>();
        let num_particles = config.num_particles;
        let world_size = config.world_size;
        let num_types = config.num_types;
        let force_matrix = config.force_matrix.clone();

        // Initialise les positions et vélocités des particules
        let mut positions = Vec::with_capacity(num_particles as usize);
        let mut velocities = Vec::with_capacity(num_particles as usize);
        let mut rng = rand::rng();

        for i in 0..num_particles {
            // Position aléatoire dans le monde
            let x = rng.random::<f32>() * world_size - world_size * 0.5;
            let y = rng.random::<f32>() * world_size - world_size * 0.5;
            let z = rng.random::<f32>() * world_size - world_size * 0.5;
            let particle_type = (i % num_types) as f32;

            positions.push([x, y, z, particle_type]);

            // Vélocité initiale petite et aléatoire
            let vx = (rng.random::<f32>() - 0.5) * 2.0;
            let vy = (rng.random::<f32>() - 0.5) * 2.0;
            let vz = (rng.random::<f32>() - 0.5) * 2.0;

            velocities.push([vx, vy, vz, 0.0]);
        }

        println!("Initializing {} particles with {} types", num_particles, num_types);

        let worker = AppComputeWorkerBuilder::new(world)
            // Paramètres de simulation
            .add_uniform("num_particles", &num_particles)
            .add_uniform("dt", &(1.0f32 / 60.0)) // 60 FPS
            .add_uniform("world_size", &world_size)
            .add_uniform("num_types", &num_types)
            // Données des particules (staging pour pouvoir lire/écrire depuis CPU)
            .add_staging("positions", &positions)
            .add_staging("velocities", &velocities)
            .add_staging("new_positions", &positions) // Copie initiale
            .add_staging("new_velocities", &velocities) // Copie initiale
            // Matrice des forces
            .add_staging("force_matrix", &force_matrix)
            // Passe de calcul
            .add_pass::<ParticleComputeShader>(
                [((num_particles + 63) / 64) as u32, 1, 1], // Workgroups de 64
                &["num_particles", "dt", "world_size", "num_types",
                    "positions", "velocities", "new_positions", "new_velocities", "force_matrix"]
            )
            .build();

        worker
    }
}

fn initialize_particle_system(world: &mut World) {
    // Force l'initialisation du compute worker
    world.resource::<AppComputeWorker<ParticleComputeWorker>>();
}

fn setup_particle_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<ParticleConfig>,
    particle_types: Res<ParticleTypes>,
    mut next_state: ResMut<NextState<GameState>>,
    compute_worker: Res<AppComputeWorker<ParticleComputeWorker>>,
) {
    println!("=== SETTING UP PARTICLE WORLD ===");
    println!("Particles: {}, Types: {}, World size: {}",
             config.num_particles, config.num_types, config.world_size);
    println!("Compute worker ready: {}", compute_worker.ready());

    // Crée un mesh sphère pour les particules
    let sphere_mesh = meshes.add(Mesh::from(Sphere::new(config.particle_size)));

    // Crée les matériaux pour chaque type de particule
    let mut type_materials = Vec::new();
    for i in 0..config.num_types {
        let color = particle_types.get_color(i);
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: color.into(),
            metallic: 0.1,
            ..default()
        });
        type_materials.push(material);
    }

    // Spawn un parent pour toutes les particules
    let parent_entity = commands.spawn((
        ParticleParent,
        Transform::default(),
        Visibility::Visible,
    )).id();

    // Lit les positions initiales depuis le compute worker
    if compute_worker.ready() {
        let positions: Vec<[f32; 4]> = compute_worker.read_vec("positions");

        // Spawn les entités particules
        for (i, pos) in positions.iter().enumerate() {
            let particle_type = pos[3] as u32;
            let position = Vec3::new(pos[0], pos[1], pos[2]);

            let material = type_materials.get(particle_type as usize)
                .cloned()
                .unwrap_or_else(|| type_materials[0].clone());

            commands.spawn((
                LifeParticle::new(particle_type, i as u32),
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(position),
                Visibility::Inherited,
            )).insert(ChildOf(parent_entity));
        }

        println!("Spawned {} particle entities", positions.len());
    } else {
        println!("Compute worker not ready, spawning placeholder particles");

        // Fallback : spawn des particules à des positions par défaut
        let mut rng = rand::rng();
        for i in 0..config.num_particles {
            let particle_type = (i % config.num_types) as u32;
            let x = rng.random::<f32>() * config.world_size - config.world_size * 0.5;
            let y = rng.random::<f32>() * config.world_size - config.world_size * 0.5;
            let z = rng.random::<f32>() * config.world_size - config.world_size * 0.5;

            let material = type_materials.get(particle_type as usize)
                .cloned()
                .unwrap_or_else(|| type_materials[0].clone());

            commands.spawn((
                LifeParticle::new(particle_type, i),
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(x, y, z)),
                Visibility::Inherited,
            )).insert(ChildOf(parent_entity));
        }
    }

    // Ajoute l'éclairage
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 15000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, -0.7, 0.0)),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        affects_lightmapped_meshes: false,
    });

    println!("Particle world setup complete!");
    println!("===============================");

    next_state.set(GameState::Running);
}

fn transition_to_running(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Running);
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

    // Lit les nouvelles positions/vélocités calculées
    let new_positions: Vec<[f32; 4]> = compute_worker.read_vec("new_positions");
    let new_velocities: Vec<[f32; 4]> = compute_worker.read_vec("new_velocities");

    // Met à jour les buffers pour le prochain cycle
    compute_worker.write_slice("positions", &new_positions);
    compute_worker.write_slice("velocities", &new_velocities);
}

fn update_particle_visualization(
    compute_worker: Res<AppComputeWorker<ParticleComputeWorker>>,
    mut query: Query<(&LifeParticle, &mut Transform)>,
    config: Res<ParticleConfig>,
) {
    if !compute_worker.ready() {
        return;
    }

    // Lit les positions actuelles depuis le GPU
    let positions: Vec<[f32; 4]> = compute_worker.read_vec("new_positions");

    // Met à jour les transforms des entités
    for (particle, mut transform) in query.iter_mut() {
        if let Some(pos) = positions.get(particle.index as usize) {
            transform.translation = Vec3::new(pos[0], pos[1], pos[2]);
        }
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: ResMut<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut config: ResMut<ParticleConfig>,
    mut compute_worker: ResMut<AppComputeWorker<ParticleComputeWorker>>,
) {
    // Espace pour pause/resume
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(match current_state.get() {
            GameState::Paused => GameState::Running,
            _ => GameState::Paused,
        });
    }

    // R pour reset avec des positions aléatoires
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        if compute_worker.ready() {
            let mut rng = rand::rng();
            let mut new_positions = Vec::new();
            let mut new_velocities = Vec::new();

            for i in 0..config.num_particles {
                let x = rng.random::<f32>() * config.world_size - config.world_size * 0.5;
                let y = rng.random::<f32>() * config.world_size - config.world_size * 0.5;
                let z = rng.random::<f32>() * config.world_size - config.world_size * 0.5;
                let particle_type = (i % config.num_types) as f32;

                new_positions.push([x, y, z, particle_type]);
                new_velocities.push([0.0, 0.0, 0.0, 0.0]);
            }

            compute_worker.write_slice("positions", &new_positions);
            compute_worker.write_slice("velocities", &new_velocities);
            println!("Reset particle positions");
        }
    }

    // F pour générer de nouvelles forces aléatoires
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        config.generate_random_forces();
        if compute_worker.ready() {
            compute_worker.write_slice("force_matrix", &config.force_matrix);
            println!("Generated new random forces");
        }
    }

    // I pour appliquer des forces intéressantes prédéfinies
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        config.set_interesting_forces();
        if compute_worker.ready() {
            compute_worker.write_slice("force_matrix", &config.force_matrix);
            println!("Applied interesting forces");
        }
    }

    // + et - pour ajuster la vitesse de simulation
    if keyboard_input.just_pressed(KeyCode::Equal) {
        let new_duration = (config.update_timer.duration().as_secs_f32() * 0.8).max(1.0 / 120.0);
        config.update_timer.set_duration(std::time::Duration::from_secs_f32(new_duration));
        println!("Increased simulation speed to {:.3}s per frame", new_duration);
    }
    if keyboard_input.just_pressed(KeyCode::Minus) {
        let new_duration = (config.update_timer.duration().as_secs_f32() * 1.2).min(1.0 / 10.0);
        config.update_timer.set_duration(std::time::Duration::from_secs_f32(new_duration));
        println!("Decreased simulation speed to {:.3}s per frame", new_duration);
    }
}

fn debug_system(
    compute_worker: Res<AppComputeWorker<ParticleComputeWorker>>,
    query: Query<&LifeParticle>,
    config: Res<ParticleConfig>,
    mut debug_timer: ResMut<DebugTimer>,
    time: Res<Time>,
) {
    debug_timer.0.tick(time.delta());
    if !debug_timer.0.just_finished() {
        return;
    }

    let total_entities = query.iter().count();

    println!("=== DEBUG INFO ===");
    println!("Total particle entities: {}", total_entities);
    println!("Config particles: {}, Types: {}", config.num_particles, config.num_types);
    println!("World size: {}, Update rate: {:.3}s",
             config.world_size, config.update_timer.duration().as_secs_f32());
    println!("Compute worker ready: {}", compute_worker.ready());

    if compute_worker.ready() {
        let positions: Vec<[f32; 4]> = compute_worker.read_vec("positions");
        let velocities: Vec<[f32; 4]> = compute_worker.read_vec("velocities");

        println!("GPU buffers: {} positions, {} velocities", positions.len(), velocities.len());

        // Affiche quelques particules pour debug
        for (i, pos) in positions.iter().take(3).enumerate() {
            if let Some(vel) = velocities.get(i) {
                println!("Particle {}: pos=({:.2}, {:.2}, {:.2}), type={}, vel=({:.2}, {:.2}, {:.2})",
                         i, pos[0], pos[1], pos[2], pos[3] as u32, vel[0], vel[1], vel[2]);
            }
        }
    }
    println!("==================");
}