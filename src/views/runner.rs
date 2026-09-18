use crate::app::AlgoApp;
use crate::models::StepKind;
use crate::settings::{ui_kbd, AppTheme};
use crate::storage::to_image_uri;
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use uuid::Uuid;

impl AlgoApp {
    pub fn reset_runner(&mut self) {
        // Если находились внутри подпроцесса, возвращаемся в корневой алгоритм
        if let Some(first_frame) = self.runner.call_stack.first().cloned() {
            self.active_algo = first_frame.parent_algo;
        }

        self.runner.call_stack.clear();
        self.runner.had_abnormality = false;
        self.runner.current_step_id = self.active_algo.first_step_id;
        self.runner.history.clear();
        self.runner.measurement_input.clear();
        self.runner.safety_acknowledged = false;

        if let Some(first_id) = self.active_algo.first_step_id {
            if let Some(rect) = self.last_canvas_rect {
                self.focus_step_on_canvas(first_id, rect.size());
            }
        }
    }

    pub fn navigate_runner(&mut self, next_id: Option<Uuid>) {
        if let Some(curr) = self.runner.current_step_id {
            self.runner.history.push(curr);
        }

        // 1. АВТО-ВХОД В ПОДПРОЦЕСС
        if let Some(nid) = next_id {
            if let Some(step) = self.active_algo.steps.iter().find(|s| s.id == nid) {
                if let StepKind::Subprocess { sub_algo, .. } = &step.kind {
                    let sub_algo_clone = (**sub_algo).clone();
                    let first_step = sub_algo_clone.first_step_id;
                    let parent_algo = self.active_algo.clone();
                    let parent_history = std::mem::take(&mut self.runner.history);

                    self.runner.call_stack.push(crate::app::RunnerCallFrame {
                        parent_step_id: nid,
                        parent_algo,
                        parent_history,
                        had_abnormality: self.runner.had_abnormality,
                    });

                    self.runner.had_abnormality = false;
                    self.active_algo = sub_algo_clone;
                    self.runner.current_step_id = first_step;
                    self.runner.measurement_input.clear();
                    self.runner.safety_acknowledged = false;

                    if let Some(id) = first_step {
                        if let Some(rect) = self.last_canvas_rect {
                            self.focus_step_on_canvas(id, rect.size());
                        }
                    }
                    return;
                }
            }
        }

        // 2. АВТО-ВЫХОД ИЗ ПОДПРОЦЕССА (если дошли до тупика/конца)
        if next_id.is_none() && !self.runner.call_stack.is_empty() {
            if let Some(frame) = self.runner.call_stack.pop() {
                let sub_had_abnormality = self.runner.had_abnormality;
                let parent_algo = frame.parent_algo;
                let parent_step_id = frame.parent_step_id;

                let mut return_target = None;
                if let Some(step) = parent_algo.steps.iter().find(|s| s.id == parent_step_id) {
                    if let StepKind::Subprocess {
                        next_if_success,
                        next_if_failure,
                        ..
                    } = &step.kind
                    {
                        return_target = if sub_had_abnormality {
                            *next_if_failure
                        } else {
                            *next_if_success
                        };
                    }
                }

                self.active_algo = parent_algo;
                self.runner.history = frame.parent_history;
                self.runner.history.push(parent_step_id);
                self.runner.had_abnormality = frame.had_abnormality || sub_had_abnormality;
                self.runner.current_step_id = return_target;
                self.runner.measurement_input.clear();
                self.runner.safety_acknowledged = false;

                if let Some(id) = return_target {
                    if let Some(rect) = self.last_canvas_rect {
                        self.focus_step_on_canvas(id, rect.size());
                    }
                }
                return;
            }
        }

        // Обычный переход
        self.runner.current_step_id = next_id;
        self.runner.measurement_input.clear();
        self.runner.safety_acknowledged = false;

        if let Some(id) = next_id {
            if let Some(rect) = self.last_canvas_rect {
                self.focus_step_on_canvas(id, rect.size());
            }
        }
    }

    pub fn navigate_runner_back(&mut self) {
        if let Some(prev) = self.runner.history.pop() {
            self.runner.current_step_id = Some(prev);
            self.runner.measurement_input.clear();
            self.runner.safety_acknowledged = false;

            if let Some(rect) = self.last_canvas_rect {
                self.focus_step_on_canvas(prev, rect.size());
            }
        } else if let Some(frame) = self.runner.call_stack.pop() {
            // Откат назад через границу подпроцесса в родительский граф
            self.active_algo = frame.parent_algo;
            self.runner.history = frame.parent_history;
            self.runner.had_abnormality = frame.had_abnormality;
            self.runner.current_step_id = Some(frame.parent_step_id);
            self.runner.measurement_input.clear();
            self.runner.safety_acknowledged = false;

            if let Some(rect) = self.last_canvas_rect {
                self.focus_step_on_canvas(frame.parent_step_id, rect.size());
            }
        }
    }

    pub fn render_runner_mode(&mut self, ui: &mut egui::Ui) {
        let view_size = ui.available_size_before_wrap();
        let lang = self.settings.language;
        let is_dark = self.settings.theme == AppTheme::Dark;

        ui.horizontal(|ui| {
            let island_frame = egui::Frame::none()
                .fill(if is_dark {
                    Color32::from_rgb(25, 30, 40)
                } else {
                    Color32::from_rgb(255, 255, 255)
                })
                .rounding(6.0_f32)
                .stroke(Stroke::new(
                    1.0_f32,
                    if is_dark {
                        Color32::from_rgb(45, 54, 70)
                    } else {
                        Color32::from_rgb(203, 213, 225)
                    },
                ))
                .inner_margin(egui::Margin::symmetric(4.0_f32, 3.0_f32));

            island_frame.show(ui, |ui| {
                if !self.runner.history.is_empty() && ui.button(lang.step_back()).clicked() {
                    self.navigate_runner_back();
                }
                if ui.button(lang.reset_test()).clicked() {
                    self.reset_runner();
                }
                if let Some(curr_id) = self.runner.current_step_id {
                    if ui.button(lang.focus_step()).clicked() {
                        self.focus_step_on_canvas(curr_id, view_size);
                    }
                }
            });

            island_frame.show(ui, |ui| {
                let center = Pos2::new(view_size.x * 0.5_f32, view_size.y * 0.5_f32).to_vec2();

                if ui.button("➖").on_hover_text(lang.zoom_out_tip()).clicked() {
                    self.zoom_by_step(false, center);
                }

                if ui
                    .button(format!("{:.0}%", (self.canvas_zoom * 100.0_f32).round()))
                    .on_hover_text(lang.zoom_reset_tip())
                    .clicked()
                {
                    self.zoom_reset(center);
                }

                if ui.button("➕").on_hover_text(lang.zoom_in_tip()).clicked() {
                    self.zoom_by_step(true, center);
                }

                if ui.button(lang.fit_view()).clicked() {
                    self.fit_to_view(view_size);
                }
            });
        });
        ui.separator();

        egui::SidePanel::right("runner_panel")
            .resizable(true)
            .default_width(360.0_f32)
            .width_range(300.0_f32..=500.0_f32)
            .show_inside(ui, |ui| {
                self.render_runner_controls(ui);
            });

        self.render_canvas(ui, true);
    }

    pub fn render_runner_controls(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        let is_dark = self.settings.theme == AppTheme::Dark;

        ui.heading(lang.runner_heading());
        ui.label(
            egui::RichText::new(format!(
                "{} ({}) | {}: {}",
                self.active_algo.metadata.name,
                self.active_algo.metadata.model,
                lang.inv_prefix(),
                self.active_algo.metadata.inv_number
            ))
            .size(12.0_f32)
            .color(if is_dark {
                Color32::from_rgb(140, 150, 168)
            } else {
                Color32::from_rgb(100, 116, 139)
            }),
        );
        ui.separator();

        if !self.runner.history.is_empty() || self.runner.current_step_id.is_some() {
            ui.label(
                egui::RichText::new(lang.route_history())
                    .strong()
                    .size(12.0_f32),
            );
            ui.add_space(2.0_f32);

            let mut rollback_to_idx: Option<usize> = None;

            egui::ScrollArea::horizontal()
                .id_source("runner_breadcrumbs")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (idx, step_id) in self.runner.history.iter().enumerate() {
                            let step_name = self
                                .active_algo
                                .steps
                                .iter()
                                .find(|s| s.id == *step_id)
                                .map_or(lang.crumb_step(), |s| s.title.as_str());

                            let crumb =
                                egui::Button::new(egui::RichText::new(step_name).size(11.0_f32))
                                    .fill(if is_dark {
                                        Color32::from_rgb(26, 34, 48)
                                    } else {
                                        Color32::from_rgb(255, 255, 255)
                                    })
                                    .stroke(Stroke::new(
                                        1.0_f32,
                                        if is_dark {
                                            Color32::from_rgb(45, 58, 80)
                                        } else {
                                            Color32::from_rgb(203, 213, 225)
                                        },
                                    ))
                                    .rounding(4.0_f32);

                            if ui
                                .add(crumb)
                                .on_hover_text(lang.crumb_rollback_tip())
                                .clicked()
                            {
                                rollback_to_idx = Some(idx);
                            }
                            ui.label("➔");
                        }

                        if let Some(curr_id) = self.runner.current_step_id {
                            let step_name = self
                                .active_algo
                                .steps
                                .iter()
                                .find(|s| s.id == curr_id)
                                .map_or(lang.crumb_current(), |s| s.title.as_str());
                            let current_tag = egui::Frame::none()
                                .fill(Color32::from_rgb(37, 99, 235))
                                .rounding(4.0_f32)
                                .inner_margin(egui::Margin::symmetric(6.0_f32, 3.0_f32));
                            current_tag.show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(step_name)
                                        .color(Color32::WHITE)
                                        .strong()
                                        .size(11.0_f32),
                                );
                            });
                        }
                    });
                });

            if let Some(idx) = rollback_to_idx {
                let target_id = self.runner.history[idx];
                self.runner.history.truncate(idx);
                self.runner.current_step_id = Some(target_id);
                self.runner.measurement_input.clear();
                self.runner.safety_acknowledged = false;
                if let Some(rect) = self.last_canvas_rect {
                    self.focus_step_on_canvas(target_id, rect.size());
                }
            }

            ui.add_space(8.0_f32);
            ui.separator();
        }

        let current_id = match self.runner.current_step_id {
            Some(id) => id,
            None => {
                ui.add_space(8.0_f32);
                ui.colored_label(
                    if is_dark {
                        Color32::from_rgb(60, 220, 130)
                    } else {
                        Color32::from_rgb(16, 185, 129)
                    },
                    egui::RichText::new(lang.diag_complete())
                        .strong()
                        .size(14.0_f32),
                );
                ui.add_space(10.0_f32);
                let start_over_btn = egui::Button::new(
                    egui::RichText::new(lang.start_over())
                        .color(Color32::WHITE)
                        .strong(),
                )
                .fill(Color32::from_rgb(37, 99, 235))
                .rounding(6.0_f32)
                .min_size(Vec2::new(ui.available_width(), 36.0_f32));

                if ui.add(start_over_btn).clicked() {
                    self.reset_runner();
                }
                return;
            }
        };

        let current_step = self
            .active_algo
            .steps
            .iter()
            .find(|s| s.id == current_id)
            .cloned();

        if let Some(step) = current_step {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.colored_label(
                    if is_dark {
                        Color32::from_rgb(255, 200, 50)
                    } else {
                        Color32::from_rgb(217, 119, 6)
                    },
                    egui::RichText::new(format!("{} {}", lang.current_node_label(), step.title))
                        .strong()
                        .size(15.0_f32),
                );
                ui.add_space(6.0_f32);

                if !step.description.is_empty() {
                    egui::Frame::none()
                        .fill(if is_dark {
                            Color32::from_rgb(22, 26, 36)
                        } else {
                            Color32::from_rgb(255, 255, 255)
                        })
                        .rounding(6.0_f32)
                        .stroke(Stroke::new(
                            1.0_f32,
                            if is_dark {
                                Color32::from_rgb(44, 54, 74)
                            } else {
                                Color32::from_rgb(203, 213, 225)
                            },
                        ))
                        .inner_margin(10.0_f32)
                        .show(ui, |ui| {
                            ui.label(&step.description);
                        });
                }

                if let Some(path) = &step.image_path {
                    ui.add_space(8.0_f32);
                    ui.add(
                        egui::Image::new(to_image_uri(path))
                            .max_width(ui.available_width())
                            .rounding(6.0_f32),
                    );
                }

                ui.add_space(12.0_f32);
                ui.separator();
                ui.add_space(4.0_f32);

                let pressed_num = if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
                    Some(0)
                } else if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
                    Some(1)
                } else if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
                    Some(2)
                } else {
                    None
                };

                match &step.kind {
                    StepKind::Standard => {
                        ui.label(egui::RichText::new(lang.choose_action()).strong());
                        ui.add_space(6.0_f32);
                        if step.options.is_empty() {
                            ui.colored_label(
                                if is_dark {
                                    Color32::from_rgb(60, 220, 130)
                                } else {
                                    Color32::from_rgb(16, 185, 129)
                                },
                                lang.final_point(),
                            );
                            ui.add_space(6.0_f32);
                            let finish_btn = egui::Button::new(
                                egui::RichText::new(lang.finish_diag_btn())
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(16, 185, 129))
                            .min_size(Vec2::new(ui.available_width(), 38.0_f32));

                            if ui.add(finish_btn).clicked() {
                                self.navigate_runner(None);
                            }
                        } else {
                            for (idx, opt) in step.options.iter().enumerate() {
                                let hotkey_triggered = pressed_num == Some(idx);

                                let card_frame = egui::Frame::none()
                                    .fill(if is_dark {
                                        Color32::from_rgb(26, 33, 46)
                                    } else {
                                        Color32::from_rgb(255, 255, 255)
                                    })
                                    .rounding(6.0_f32)
                                    .stroke(Stroke::new(
                                        1.0_f32,
                                        if is_dark {
                                            Color32::from_rgb(50, 64, 88)
                                        } else {
                                            Color32::from_rgb(203, 213, 225)
                                        },
                                    ))
                                    .inner_margin(egui::Margin::symmetric(10.0_f32, 8.0_f32));

                                let response = card_frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui_kbd(ui, &format!("{}", idx + 1), is_dark);
                                        ui.add_space(4.0_f32);
                                        ui.label(
                                            egui::RichText::new(&opt.text).strong().size(13.5_f32),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(egui::RichText::new("➔").color(
                                                    if is_dark {
                                                        Color32::from_rgb(80, 150, 255)
                                                    } else {
                                                        Color32::from_rgb(37, 99, 235)
                                                    },
                                                ));
                                            },
                                        );
                                    });
                                });

                                let clicked =
                                    response.response.interact(egui::Sense::click()).clicked();
                                if clicked || hotkey_triggered {
                                    self.navigate_runner(opt.next_step_id);
                                }
                                ui.add_space(4.0_f32);
                            }
                        }
                    }

                    StepKind::Subprocess {
                        sub_algo,
                        next_if_success: _,
                        next_if_failure: _,
                    } => {
                        egui::Frame::none()
                            .fill(if is_dark {
                                Color32::from_rgb(38, 26, 54)
                            } else {
                                Color32::from_rgb(250, 245, 255)
                            })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if is_dark {
                                    Color32::from_rgb(90, 50, 130)
                                } else {
                                    Color32::from_rgb(192, 132, 252)
                                },
                            ))
                            .rounding(4.0_f32)
                            .inner_margin(10.0_f32)
                            .show(ui, |ui| {
                                ui.colored_label(
                                    if is_dark {
                                        Color32::from_rgb(216, 180, 254)
                                    } else {
                                        Color32::from_rgb(147, 51, 234)
                                    },
                                    lang.badge_subprocess(),
                                );
                                ui.label(lang.subprocess_steps_count(sub_algo.steps.len()));
                            });

                        ui.add_space(8.0_f32);
                        let enter_btn = egui::Button::new(
                            egui::RichText::new(lang.btn_enter_subprocess())
                                .color(Color32::WHITE)
                                .strong(),
                        )
                        .fill(Color32::from_rgb(147, 51, 234))
                        .min_size(Vec2::new(ui.available_width(), 38.0_f32));

                        if ui.add(enter_btn).clicked() {
                            self.navigate_runner(Some(step.id));
                        }
                    }

                    StepKind::Measurement {
                        unit,
                        min_val,
                        max_val,
                        next_if_normal,
                        next_if_abnormal,
                    } => {
                        ui.label(
                            egui::RichText::new(lang.measurement_prompt(*min_val, *max_val, unit))
                                .strong(),
                        );
                        ui.add_space(4.0_f32);
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.runner.measurement_input);
                            ui.label(unit);
                        });

                        if let Ok(val) = self.runner.measurement_input.trim().parse::<f64>() {
                            ui.add_space(6.0_f32);
                            let in_range = val >= *min_val && val <= *max_val;
                            if in_range {
                                ui.colored_label(
                                    if is_dark {
                                        Color32::from_rgb(60, 220, 130)
                                    } else {
                                        Color32::from_rgb(16, 185, 129)
                                    },
                                    format!("✔ {} {} {}", val, unit, lang.in_range_msg()),
                                );
                                ui.add_space(6.0_f32);

                                let accept_btn = egui::Button::new(
                                    egui::RichText::new(lang.accept_normal())
                                        .color(Color32::WHITE)
                                        .strong(),
                                )
                                .fill(Color32::from_rgb(16, 185, 129))
                                .min_size(Vec2::new(ui.available_width(), 38.0_f32));

                                if ui.add(accept_btn).clicked()
                                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    self.navigate_runner(*next_if_normal);
                                }
                            } else {
                                ui.colored_label(
                                    Color32::from_rgb(220, 38, 38),
                                    lang.abnormal_err(*min_val, *max_val, unit),
                                );
                                ui.add_space(6.0_f32);

                                let reject_btn = egui::Button::new(
                                    egui::RichText::new(lang.accept_abnormal())
                                        .color(Color32::WHITE)
                                        .strong(),
                                )
                                .fill(Color32::from_rgb(220, 38, 38))
                                .min_size(Vec2::new(ui.available_width(), 38.0_f32));

                                if ui.add(reject_btn).clicked()
                                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    self.runner.had_abnormality = true;
                                    self.navigate_runner(*next_if_abnormal);
                                }
                            }
                        }
                    }

                    StepKind::SafetyWarning {
                        ack_text,
                        next_step_id,
                    } => {
                        egui::Frame::none()
                            .fill(if is_dark {
                                Color32::from_rgb(50, 35, 12)
                            } else {
                                Color32::from_rgb(254, 243, 199)
                            })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if is_dark {
                                    Color32::from_rgb(80, 60, 20)
                                } else {
                                    Color32::from_rgb(245, 158, 11)
                                },
                            ))
                            .rounding(4.0_f32)
                            .inner_margin(8.0_f32)
                            .show(ui, |ui| {
                                ui.colored_label(
                                    if is_dark {
                                        Color32::from_rgb(255, 185, 45)
                                    } else {
                                        Color32::from_rgb(180, 83, 9)
                                    },
                                    lang.safety_title(),
                                );
                                ui.checkbox(&mut self.runner.safety_acknowledged, ack_text);
                            });

                        ui.add_space(8.0_f32);
                        ui.add_enabled_ui(self.runner.safety_acknowledged, |ui| {
                            let proceed_btn = egui::Button::new(
                                egui::RichText::new(lang.confirm_and_proceed())
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(217, 119, 6))
                            .min_size(Vec2::new(ui.available_width(), 38.0_f32));

                            if ui.add(proceed_btn).clicked() {
                                self.navigate_runner(*next_step_id);
                            }
                        });
                    }
                }
            });
        }
    }
}
