use crate::states::game_state::*;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera).add_systems(
            Update,
            (camera_movement, camera_rotation).run_if(in_state(GameState::Running)),
        );
    }
}

#[derive(Component)]
pub struct CameraController {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            sensitivity: 0.1,
            speed: 15.0,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    // Spawn la caméra avec un controller - positionnée pour voir le monde des particules
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(35.0, 35.0, 35.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController::default(),
    ));
}

fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<CameraController>>,
    time: Res<Time>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let speed = 25.0;

        // ZQSD pour le mouvement (ou WASD en QWERTY)
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::KeyZ) {
            direction += transform.forward().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction += transform.back().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::KeyQ) {
            direction += transform.left().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += transform.right().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }

        // Normalise le vecteur direction pour éviter un mouvement plus rapide en diagonal
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        // Applique le mouvement
        transform.translation += direction * speed * time.delta_secs();
    }
}

fn camera_rotation(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut Transform, With<CameraController>>,
) {
    // Rotation seulement quand le bouton droit de la souris est pressé
    if !mouse_button_input.pressed(MouseButton::Right) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    if delta.length() == 0.0 {
        return;
    }

    for mut transform in query.iter_mut() {
        let sensitivity = 0.002;

        // Rotation horizontale (yaw)
        let yaw = -delta.x * sensitivity;
        // Rotation verticale (pitch)
        let pitch = -delta.y * sensitivity;

        // Applique la rotation en gardant Y comme axe up global
        let yaw_rotation = Quat::from_rotation_y(yaw);
        let pitch_rotation = Quat::from_axis_angle(*transform.local_x(), pitch);

        transform.rotation = yaw_rotation * transform.rotation * pitch_rotation;
    }
}