use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashSet;

use crate::components::simulation::Simulation;
use crate::resources::particle_config::ParticleConfig;

#[derive(Resource)]
pub struct SimulationUI {
    pub selected_simulation: Option<usize>,
    pub show_matrix_window: bool,
    pub show_simulations_list: bool,
    pub selected_simulations: HashSet<usize>,
    pub right_panel_width: f32,
}

impl Default for SimulationUI {
    fn default() -> Self {
        let mut selected_simulations = HashSet::new();
        selected_simulations.insert(0);

        Self {
            selected_simulation: None,
            show_matrix_window: false,
            show_simulations_list: true,
            selected_simulations,
            right_panel_width: 0.0,
        }
    }
}

pub fn simulations_list_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<SimulationUI>,
    simulations: Query<&Simulation>,
) {
    let ctx = contexts.ctx_mut();

    if !ui_state.show_simulations_list {
        ui_state.right_panel_width = 0.0;
        return
    }

    let panel_width = 350.0;

    egui::SidePanel::right("simulations_panel")
        .exact_width(panel_width)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("🔬 Simulations");

            ui.horizontal(|ui| {
                if ui.button("Tout sélectionner").clicked() {
                    for (i, _) in simulations.iter().enumerate() {
                        ui_state.selected_simulations.insert(i);
                    }
                }
                if ui.button("Tout désélectionner").clicked() {
                    ui_state.selected_simulations.clear();
                }
            });

            ui.separator();

            let sim_list: Vec<(usize, &Simulation)> = simulations.iter().enumerate().collect();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("simulations_grid")
                    .num_columns(4)
                    .spacing([15.0, 5.0])
                    .striped(true)
                    .min_col_width(40.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Vue").strong());
                        ui.label(egui::RichText::new("Sim").strong());
                        ui.label(egui::RichText::new("Score").strong());
                        ui.label(egui::RichText::new("Matrice").strong());
                        ui.end_row();

                        ui.separator();
                        ui.separator();
                        ui.separator();
                        ui.separator();
                        ui.end_row();

                        for (sim_id, sim) in sim_list {
                            let is_selected_for_matrix = ui_state.selected_simulation == Some(sim_id);

                            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                let mut is_selected_for_view = ui_state.selected_simulations.contains(&sim_id);
                                if ui.checkbox(&mut is_selected_for_view, "").changed() {
                                    if is_selected_for_view {
                                        ui_state.selected_simulations.insert(sim_id);
                                    } else {
                                        ui_state.selected_simulations.remove(&sim_id);
                                    }
                                }
                            });

                            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                let sim_label = if is_selected_for_matrix {
                                    egui::RichText::new(format!("#{}", sim_id + 1))
                                        .color(egui::Color32::from_rgb(100, 200, 255))
                                        .strong()
                                } else {
                                    egui::RichText::new(format!("#{}", sim_id + 1))
                                };

                                if ui.selectable_label(false, sim_label).clicked() {
                                    ui_state.selected_simulation = Some(sim_id);
                                    ui_state.show_matrix_window = true;
                                }
                            });

                            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                let score_color = if sim.score > 50.0 {
                                    egui::Color32::from_rgb(0, 255, 0)
                                } else if sim.score > 20.0 {
                                    egui::Color32::from_rgb(255, 255, 0)
                                } else if sim.score > 10.0 {
                                    egui::Color32::from_rgb(255, 150, 0)
                                } else {
                                    egui::Color32::from_rgb(200, 200, 200)
                                };
                                ui.label(egui::RichText::new(format!("{:.0}", sim.score))
                                    .color(score_color)
                                    .monospace());
                            });

                            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.button("📊").on_hover_text("Voir matrice").clicked() {
                                    ui_state.selected_simulation = Some(sim_id);
                                    ui_state.show_matrix_window = true;
                                }
                            });

                            ui.end_row();
                        }
                    });
            });

            ui.separator();
            ui.label(format!("👁 {} vue(s) active(s)", ui_state.selected_simulations.len()));
        });

    ui_state.right_panel_width = panel_width;
}

pub fn force_matrix_window(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<SimulationUI>,
    config: Res<ParticleConfig>,
    simulations: Query<&Simulation>,
) {
    if !ui_state.show_matrix_window || ui_state.selected_simulation.is_none() {
        return
    }

    let ctx = contexts.ctx_mut();
    let selected_sim = ui_state.selected_simulation.unwrap();

    egui::Window::new(format!("🔬 Matrice des Forces - Simulation #{}", selected_sim + 1))
        .resizable(true)
        .collapsible(true)
        .min_width(500.0)
        .open(&mut ui_state.show_matrix_window)
        .show(ctx, |ui| {
            if let Some(simulation) = simulations.iter().nth(selected_sim) {
                ui.label(format!("Types de particules: {}", config.num_types));
                ui.label(egui::RichText::new("Forces normalisées entre -2.000 et +2.000")
                    .small()
                    .color(egui::Color32::from_rgb(150, 150, 150)));
                ui.separator();

                // Matrice des forces
                ui.label(egui::RichText::new("⚡ Forces d'Interaction").size(14.0).strong());
                ui.add_space(5.0);

                egui::Grid::new("force_matrix_grid")
                    .num_columns(config.num_types as usize + 1)
                    .spacing([10.0, 4.0])
                    .min_col_width(70.0)
                    .show(ui, |ui| {
                        ui.label("De\\Vers");

                        for j in 0..config.num_types {
                            let color = get_type_color(j as usize, config.num_types as usize);
                            ui.label(egui::RichText::new(format!("T{}", j))
                                .color(egui::Color32::from_rgb(
                                    (color.to_srgba().red * 255.0) as u8,
                                    (color.to_srgba().green * 255.0) as u8,
                                    (color.to_srgba().blue * 255.0) as u8,
                                ))
                                .strong());
                        }
                        ui.end_row();

                        for _ in 0..=config.num_types {
                            ui.separator();
                        }
                        ui.end_row();

                        for i in 0..config.num_types {
                            let color = get_type_color(i as usize, config.num_types as usize);
                            ui.label(egui::RichText::new(format!("T{}", i))
                                .color(egui::Color32::from_rgb(
                                    (color.to_srgba().red * 255.0) as u8,
                                    (color.to_srgba().green * 255.0) as u8,
                                    (color.to_srgba().blue * 255.0) as u8,
                                ))
                                .strong());

                            for j in 0..config.num_types {
                                let force = config.get_force(i, j);

                                let force_color = if force.abs() < 0.05 {
                                    egui::Color32::from_rgb(120, 120, 120)
                                } else if force > 0.0 {
                                    let intensity = (force.abs() * 127.5 + 127.5) as u8;
                                    egui::Color32::from_rgb(0, intensity.max(100), 0)
                                } else {
                                    let intensity = (force.abs() * 127.5 + 127.5) as u8;
                                    egui::Color32::from_rgb(intensity.max(100), 0, 0)
                                };

                                ui.label(egui::RichText::new(format!("{:+.3}", force))
                                    .color(force_color)
                                    .monospace()
                                    .size(11.0));
                            }
                            ui.end_row();
                        }
                    });

                ui.add_space(10.0);
                ui.separator();

                ui.collapsing("🔧 Détails techniques", |ui| {
                    ui.label(format!("Score actuel: {:.1}", simulation.score));
                    ui.label(format!("ID simulation: {}", simulation.id));
                    ui.label(format!("Forces stockées: {}", config.force_matrix.len()));
                    ui.separator();
                    ui.label(egui::RichText::new("Facteur d'échelle: 80.0").strong());
                    ui.label("Forces réelles = valeurs × 80.0");
                });
            }
        });
}

// Fonction helper pour obtenir la couleur d'un type
fn get_type_color(type_index: usize, total_types: usize) -> Color {
    let hue = (type_index as f32 / total_types as f32) * 360.0;
    Color::hsl(hue, 0.8, 0.6)
}