// Compute Shader pour les Particules de Vie

// Paramètres de simulation
@group(0) @binding(0) var<uniform> num_particles: u32;
@group(0) @binding(1) var<uniform> dt: f32;
@group(0) @binding(2) var<uniform> world_size: f32;
@group(0) @binding(3) var<uniform> num_types: u32;

// Positions des particules (x, y, z, type)
@group(0) @binding(4) var<storage, read> positions: array<vec4<f32>>;

// Vélocités des particules (x, y, z, unused)
@group(0) @binding(5) var<storage, read> velocities: array<vec4<f32>>;

// Nouvelles positions (output)
@group(0) @binding(6) var<storage, read_write> new_positions: array<vec4<f32>>;

// Nouvelles vélocités (output)
@group(0) @binding(7) var<storage, read_write> new_velocities: array<vec4<f32>>;

// Matrice des forces d'interaction entre types (format linéaire)
@group(0) @binding(8) var<storage, read> force_matrix: array<f32>;

// Constantes physiques
const MAX_FORCE: f32 = 1000000.0;
const MIN_DISTANCE: f32 = 0.5;
const MAX_DISTANCE: f32 = 50.0;
const FRICTION: f32 = 0.98;
const MAX_VELOCITY: f32 = 10000.0;

// Fonction pour obtenir la force entre deux types de particules
fn get_force_between_types(type_a: u32, type_b: u32) -> f32 {
    let index = type_a * num_types + type_b;
    return force_matrix[index];
}

// Fonction pour calculer la force entre deux particules
fn calculate_force(pos_a: vec3<f32>, pos_b: vec3<f32>, type_a: u32, type_b: u32) -> vec3<f32> {
    let diff = pos_b - pos_a;
    let distance = length(diff);

    // Évite la division par zéro et les forces trop importantes
    if (distance < MIN_DISTANCE || distance > MAX_DISTANCE) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let direction = normalize(diff);
    let force_strength = get_force_between_types(type_a, type_b);

    // Force inversement proportionnelle au carré de la distance
    let force_magnitude = force_strength / (distance * distance);
    let clamped_force = clamp(force_magnitude, -MAX_FORCE, MAX_FORCE);

    return direction * clamped_force;
}

// Fonction pour appliquer les limites du monde (rebonds)
fn apply_world_bounds(pos: vec3<f32>, vel: vec3<f32>) -> vec3<f32> {
    var new_vel = vel;
    let half_size = world_size * 0.5;

    // Rebond sur les murs avec atténuation
    if (pos.x > half_size || pos.x < -half_size) {
        new_vel.x = -new_vel.x * 0.8;
    }
    if (pos.y > half_size || pos.y < -half_size) {
        new_vel.y = -new_vel.y * 0.8;
    }
    if (pos.z > half_size || pos.z < -half_size) {
        new_vel.z = -new_vel.z * 0.8;
    }

    return new_vel;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle_index = global_id.x;

    if (particle_index >= num_particles) {
        return;
    }

    let current_pos = positions[particle_index].xyz;
    let current_type = u32(positions[particle_index].w);
    let current_vel = velocities[particle_index].xyz;

    var total_force = vec3<f32>(0.0, 0.0, 0.0);
    var interaction_count = 0u;

    // Parcourt toutes les autres particules
    for (var i: u32 = 0u; i < num_particles; i++) {
        if (i == particle_index) {
            continue;
        }

        let other_pos = positions[i].xyz;
        let other_type = u32(positions[i].w);
        let distance = length(other_pos - current_pos);

        // Compte les interactions dans la portée
        if (distance >= MIN_DISTANCE && distance <= MAX_DISTANCE) {
            interaction_count++;
            let force = calculate_force(current_pos, other_pos, current_type, other_type);
            total_force += force;
        }
    }

    // Appliquer la physique seulement si il y a des forces
    var new_vel = current_vel;
    if (length(total_force) > 0.01) {
        new_vel = current_vel + total_force * dt;
    }

    // Applique la friction
    new_vel *= FRICTION;

    // Limite la vélocité maximale
    let vel_magnitude = length(new_vel);
    if (vel_magnitude > MAX_VELOCITY) {
        new_vel = normalize(new_vel) * MAX_VELOCITY;
    }

    // Calcule la nouvelle position
    var new_pos = current_pos + new_vel * dt;

    // Applique les limites du monde et ajuste la vélocité si nécessaire
    let half_size = world_size * 0.5;
    if (new_pos.x > half_size) {
        new_pos.x = half_size;
        new_vel.x = -abs(new_vel.x) * 0.8;
    }
    if (new_pos.x < -half_size) {
        new_pos.x = -half_size;
        new_vel.x = abs(new_vel.x) * 0.8;
    }
    if (new_pos.y > half_size) {
        new_pos.y = half_size;
        new_vel.y = -abs(new_vel.y) * 0.8;
    }
    if (new_pos.y < -half_size) {
        new_pos.y = -half_size;
        new_vel.y = abs(new_vel.y) * 0.8;
    }
    if (new_pos.z > half_size) {
        new_pos.z = half_size;
        new_vel.z = -abs(new_vel.z) * 0.8;
    }
    if (new_pos.z < -half_size) {
        new_pos.z = -half_size;
        new_vel.z = abs(new_vel.z) * 0.8;
    }

    // Écrit les nouveaux états
    new_positions[particle_index] = vec4<f32>(new_pos, f32(current_type));
    new_velocities[particle_index] = vec4<f32>(new_vel, 0.0);
}