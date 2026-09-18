use crate::models::Step;
use eframe::egui::{Color32, FontId, Painter, PointerButton, Pos2, Rect, Stroke, Ui, Vec2};
use std::collections::HashMap;
use uuid::Uuid;

const LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

/// Собирает все целевые блоки, на которые есть обратные ссылки снизу вверх, и сопоставляет им буквы
pub fn build_connector_letter_map(
    steps: &[Step],
    node_rects: &[(Uuid, Rect)],
) -> HashMap<Uuid, String> {
    let mut upward_targets = Vec::new();

    for step in steps {
        let from_rect = match node_rects.iter().find(|(id, _)| *id == step.id) {
            Some((_, r)) => *r,
            None => continue,
        };

        let mut targets = Vec::new();
        match &step.kind {
            crate::models::StepKind::SafetyWarning { next_step_id, .. } => {
                if let Some(nid) = next_step_id {
                    targets.push(*nid);
                }
            }
            crate::models::StepKind::Subprocess {
                next_if_success,
                next_if_failure,
                ..
            } => {
                if let Some(sid) = next_if_success {
                    targets.push(*sid);
                }
                if let Some(fid) = next_if_failure {
                    targets.push(*fid);
                }
            }
            crate::models::StepKind::Standard => {
                for opt in &step.options {
                    if let Some(tid) = opt.next_step_id {
                        targets.push(tid);
                    }
                }
            }
            crate::models::StepKind::Measurement {
                next_if_normal,
                next_if_abnormal,
                ..
            } => {
                if let Some(nid) = next_if_normal {
                    targets.push(*nid);
                }
                if let Some(aid) = next_if_abnormal {
                    targets.push(*aid);
                }
            }
        }

        for tid in targets {
            if let Some((_, to_rect)) = node_rects.iter().find(|(id, _)| *id == tid) {
                if to_rect.center().y < from_rect.center().y - 10.0_f32
                    && !upward_targets.contains(&tid)
                {
                    upward_targets.push(tid);
                }
            }
        }
    }

    upward_targets
        .iter()
        .enumerate()
        .map(|(idx, &id)| {
            let letter = LETTERS.get(idx).unwrap_or(&"A").to_string();
            (id, letter)
        })
        .collect()
}

/// Отрисовывает входные кружки-коннекторы слева от верхних блоков
pub fn draw_incoming_connectors(
    painter: &Painter,
    ui: &Ui,
    node_rects: &[(Uuid, Rect)],
    target_to_letter: &HashMap<Uuid, String>,
    hovered_connector: &mut Option<String>,
    zoom: f32,
    is_dark: bool,
) {
    let pointer = ui.input(|i| i.pointer.clone());

    for (&tid, letter) in target_to_letter {
        if let Some((_, to_rect)) = node_rects.iter().find(|(id, _)| *id == tid) {
            let r = 12.0_f32 * zoom;
            let in_center = Pos2::new(to_rect.min.x - 36.0_f32 * zoom, to_rect.center().y);
            let in_circle_rect = Rect::from_center_size(in_center, Vec2::splat(r * 2.0_f32));

            let is_hovered = pointer
                .hover_pos()
                .map_or(false, |p| in_circle_rect.contains(p));
            let is_active_pair = hovered_connector.as_ref() == Some(letter);

            let circle_bg = if is_active_pair || is_hovered {
                Color32::from_rgb(217, 119, 6)
            } else if is_dark {
                Color32::from_rgb(32, 40, 56)
            } else {
                Color32::from_rgb(255, 255, 255)
            };

            let circle_stroke = Stroke::new(
                1.6_f32 * zoom,
                if is_active_pair || is_hovered {
                    Color32::from_rgb(255, 205, 50)
                } else if is_dark {
                    Color32::from_rgb(70, 140, 255)
                } else {
                    Color32::from_rgb(37, 99, 235)
                },
            );

            painter.circle_filled(in_center, r, circle_bg);
            painter.circle_stroke(in_center, r, circle_stroke);
            painter.text(
                in_center,
                eframe::egui::Align2::CENTER_CENTER,
                letter,
                FontId::proportional(12.0_f32 * zoom),
                if is_active_pair || is_hovered || is_dark {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(15, 23, 42)
                },
            );

            // Стрелка входа в блок
            let p_start = Pos2::new(in_center.x + r, in_center.y);
            let p_end = Pos2::new(to_rect.min.x, to_rect.center().y);
            painter.line_segment([p_start, p_end], circle_stroke);

            let arr_len = 5.0_f32 * zoom;
            painter.line_segment(
                [p_end, p_end + Vec2::new(-arr_len, arr_len * 0.7_f32)],
                circle_stroke,
            );
            painter.line_segment(
                [p_end, p_end + Vec2::new(-arr_len, -arr_len * 0.7_f32)],
                circle_stroke,
            );

            if is_hovered {
                *hovered_connector = Some(letter.clone());
                egui_tooltip(
                    ui,
                    &format!("in_conn_{}", tid),
                    &format!("Точка входа по ссылке [{}]", letter),
                );
            }
        }
    }
}

/// Отрисовывает выходной кружок-коннектор снизу блока при обратном переходе вверх
pub fn draw_outgoing_connector(
    painter: &Painter,
    ui: &Ui,
    start_pt: Pos2,
    letter: &str,
    label: &str,
    target_title: &str,
    line_color: Color32,
    hovered_connector: &mut Option<String>,
    zoom: f32,
    is_dark: bool,
    is_test_mode: bool,
) -> (Option<Rect>, bool) {
    let r = 12.0_f32 * zoom;
    let out_center = Pos2::new(start_pt.x, start_pt.y + 32.0_f32 * zoom);
    let out_circle_rect = Rect::from_center_size(out_center, Vec2::splat(r * 2.0_f32));

    let pointer = ui.input(|i| i.pointer.clone());
    let is_hovered = pointer
        .hover_pos()
        .map_or(false, |p| out_circle_rect.contains(p));
    let is_active_pair = hovered_connector.as_ref() == Some(&letter.to_string());

    let stroke = Stroke::new(
        1.6_f32 * zoom,
        if is_active_pair || is_hovered {
            Color32::from_rgb(255, 205, 50)
        } else {
            line_color
        },
    );

    painter.line_segment(
        [start_pt, Pos2::new(out_center.x, out_center.y - r)],
        stroke,
    );

    let bg = if is_active_pair || is_hovered {
        Color32::from_rgb(217, 119, 6)
    } else if is_dark {
        Color32::from_rgb(32, 40, 56)
    } else {
        Color32::from_rgb(255, 255, 255)
    };

    painter.circle_filled(out_center, r, bg);
    painter.circle_stroke(out_center, r, stroke);
    painter.text(
        out_center,
        eframe::egui::Align2::CENTER_CENTER,
        letter,
        FontId::proportional(12.0_f32 * zoom),
        if is_active_pair || is_hovered || is_dark {
            Color32::WHITE
        } else {
            Color32::from_rgb(15, 23, 42)
        },
    );

    // Плашка с текстом перехода
    let mut badge_rect_out = None;
    if !label.is_empty() {
        let badge_pos = Pos2::new(out_center.x + r + 8.0_f32 * zoom, out_center.y);
        let badge_rect = Rect::from_center_size(
            badge_pos + Vec2::new((label.len() as f32 * 3.5_f32) * zoom, 0.0_f32),
            Vec2::new(
                (label.len() as f32 * 7.5_f32 + 10.0_f32) * zoom,
                16.0_f32 * zoom,
            ),
        );
        painter.rect_filled(
            badge_rect,
            3.0_f32 * zoom,
            if is_dark {
                Color32::from_rgb(20, 24, 30)
            } else {
                Color32::WHITE
            },
        );
        painter.rect_stroke(
            badge_rect,
            3.0_f32 * zoom,
            Stroke::new(1.0_f32 * zoom, line_color),
        );
        painter.text(
            badge_rect.center(),
            eframe::egui::Align2::CENTER_CENTER,
            label,
            FontId::proportional(10.5_f32 * zoom),
            if is_dark {
                line_color
            } else {
                Color32::from_rgb(15, 23, 42)
            },
        );
        badge_rect_out = Some(badge_rect);
    }

    let mut jump_requested = false;
    if is_hovered {
        *hovered_connector = Some(letter.to_string());
        egui_tooltip(
            ui,
            &format!("out_conn_{}", letter),
            &format!(
                "Переход к [{}]: «{}»\n(Кликните для перехода)",
                letter, target_title
            ),
        );

        if pointer.button_clicked(PointerButton::Primary) && !is_test_mode {
            jump_requested = true;
        }
    }

    (badge_rect_out, jump_requested)
}

fn egui_tooltip(ui: &Ui, id_str: &str, text: &str) {
    eframe::egui::show_tooltip(ui.ctx(), eframe::egui::Id::new(id_str), |ui| {
        ui.label(text);
    });
}
