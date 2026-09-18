use crate::app::AlgoApp;
use crate::settings::AppTheme;
use crate::storage::{delete_from_archive, save_algorithm_to_archive};
use eframe::egui::{self, Color32, Stroke, Vec2};

impl AlgoApp {
    pub fn render_modals(&mut self, ctx: &egui::Context) {
        let lang = self.settings.language;
        let is_dark = self.settings.theme == AppTheme::Dark;

        // 1. Подтверждение закрытия приложения при наличии несохраненных данных
        if self.show_close_modal {
            egui::Window::new(lang.confirm_close_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0_f32, 0.0_f32])
                .show(ctx, |ui| {
                    ui.set_width(360.0_f32);
                    ui.add_space(4.0_f32);
                    ui.label(lang.confirm_close_msg());
                    ui.add_space(12.0_f32);
                    ui.separator();
                    ui.add_space(6.0_f32);
                    ui.horizontal(|ui| {
                        if ui.button(lang.btn_save_and_exit()).clicked() {
                            let mut root = self.sync_root_algorithm();
                            if save_algorithm_to_archive(&mut root).is_ok() {
                                self.force_close = true;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                        if ui.button(lang.btn_discard_and_exit()).clicked() {
                            self.force_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button(lang.btn_cancel()).clicked() {
                            self.show_close_modal = false;
                        }
                    });
                });
        }

        // 2. Паспорт оборудования (метаданные станка и технологической карты)
        if self.show_passport_modal {
            let modal_bg = if is_dark {
                Color32::from_rgb(24, 29, 39)
            } else {
                Color32::from_rgb(255, 255, 255)
            };
            let section_bg = if is_dark {
                Color32::from_rgb(18, 22, 30)
            } else {
                Color32::from_rgb(244, 247, 252)
            };
            let border_c = if is_dark {
                Color32::from_rgb(46, 56, 76)
            } else {
                Color32::from_rgb(203, 213, 225)
            };

            egui::Window::new(lang.passport_window_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0_f32, 0.0_f32])
                .fixed_size(Vec2::new(510.0_f32, 0.0_f32))
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(modal_bg)
                        .stroke(Stroke::new(1.0_f32, border_c))
                        .rounding(8.0_f32)
                        .inner_margin(16.0_f32),
                )
                .show(ctx, |ui| {
                    ui.set_width(478.0_f32);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("📋").size(24.0_f32));
                        ui.add_space(4.0_f32);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(lang.passport_window_title())
                                    .strong()
                                    .size(15.0_f32),
                            );
                            ui.label(
                                egui::RichText::new(lang.passport_desc())
                                    .size(11.5_f32)
                                    .color(if is_dark {
                                        Color32::from_rgb(140, 150, 168)
                                    } else {
                                        Color32::from_rgb(100, 116, 139)
                                    }),
                            );
                        });
                    });

                    ui.add_space(8.0_f32);
                    ui.separator();
                    ui.add_space(6.0_f32);

                    let card_frame = egui::Frame::none()
                        .fill(section_bg)
                        .stroke(Stroke::new(1.0_f32, border_c))
                        .rounding(6.0_f32)
                        .inner_margin(12.0_f32);

                    // Секция данных оборудования
                    card_frame.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(lang.passport_sec_equipment())
                                .strong()
                                .size(12.5_f32)
                                .color(if is_dark {
                                    Color32::from_rgb(80, 150, 255)
                                } else {
                                    Color32::from_rgb(29, 78, 216)
                                }),
                        );
                        ui.add_space(8.0_f32);

                        ui.columns(2, |cols| {
                            cols[0].label(
                                egui::RichText::new(lang.field_manufacturer()).size(11.5_f32),
                            );
                            cols[0].add(
                                egui::TextEdit::singleline(
                                    &mut self.active_algo.metadata.manufacturer,
                                )
                                .hint_text(lang.hint_manufacturer())
                                .desired_width(f32::INFINITY),
                            );

                            cols[1].label(egui::RichText::new(lang.field_model()).size(11.5_f32));
                            cols[1].add(
                                egui::TextEdit::singleline(&mut self.active_algo.metadata.model)
                                    .hint_text(lang.hint_model())
                                    .desired_width(f32::INFINITY),
                            );
                        });

                        ui.add_space(8.0_f32);

                        ui.columns(2, |cols| {
                            cols[0].label(egui::RichText::new(lang.field_type()).size(11.5_f32));
                            cols[0].add(
                                egui::TextEdit::singleline(&mut self.active_algo.metadata.eq_type)
                                    .hint_text(lang.hint_type())
                                    .desired_width(f32::INFINITY),
                            );

                            cols[1].label(egui::RichText::new(lang.field_inv()).size(11.5_f32));
                            cols[1].add(
                                egui::TextEdit::singleline(
                                    &mut self.active_algo.metadata.inv_number,
                                )
                                .hint_text(lang.hint_inv())
                                .desired_width(f32::INFINITY),
                            );
                        });
                    });

                    ui.add_space(6.0_f32);

                    // Секция параметров регламента
                    card_frame.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(lang.passport_sec_doc())
                                .strong()
                                .size(12.5_f32)
                                .color(if is_dark {
                                    Color32::from_rgb(80, 150, 255)
                                } else {
                                    Color32::from_rgb(29, 78, 216)
                                }),
                        );
                        ui.add_space(8.0_f32);

                        ui.label(egui::RichText::new(lang.field_name()).size(11.5_f32));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.active_algo.metadata.name)
                                .hint_text(lang.hint_name())
                                .desired_width(f32::INFINITY),
                        );

                        ui.add_space(8.0_f32);

                        ui.columns(2, |cols| {
                            cols[0].label(egui::RichText::new(lang.field_author()).size(11.5_f32));
                            cols[0].add(
                                egui::TextEdit::singleline(&mut self.active_algo.metadata.author)
                                    .hint_text(lang.hint_author())
                                    .desired_width(f32::INFINITY),
                            );

                            cols[1].label(
                                egui::RichText::new(lang.field_updated_date()).size(11.5_f32),
                            );
                            let mut date_str = self
                                .active_algo
                                .metadata
                                .updated_at
                                .format("%d.%m.%Y %H:%M")
                                .to_string();
                            cols[1].add_enabled(
                                false,
                                egui::TextEdit::singleline(&mut date_str)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    });

                    ui.add_space(10.0_f32);
                    ui.separator();
                    ui.add_space(6.0_f32);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let apply_btn = egui::Button::new(
                            egui::RichText::new(lang.btn_apply())
                                .color(Color32::WHITE)
                                .strong(),
                        )
                        .fill(Color32::from_rgb(37, 99, 235))
                        .rounding(6.0_f32)
                        .min_size(Vec2::new(110.0_f32, 32.0_f32));

                        if ui.add(apply_btn).clicked() {
                            self.show_passport_modal = false;
                        }

                        if ui.button(lang.btn_cancel()).clicked() {
                            self.show_passport_modal = false;
                        }
                    });
                });
        }

        // 3. Подтверждение удаления регламента из базы
        if let Some((del_id, del_name)) = self.confirm_delete_target.clone() {
            let mut close_dialog = false;
            egui::Window::new(lang.confirm_delete_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0_f32, 0.0_f32])
                .show(ctx, |ui| {
                    ui.set_width(340.0_f32);
                    ui.add_space(4.0_f32);
                    ui.label(lang.confirm_delete_msg());
                    ui.colored_label(Color32::from_rgb(255, 205, 50), format!("«{}»", del_name));
                    ui.add_space(10.0_f32);
                    ui.separator();
                    ui.add_space(6.0_f32);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new(lang.btn_delete_archive())
                                    .color(Color32::from_rgb(255, 90, 90)),
                            )
                            .clicked()
                        {
                            delete_from_archive(del_id);
                            self.refresh_archive();
                            close_dialog = true;
                        }
                        if ui.button(lang.btn_cancel()).clicked() {
                            close_dialog = true;
                        }
                    });
                });

            if close_dialog {
                self.confirm_delete_target = None;
            }
        }
    }
}
