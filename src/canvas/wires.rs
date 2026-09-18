use crate::models::WireStyle;
use eframe::egui::{
    self, epaint::CubicBezierShape, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2,
};

/// Вычисляет кратчайшее расстояние от точки до отрезка (для детекции клика ПКМ по проводу)
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

/// Отрисовывает линию связи (сплайн или 90°), стрелку на конце и бейдж с текстом варианта перехода
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
) -> (Option<Rect>, Vec<[Pos2; 2]>) {
    let base_width = if is_highlighted { 3.2_f32 } else { 2.0_f32 };
    let stroke_width = (base_width * zoom).clamp(1.2_f32, 5.0_f32);
    let stroke = Stroke::new(stroke_width, color);

    let mut segments = Vec::new();
    let text_pos;

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

            // Аппроксимация сплайна отрезками для проверки попадания клика
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
            let mid_y = ((start.y + end.y) * 0.5_f32).round();
            let p1 = Pos2::new(start.x, mid_y);
            let p2 = Pos2::new(end.x, mid_y);

            painter.line_segment([start, p1], stroke);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, end], stroke);

            segments.push([start, p1]);
            segments.push([p1, p2]);
            segments.push([p2, end]);

            text_pos = Pos2::new((start.x + end.x) * 0.5_f32, mid_y);
        }
    }

    // Стрелка входа в блок
    let dir = Vec2::new(0.0_f32, 1.0_f32);
    let normal = Vec2::new(-1.0_f32, 0.0_f32);
    let arrow_size = if is_highlighted {
        8.5_f32 * zoom
    } else {
        7.0_f32 * zoom
    };
    let arrow_pt1 = end - dir * arrow_size + normal * (arrow_size * 0.65_f32);
    let arrow_pt2 = end - dir * arrow_size - normal * (arrow_size * 0.65_f32);
    painter.line_segment([end, arrow_pt1], stroke);
    painter.line_segment([end, arrow_pt2], stroke);

    // Плашка с текстом перехода
    if !label.is_empty() {
        let badge_rect = Rect::from_center_size(
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

        painter.rect_filled(badge_rect, 3.0_f32 * zoom, badge_bg);
        let border_stroke = Stroke::new(
            if is_highlighted {
                1.5_f32 * zoom
            } else {
                1.0_f32 * zoom
            },
            color,
        );
        painter.rect_stroke(badge_rect, 3.0_f32 * zoom, border_stroke);
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

        (Some(badge_rect), segments)
    } else {
        (None, segments)
    }
}
