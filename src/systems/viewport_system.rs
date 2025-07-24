use bevy::prelude::*;
use bevy::render::view::{Layer, RenderLayers};
use bevy::render::camera::{Viewport, ClearColorConfig};
use bevy::window::WindowResized;

use crate::resources::simulation_config::SimulationConfig;

#[derive(Component)]
pub struct SimulationCamera {
    pub simulation_id: u32,
}

pub fn setup_viewports(
    mut commands: Commands,
    sim_config: Res<SimulationConfig>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    
    let (window_width, window_height) = {
        let window = window.unwrap();
        (window.width(), window.height())
    };

    let viewport_width = window_width / sim_config.viewport_cols as f32;
    let viewport_height = window_height / sim_config.viewport_rows as f32;

    for sim_id in 0..sim_config.simulation_count {
        let row = sim_id / sim_config.viewport_cols as usize;
        let col = sim_id % sim_config.viewport_cols as usize;

        let x = (col as f32 * viewport_width) as u32;
        let y = (row as f32 * viewport_height) as u32;
        let width = viewport_width as u32;
        let height = viewport_height as u32;

        // Position caméra pour voir toute la simulation
        let camera_distance = sim_config.world_size * 1.5;
        let camera_pos = Vec3::new(
            camera_distance * 0.7,
            camera_distance * 0.5,
            camera_distance * 0.7,
        );

        commands.spawn((
            Camera {
                viewport: Some(Viewport {
                    physical_position: UVec2::new(x, y),
                    physical_size: UVec2::new(width, height),
                    ..default()
                }),
                order: sim_id as isize,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
                ..default()
            },
            Camera3d::default(),
            Transform::from_translation(camera_pos).looking_at(Vec3::ZERO, Vec3::Y),
            SimulationCamera { simulation_id: sim_id as u32 },
            RenderLayers::from_layers(&[0, (sim_id + 1) as Layer]),
        ));
    }

    println!("Setup {} viewport cameras in {}x{} grid",
             sim_config.simulation_count,
             sim_config.viewport_cols,
             sim_config.viewport_rows);
}

pub fn update_viewports_on_resize(
    mut resize_events: EventReader<WindowResized>,
    sim_config: Res<SimulationConfig>,
    mut cameras: Query<(&mut Camera, &SimulationCamera)>,
) {
    for event in resize_events.read() {
        let viewport_width = event.width / sim_config.viewport_cols as f32;
        let viewport_height = event.height / sim_config.viewport_rows as f32;

        for (mut camera, sim_camera) in cameras.iter_mut() {
            let sim_id = sim_camera.simulation_id as usize;
            let row = sim_id / sim_config.viewport_cols as usize;
            let col = sim_id % sim_config.viewport_cols as usize;

            let x = (col as f32 * viewport_width) as u32;
            let y = (row as f32 * viewport_height) as u32;
            let width = viewport_width as u32;
            let height = viewport_height as u32;

            if let Some(ref mut viewport) = camera.viewport {
                viewport.physical_position = UVec2::new(x, y);
                viewport.physical_size = UVec2::new(width, height);
            }
        }
    }
}