pub mod connectors;
pub mod wires;

use crate::app::{AlgoApp, ContextMenuState};
use crate::canvas::connectors::*;
use crate::canvas::wires::*;
use crate::models::{Step, StepKind, WireStyle};
use crate::settings::{AppTheme, GRID_SNAP_STEP};
use eframe::egui::{self, Color32, FontId, PointerButton, Pos2, Rect, Stroke, Vec2};
use uuid::Uuid;

impl AlgoApp {
    pub fn render_canvas(&mut self, ui: &mut egui::Ui, is_test_mode: bool) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            egui::Sense::click_and_drag(),
        );
        let canvas_rect = response.rect;
        self.last_canvas_rect = Some(canvas_rect);

        let lang = self.settings.language;
        let is_dark = self.settings.theme == AppTheme::Dark;

        let canvas_bg = if is_dark {
            Color32::from_rgb(16, 19, 24)
        } else {
            Color32::from_rgb(234, 239, 246)
        };
        painter.rect_filled(canvas_rect, 0.0_f32, canvas_bg);

        let pointer = ui.input(|i| i.pointer.clone());
        let is_in_canvas = pointer
            .hover_pos()
            .map_or(false, |p| canvas_rect.contains(p));

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.context_menu = None;
            self.context_menu_rect = None;
            self.wire_drag_source_id = None;
            self.selection_box_start = None;
            self.show_search_bar = false;
        }

        let clicked_in_menu = self.context_menu_rect.map_or(false, |r| {
            pointer.latest_pos().map_or(false, |pos| r.contains(pos))
        });

        let minimap_size = Vec2::new(180.0_f32, 115.0_f32);
        let minimap_rect = Rect::from_min_size(
            canvas_rect.max - minimap_size - Vec2::new(12.0_f32, 12.0_f32),
            minimap_size,
        );
        let clicked_in_minimap = pointer
            .latest_pos()
            .map_or(false, |pos| minimap_rect.contains(pos));

        // Масштабирование колесиком мыши
        let scroll_delta_y = ui.input(|i| i.raw_scroll_delta.y);
        if is_in_canvas
            && scroll_delta_y.abs() > 0.1_f32
            && !clicked_in_menu
            && !clicked_in_minimap
            && !self.show_close_modal
        {
            self.target_pan = None;
            if let Some(mouse_pos) = pointer.hover_pos() {
                self.zoom_by_step(scroll_delta_y > 0.0_f32, mouse_pos.to_vec2());
                ui.ctx().request_repaint();
            }
        }

        // Панорамирование холста СКМ
        if pointer.middle_down() && is_in_canvas && !self.show_close_modal {
            self.target_pan = None;
            let delta = pointer.delta();
            if delta.length_sq() > 0.0_f32 {
                self.canvas_pan += delta;
                ui.ctx().request_repaint();
            }
            ui.ctx().set_cursor_icon(egui::CursorIcon::AllScroll);
        }

        // Точечная сетка
        let grid_size = 36.0_f32 * self.canvas_zoom;
        let (dot_color, dot_radius) = if is_dark {
            (Color32::from_rgba_unmultiplied(80, 100, 130, 80), 1.2_f32)
        } else {
            (Color32::from_rgba_unmultiplied(51, 65, 85, 140), 1.35_f32)
        };

        let min_kx = ((canvas_rect.min.x - self.canvas_pan.x) / grid_size).floor() as i32;
        let max_kx = ((canvas_rect.max.x - self.canvas_pan.x) / grid_size).ceil() as i32;
        let min_ky = ((canvas_rect.min.y - self.canvas_pan.y) / grid_size).floor() as i32;
        let max_ky = ((canvas_rect.max.y - self.canvas_pan.y) / grid_size).ceil() as i32;

        for kx in min_kx..=max_kx {
            let x = (kx as f32) * grid_size + self.canvas_pan.x;
            for ky in min_ky..=max_ky {
                let y = (ky as f32) * grid_size + self.canvas_pan.y;
                painter.circle_filled(Pos2::new(x, y), dot_radius, dot_color);
            }
        }

        // Экран пустого состояния (первый запуск)
        if self.active_algo.steps.is_empty() && !is_test_mode {
            let card_size = Vec2::new(420.0_f32, 220.0_f32);
            let card_rect = Rect::from_center_size(canvas_rect.center(), card_size);

            let empty_bg = if is_dark {
                Color32::from_rgba_unmultiplied(22, 27, 36, 245)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 250)
            };
            let empty_border = if is_dark {
                Color32::from_rgb(52, 64, 84)
            } else {
                Color32::from_rgb(180, 195, 215)
            };

            painter.rect_filled(card_rect, 10.0_f32, empty_bg);
            painter.rect_stroke(card_rect, 10.0_f32, Stroke::new(1.5_f32, empty_border));

            ui.allocate_ui_at_rect(card_rect, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(22.0_f32);
                    ui.label(egui::RichText::new("🕸").size(34.0_f32));
                    ui.add_space(4.0_f32);
                    ui.label(
                        egui::RichText::new(lang.empty_canvas_title())
                            .strong()
                            .size(17.0_f32)
                            .color(if is_dark {
                                Color32::from_rgb(240, 245, 255)
                            } else {
                                Color32::from_rgb(15, 23, 42)
                            }),
                    );
                    ui.add_space(4.0_f32);
                    ui.label(
                        egui::RichText::new(lang.empty_canvas_hint())
                            .size(12.5_f32)
                            .color(if is_dark {
                                Color32::from_rgb(140, 150, 168)
                            } else {
                                Color32::from_rgb(100, 116, 139)
                            }),
                    );
                    ui.add_space(16.0_f32);

                    let big_add_btn = egui::Button::new(
                        egui::RichText::new(lang.empty_canvas_btn())
                            .color(Color32::WHITE)
                            .strong()
                            .size(13.5_f32),
                    )
                    .fill(Color32::from_rgb(37, 99, 235))
                    .rounding(6.0_f32)
                    .min_size(Vec2::new(220.0_f32, 36.0_f32));

                    if ui.add(big_add_btn).clicked() {
                        self.snapshot_for_undo();
                        let mut world_x = (-self.canvas_pan.x + canvas_rect.center().x - 105.0_f32)
                            / self.canvas_zoom;
                        let mut world_y = (-self.canvas_pan.y + canvas_rect.center().y - 50.0_f32)
                            / self.canvas_zoom;
                        if self.settings.snap_to_grid {
                            world_x = (world_x / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                            world_y = (world_y / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                        }

                        let title = lang.step_default_name(1);
                        let step = Step::new(title, [world_x, world_y]);
                        let id = step.id;
                        self.active_algo.first_step_id = Some(id);
                        self.active_algo.steps.push(step);
                        self.selected_step_ids.clear();
                        self.selected_step_ids.insert(id);
                    }
                });
            });
        }

        let node_size = Vec2::new(210.0_f32, 100.0_f32) * self.canvas_zoom;

        let mut node_rects: Vec<(Uuid, Rect)> = Vec::new();
        for step in &self.active_algo.steps {
            let screen_pos = Pos2::new(
                step.pos[0] * self.canvas_zoom + self.canvas_pan.x,
                step.pos[1] * self.canvas_zoom + self.canvas_pan.y,
            );
            node_rects.push((step.id, Rect::from_min_size(screen_pos, node_size)));
        }

        // Двойной клик ЛКМ — создать блок
        if !is_test_mode
            && !self.show_close_modal
            && is_in_canvas
            && !clicked_in_menu
            && !clicked_in_minimap
        {
            if pointer.button_double_clicked(PointerButton::Primary) {
                if let Some(pos) = pointer.latest_pos() {
                    let hit_any = node_rects.iter().any(|(_, r)| r.contains(pos));
                    if !hit_any {
                        self.snapshot_for_undo();
                        let count = self.active_algo.steps.len() + 1;
                        let mut world_x = (pos.x - self.canvas_pan.x) / self.canvas_zoom;
                        let mut world_y = (pos.y - self.canvas_pan.y) / self.canvas_zoom;
                        if self.settings.snap_to_grid {
                            world_x = (world_x / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                            world_y = (world_y / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                        }

                        let title = lang.step_default_name(count);
                        let step = Step::new(title, [world_x, world_y]);
                        let id = step.id;
                        if self.active_algo.first_step_id.is_none() {
                            self.active_algo.first_step_id = Some(id);
                        }
                        self.active_algo.steps.push(step);
                        self.selected_step_ids.clear();
                        self.selected_step_ids.insert(id);
                    }
                }
            }
        }

        // Перетаскивание и выделение блоков
        if !is_test_mode && !self.show_close_modal {
            if pointer.primary_pressed() && is_in_canvas && !clicked_in_menu && !clicked_in_minimap
            {
                self.context_menu = None;
                self.context_menu_rect = None;

                if let Some(pos) = pointer.latest_pos() {
                    let mut started_wire = false;
                    for (id, r) in &node_rects {
                        let port_out = Pos2::new(r.center().x, r.max.y);
                        if pos.distance(port_out) <= 12.0_f32 * self.canvas_zoom {
                            self.wire_drag_source_id = Some(*id);
                            started_wire = true;
                            break;
                        }
                    }

                    if !started_wire {
                        let hit_step = self.active_algo.steps.iter().rev().find(|step| {
                            let screen_pos = Pos2::new(
                                step.pos[0] * self.canvas_zoom + self.canvas_pan.x,
                                step.pos[1] * self.canvas_zoom + self.canvas_pan.y,
                            );
                            Rect::from_min_size(screen_pos, node_size).contains(pos)
                        });

                        let shift_down = ui.input(|i| i.modifiers.shift);

                        if let Some(step) = hit_step {
                            let step_id = step.id;
                            if shift_down {
                                if self.selected_step_ids.contains(&step_id) {
                                    self.selected_step_ids.remove(&step_id);
                                } else {
                                    self.selected_step_ids.insert(step_id);
                                }
                            } else if !self.selected_step_ids.contains(&step_id) {
                                self.selected_step_ids.clear();
                                self.selected_step_ids.insert(step_id);
                            }

                            self.is_dragging_nodes = true;
                            self.drag_start_mouse = pos;
                            self.dragged_any_distance = false;
                            self.drag_start_positions.clear();
                            for s in &self.active_algo.steps {
                                if self.selected_step_ids.contains(&s.id) {
                                    self.drag_start_positions.insert(s.id, s.pos);
                                }
                            }
                        } else {
                            if !shift_down {
                                self.selected_step_ids.clear();
                            }
                            self.selection_box_start = Some(pos);
                        }
                    }
                }
            }

            if pointer.primary_down() && !clicked_in_menu && !clicked_in_minimap {
                if self.is_dragging_nodes {
                    if let Some(cur_mouse) = pointer.latest_pos() {
                        let total_mouse_delta = cur_mouse - self.drag_start_mouse;
                        if total_mouse_delta.length_sq() > 4.0_f32 {
                            if !self.dragged_any_distance {
                                self.snapshot_for_undo();
                                self.dragged_any_distance = true;
                            }
                            for step in &mut self.active_algo.steps {
                                if let Some(init_pos) = self.drag_start_positions.get(&step.id) {
                                    let mut new_x =
                                        init_pos[0] + (total_mouse_delta.x / self.canvas_zoom);
                                    let mut new_y =
                                        init_pos[1] + (total_mouse_delta.y / self.canvas_zoom);

                                    if self.settings.snap_to_grid {
                                        new_x = (new_x / (GRID_SNAP_STEP * 0.5_f32)).round()
                                            * (GRID_SNAP_STEP * 0.5_f32);
                                        new_y = (new_y / (GRID_SNAP_STEP * 0.5_f32)).round()
                                            * (GRID_SNAP_STEP * 0.5_f32);
                                    }

                                    step.pos[0] = new_x;
                                    step.pos[1] = new_y;
                                }
                            }
                            ui.ctx().request_repaint();
                        }
                    }
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }
            } else {
                if self.is_dragging_nodes && self.settings.snap_to_grid {
                    for step in &mut self.active_algo.steps {
                        if self.selected_step_ids.contains(&step.id) {
                            step.pos[0] = (step.pos[0] / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                            step.pos[1] = (step.pos[1] / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                        }
                    }
                }

                self.is_dragging_nodes = false;
                self.drag_start_positions.clear();

                if let (Some(start_pt), Some(cur_pos)) =
                    (self.selection_box_start, pointer.latest_pos())
                {
                    let box_rect = Rect::from_two_pos(start_pt, cur_pos);
                    if box_rect.size().length_sq() > 16.0_f32 {
                        for (id, r) in &node_rects {
                            if box_rect.intersects(*r) {
                                self.selected_step_ids.insert(*id);
                            }
                        }
                    }
                }
                self.selection_box_start = None;

                if let Some(source_id) = self.wire_drag_source_id {
                    if let Some(rel_pos) = pointer.latest_pos() {
                        if let Some((target_id, _)) =
                            node_rects.iter().find(|(_, r)| r.contains(rel_pos))
                        {
                            self.connect_nodes(source_id, *target_id);
                        }
                    }
                }
                self.wire_drag_source_id = None;
            }
        }

        let mouse_clicked = pointer.primary_clicked()
            && is_in_canvas
            && !clicked_in_menu
            && !clicked_in_minimap
            && !self.show_close_modal;

        // В РЕЖИМЕ ДИАГНОСТИКИ ПКМ ПОЛНОСТЬЮ БЛОКИРУЕТСЯ
        let right_clicked = !is_test_mode
            && pointer.secondary_clicked()
            && is_in_canvas
            && !clicked_in_minimap
            && !self.show_close_modal;

        let click_pos = pointer.latest_pos();
        let mut link_to_disconnect: Option<(Uuid, usize, u8)> = None;
        let mut wire_to_toggle: Option<(String, WireStyle)> = None;
        let mut connector_jump_target: Option<Uuid> = None;

        let norm_text = lang.link_normal();
        let abn_text = lang.link_abnormal();
        let safe_text = lang.link_safety();

        // 1. СОЕДИНИТЕЛИ ГОСТ (ОБРАТНЫЕ СВЯЗИ A, B, C...)
        let target_to_letter = build_connector_letter_map(&self.active_algo.steps, &node_rects);
        draw_incoming_connectors(
            &painter,
            ui,
            &node_rects,
            &target_to_letter,
            &mut self.hovered_connector,
            self.canvas_zoom,
            is_dark,
        );

        // 2. ОТРИСОВКА СВЯЗЕЙ (ПРЯМЫЕ И ВЫХОДНЫЕ СОЕДИНИТЕЛИ)
        for step in &self.active_algo.steps {
            let from_rect = match node_rects.iter().find(|(id, _)| *id == step.id) {
                Some((_, r)) => *r,
                None => continue,
            };
            let start_pt = Pos2::new(from_rect.center().x, from_rect.max.y);

            let mut connections = Vec::new();
            match &step.kind {
                StepKind::Standard => {
                    for (idx, opt) in step.options.iter().enumerate() {
                        if let Some(target_id) = opt.next_step_id {
                            let link_c = if is_dark {
                                Color32::from_rgb(120, 160, 255)
                            } else {
                                Color32::from_rgb(37, 99, 235)
                            };
                            connections.push((target_id, opt.text.clone(), link_c, idx, 0));
                        }
                    }
                }
                StepKind::Measurement {
                    next_if_normal,
                    next_if_abnormal,
                    ..
                } => {
                    if let Some(nid) = next_if_normal {
                        let norm_c = if is_dark {
                            Color32::from_rgb(60, 210, 130)
                        } else {
                            Color32::from_rgb(16, 185, 129)
                        };
                        connections.push((*nid, norm_text.to_string(), norm_c, 0, 1));
                    }
                    if let Some(aid) = next_if_abnormal {
                        let abn_c = if is_dark {
                            Color32::from_rgb(255, 90, 90)
                        } else {
                            Color32::from_rgb(220, 38, 38)
                        };
                        connections.push((*aid, abn_text.to_string(), abn_c, 0, 2));
                    }
                }
                StepKind::SafetyWarning { next_step_id, .. } => {
                    if let Some(nid) = next_step_id {
                        let safe_c = if is_dark {
                            Color32::from_rgb(255, 175, 45)
                        } else {
                            Color32::from_rgb(217, 119, 6)
                        };
                        connections.push((*nid, safe_text.to_string(), safe_c, 0, 3));
                    }
                }
            }

            let has_selection = !self.selected_step_ids.is_empty();

            for (target_id, label, color, conn_idx, conn_type) in connections {
                if let Some((_, to_rect)) = node_rects.iter().find(|(id, _)| *id == target_id) {
                    let is_backward = to_rect.center().y < from_rect.center().y - 10.0_f32;

                    let is_outgoing = !is_test_mode && self.selected_step_ids.contains(&step.id);
                    let is_incoming = !is_test_mode && self.selected_step_ids.contains(&target_id);
                    let is_highlighted = is_outgoing || is_incoming;

                    let line_color = if is_test_mode {
                        let is_traversed = self
                            .runner
                            .history
                            .windows(2)
                            .any(|w| w[0] == step.id && w[1] == target_id)
                            || (self.runner.history.last() == Some(&step.id)
                                && self.runner.current_step_id == Some(target_id));
                        let is_current_outgoing = self.runner.current_step_id == Some(step.id);

                        if is_traversed {
                            if is_dark {
                                Color32::from_rgb(60, 225, 110)
                            } else {
                                Color32::from_rgb(16, 185, 129)
                            }
                        } else if is_current_outgoing {
                            if is_dark {
                                Color32::from_rgb(255, 205, 50)
                            } else {
                                Color32::from_rgb(217, 119, 6)
                            }
                        } else if is_dark {
                            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 40)
                        } else {
                            Color32::from_rgba_unmultiplied(148, 163, 184, 80)
                        }
                    } else if is_outgoing {
                        if is_dark {
                            Color32::from_rgb(255, 205, 50)
                        } else {
                            Color32::from_rgb(217, 119, 6)
                        }
                    } else if is_incoming {
                        if is_dark {
                            Color32::from_rgb(56, 189, 248)
                        } else {
                            Color32::from_rgb(2, 132, 199)
                        }
                    } else if has_selection {
                        if is_dark {
                            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 35)
                        } else {
                            Color32::from_rgba_unmultiplied(148, 163, 184, 90)
                        }
                    } else {
                        color
                    };

                    if is_backward {
                        // Выходной коннектор-кружок A снизу
                        let letter = target_to_letter
                            .get(&target_id)
                            .cloned()
                            .unwrap_or_else(|| "A".to_string());
                        let target_title = self
                            .active_algo
                            .steps
                            .iter()
                            .find(|s| s.id == target_id)
                            .map_or("", |s| s.title.as_str());

                        let (badge_rect_out, jump) = draw_outgoing_connector(
                            &painter,
                            ui,
                            start_pt,
                            &letter,
                            &label,
                            target_title,
                            line_color,
                            &mut self.hovered_connector,
                            self.canvas_zoom,
                            is_dark,
                            is_test_mode,
                        );

                        if jump {
                            connector_jump_target = Some(target_id);
                        }

                        if !is_test_mode && mouse_clicked && self.wire_drag_source_id.is_none() {
                            if let (Some(b_rect), Some(pos)) = (badge_rect_out, click_pos) {
                                if b_rect.contains(pos) {
                                    link_to_disconnect = Some((step.id, conn_idx, conn_type));
                                }
                            }
                        }
                    } else {
                        // Обычная прямая связь сверху вниз
                        let end_pt = Pos2::new(to_rect.center().x, to_rect.min.y);
                        let conn_key = format!("{}:{}", step.id, target_id);
                        let wire_style = self
                            .active_algo
                            .wire_styles
                            .get(&conn_key)
                            .copied()
                            .unwrap_or(self.settings.default_wire_style);

                        let (badge_rect, segments) = draw_connection_wire(
                            &painter,
                            start_pt,
                            end_pt,
                            &label,
                            line_color,
                            self.canvas_zoom,
                            is_dark,
                            is_highlighted,
                            wire_style,
                        );

                        if !is_test_mode && mouse_clicked && self.wire_drag_source_id.is_none() {
                            if let (Some(b_rect), Some(pos)) = (badge_rect, click_pos) {
                                if b_rect.contains(pos) {
                                    link_to_disconnect = Some((step.id, conn_idx, conn_type));
                                }
                            }
                        }

                        if !is_test_mode && right_clicked && wire_to_toggle.is_none() {
                            if let Some(pos) = click_pos {
                                let hit_badge =
                                    badge_rect.map_or(false, |r| r.expand(5.0_f32).contains(pos));
                                let hit_line = segments.iter().any(|seg| {
                                    dist_to_segment(pos, seg[0], seg[1])
                                        <= 6.0_f32 * self.canvas_zoom
                                });

                                if hit_badge || hit_line {
                                    wire_to_toggle = Some((conn_key, wire_style));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(target_id) = connector_jump_target {
            self.focus_step_on_canvas(target_id, canvas_rect.size());
        }

        if pointer.hover_pos().is_none() {
            self.hovered_connector = None;
        }

        // Временный провод при вытягивании связи
        if let Some(source_id) = self.wire_drag_source_id {
            if let Some((_, r)) = node_rects.iter().find(|(id, _)| *id == source_id) {
                if let Some(mouse_pos) = pointer.latest_pos() {
                    let start_pt = Pos2::new(r.center().x, r.max.y);
                    let wire_c = if is_dark {
                        Color32::from_rgb(255, 205, 50)
                    } else {
                        Color32::from_rgb(217, 119, 6)
                    };
                    draw_connection_wire(
                        &painter,
                        start_pt,
                        mouse_pos,
                        "",
                        wire_c,
                        self.canvas_zoom,
                        is_dark,
                        true,
                        self.settings.default_wire_style,
                    );
                }
            }
        }

        // Разрыв связи (ЛКМ)
        if let Some((step_id, idx, conn_type)) = link_to_disconnect {
            self.snapshot_for_undo();
            if let Some(step) = self.active_algo.steps.iter_mut().find(|s| s.id == step_id) {
                match conn_type {
                    0 => {
                        if let Some(opt) = step.options.get_mut(idx) {
                            opt.next_step_id = None;
                        }
                    }
                    1 => {
                        if let StepKind::Measurement { next_if_normal, .. } = &mut step.kind {
                            *next_if_normal = None;
                        }
                    }
                    2 => {
                        if let StepKind::Measurement {
                            next_if_abnormal, ..
                        } = &mut step.kind
                        {
                            *next_if_abnormal = None;
                        }
                    }
                    3 => {
                        if let StepKind::SafetyWarning { next_step_id, .. } = &mut step.kind {
                            *next_step_id = None;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Переключение стиля связи (ПКМ) или вызов контекстного меню
        if !is_test_mode {
            if let Some((conn_key, cur_style)) = wire_to_toggle {
                self.snapshot_for_undo();
                let new_style = match cur_style {
                    WireStyle::Curved => WireStyle::Orthogonal,
                    WireStyle::Orthogonal => WireStyle::Curved,
                };
                self.active_algo.wire_styles.insert(conn_key, new_style);
                let msg = match new_style {
                    WireStyle::Curved => lang.status_wire_style_curved(),
                    WireStyle::Orthogonal => lang.status_wire_style_ortho(),
                };
                self.set_status(msg.to_string());
                ui.ctx().request_repaint();
            } else if right_clicked {
                if let Some(pos) = click_pos {
                    let hit_step = self.active_algo.steps.iter().rev().find(|step| {
                        let screen_pos = Pos2::new(
                            step.pos[0] * self.canvas_zoom + self.canvas_pan.x,
                            step.pos[1] * self.canvas_zoom + self.canvas_pan.y,
                        );
                        Rect::from_min_size(screen_pos, node_size).contains(pos)
                    });

                    self.context_menu = Some(ContextMenuState {
                        pos,
                        node_id: hit_step.map(|s| s.id),
                    });
                    self.context_menu_rect = None;

                    if let Some(step) = hit_step {
                        if !self.selected_step_ids.contains(&step.id) {
                            self.selected_step_ids.clear();
                            self.selected_step_ids.insert(step.id);
                        }
                    }
                }
            }
        }

        // Отрисовка карточек шагов
        for (id, rect) in &node_rects {
            let step = match self.active_algo.steps.iter().find(|s| s.id == *id) {
                Some(s) => s,
                None => continue,
            };

            let is_start = self.active_algo.first_step_id == Some(*id);
            let is_current_run = is_test_mode && self.runner.current_step_id == Some(*id);
            let is_passed_run = is_test_mode && self.runner.history.contains(id);
            let is_selected = !is_test_mode && self.selected_step_ids.contains(id);
            let is_search_match = !is_test_mode && self.search_results.contains(id);
            let is_current_search =
                is_search_match && self.search_results.get(self.current_search_idx) == Some(id);

            let accent_color = if is_current_run {
                if is_dark {
                    Color32::from_rgb(255, 195, 0)
                } else {
                    Color32::from_rgb(217, 119, 6)
                }
            } else if is_passed_run {
                if is_dark {
                    Color32::from_rgb(60, 210, 130)
                } else {
                    Color32::from_rgb(16, 185, 129)
                }
            } else {
                match &step.kind {
                    StepKind::Standard => {
                        if is_dark {
                            Color32::from_rgb(70, 140, 255)
                        } else {
                            Color32::from_rgb(37, 99, 235)
                        }
                    }
                    StepKind::Measurement { .. } => {
                        if is_dark {
                            Color32::from_rgb(45, 210, 180)
                        } else {
                            Color32::from_rgb(13, 148, 136)
                        }
                    }
                    StepKind::SafetyWarning { .. } => {
                        if is_dark {
                            Color32::from_rgb(255, 165, 40)
                        } else {
                            Color32::from_rgb(217, 119, 6)
                        }
                    }
                }
            };

            let card_bg = if is_test_mode {
                if is_current_run {
                    if is_dark {
                        Color32::from_rgb(32, 38, 48)
                    } else {
                        Color32::from_rgb(254, 252, 232)
                    }
                } else if is_passed_run {
                    if is_dark {
                        Color32::from_rgb(20, 29, 25)
                    } else {
                        Color32::from_rgb(240, 253, 244)
                    }
                } else if is_dark {
                    Color32::from_rgb(23, 27, 34)
                } else {
                    Color32::from_rgb(255, 255, 255)
                }
            } else if is_current_search {
                if is_dark {
                    Color32::from_rgb(42, 36, 22)
                } else {
                    Color32::from_rgb(254, 243, 199)
                }
            } else if is_search_match {
                if is_dark {
                    Color32::from_rgb(32, 30, 24)
                } else {
                    Color32::from_rgb(254, 249, 195)
                }
            } else if is_selected {
                if is_dark {
                    Color32::from_rgb(26, 32, 44)
                } else {
                    Color32::from_rgb(248, 250, 255)
                }
            } else if is_dark {
                Color32::from_rgb(23, 27, 34)
            } else {
                Color32::from_rgb(255, 255, 255)
            };

            let border_color = if is_current_run {
                if is_dark {
                    Color32::from_rgb(255, 195, 0)
                } else {
                    Color32::from_rgb(217, 119, 6)
                }
            } else if is_passed_run {
                if is_dark {
                    Color32::from_rgb(60, 210, 130)
                } else {
                    Color32::from_rgb(16, 185, 129)
                }
            } else if is_selected {
                Color32::from_rgb(37, 99, 235)
            } else if is_start {
                if is_dark {
                    Color32::from_rgb(60, 200, 120)
                } else {
                    Color32::from_rgb(16, 185, 129)
                }
            } else if is_dark {
                Color32::from_rgb(52, 60, 75)
            } else {
                Color32::from_rgb(190, 204, 222)
            };

            let border_w = if is_current_run {
                3.0_f32 * self.canvas_zoom
            } else if is_selected {
                2.2_f32 * self.canvas_zoom
            } else if is_start {
                1.8_f32 * self.canvas_zoom
            } else {
                1.0_f32 * self.canvas_zoom
            };

            let card_rounding = 4.0_f32 * self.canvas_zoom;
            painter.rect_filled(*rect, card_rounding, card_bg);
            painter.rect_stroke(*rect, card_rounding, Stroke::new(border_w, border_color));

            let stripe_w = 4.5_f32 * self.canvas_zoom;
            let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(stripe_w, rect.height()));
            painter.rect_filled(stripe_rect, card_rounding, accent_color);

            let divider_y = rect.min.y + 26.0_f32 * self.canvas_zoom;
            painter.line_segment(
                [
                    Pos2::new(rect.min.x + stripe_w, divider_y),
                    Pos2::new(rect.max.x, divider_y),
                ],
                Stroke::new(
                    1.0_f32,
                    if is_dark {
                        Color32::from_rgb(38, 44, 56)
                    } else {
                        Color32::from_rgb(226, 232, 240)
                    },
                ),
            );

            // Линтер целостности графа
            let mut lint_warning: Option<&'static str> = None;
            if !is_test_mode {
                let has_incoming = is_start
                    || self
                        .active_algo
                        .steps
                        .iter()
                        .any(|other| match &other.kind {
                            StepKind::Standard => other
                                .options
                                .iter()
                                .any(|o| o.next_step_id == Some(step.id)),
                            StepKind::Measurement {
                                next_if_normal,
                                next_if_abnormal,
                                ..
                            } => {
                                *next_if_normal == Some(step.id)
                                    || *next_if_abnormal == Some(step.id)
                            }
                            StepKind::SafetyWarning { next_step_id, .. } => {
                                *next_step_id == Some(step.id)
                            }
                        });

                if !has_incoming {
                    lint_warning = Some(lang.lint_unreachable());
                } else {
                    let has_unlinked_branch = match &step.kind {
                        StepKind::Standard => {
                            !step.options.is_empty()
                                && step.options.iter().any(|o| o.next_step_id.is_none())
                        }
                        StepKind::Measurement {
                            next_if_normal,
                            next_if_abnormal,
                            ..
                        } => next_if_normal.is_none() || next_if_abnormal.is_none(),
                        StepKind::SafetyWarning { next_step_id, .. } => next_step_id.is_none(),
                    };
                    if has_unlinked_branch {
                        lint_warning = Some(lang.lint_incomplete());
                    }
                }
            }

            let title_prefix = if is_start && !is_test_mode {
                "🚩 "
            } else {
                ""
            };
            painter.text(
                Pos2::new(
                    rect.min.x + 12.0_f32 * self.canvas_zoom,
                    rect.min.y + 13.0_f32 * self.canvas_zoom,
                ),
                egui::Align2::LEFT_CENTER,
                format!("{}{}", title_prefix, step.title),
                FontId::proportional(12.5_f32 * self.canvas_zoom),
                if is_dark {
                    Color32::from_rgb(240, 245, 255)
                } else {
                    Color32::from_rgb(15, 23, 42)
                },
            );

            let mut badge_right_offset = 8.0_f32 * self.canvas_zoom;
            if let Some(warning) = lint_warning {
                let lint_icon_pos = Pos2::new(
                    rect.max.x - badge_right_offset,
                    rect.min.y + 13.0_f32 * self.canvas_zoom,
                );
                let lint_rect =
                    Rect::from_center_size(lint_icon_pos, Vec2::splat(16.0_f32 * self.canvas_zoom));
                painter.text(
                    lint_icon_pos,
                    egui::Align2::RIGHT_CENTER,
                    "⚠",
                    FontId::proportional(12.5_f32 * self.canvas_zoom),
                    Color32::from_rgb(217, 119, 6),
                );

                if pointer.hover_pos().map_or(false, |p| lint_rect.contains(p)) {
                    egui::show_tooltip(
                        ui.ctx(),
                        egui::Id::new(format!("lint_{}", step.id)),
                        |ui| {
                            ui.label(warning);
                        },
                    );
                }
                badge_right_offset += 16.0_f32 * self.canvas_zoom;
            }

            let badge_text = if is_current_run {
                lang.badge_active()
            } else if is_passed_run {
                lang.badge_done()
            } else {
                match &step.kind {
                    StepKind::Standard => lang.badge_choice(),
                    StepKind::Measurement { .. } => lang.badge_measure(),
                    StepKind::SafetyWarning { .. } => lang.badge_safety(),
                }
            };

            painter.text(
                Pos2::new(
                    rect.max.x - badge_right_offset,
                    rect.min.y + 13.0_f32 * self.canvas_zoom,
                ),
                egui::Align2::RIGHT_CENTER,
                badge_text,
                FontId::proportional(10.5_f32 * self.canvas_zoom),
                accent_color,
            );

            let preview_text = if step.description.trim().is_empty() {
                lang.no_desc().to_string()
            } else {
                let line = step.description.lines().next().unwrap_or("");
                if line.chars().count() > 23 {
                    format!("{}...", line.chars().take(23).collect::<String>())
                } else {
                    line.to_string()
                }
            };

            painter.text(
                Pos2::new(
                    rect.min.x + 12.0_f32 * self.canvas_zoom,
                    rect.min.y + 35.0_f32 * self.canvas_zoom,
                ),
                egui::Align2::LEFT_TOP,
                preview_text,
                FontId::proportional(11.5_f32 * self.canvas_zoom),
                if is_dark {
                    Color32::from_rgb(160, 168, 180)
                } else {
                    Color32::from_rgb(71, 85, 105)
                },
            );

            if let StepKind::Measurement {
                unit,
                min_val,
                max_val,
                ..
            } = &step.kind
            {
                let metric_str = format!(
                    "{} {} ... {} {}",
                    lang.tolerance_prefix(),
                    min_val,
                    max_val,
                    unit
                );
                painter.text(
                    Pos2::new(
                        rect.min.x + 12.0_f32 * self.canvas_zoom,
                        rect.min.y + 55.0_f32 * self.canvas_zoom,
                    ),
                    egui::Align2::LEFT_TOP,
                    metric_str,
                    FontId::monospace(10.5_f32 * self.canvas_zoom),
                    accent_color,
                );
            }

            if step.image_path.is_some() {
                painter.text(
                    Pos2::new(
                        rect.min.x + 12.0_f32 * self.canvas_zoom,
                        rect.min.y + 74.0_f32 * self.canvas_zoom,
                    ),
                    egui::Align2::LEFT_TOP,
                    lang.photo_attached(),
                    FontId::proportional(10.5_f32 * self.canvas_zoom),
                    if is_dark {
                        Color32::from_rgb(100, 180, 255)
                    } else {
                        Color32::from_rgb(29, 78, 216)
                    },
                );
            }

            let port_in = Pos2::new(rect.center().x, rect.min.y);
            let port_out = Pos2::new(rect.center().x, rect.max.y);

            // Порт входа (IN)
            if is_selected {
                let glow_in = if is_dark {
                    Color32::from_rgb(56, 189, 248)
                } else {
                    Color32::from_rgb(2, 132, 199)
                };
                painter.circle_stroke(
                    port_in,
                    6.0_f32 * self.canvas_zoom,
                    Stroke::new(1.4_f32 * self.canvas_zoom, glow_in),
                );
                painter.circle_filled(port_in, 4.0_f32 * self.canvas_zoom, glow_in);
                painter.circle_stroke(
                    port_in,
                    4.0_f32 * self.canvas_zoom,
                    Stroke::new(1.0_f32 * self.canvas_zoom, Color32::WHITE),
                );
            } else {
                painter.circle_filled(
                    port_in,
                    3.2_f32 * self.canvas_zoom,
                    if is_dark {
                        Color32::from_rgb(120, 135, 160)
                    } else {
                        Color32::from_rgb(148, 163, 184)
                    },
                );
            }

            // Порт выхода (OUT)
            if !is_test_mode {
                let is_port_hovered = pointer.hover_pos().map_or(false, |p| {
                    p.distance(port_out) <= 12.0_f32 * self.canvas_zoom
                });

                if is_selected {
                    let glow_out = if is_dark {
                        Color32::from_rgb(255, 205, 50)
                    } else {
                        Color32::from_rgb(217, 119, 6)
                    };
                    painter.circle_stroke(
                        port_out,
                        6.5_f32 * self.canvas_zoom,
                        Stroke::new(1.5_f32 * self.canvas_zoom, glow_out),
                    );
                    painter.circle_filled(port_out, 4.5_f32 * self.canvas_zoom, glow_out);
                    painter.circle_stroke(
                        port_out,
                        4.5_f32 * self.canvas_zoom,
                        Stroke::new(1.2_f32 * self.canvas_zoom, Color32::WHITE),
                    );
                } else {
                    let p_color = if is_port_hovered || self.wire_drag_source_id == Some(step.id) {
                        if is_dark {
                            Color32::from_rgb(255, 205, 50)
                        } else {
                            Color32::from_rgb(217, 119, 6)
                        }
                    } else {
                        accent_color
                    };

                    painter.circle_filled(port_out, 4.2_f32 * self.canvas_zoom, p_color);
                    painter.circle_stroke(
                        port_out,
                        4.2_f32 * self.canvas_zoom,
                        Stroke::new(1.2_f32 * self.canvas_zoom, Color32::WHITE),
                    );
                }
            }
        }

        // Рамка множественного выделения
        if let (Some(start_pt), Some(cur_pos)) = (self.selection_box_start, pointer.latest_pos()) {
            let box_rect = Rect::from_two_pos(start_pt, cur_pos);
            let fill_color = if is_dark {
                Color32::from_rgba_unmultiplied(70, 140, 255, 35)
            } else {
                Color32::from_rgba_unmultiplied(37, 99, 235, 25)
            };
            painter.rect_filled(box_rect, 2.0_f32, fill_color);
            painter.rect_stroke(
                box_rect,
                2.0_f32,
                Stroke::new(
                    1.2_f32,
                    if is_dark {
                        Color32::from_rgb(90, 160, 255)
                    } else {
                        Color32::from_rgb(37, 99, 235)
                    },
                ),
            );
        }

        // Интерактивная миникарта
        if !self.active_algo.steps.is_empty() {
            let toggle_size = Vec2::splat(18.0_f32);
            let toggle_rect = Rect::from_min_size(
                Pos2::new(minimap_rect.max.x - 22.0_f32, minimap_rect.min.y + 4.0_f32),
                toggle_size,
            );

            let is_toggle_hovered = pointer
                .hover_pos()
                .map_or(false, |p| toggle_rect.contains(p));

            if is_toggle_hovered {
                let tooltip = if self.settings.minimap_show_links {
                    lang.minimap_links_on()
                } else {
                    lang.minimap_links_off()
                };
                egui::show_tooltip(ui.ctx(), egui::Id::new("minimap_links_tip"), |ui| {
                    ui.label(tooltip);
                });
            }

            if pointer.primary_clicked() && is_toggle_hovered {
                self.settings.minimap_show_links = !self.settings.minimap_show_links;
                self.settings.save();
                ui.ctx().request_repaint();
            }

            let minimap_bg = if is_dark {
                Color32::from_rgba_unmultiplied(20, 24, 32, 235)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 245)
            };
            let minimap_border = if is_dark {
                Stroke::new(1.0_f32, Color32::from_rgb(50, 58, 72))
            } else {
                Stroke::new(1.0_f32, Color32::from_rgb(180, 195, 215))
            };

            painter.rect_filled(minimap_rect, 4.0_f32, minimap_bg);
            painter.rect_stroke(minimap_rect, 4.0_f32, minimap_border);

            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            for s in &self.active_algo.steps {
                min_x = min_x.min(s.pos[0]);
                max_x = max_x.max(s.pos[0] + 210.0_f32);
                min_y = min_y.min(s.pos[1]);
                max_y = max_y.max(s.pos[1] + 100.0_f32);
            }

            let margin = 180.0_f32;
            min_x -= margin;
            max_x += margin;
            min_y -= margin;
            max_y += margin;

            let world_w = (max_x - min_x).max(100.0_f32);
            let world_h = (max_y - min_y).max(100.0_f32);

            let scale_x = (minimap_rect.width() - 16.0_f32) / world_w;
            let scale_y = (minimap_rect.height() - 16.0_f32) / world_h;
            let scale = scale_x.min(scale_y);

            let map_origin = minimap_rect.min + Vec2::new(8.0_f32, 8.0_f32);

            if self.settings.minimap_show_links {
                let default_link_stroke = Stroke::new(
                    1.2_f32,
                    if is_dark {
                        Color32::from_rgba_unmultiplied(120, 150, 190, 70)
                    } else {
                        Color32::from_rgba_unmultiplied(51, 65, 85, 160)
                    },
                );

                for step in &self.active_algo.steps {
                    let start_pt = map_origin
                        + Vec2::new(
                            (step.pos[0] + 105.0_f32 - min_x) * scale,
                            (step.pos[1] + 100.0_f32 - min_y) * scale,
                        );

                    let mut target_ids = Vec::new();
                    match &step.kind {
                        StepKind::Standard => {
                            for opt in &step.options {
                                if let Some(tid) = opt.next_step_id {
                                    target_ids.push(tid);
                                }
                            }
                        }
                        StepKind::Measurement {
                            next_if_normal,
                            next_if_abnormal,
                            ..
                        } => {
                            if let Some(nid) = next_if_normal {
                                target_ids.push(*nid);
                            }
                            if let Some(aid) = next_if_abnormal {
                                target_ids.push(*aid);
                            }
                        }
                        StepKind::SafetyWarning { next_step_id, .. } => {
                            if let Some(nid) = next_step_id {
                                target_ids.push(*nid);
                            }
                        }
                    }

                    for tid in target_ids {
                        if let Some(target) = self.active_algo.steps.iter().find(|s| s.id == tid) {
                            let end_pt = map_origin
                                + Vec2::new(
                                    (target.pos[0] + 105.0_f32 - min_x) * scale,
                                    (target.pos[1] - min_y) * scale,
                                );

                            let stroke = if is_test_mode {
                                let is_traversed = self
                                    .runner
                                    .history
                                    .windows(2)
                                    .any(|w| w[0] == step.id && w[1] == tid)
                                    || (self.runner.history.last() == Some(&step.id)
                                        && self.runner.current_step_id == Some(tid));
                                let is_current_outgoing =
                                    self.runner.current_step_id == Some(step.id);

                                if is_traversed {
                                    Stroke::new(
                                        2.0_f32,
                                        if is_dark {
                                            Color32::from_rgb(60, 225, 110)
                                        } else {
                                            Color32::from_rgb(16, 185, 129)
                                        },
                                    )
                                } else if is_current_outgoing {
                                    Stroke::new(
                                        1.6_f32,
                                        if is_dark {
                                            Color32::from_rgba_unmultiplied(255, 205, 50, 200)
                                        } else {
                                            Color32::from_rgb(217, 119, 6)
                                        },
                                    )
                                } else {
                                    Stroke::new(
                                        1.0_f32,
                                        if is_dark {
                                            Color32::from_rgba_unmultiplied(100, 120, 150, 40)
                                        } else {
                                            Color32::from_rgba_unmultiplied(100, 116, 139, 120)
                                        },
                                    )
                                }
                            } else {
                                default_link_stroke
                            };

                            painter.line_segment([start_pt, end_pt], stroke);
                        }
                    }
                }
            }

            let time = ui.input(|i| i.time);
            let pulse_speed = 0.4_f64;
            let pulse_phase = (time * pulse_speed).fract() as f32;
            let pulse_phase2 = ((time * pulse_speed) + 0.5_f64).fract() as f32;

            for s in &self.active_algo.steps {
                let m_pos =
                    map_origin + Vec2::new((s.pos[0] - min_x) * scale, (s.pos[1] - min_y) * scale);
                let m_size =
                    Vec2::new(210.0_f32 * scale, 100.0_f32 * scale).max(Vec2::splat(3.0_f32));
                let m_rect = Rect::from_min_size(m_pos, m_size);

                let is_current_run = is_test_mode && self.runner.current_step_id == Some(s.id);
                let is_passed_run = is_test_mode && self.runner.history.contains(&s.id);
                let is_sel = !is_test_mode && self.selected_step_ids.contains(&s.id);

                if is_current_run {
                    ui.ctx().request_repaint();

                    let expand1 = pulse_phase * 12.0_f32;
                    let alpha1 = ((1.0_f32 - pulse_phase) * 140.0_f32) as u8;
                    let ring1 = m_rect.expand(expand1);
                    painter.rect_stroke(
                        ring1.intersect(minimap_rect),
                        3.0_f32,
                        Stroke::new(
                            1.2_f32,
                            if is_dark {
                                Color32::from_rgba_unmultiplied(255, 205, 50, alpha1)
                            } else {
                                Color32::from_rgba_unmultiplied(217, 119, 6, alpha1)
                            },
                        ),
                    );

                    let expand2 = pulse_phase2 * 12.0_f32;
                    let alpha2 = ((1.0_f32 - pulse_phase2) * 140.0_f32) as u8;
                    let ring2 = m_rect.expand(expand2);
                    painter.rect_stroke(
                        ring2.intersect(minimap_rect),
                        3.0_f32,
                        Stroke::new(
                            1.0_f32,
                            if is_dark {
                                Color32::from_rgba_unmultiplied(255, 205, 50, alpha2)
                            } else {
                                Color32::from_rgba_unmultiplied(217, 119, 6, alpha2)
                            },
                        ),
                    );

                    let glow = ((time * 1.5_f64).sin() * 0.5_f64 + 0.5_f64) as f32;
                    let active_color = if is_dark {
                        Color32::from_rgb(
                            255,
                            (195.0_f32 + glow * 30.0_f32) as u8,
                            (50.0_f32 + glow * 40.0_f32) as u8,
                        )
                    } else {
                        Color32::from_rgb(217, (119.0_f32 + glow * 25.0_f32) as u8, 6)
                    };
                    painter.rect_filled(m_rect, 1.5_f32, active_color);
                } else {
                    let m_color = if is_passed_run {
                        if is_dark {
                            Color32::from_rgb(60, 210, 130)
                        } else {
                            Color32::from_rgb(16, 185, 129)
                        }
                    } else if is_sel {
                        Color32::from_rgb(37, 99, 235)
                    } else if self.active_algo.first_step_id == Some(s.id) {
                        if is_dark {
                            Color32::from_rgb(60, 200, 120)
                        } else {
                            Color32::from_rgb(16, 185, 129)
                        }
                    } else if is_dark {
                        Color32::from_rgb(72, 80, 96)
                    } else {
                        Color32::from_rgb(100, 116, 139)
                    };
                    painter.rect_filled(m_rect, 1.5_f32, m_color);
                }
            }

            let view_world_min = Pos2::new(
                -self.canvas_pan.x / self.canvas_zoom,
                -self.canvas_pan.y / self.canvas_zoom,
            );
            let view_world_max = Pos2::new(
                (-self.canvas_pan.x + canvas_rect.width()) / self.canvas_zoom,
                (-self.canvas_pan.y + canvas_rect.height()) / self.canvas_zoom,
            );

            let cam_p1 = map_origin
                + Vec2::new(
                    (view_world_min.x - min_x) * scale,
                    (view_world_min.y - min_y) * scale,
                );
            let cam_p2 = map_origin
                + Vec2::new(
                    (view_world_max.x - min_x) * scale,
                    (view_world_max.y - min_y) * scale,
                );
            let cam_rect = Rect::from_two_pos(cam_p1, cam_p2);

            let cam_stroke = Stroke::new(
                1.3_f32,
                if is_dark {
                    Color32::from_rgb(255, 205, 50)
                } else {
                    Color32::from_rgb(37, 99, 235)
                },
            );
            painter.rect_stroke(cam_rect.intersect(minimap_rect), 2.0_f32, cam_stroke);

            let btn_bg = if is_toggle_hovered {
                if is_dark {
                    Color32::from_rgb(45, 52, 65)
                } else {
                    Color32::from_rgb(230, 238, 250)
                }
            } else {
                Color32::TRANSPARENT
            };
            painter.rect_filled(toggle_rect, 3.0_f32, btn_bg);

            let icon_color = if self.settings.minimap_show_links {
                Color32::from_rgb(37, 99, 235)
            } else if is_dark {
                Color32::from_rgb(85, 90, 105)
            } else {
                Color32::from_rgb(100, 116, 139)
            };
            painter.text(
                toggle_rect.center(),
                egui::Align2::CENTER_CENTER,
                "🔗",
                FontId::proportional(11.0_f32),
                icon_color,
            );

            if pointer.primary_down() && clicked_in_minimap && !is_toggle_hovered {
                self.target_pan = None;
                if let Some(mouse_p) = pointer.latest_pos() {
                    let target_world_x = min_x + (mouse_p.x - map_origin.x) / scale;
                    let target_world_y = min_y + (mouse_p.y - map_origin.y) / scale;
                    self.canvas_pan.x =
                        (canvas_rect.width() * 0.5_f32) - (target_world_x * self.canvas_zoom);
                    self.canvas_pan.y =
                        (canvas_rect.height() * 0.5_f32) - (target_world_y * self.canvas_zoom);
                    ui.ctx().request_repaint();
                }
            }
        }

        // Поиск по узлам (Ctrl+F)
        if self.show_search_bar && !is_test_mode {
            let search_area_pos =
                Pos2::new(canvas_rect.max.x - 310.0_f32, canvas_rect.min.y + 12.0_f32);
            let mut close_search = false;
            let mut next_match = false;
            let mut prev_match = false;

            egui::Area::new(egui::Id::new("canvas_search_panel"))
                .fixed_pos(search_area_pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(8.0_f32)
                        .rounding(4.0_f32)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let edit_res = ui.add(
                                    egui::TextEdit::singleline(&mut self.search_query)
                                        .id(egui::Id::new("canvas_search_input"))
                                        .desired_width(150.0_f32)
                                        .hint_text(lang.search_nodes_placeholder()),
                                );

                                if edit_res.changed() {
                                    self.update_search_query();
                                    self.jump_to_current_search_match(canvas_rect.size());
                                }

                                if edit_res.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    if ui.input(|i| i.modifiers.shift) {
                                        prev_match = true;
                                    } else {
                                        next_match = true;
                                    }
                                }

                                if !self.search_results.is_empty() {
                                    ui.label(format!(
                                        "{}/{}",
                                        self.current_search_idx + 1,
                                        self.search_results.len()
                                    ));
                                } else if !self.search_query.trim().is_empty() {
                                    ui.colored_label(
                                        Color32::from_rgb(220, 38, 38),
                                        lang.search_no_matches(),
                                    );
                                }

                                ui.add_enabled_ui(!self.search_results.is_empty(), |ui| {
                                    if ui.button("▲").clicked() {
                                        prev_match = true;
                                    }
                                    if ui.button("▼").clicked() {
                                        next_match = true;
                                    }
                                });

                                if ui.button("✖").clicked() {
                                    close_search = true;
                                }
                            });
                        });
                });

            if next_match && !self.search_results.is_empty() {
                self.current_search_idx = (self.current_search_idx + 1) % self.search_results.len();
                self.jump_to_current_search_match(canvas_rect.size());
            }

            if prev_match && !self.search_results.is_empty() {
                if self.current_search_idx == 0 {
                    self.current_search_idx = self.search_results.len().saturating_sub(1);
                } else {
                    self.current_search_idx -= 1;
                }
                self.jump_to_current_search_match(canvas_rect.size());
            }

            if close_search {
                self.show_search_bar = false;
            }
        }

        // Контекстное меню ПКМ (только в Конструкторе)
        if let Some(menu) = self.context_menu {
            let mut close_menu = false;
            let mut action_duplicate = false;
            let mut action_delete = false;
            let mut action_add_here = None;
            let mut action_make_start = None;

            let area_response = egui::Area::new(egui::Id::new("canvas_context_menu"))
                .fixed_pos(menu.pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(6.0_f32)
                        .show(ui, |ui| {
                            ui.set_width(175.0_f32);
                            ui.spacing_mut().item_spacing = Vec2::new(0.0_f32, 3.0_f32);

                            ui.vertical_centered_justified(|ui| {
                                if let Some(node_id) = menu.node_id {
                                    if ui.button(lang.ctx_make_start()).clicked() {
                                        action_make_start = Some(node_id);
                                        close_menu = true;
                                    }
                                    if ui.button(lang.ctx_duplicate()).clicked() {
                                        action_duplicate = true;
                                        close_menu = true;
                                    }
                                    ui.separator();
                                    if ui.button(lang.ctx_delete()).clicked() {
                                        action_delete = true;
                                        close_menu = true;
                                    }
                                } else {
                                    if ui.button(lang.ctx_add_here()).clicked() {
                                        action_add_here = Some(menu.pos);
                                        close_menu = true;
                                    }
                                    ui.separator();
                                    if ui.button(lang.fit_view()).clicked() {
                                        self.fit_to_view(canvas_rect.size());
                                        close_menu = true;
                                    }
                                }
                            });
                        });
                });

            self.context_menu_rect = Some(area_response.response.rect);

            if let Some(id) = action_make_start {
                self.snapshot_for_undo();
                self.active_algo.first_step_id = Some(id);
            }
            if action_duplicate {
                self.duplicate_selected();
            }
            if action_delete {
                self.delete_selected();
            }
            if let Some(pos) = action_add_here {
                self.snapshot_for_undo();
                let count = self.active_algo.steps.len() + 1;
                let mut world_x = (pos.x - self.canvas_pan.x) / self.canvas_zoom;
                let mut world_y = (pos.y - self.canvas_pan.y) / self.canvas_zoom;
                if self.settings.snap_to_grid {
                    world_x = (world_x / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                    world_y = (world_y / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                }

                let title = lang.step_default_name(count);
                let step = Step::new(title, [world_x, world_y]);
                let id = step.id;
                if self.active_algo.first_step_id.is_none() {
                    self.active_algo.first_step_id = Some(id);
                }
                self.active_algo.steps.push(step);
                self.selected_step_ids.clear();
                self.selected_step_ids.insert(id);
            }

            if close_menu {
                self.context_menu = None;
                self.context_menu_rect = None;
            }
        }
    }
}
