use crate::app::AlgoApp;
use crate::models::{OptionLink, StepKind};
use crate::settings::{ui_kbd, AppTheme};
use crate::storage::to_image_uri;
use eframe::egui::{self, Color32, Stroke};
use uuid::Uuid;

impl AlgoApp {
    pub fn render_inspector(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        let is_dark = self.settings.theme == AppTheme::Dark;

        ui.heading(lang.inspector_heading());
        ui.separator();

        if self.selected_step_ids.len() > 1 {
            ui.label(format!(
                "{}{}",
                lang.multi_select_hint(),
                self.selected_step_ids.len()
            ));
            ui.add_space(6.0_f32);
            if ui.button(lang.delete_btn()).clicked() {
                self.delete_selected();
            }
            return;
        }

        let single_selected_id = self.selected_step_ids.iter().next().copied();

        let step_lookup: Vec<(Option<Uuid>, String)> =
            std::iter::once((None, lang.finish_diag().to_string()))
                .chain(
                    self.active_algo
                        .steps
                        .iter()
                        .map(|s| (Some(s.id), s.title.clone())),
                )
                .collect();

        let mut do_delete = false;
        let mut make_start_id: Option<Uuid> = None;
        let is_curr_start =
            single_selected_id.map_or(false, |id| self.active_algo.first_step_id == Some(id));

        let card_frame = egui::Frame::none()
            .fill(if is_dark {
                Color32::from_rgb(24, 29, 39)
            } else {
                Color32::from_rgb(255, 255, 255)
            })
            .stroke(Stroke::new(
                1.0_f32,
                if is_dark {
                    Color32::from_rgb(44, 54, 72)
                } else {
                    Color32::from_rgb(203, 213, 225)
                },
            ))
            .rounding(6.0_f32)
            .inner_margin(10.0_f32);

        if let Some(selected_id) = single_selected_id {
            if let Some(step) = self
                .active_algo
                .steps
                .iter_mut()
                .find(|s| s.id == selected_id)
            {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Карточка 1: Основные параметры
                    card_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(lang.section_general()).strong().color(
                                if is_dark {
                                    Color32::from_rgb(80, 150, 255)
                                } else {
                                    Color32::from_rgb(29, 78, 216)
                                },
                            ));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(
                                            egui::RichText::new("🗑")
                                                .color(Color32::from_rgb(220, 38, 38)),
                                        )
                                        .on_hover_text(lang.delete_btn())
                                        .clicked()
                                    {
                                        do_delete = true;
                                    }
                                },
                            );
                        });
                        ui.add_space(4.0_f32);

                        let mut is_start = is_curr_start;
                        if ui.checkbox(&mut is_start, lang.start_node()).changed() && is_start {
                            make_start_id = Some(step.id);
                        }

                        ui.add_space(4.0_f32);
                        ui.label(lang.block_title());
                        ui.text_edit_singleline(&mut step.title);

                        ui.add_space(4.0_f32);
                        ui.label(lang.behavior_type());
                        let mut kind_idx = match step.kind {
                            StepKind::Standard => 0,
                            StepKind::Measurement { .. } => 1,
                            StepKind::SafetyWarning { .. } => 2,
                        };
                        let old_idx = kind_idx;
                        egui::ComboBox::from_id_source("inspector_kind_select")
                            .selected_text(match kind_idx {
                                0 => lang.kind_standard(),
                                1 => lang.kind_measurement(),
                                2 => lang.kind_safety(),
                                _ => "",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut kind_idx, 0, lang.kind_standard());
                                ui.selectable_value(&mut kind_idx, 1, lang.kind_measurement());
                                ui.selectable_value(&mut kind_idx, 2, lang.kind_safety());
                            });

                        if kind_idx != old_idx {
                            step.kind = match kind_idx {
                                0 => StepKind::Standard,
                                1 => StepKind::Measurement {
                                    unit: lang.default_unit().to_string(),
                                    min_val: 200.0_f64,
                                    max_val: 240.0_f64,
                                    next_if_normal: None,
                                    next_if_abnormal: None,
                                },
                                2 => StepKind::SafetyWarning {
                                    ack_text: lang.default_safety_ack().to_string(),
                                    next_step_id: None,
                                },
                                _ => StepKind::Standard,
                            };
                        }
                    });
                    ui.add_space(6.0_f32);

                    // Карточка 2: Инструкция специалисту и прикрепленное фото
                    card_frame.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(lang.section_instructions())
                                .strong()
                                .color(if is_dark {
                                    Color32::from_rgb(80, 150, 255)
                                } else {
                                    Color32::from_rgb(29, 78, 216)
                                }),
                        );
                        ui.add_space(4.0_f32);
                        ui.text_edit_multiline(&mut step.description);

                        ui.add_space(4.0_f32);
                        ui.horizontal(|ui| {
                            if ui.button(lang.photo_btn()).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg"])
                                    .pick_file()
                                {
                                    step.image_path = Some(path.display().to_string());
                                }
                            }
                            if step.image_path.is_some() && ui.button("❌").clicked() {
                                step.image_path = None;
                            }
                        });

                        if let Some(path) = &step.image_path {
                            ui.add_space(4.0_f32);
                            ui.add(
                                egui::Image::new(to_image_uri(path))
                                    .max_width(260.0_f32)
                                    .rounding(4.0_f32),
                            );
                        }
                    });
                    ui.add_space(6.0_f32);

                    // Карточка 3: Логика ветвлений и переходов
                    card_frame.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(lang.section_branching())
                                .strong()
                                .color(if is_dark {
                                    Color32::from_rgb(80, 150, 255)
                                } else {
                                    Color32::from_rgb(29, 78, 216)
                                }),
                        );
                        ui.add_space(4.0_f32);

                        match &mut step.kind {
                            StepKind::Standard => {
                                if ui.button(lang.add_branch()).clicked() {
                                    step.options.push(OptionLink {
                                        text: lang.default_branch_yes().to_string(),
                                        next_step_id: None,
                                    });
                                }

                                let mut opt_to_del = None;
                                for (idx, opt) in step.options.iter_mut().enumerate() {
                                    ui.add_space(2.0_f32);
                                    egui::Frame::none()
                                        .fill(if is_dark {
                                            Color32::from_rgb(30, 37, 50)
                                        } else {
                                            Color32::from_rgb(241, 245, 249)
                                        })
                                        .stroke(Stroke::new(
                                            1.0_f32,
                                            if is_dark {
                                                Color32::from_rgb(45, 58, 80)
                                            } else {
                                                Color32::from_rgb(203, 213, 225)
                                            },
                                        ))
                                        .rounding(4.0_f32)
                                        .inner_margin(6.0_f32)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(lang.branch_text());
                                                ui.text_edit_singleline(&mut opt.text);
                                                if ui.button("✖").clicked() {
                                                    opt_to_del = Some(idx);
                                                }
                                            });

                                            let cur_title = step_lookup
                                                .iter()
                                                .find(|(id, _)| *id == opt.next_step_id)
                                                .map(|(_, t)| t.as_str())
                                                .unwrap_or(lang.not_selected());
                                            egui::ComboBox::from_id_source(format!(
                                                "combo_{}_{}",
                                                step.id, idx
                                            ))
                                            .selected_text(cur_title)
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    for (id_opt, title) in &step_lookup {
                                                        ui.selectable_value(
                                                            &mut opt.next_step_id,
                                                            *id_opt,
                                                            title,
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                }
                                if let Some(del) = opt_to_del {
                                    step.options.remove(del);
                                }
                            }

                            StepKind::Measurement {
                                unit,
                                min_val,
                                max_val,
                                next_if_normal,
                                next_if_abnormal,
                            } => {
                                ui.horizontal(|ui| {
                                    ui.label(lang.unit());
                                    ui.text_edit_singleline(unit);
                                });
                                ui.horizontal(|ui| {
                                    ui.label(lang.min_val());
                                    ui.add(egui::DragValue::new(min_val).speed(0.1_f64));
                                    ui.label(lang.max_val());
                                    ui.add(egui::DragValue::new(max_val).speed(0.1_f64));
                                });

                                ui.add_space(4.0_f32);
                                ui.label(lang.goto_normal());
                                let norm_title = step_lookup
                                    .iter()
                                    .find(|(id, _)| *id == *next_if_normal)
                                    .map(|(_, t)| t.as_str())
                                    .unwrap_or(lang.not_selected());
                                egui::ComboBox::from_id_source(format!("norm_{}", step.id))
                                    .selected_text(norm_title)
                                    .show_ui(ui, |ui| {
                                        for (id_opt, title) in &step_lookup {
                                            ui.selectable_value(next_if_normal, *id_opt, title);
                                        }
                                    });

                                ui.label(lang.goto_abnormal());
                                let abn_title = step_lookup
                                    .iter()
                                    .find(|(id, _)| *id == *next_if_abnormal)
                                    .map(|(_, t)| t.as_str())
                                    .unwrap_or(lang.not_selected());
                                egui::ComboBox::from_id_source(format!("abn_{}", step.id))
                                    .selected_text(abn_title)
                                    .show_ui(ui, |ui| {
                                        for (id_opt, title) in &step_lookup {
                                            ui.selectable_value(next_if_abnormal, *id_opt, title);
                                        }
                                    });
                            }

                            StepKind::SafetyWarning {
                                ack_text,
                                next_step_id,
                            } => {
                                ui.label(lang.safety_ack());
                                ui.text_edit_singleline(ack_text);
                                ui.label(lang.goto_after_ack());
                                let next_title = step_lookup
                                    .iter()
                                    .find(|(id, _)| *id == *next_step_id)
                                    .map(|(_, t)| t.as_str())
                                    .unwrap_or(lang.not_selected());
                                egui::ComboBox::from_id_source(format!("safe_{}", step.id))
                                    .selected_text(next_title)
                                    .show_ui(ui, |ui| {
                                        for (id_opt, title) in &step_lookup {
                                            ui.selectable_value(next_step_id, *id_opt, title);
                                        }
                                    });
                            }
                        }
                    });
                });
            }
        } else {
            // Справочный центр
            card_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("💡").size(18.0_f32));
                    ui.label(
                        egui::RichText::new(lang.inspector_empty_title())
                            .strong()
                            .size(13.5_f32),
                    );
                });
                ui.add_space(4.0_f32);
                ui.label(
                    egui::RichText::new(lang.inspector_empty_desc())
                        .size(11.5_f32)
                        .color(if is_dark {
                            Color32::from_rgb(140, 150, 168)
                        } else {
                            Color32::from_rgb(100, 116, 139)
                        }),
                );

                ui.add_space(10.0_f32);
                ui.separator();
                ui.add_space(6.0_f32);

                ui.label(
                    egui::RichText::new(lang.cheat_sheet_title())
                        .strong()
                        .size(12.0_f32),
                );
                ui.add_space(6.0_f32);

                let mut kbd_row = |keys: &[&str], desc: &str| {
                    ui.horizontal(|ui| {
                        for k in keys {
                            ui_kbd(ui, k, is_dark);
                        }
                        ui.label(egui::RichText::new(desc).size(11.5_f32).color(if is_dark {
                            Color32::from_rgb(160, 170, 185)
                        } else {
                            Color32::from_rgb(51, 65, 85)
                        }));
                    });
                    ui.add_space(3.0_f32);
                };

                let l_2x_lmb = format!("2x {}", lang.key_lmb());
                let l_lmb = lang.key_lmb();
                let l_mmb = lang.key_mmb();
                let l_dot = lang.key_dot();
                let l_badge = lang.key_badge();

                kbd_row(&[&l_2x_lmb], lang.cheat_create_node());
                kbd_row(&[l_lmb, l_dot], lang.cheat_drag_wire());
                kbd_row(&[l_lmb, l_badge], lang.cheat_cut_wire());
                kbd_row(&[lang.key_rmb(), lang.key_wire()], lang.cheat_toggle_wire());
                kbd_row(&["Shift", l_lmb], lang.cheat_multiselect());
                kbd_row(&["Del"], lang.cheat_del_selected());
                kbd_row(&["Ctrl", "Z"], lang.cheat_undo_act());
                kbd_row(&["Ctrl", "Y"], lang.cheat_redo_act());
                kbd_row(&["Ctrl", "F"], lang.cheat_search_act());
                kbd_row(&[l_mmb], lang.cheat_pan_act());
            });
        }

        if let Some(id) = make_start_id {
            self.snapshot_for_undo();
            self.active_algo.first_step_id = Some(id);
        }
        if do_delete {
            self.delete_selected();
        }
    }
}
