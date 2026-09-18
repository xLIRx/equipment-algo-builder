pub const GRID_SNAP_STEP: f32 = 36.0_f32;

use crate::i18n::Language;
use crate::models::WireStyle;
use eframe::egui::{self, Color32, Stroke};
use serde::{Deserialize, Serialize};
use std::fs;

const SETTINGS_FILE: &str = "settings.json";

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Dark,
    Light,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub language: Language,
    pub theme: AppTheme,
    #[serde(default = "default_true")]
    pub minimap_show_links: bool,
    #[serde(default = "default_true")]
    pub snap_to_grid: bool,
    #[serde(default)]
    pub default_wire_style: WireStyle,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::Ru,
            theme: AppTheme::Dark,
            minimap_show_links: true,
            snap_to_grid: true,
            default_wire_style: WireStyle::Curved,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        if let Ok(data) = fs::read_to_string(SETTINGS_FILE) {
            if let Ok(settings) = serde_json::from_str(&data) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(SETTINGS_FILE, json);
        }
    }
}

pub fn configure_app_visuals(ctx: &egui::Context, theme: AppTheme) {
    match theme {
        AppTheme::Dark => {
            let mut visuals = egui::Visuals::dark();
            visuals.override_text_color = Some(Color32::from_rgb(235, 240, 248));
            visuals.panel_fill = Color32::from_rgb(18, 22, 29);
            visuals.window_fill = Color32::from_rgb(24, 29, 39);
            visuals.extreme_bg_color = Color32::from_rgb(12, 15, 20);
            visuals.faint_bg_color = Color32::from_rgb(26, 32, 44);

            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(22, 27, 36);
            visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0_f32);
            visuals.widgets.noninteractive.bg_stroke =
                Stroke::new(1.0_f32, Color32::from_rgb(42, 52, 68));

            visuals.widgets.inactive.bg_fill = Color32::from_rgb(26, 33, 45);
            visuals.widgets.inactive.rounding = egui::Rounding::same(6.0_f32);
            visuals.widgets.inactive.bg_stroke =
                Stroke::new(1.0_f32, Color32::from_rgb(46, 58, 78));

            visuals.widgets.hovered.bg_fill = Color32::from_rgb(36, 48, 68);
            visuals.widgets.hovered.rounding = egui::Rounding::same(6.0_f32);
            visuals.widgets.hovered.bg_stroke =
                Stroke::new(1.2_f32, Color32::from_rgb(70, 130, 220));

            visuals.widgets.active.bg_fill = Color32::from_rgb(42, 60, 88);
            visuals.widgets.active.rounding = egui::Rounding::same(6.0_f32);
            visuals.widgets.active.bg_stroke =
                Stroke::new(1.5_f32, Color32::from_rgb(90, 150, 250));

            visuals.widgets.open.bg_fill = Color32::from_rgb(30, 38, 52);
            visuals.widgets.open.rounding = egui::Rounding::same(6.0_f32);

            ctx.set_visuals(visuals);
        }
        AppTheme::Light => {
            let mut visuals = egui::Visuals::light();
            visuals.override_text_color = Some(Color32::from_rgb(15, 23, 42));
            visuals.panel_fill = Color32::from_rgb(241, 245, 249);
            visuals.window_fill = Color32::from_rgb(255, 255, 255);
            visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
            visuals.faint_bg_color = Color32::from_rgb(248, 250, 252);

            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
            visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0_f32);
            visuals.widgets.noninteractive.bg_stroke =
                Stroke::new(1.0_f32, Color32::from_rgb(203, 213, 225));

            visuals.widgets.inactive.bg_fill = Color32::from_rgb(255, 255, 255);
            visuals.widgets.inactive.rounding = egui::Rounding::same(6.0_f32);
            visuals.widgets.inactive.bg_stroke =
                Stroke::new(1.0_f32, Color32::from_rgb(203, 213, 225));

            visuals.widgets.hovered.bg_fill = Color32::from_rgb(238, 242, 255);
            visuals.widgets.hovered.rounding = egui::Rounding::same(6.0_f32);
            visuals.widgets.hovered.bg_stroke =
                Stroke::new(1.2_f32, Color32::from_rgb(59, 130, 246));

            visuals.widgets.active.bg_fill = Color32::from_rgb(224, 231, 255);
            visuals.widgets.active.rounding = egui::Rounding::same(6.0_f32);
            visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, Color32::from_rgb(37, 99, 235));

            visuals.widgets.open.bg_fill = Color32::from_rgb(241, 245, 249);
            visuals.widgets.open.rounding = egui::Rounding::same(6.0_f32);

            ctx.set_visuals(visuals);
        }
    }
}

pub fn ui_kbd(ui: &mut egui::Ui, text: &str, is_dark: bool) {
    let bg = if is_dark {
        Color32::from_rgb(34, 42, 56)
    } else {
        Color32::from_rgb(238, 242, 248)
    };
    let border = if is_dark {
        Color32::from_rgb(60, 74, 98)
    } else {
        Color32::from_rgb(180, 195, 215)
    };
    let text_color = if is_dark {
        Color32::from_rgb(230, 240, 255)
    } else {
        Color32::from_rgb(30, 41, 59)
    };

    egui::Frame::none()
        .fill(bg)
        .rounding(4.0_f32)
        .stroke(Stroke::new(1.0_f32, border))
        .inner_margin(egui::Margin::symmetric(5.0_f32, 2.0_f32))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .monospace()
                    .size(11.0_f32)
                    .color(text_color)
                    .strong(),
            );
        });
}
