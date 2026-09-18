#![allow(float_literal_f32_fallback)]

mod app;
mod canvas;
mod i18n;
mod models;
mod settings;
mod storage;
mod views;

use app::AlgoApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0_f32, 820.0_f32])
            .with_title("Equipment Algo Builder"),
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    eframe::run_native(
        "Equipment Algo Builder",
        options,
        Box::new(|cc| Box::new(AlgoApp::new(cc))),
    )
}