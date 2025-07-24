use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::resources::simulation_config::{SimulationConfig, ParticleTypesConfig};
use crate::states::app_state::AppState;
use crate::globals::*;

#[derive(Resource)]
pub struct MenuConfig {
    pub world_size: f32,
    pub simulation_count: usize,
    pub particle_count: usize,
    pub particle_types: usize,
    pub max_force_range: f32,
    pub food_count: usize,
    pub food_respawn_enabled: bool,
    pub food_respawn_time: f32,
    pub food_value: f32,
    pub elite_ratio: f32,
    pub mutation_rate: f32,
    pub crossover_rate: f32,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            world_size: DEFAULT_WORLD_SIZE,
            simulation_count: DEFAULT_SIMULATION_COUNT,
            particle_count: DEFAULT_PARTICLE_COUNT,
            particle_types: DEFAULT_PARTICLE_TYPES,
            max_force_range: DEFAULT_MAX_FORCE_RANGE,
            food_count: DEFAULT_FOOD_COUNT,
            food_respawn_enabled: true,
            food_respawn_time: DEFAULT_FOOD_RESPAWN_TIME,
            food_value: DEFAULT_FOOD_VALUE,
            elite_ratio: DEFAULT_ELITE_RATIO,
            mutation_rate: DEFAULT_MUTATION_RATE,
            crossover_rate: DEFAULT_CROSSOVER_RATE,
        }
    }
}

pub fn main_menu_ui(
    mut contexts: EguiContexts,
    mut menu_config: ResMut<MenuConfig>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {

    println!("MainMenu ui");

    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Particle Life 3D")
                .size(28.0)
                .strong()
                .color(egui::Color32::from_rgb(100, 200, 255)));
            ui.label(egui::RichText::new("8 Simulations Parallèles")
                .size(14.0)
                .italics()
                .color(egui::Color32::GRAY));
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Paramètres de monde
            ui.group(|ui| {
                ui.label(egui::RichText::new("🌍 Monde").size(16.0).strong());
                ui.separator();

                egui::Grid::new("world_params")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Taille du monde:");
                        ui.add(egui::DragValue::new(&mut menu_config.world_size)
                            .range(20.0..=200.0)
                            .suffix(" unités"));
                        ui.end_row();
                    });
            });

            ui.add_space(10.0);

            // Paramètres de simulation
            ui.group(|ui| {
                ui.label(egui::RichText::new("⚙ Simulation").size(16.0).strong());
                ui.separator();

                egui::Grid::new("sim_params")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Nombre de simulations:");
                        ui.add(egui::DragValue::new(&mut menu_config.simulation_count)
                            .range(1..=16));
                        ui.end_row();

                        ui.label("Particules totales:");
                        ui.add(egui::DragValue::new(&mut menu_config.particle_count)
                            .range(100..=3200));
                        ui.end_row();

                        ui.label("Types de particules:");
                        ui.add(egui::DragValue::new(&mut menu_config.particle_types)
                            .range(2..=8));
                        ui.end_row();

                        ui.label("Portée des forces:");
                        ui.add(egui::DragValue::new(&mut menu_config.max_force_range)
                            .range(10.0..=200.0)
                            .suffix(" unités"));
                        ui.end_row();
                    });

                ui.add_space(5.0);
                let particles_per_sim = menu_config.particle_count / menu_config.simulation_count;
                ui.label(egui::RichText::new(format!("≈ {} particules par simulation", particles_per_sim))
                    .small()
                    .color(egui::Color32::GRAY));
            });

            ui.add_space(10.0);

            // Paramètres génétiques
            ui.group(|ui| {
                ui.label(egui::RichText::new("🧬 Génétique").size(16.0).strong());
                ui.separator();

                egui::Grid::new("genetic_params")
                    .num_columns(3)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Élites:");
                        ui.add(egui::DragValue::new(&mut menu_config.elite_ratio)
                            .range(0.01..=0.5)
                            .speed(0.01)
                            .fixed_decimals(2));
                        ui.label(format!("({:.0}%)", menu_config.elite_ratio * 100.0));
                        ui.end_row();

                        ui.label("Mutation:");
                        ui.add(egui::DragValue::new(&mut menu_config.mutation_rate)
                            .range(0.0..=1.0)
                            .speed(0.01)
                            .fixed_decimals(2));
                        ui.label(format!("({:.0}%)", menu_config.mutation_rate * 100.0));
                        ui.end_row();

                        ui.label("Crossover:");
                        ui.add(egui::DragValue::new(&mut menu_config.crossover_rate)
                            .range(0.0..=1.0)
                            .speed(0.01)
                            .fixed_decimals(2));
                        ui.label(format!("({:.0}%)", menu_config.crossover_rate * 100.0));
                        ui.end_row();
                    });
            });

            ui.add_space(10.0);

            // Paramètres de nourriture
            ui.group(|ui| {
                ui.label(egui::RichText::new("🍎 Nourriture").size(16.0).strong());
                ui.separator();

                egui::Grid::new("food_params")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Nombre:");
                        ui.add(egui::DragValue::new(&mut menu_config.food_count)
                            .range(0..=500));
                        ui.end_row();

                        ui.label("Réapparition:");
                        ui.checkbox(&mut menu_config.food_respawn_enabled, "Activée");
                        ui.end_row();

                        if menu_config.food_respawn_enabled {
                            ui.label("Temps de respawn:");
                            ui.add(egui::DragValue::new(&mut menu_config.food_respawn_time)
                                .range(1.0..=30.0)
                                .suffix(" sec"));
                            ui.end_row();
                        }

                        ui.label("Valeur nutritive:");
                        ui.add(egui::DragValue::new(&mut menu_config.food_value)
                            .range(0.1..=10.0)
                            .fixed_decimals(1));
                        ui.end_row();
                    });
            });

            ui.add_space(20.0);

            // Boutons d'action
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    if ui.add_sized([200.0, 50.0],
                                    egui::Button::new(egui::RichText::new("🚀 Lancer Simulation").size(18.0))
                                        .fill(egui::Color32::from_rgb(0, 120, 215)))
                        .clicked() {
                        apply_configuration(&mut commands, &menu_config);
                        next_state.set(AppState::Simulation);
                    }

                    ui.add_space(20.0);

                    if ui.add_sized([180.0, 50.0],
                                    egui::Button::new(egui::RichText::new("📊 Visualiseur").size(16.0))
                                        .fill(egui::Color32::from_rgb(80, 80, 80)))
                        .clicked() {
                        // TODO: À implémenter plus tard
                        info!("Visualiseur pas encore implémenté");
                    }
                });

                ui.add_space(10.0);

                if ui.button("🔄 Réinitialiser").clicked() {
                    *menu_config = MenuConfig::default();
                }
            });

            ui.add_space(20.0);
        });
    });
}

fn apply_configuration(commands: &mut Commands, config: &MenuConfig) {
    commands.insert_resource(SimulationConfig {
        world_size: config.world_size,
        simulation_count: config.simulation_count,
        particle_count: config.particle_count,
        particle_types: config.particle_types,
        particles_per_simulation: config.particle_count / config.simulation_count,
        max_force_range: config.max_force_range,
        velocity_half_life: VELOCITY_HALF_LIFE,
        food_count: config.food_count,
        food_respawn_enabled: config.food_respawn_enabled,
        food_respawn_time: config.food_respawn_time,
        food_value: config.food_value,
        elite_ratio: config.elite_ratio,
        mutation_rate: config.mutation_rate,
        crossover_rate: config.crossover_rate,
        viewport_cols: if config.simulation_count <= 4 {
            config.simulation_count.min(4) as u32
        } else { 4 },
        viewport_rows: if config.simulation_count <= 4 { 1 } else {
            ((config.simulation_count + 3) / 4) as u32
        },
    });

    commands.insert_resource(ParticleTypesConfig::new(config.particle_types));

    info!("Configuration appliquée:");
    info!("  • {} simulations avec {} particules totales", config.simulation_count, config.particle_count);
    info!("  • {} types de particules", config.particle_types);
    info!("  • Monde {}x{} unités", config.world_size, config.world_size);
}