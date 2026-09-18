use crate::models::WireStyle;
use eframe::egui::{
    self, epaint::CubicBezierShape, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2,
};

pub fn dist_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_sq();
    if len_sq == 0.0_f32 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0_f32, 1.0_f32);
    let proj = a + ab * t;
    p.distance(proj)
}

/// Распределитель каналов: гарантирует уникальную высоту Y для всех перекрывающихся по X линий
pub fn compute_channel_mid_ys(wires: &[(Pos2, Pos2)], zoom: f32) -> Vec<f32> {
    let mut mid_ys = Vec::with_capacity(wires.len());
    let mut placed: Vec<(f32, f32, f32)> = Vec::new(); // (x_min, x_max, mid_y)

    for (start, end) in wires {
        let dx = (end.x - start.x).abs();
        let dy = end.y - start.y;

        // Строго вертикальные линии не требуют горизонтального канала
        if dx < 6.0_f32 * zoom {
            mid_ys.push((start.y + end.y) * 0.5_f32);
            continue;
        }

        let is_long_jump = dy > 340.0_f32 * zoom;
        let x_min = start.x.min(end.x) - 16.0_f32 * zoom;
        let x_max = start.x.max(end.x) + 16.0_f32 * zoom;

        let mut chosen_y = 0.0_f32;

        for track in 0..12 {
            let candidate_y = if is_long_jump {
                // Обходные магистрали заходят к целевому блоку снизу
                end.y - (54.0_f32 + track as f32 * 36.0_f32) * zoom
            } else {
                // Стандартный переход между соседними этажами
                start.y + (48.0_f32 + track as f32 * 36.0_f32) * zoom
            };

            // Проверяем коллизию по горизонтали с уже размещенными проводами
            let conflict = placed.iter().any(|(p_min, p_max, p_y)| {
                let y_clash = (p_y - candidate_y).abs() < 24.0_f32 * zoom;
                let x_overlap = x_min <= *p_max && x_max >= *p_min;
                y_clash && x_overlap
            });

            if !conflict {
                chosen_y = candidate_y;
                break;
            }
        }

        placed.push((x_min, x_max, chosen_y));
        mid_ys.push(chosen_y);
    }

    mid_ys
}

/// Отрисовывает связь с промежуточными стрелками ТОЛЬКО на горизонтали
pub fn draw_connection_wire(
    painter: &Painter,
    start: Pos2,
    end: Pos2,
    label: &str,
    color: Color32,
    zoom: f32,
    is_dark: bool,
    is_highlighted: bool,
    style: WireStyle,
    exact_mid_y: f32,
) -> (Option<Rect>, Vec<[Pos2; 2]>) {
    let base_width = if is_highlighted { 3.2_f32 } else { 2.0_f32 };
    let stroke_width = (base_width * zoom).clamp(1.2_f32, 5.0_f32);
    let stroke = Stroke::new(stroke_width, color);

    let mut segments = Vec::new();
    let text_pos;
    let mut p1_opt = None;
    let mut p2_opt = None;

    match style {
        WireStyle::Curved => {
            let dy = (end.y - start.y).abs();
            let control_offset = (dy * 0.5_f32).max(45.0_f32 * zoom);
            let cp1 = Pos2::new(start.x, start.y + control_offset);
            let cp2 = Pos2::new(end.x, end.y - control_offset);

            let bezier_shape = CubicBezierShape::from_points_stroke(
                [start, cp1, cp2, end],
                false,
                Color32::TRANSPARENT,
                stroke,
            );
            painter.add(bezier_shape);

            let mut prev = start;
            for i in 1..=8 {
                let t = i as f32 / 8.0_f32;
                let it = 1.0_f32 - t;
                let pt = Pos2::new(
                    it.powi(3) * start.x
                        + 3.0 * it.powi(2) * t * cp1.x
                        + 3.0 * it * t.powi(2) * cp2.x
                        + t.powi(3) * end.x,
                    it.powi(3) * start.y
                        + 3.0 * it.powi(2) * t * cp1.y
                        + 3.0 * it * t.powi(2) * cp2.y
                        + t.powi(3) * end.y,
                );
                segments.push([prev, pt]);
                prev = pt;
            }

            text_pos = Pos2::new(
                0.125_f32 * start.x + 0.375_f32 * cp1.x + 0.375_f32 * cp2.x + 0.125_f32 * end.x,
                0.125_f32 * start.y + 0.375_f32 * cp1.y + 0.375_f32 * cp2.y + 0.125_f32 * end.y,
            );
        }
        WireStyle::Orthogonal => {
            let dx = end.x - start.x;
            let is_strictly_vertical = dx.abs() < 6.0_f32 * zoom;

            if is_strictly_vertical {
                painter.line_segment([start, end], stroke);
                segments.push([start, end]);
                text_pos = Pos2::new(start.x, (start.y + end.y) * 0.5_f32);
            } else {
                let mid_y = exact_mid_y.round();
                let p1 = Pos2::new(start.x, mid_y);
                let p2 = Pos2::new(end.x, mid_y);

                painter.line_segment([start, p1], stroke);
                painter.line_segment([p1, p2], stroke);
                painter.line_segment([p2, end], stroke);

                segments.push([start, p1]);
                segments.push([p1, p2]);
                segments.push([p2, end]);

                p1_opt = Some(p1);
                p2_opt = Some(p2);

                let badge_w = (label.len() as f32 * 7.5_f32 + 16.0_f32) * zoom;
                let horiz_len = dx.abs();
                let safe_pad = 14.0_f32 * zoom;

                if horiz_len >= badge_w + safe_pad * 2.0_f32 {
                    let half_w = badge_w * 0.5_f32;
                    let mut cx = (start.x + end.x) * 0.5_f32;
                    if dx > 0.0_f32 {
                        cx = cx.clamp(start.x + safe_pad + half_w, end.x - safe_pad - half_w);
                    } else {
                        cx = cx.clamp(end.x + safe_pad + half_w, start.x - safe_pad - half_w);
                    }
                    text_pos = Pos2::new(cx, mid_y);
                } else {
                    text_pos = Pos2::new(end.x, (mid_y + end.y) * 0.5_f32);
                }
            }
        }
    }

    // Единственная вертикальная стрелка — на входе в порт карточки
    let dir_down = Vec2::new(0.0_f32, 1.0_f32);
    let normal_down = Vec2::new(-1.0_f32, 0.0_f32);
    let end_arrow_size = if is_highlighted {
        8.5_f32 * zoom
    } else {
        7.0_f32 * zoom
    };
    let arrow_pt1 = end - dir_down * end_arrow_size + normal_down * (end_arrow_size * 0.65_f32);
    let arrow_pt2 = end - dir_down * end_arrow_size - normal_down * (end_arrow_size * 0.65_f32);
    painter.line_segment([end, arrow_pt1], stroke);
    painter.line_segment([end, arrow_pt2], stroke);

    // Плашка с текстом
    let badge_rect = if !label.is_empty() {
        let b_rect = Rect::from_center_size(
            text_pos,
            Vec2::new(
                (label.len() as f32 * 7.5_f32 + 14.0_f32) * zoom,
                18.0_f32 * zoom,
            ),
        );

        let badge_bg = if is_dark {
            if is_highlighted {
                Color32::from_rgb(26, 32, 42)
            } else {
                Color32::from_rgb(20, 24, 30)
            }
        } else {
            Color32::from_rgb(255, 255, 255)
        };

        painter.rect_filled(b_rect, 3.0_f32 * zoom, badge_bg);
        let border_stroke = Stroke::new(
            if is_highlighted {
                1.5_f32 * zoom
            } else {
                1.0_f32 * zoom
            },
            color,
        );
        painter.rect_stroke(b_rect, 3.0_f32 * zoom, border_stroke);
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.0_f32 * zoom),
            if is_dark {
                color
            } else {
                Color32::from_rgb(15, 23, 42)
            },
        );

        Some(b_rect)
    } else {
        None
    };

    // Стрелки направления ТОЛЬКО на горизонтальном сегменте
    if let (Some(p1), Some(p2)) = (p1_opt, p2_opt) {
        let dx = end.x - start.x;
        let h_len = dx.abs();
        let h_dir = Vec2::new(dx.signum(), 0.0_f32);

        let draw_horiz_arrow = |tip: Pos2| {
            if let Some(r) = badge_rect {
                if r.expand(6.0_f32 * zoom).contains(tip) {
                    return;
                }
            }
            let normal = Vec2::new(-h_dir.y, h_dir.x);
            let arrow_sz = (6.0_f32 * zoom).clamp(4.0_f32, 8.5_f32);
            let a1 = tip - h_dir * arrow_sz + normal * (arrow_sz * 0.55_f32);
            let a2 = tip - h_dir * arrow_sz - normal * (arrow_sz * 0.55_f32);
            painter.line_segment([tip, a1], stroke);
            painter.line_segment([tip, a2], stroke);
        };

        if h_len >= 48.0_f32 * zoom {
            draw_horiz_arrow(p1 + h_dir * (20.0_f32 * zoom));
            draw_horiz_arrow(p2 - h_dir * (14.0_f32 * zoom));
        } else if h_len >= 18.0_f32 * zoom {
            draw_horiz_arrow(p1 + h_dir * (h_len * 0.5_f32));
        }
    }

    (badge_rect, segments)
}
