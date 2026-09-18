use crate::app::AlgoApp;
use crate::models::Algorithm;
use crate::settings::AppTheme;
use crate::storage::load_all_from_archive;
use eframe::egui::{self, Color32, Stroke};
use std::collections::BTreeMap;
use uuid::Uuid;

impl AlgoApp {
    pub fn refresh_archive(&mut self) {
        self.archive_list = load_all_from_archive();
        self.archive_loaded = true;
    }

    pub fn render_archive_mode(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        let is_dark = self.settings.theme == AppTheme::Dark;

        if !self.archive_loaded {
            self.refresh_archive();
        }

        ui.horizontal(|ui| {
            ui.heading(lang.tab_archive());
            ui.separator();
            ui.add(
                egui::TextEdit::singleline(&mut self.archive_search)
                    .hint_text(lang.search_placeholder())
                    .desired_width(320.0_f32),
            );
            if ui.button(lang.btn_refresh()).clicked() {
                self.refresh_archive();
            }
        });
        ui.separator();

        if self.archive_list.is_empty() {
            ui.label(lang.empty_archive());
            return;
        }

        let query = self.archive_search.trim().to_lowercase();

        let mut tree: BTreeMap<String, BTreeMap<String, Vec<&Algorithm>>> = BTreeMap::new();
        for algo in &self.archive_list {
            let mfg = if algo.metadata.manufacturer.trim().is_empty() {
                lang.no_manufacturer().to_string()
            } else {
                algo.metadata.manufacturer.trim().to_string()
            };

            let model = if algo.metadata.model.trim().is_empty() {
                lang.no_model().to_string()
            } else {
                algo.metadata.model.trim().to_string()
            };

            tree.entry(mfg)
                .or_default()
                .entry(model)
                .or_default()
                .push(algo);
        }

        let mut id_to_load_builder: Option<Algorithm> = None;
        let mut id_to_load_runner: Option<Algorithm> = None;
        let mut id_to_delete: Option<(Uuid, String)> = None;

        let panel_bg = if is_dark {
            Color32::from_rgb(22, 26, 35)
        } else {
            Color32::from_rgb(234, 239, 246)
        };

        egui::SidePanel::left("archive_category_tree")
            .resizable(true)
            .default_width(260.0_f32)
            .width_range(200.0_f32..=360.0_f32)
            .frame(egui::Frame::none().fill(panel_bg).inner_margin(8.0_f32))
            .show_inside(ui, |ui| {
                ui.label(
                    egui::RichText::new(lang.equipment_catalog())
                        .strong()
                        .size(13.0_f32),
                );
                ui.add_space(4.0_f32);

                let is_all_sel =
                    self.archive_selected_mfg.is_none() && self.archive_selected_model.is_none();
                let all_btn = ui.selectable_label(
                    is_all_sel,
                    format!("📁 {} ({})", lang.all_equipment(), self.archive_list.len()),
                );
                if all_btn.clicked() {
                    self.archive_selected_mfg = None;
                    self.archive_selected_model = None;
                }
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_source("archive_tree_scroll")
                    .show(ui, |ui| {
                        for (mfg, models) in &tree {
                            let total_in_mfg: usize = models.values().map(|v| v.len()).sum();
                            let is_mfg_active = self.archive_selected_mfg.as_deref() == Some(mfg)
                                && self.archive_selected_model.is_none();

                            ui.horizontal(|ui| {
                                let label =
                                    egui::RichText::new(format!("🏭 {} ({})", mfg, total_in_mfg))
                                        .strong();
                                if ui.selectable_label(is_mfg_active, label).clicked() {
                                    self.archive_selected_mfg = Some(mfg.clone());
                                    self.archive_selected_model = None;
                                }
                            });

                            ui.indent(format!("tree_indent_{}", mfg), |ui| {
                                for (model, algos) in models {
                                    let is_model_active = self.archive_selected_mfg.as_deref()
                                        == Some(mfg)
                                        && self.archive_selected_model.as_deref() == Some(model);

                                    let model_label = format!("⚙ {} ({})", model, algos.len());
                                    if ui.selectable_label(is_model_active, model_label).clicked() {
                                        self.archive_selected_mfg = Some(mfg.clone());
                                        self.archive_selected_model = Some(model.clone());
                                    }
                                }
                            });
                            ui.add_space(2.0_f32);
                        }
                    });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let filtered: Vec<&Algorithm> = self
                .archive_list
                .iter()
                .filter(|a| {
                    if let Some(sel_mfg) = &self.archive_selected_mfg {
                        let mfg = if a.metadata.manufacturer.trim().is_empty() {
                            lang.no_manufacturer()
                        } else {
                            a.metadata.manufacturer.trim()
                        };
                        if mfg != sel_mfg {
                            return false;
                        }
                    }
                    if let Some(sel_model) = &self.archive_selected_model {
                        let model = if a.metadata.model.trim().is_empty() {
                            lang.no_model()
                        } else {
                            a.metadata.model.trim()
                        };
                        if model != sel_model {
                            return false;
                        }
                    }

                    if !query.is_empty() {
                        let matches_query = a.metadata.name.to_lowercase().contains(&query)
                            || a.metadata.manufacturer.to_lowercase().contains(&query)
                            || a.metadata.model.to_lowercase().contains(&query)
                            || a.metadata.eq_type.to_lowercase().contains(&query)
                            || a.metadata.inv_number.to_lowercase().contains(&query)
                            || a.metadata.author.to_lowercase().contains(&query);
                        if !matches_query {
                            return false;
                        }
                    }
                    true
                })
                .collect();

            if filtered.is_empty() {
                ui.add_space(20.0_f32);
                ui.label(lang.not_found());
                return;
            }

            let card_bg = if is_dark {
                Color32::from_rgb(26, 31, 42)
            } else {
                Color32::from_rgb(255, 255, 255)
            };
            let card_border = if is_dark {
                Color32::from_rgb(46, 56, 76)
            } else {
                Color32::from_rgb(203, 213, 225)
            };

            egui::ScrollArea::vertical()
                .id_source("archive_items_scroll")
                .show(ui, |ui| {
                    ui.add_space(2.0_f32);
                    for algo in filtered {
                        let display_name = if algo.metadata.name.trim().is_empty() {
                            lang.untitled().to_string()
                        } else {
                            algo.metadata.name.clone()
                        };

                        egui::Frame::none()
                            .fill(card_bg)
                            .stroke(Stroke::new(1.0_f32, card_border))
                            .rounding(6.0_f32)
                            .inner_margin(12.0_f32)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(&display_name)
                                                .strong()
                                                .size(15.0_f32),
                                        );
                                        ui.add_space(3.0_f32);

                                        ui.horizontal(|ui| {
                                            if !algo.metadata.manufacturer.is_empty() {
                                                ui.colored_label(
                                                    if is_dark {
                                                        Color32::from_rgb(80, 160, 255)
                                                    } else {
                                                        Color32::from_rgb(29, 78, 216)
                                                    },
                                                    &algo.metadata.manufacturer,
                                                );
                                                ui.label("•");
                                            }
                                            if !algo.metadata.model.is_empty() {
                                                ui.label(format!(
                                                    "{} {}",
                                                    lang.model_prefix(),
                                                    algo.metadata.model
                                                ));
                                                ui.label("•");
                                            }
                                            if !algo.metadata.inv_number.is_empty() {
                                                ui.label(format!(
                                                    "{} {}",
                                                    lang.inv_prefix(),
                                                    algo.metadata.inv_number
                                                ));
                                                ui.label("•");
                                            }
                                            ui.colored_label(
                                                if is_dark {
                                                    Color32::from_rgb(140, 150, 165)
                                                } else {
                                                    Color32::from_rgb(100, 116, 139)
                                                },
                                                format!(
                                                    "{} {}",
                                                    algo.steps.len(),
                                                    lang.card_steps()
                                                ),
                                            );
                                        });

                                        ui.add_space(2.0_f32);
                                        ui.horizontal(|ui| {
                                            if !algo.metadata.author.is_empty() {
                                                ui.label(format!(
                                                    "{} {}",
                                                    lang.author_prefix(),
                                                    algo.metadata.author
                                                ));
                                                ui.label("•");
                                            }
                                            ui.label(format!(
                                                "{} {}",
                                                lang.card_updated(),
                                                algo.metadata.updated_at.format("%d.%m.%Y %H:%M")
                                            ));
                                        });
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .button(
                                                    egui::RichText::new(lang.btn_delete_archive())
                                                        .color(Color32::from_rgb(220, 38, 38)),
                                                )
                                                .clicked()
                                            {
                                                id_to_delete =
                                                    Some((algo.id, display_name.clone()));
                                            }
                                            if ui.button(lang.btn_open_builder()).clicked() {
                                                id_to_load_builder = Some(algo.clone());
                                            }

                                            let run_btn = egui::Button::new(
                                                egui::RichText::new(lang.btn_run_diag())
                                                    .color(Color32::WHITE)
                                                    .strong(),
                                            )
                                            .fill(Color32::from_rgb(37, 99, 235))
                                            .rounding(5.0_f32);

                                            if ui.add(run_btn).clicked() {
                                                id_to_load_runner = Some(algo.clone());
                                            }
                                        },
                                    );
                                });
                            });
                        ui.add_space(6.0_f32);
                    }
                });
        });

        if let Some(algo) = id_to_load_builder {
            self.active_algo = algo.clone();
            self.last_saved_algo = Some(algo);
            self.selected_step_ids.clear();
            if let Some(first) = self.active_algo.first_step_id {
                self.selected_step_ids.insert(first);
            }
            self.current_tab = "Конструктор".to_string();
        }

        if let Some(algo) = id_to_load_runner {
            self.active_algo = algo.clone();
            self.last_saved_algo = Some(algo);
            self.reset_runner();
            self.current_tab = "Диагностика".to_string();
        }

        if let Some(target) = id_to_delete {
            self.confirm_delete_target = Some(target);
        }
    }
}
