mod i18n;

use chrono::{DateTime, Utc};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use i18n::Language;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use uuid::Uuid;

const SETTINGS_FILE: &str = "settings.json";
const ARCHIVE_DIR: &str = "archive";

// --- НАСТРОЙКИ ПРИЛОЖЕНИЯ ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub language: Language,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::Ru,
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

// --- СТРУКТУРЫ ДАННЫХ АЛГОРИТМА ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EquipmentInfo {
    pub eq_type: String,
    pub model: String,
    pub name: String,
    pub inv_number: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: String,
}

impl Default for EquipmentInfo {
    fn default() -> Self {
        Self {
            eq_type: String::new(),
            model: String::new(),
            name: "Новое оборудование".to_string(),
            inv_number: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            author: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionLink {
    pub text: String,
    pub next_step_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum StepKind {
    Standard,
    Measurement {
        unit: String,
        min_val: f64,
        max_val: f64,
        next_if_normal: Option<Uuid>,
        next_if_abnormal: Option<Uuid>,
    },
    SafetyWarning {
        ack_text: String,
        next_step_id: Option<Uuid>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub kind: StepKind,
    pub options: Vec<OptionLink>,
    pub image_path: Option<String>,
    pub pos: [f32; 2],
}

impl Step {
    pub fn new(title: String, pos: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description: String::new(),
            kind: StepKind::Standard,
            options: Vec::new(),
            image_path: None,
            pos,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Algorithm {
    pub id: Uuid,
    pub metadata: EquipmentInfo,
    pub steps: Vec<Step>,
    pub first_step_id: Option<Uuid>,
}

impl Default for Algorithm {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: EquipmentInfo::default(),
            steps: Vec::new(),
            first_step_id: None,
        }
    }
}

// --- СОСТОЯНИЕ РЕЖИМА ДИАГНОСТИКИ ---

#[derive(Default)]
struct RunnerState {
    current_step_id: Option<Uuid>,
    history: Vec<Uuid>,
    measurement_input: String,
    safety_acknowledged: bool,
}

// --- УПРАВЛЕНИЕ АРХИВОМ ---

fn save_algorithm_to_archive(algo: &mut Algorithm) -> Result<(), String> {
    let project_dir = PathBuf::from(ARCHIVE_DIR).join(algo.id.to_string());
    let images_dir = project_dir.join("images");

    fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;

    for step in &mut algo.steps {
        if let Some(src_str) = &step.image_path {
            let src_path = PathBuf::from(src_str);
            if src_path.exists() {
                let inside_proj = src_path.starts_with(&images_dir);
                if !inside_proj {
                    let file_name = src_path.file_name().unwrap_or_default().to_string_lossy();
                    let target_name = format!("{}_{}", step.id, file_name);
                    let target_path = images_dir.join(target_name);

                    if fs::copy(&src_path, &target_path).is_ok() {
                        step.image_path = Some(target_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    algo.metadata.updated_at = Utc::now();

    let json = serde_json::to_string_pretty(&algo).map_err(|e| e.to_string())?;
    let data_file = project_dir.join("data.json");
    fs::write(data_file, json).map_err(|e| e.to_string())?;

    Ok(())
}

fn load_all_from_archive() -> Vec<Algorithm> {
    let mut list = Vec::new();
    let archive_path = Path::new(ARCHIVE_DIR);
    if !archive_path.exists() {
        return list;
    }

    if let Ok(entries) = fs::read_dir(archive_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let data_file = entry.path().join("data.json");
                if data_file.exists() {
                    if let Ok(content) = fs::read_to_string(data_file) {
                        if let Ok(algo) = serde_json::from_str::<Algorithm>(&content) {
                            list.push(algo);
                        }
                    }
                }
            }
        }
    }

    list.sort_by(|a, b| b.metadata.updated_at.cmp(&a.metadata.updated_at));
    list
}

fn delete_from_archive(id: Uuid) {
    let project_dir = PathBuf::from(ARCHIVE_DIR).join(id.to_string());
    if project_dir.exists() {
        let _ = fs::remove_dir_all(project_dir);
    }
}

// Преобразование пути в URI для egui
fn to_image_uri(path_str: &str) -> String {
    if let Ok(abs) = fs::canonicalize(path_str) {
        let clean = abs.to_string_lossy().replace('\\', "/");
        let formatted = clean.trim_start_matches("//?/").trim_start_matches("/?");
        format!("file://{}", formatted)
    } else {
        format!("file://{}", path_str.replace('\\', "/"))
    }
}

// --- СОСТОЯНИЕ GUI ---

struct AlgoApp {
    current_tab: String,
    settings: AppSettings,
    active_algo: Algorithm,
    selected_step_id: Option<Uuid>,
    canvas_pan: Vec2,
    canvas_zoom: f32,
    dragging_node_id: Option<Uuid>,
    drag_start_mouse: Pos2,
    drag_start_node_pos: [f32; 2],
    runner: RunnerState,
    // Состояние архива и интерфейса
    show_passport_modal: bool,
    status_msg: Option<(String, Instant)>,
    archive_list: Vec<Algorithm>,
    archive_search: String,
    archive_loaded: bool,
}

impl AlgoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            current_tab: "Конструктор".to_owned(),
            settings: AppSettings::load(),
            active_algo: Algorithm::default(),
            selected_step_id: None,
            canvas_pan: Vec2::new(120.0, 100.0),
            canvas_zoom: 1.0_f32,
            dragging_node_id: None,
            drag_start_mouse: Pos2::ZERO,
            drag_start_node_pos: [0.0, 0.0],
            runner: RunnerState::default(),
            show_passport_modal: false,
            status_msg: None,
            archive_list: Vec::new(),
            archive_search: String::new(),
            archive_loaded: false,
        }
    }

    fn refresh_archive(&mut self) {
        self.archive_list = load_all_from_archive();
        self.archive_loaded = true;
    }

    fn set_status(&mut self, text: String) {
        self.status_msg = Some((text, Instant::now()));
    }

    fn reset_runner(&mut self) {
        self.runner.current_step_id = self.active_algo.first_step_id;
        self.runner.history.clear();
        self.runner.measurement_input.clear();
        self.runner.safety_acknowledged = false;
    }

    fn navigate_runner(&mut self, next_id: Option<Uuid>) {
        if let Some(curr) = self.runner.current_step_id {
            self.runner.history.push(curr);
        }
        self.runner.current_step_id = next_id;
        self.runner.measurement_input.clear();
        self.runner.safety_acknowledged = false;
    }

    fn navigate_runner_back(&mut self) {
        if let Some(prev) = self.runner.history.pop() {
            self.runner.current_step_id = Some(prev);
            self.runner.measurement_input.clear();
            self.runner.safety_acknowledged = false;
        }
    }

    fn focus_step_on_canvas(&mut self, step_id: Uuid, view_size: Vec2) {
        if let Some(step) = self.active_algo.steps.iter().find(|s| s.id == step_id) {
            let target_center_x = (step.pos[0] + 100.0) * self.canvas_zoom;
            let target_center_y = (step.pos[1] + 47.5) * self.canvas_zoom;
            self.canvas_pan.x = (view_size.x * 0.5) - target_center_x;
            self.canvas_pan.y = (view_size.y * 0.5) - target_center_y;
        }
    }

    fn fit_to_view(&mut self, view_size: Vec2) {
        if self.active_algo.steps.is_empty() {
            return;
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for s in &self.active_algo.steps {
            min_x = min_x.min(s.pos[0]);
            max_x = max_x.max(s.pos[0] + 200.0);
            min_y = min_y.min(s.pos[1]);
            max_y = max_y.max(s.pos[1] + 95.0);
        }

        let content_w = (max_x - min_x) + 120.0;
        let content_h = (max_y - min_y) + 120.0;

        let zoom_x = (view_size.x / content_w).clamp(0.35_f32, 1.5_f32);
        let zoom_y = (view_size.y / content_h).clamp(0.35_f32, 1.5_f32);
        self.canvas_zoom = zoom_x.min(zoom_y);

        let center_x = ((min_x + max_x) * 0.5) * self.canvas_zoom;
        let center_y = ((min_y + max_y) * 0.5) * self.canvas_zoom;

        self.canvas_pan.x = (view_size.x * 0.5) - center_x;
        self.canvas_pan.y = (view_size.y * 0.5) - center_y;
    }
}

impl eframe::App for AlgoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.settings.language;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.selectable_value(&mut self.current_tab, "Конструктор".to_string(), lang.tab_builder());
                if ui.selectable_value(&mut self.current_tab, "Диагностика".to_string(), lang.tab_runner()).clicked() {
                    self.reset_runner();
                }
                if ui.selectable_value(&mut self.current_tab, "Архив".to_string(), lang.tab_archive()).clicked() {
                    self.refresh_archive();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut current_lang = self.settings.language;
                    let label = match current_lang {
                        Language::Ru => "🇷🇺 RU",
                        Language::En => "🇬🇧 EN",
                    };

                    egui::ComboBox::from_id_source("lang_selector")
                        .selected_text(label)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut current_lang, Language::Ru, "🇷🇺 Русский").clicked()
                                || ui.selectable_value(&mut current_lang, Language::En, "🇬🇧 English").clicked()
                            {
                                self.settings.language = current_lang;
                                self.settings.save();
                            }
                        });
                });
            });
        });

        // Модальное окно паспорта оборудования
        if self.show_passport_modal {
            egui::Window::new(lang.passport_window_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    egui::Grid::new("passport_fields").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                        ui.label(lang.field_name());
                        ui.text_edit_singleline(&mut self.active_algo.metadata.name);
                        ui.end_row();

                        ui.label(lang.field_type());
                        ui.text_edit_singleline(&mut self.active_algo.metadata.eq_type);
                        ui.end_row();

                        ui.label(lang.field_model());
                        ui.text_edit_singleline(&mut self.active_algo.metadata.model);
                        ui.end_row();

                        ui.label(lang.field_inv());
                        ui.text_edit_singleline(&mut self.active_algo.metadata.inv_number);
                        ui.end_row();

                        ui.label(lang.field_author());
                        ui.text_edit_singleline(&mut self.active_algo.metadata.author);
                        ui.end_row();
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(lang.btn_close()).clicked() {
                            self.show_passport_modal = false;
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab.as_str() {
                "Конструктор" => self.render_builder_mode(ui),
                "Диагностика" => self.render_runner_mode(ui),
                "Архив" => self.render_archive_mode(ui),
                _ => {}
            }
        });
    }
}

impl AlgoApp {
    fn render_builder_mode(&mut self, ui: &mut egui::Ui) {
        let view_size = ui.available_size_before_wrap();
        let lang = self.settings.language;

        ui.horizontal(|ui| {
            if ui.button(lang.btn_new()).clicked() {
                self.active_algo = Algorithm::default();
                self.selected_step_id = None;
                self.canvas_pan = Vec2::new(120.0, 100.0);
                self.canvas_zoom = 1.0;
            }

            if ui.button(lang.btn_save()).clicked() {
                if save_algorithm_to_archive(&mut self.active_algo).is_ok() {
                    self.set_status(lang.save_success().to_string());
                    self.refresh_archive();
                }
            }

            if ui.button(lang.btn_passport()).clicked() {
                self.show_passport_modal = true;
            }

            ui.separator();

            if ui.button(lang.add_block()).clicked() {
                let count = self.active_algo.steps.len() + 1;
                let new_pos = [
                    (-self.canvas_pan.x + 220.0 + (count as f32 * 30.0)) / self.canvas_zoom,
                    (-self.canvas_pan.y + 120.0 + (count as f32 * 30.0)) / self.canvas_zoom,
                ];
                let title = match lang {
                    Language::Ru => format!("Шаг {}", count),
                    Language::En => format!("Step {}", count),
                };
                let step = Step::new(title, new_pos);
                let id = step.id;
                if self.active_algo.first_step_id.is_none() {
                    self.active_algo.first_step_id = Some(id);
                }
                self.active_algo.steps.push(step);
                self.selected_step_id = Some(id);
            }

            if ui.button(lang.fit_view()).clicked() {
                self.fit_to_view(view_size);
            }

            ui.separator();
            ui.label(format!("{} {:.0}%", lang.zoom_label(), self.canvas_zoom * 100.0));
            if ui.button("100%").clicked() {
                self.canvas_zoom = 1.0_f32;
            }

            // Статус сохранения
            if let Some((msg, time)) = &self.status_msg {
                if time.elapsed().as_secs() < 3 {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(90, 220, 120), msg);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(lang.toolbar_hint());
            });
        });
        ui.separator();

        let wants_kb = ui.ctx().wants_keyboard_input();
        if !wants_kb && ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            if let Some(del_id) = self.selected_step_id {
                self.delete_step(del_id);
            }
        }

        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(330.0)
            .width_range(280.0..=480.0)
            .show_inside(ui, |ui| {
                self.render_inspector(ui);
            });

        self.render_canvas(ui, false);
    }

    fn render_runner_mode(&mut self, ui: &mut egui::Ui) {
        let view_size = ui.available_size_before_wrap();
        let lang = self.settings.language;

        ui.horizontal(|ui| {
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
            if ui.button(lang.fit_view()).clicked() {
                self.fit_to_view(view_size);
            }
            ui.separator();
            ui.label(format!("{} {:.0}%", lang.zoom_label(), self.canvas_zoom * 100.0));
        });
        ui.separator();

        egui::SidePanel::right("runner_panel")
            .resizable(true)
            .default_width(360.0)
            .width_range(300.0..=500.0)
            .show_inside(ui, |ui| {
                self.render_runner_controls(ui);
            });

        self.render_canvas(ui, true);
    }

    fn render_archive_mode(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;

        if !self.archive_loaded {
            self.refresh_archive();
        }

        ui.horizontal(|ui| {
            ui.heading(lang.tab_archive());
            ui.separator();
            ui.add(egui::TextEdit::singleline(&mut self.archive_search).hint_text(lang.search_placeholder()));
            if ui.button(lang.btn_refresh()).clicked() {
                self.refresh_archive();
            }
        });
        ui.separator();

        let query = self.archive_search.trim().to_lowercase();
        let filtered: Vec<&Algorithm> = self.archive_list
            .iter()
            .filter(|a| {
                if query.is_empty() {
                    return true;
                }
                a.metadata.name.to_lowercase().contains(&query)
                    || a.metadata.model.to_lowercase().contains(&query)
                    || a.metadata.eq_type.to_lowercase().contains(&query)
                    || a.metadata.inv_number.to_lowercase().contains(&query)
                    || a.metadata.author.to_lowercase().contains(&query)
            })
            .collect();

        if self.archive_list.is_empty() {
            ui.label(lang.empty_archive());
            return;
        }

        if filtered.is_empty() {
            ui.label(lang.not_found());
            return;
        }

        let mut id_to_load_builder: Option<Algorithm> = None;
        let mut id_to_load_runner: Option<Algorithm> = None;
        let mut id_to_delete: Option<Uuid> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for algo in filtered {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(26, 30, 38))
                    .rounding(6.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                let display_name = if algo.metadata.name.is_empty() {
                                    "Без названия".to_string()
                                } else {
                                    algo.metadata.name.clone()
                                };
                                ui.heading(display_name);

                                ui.add_space(3.0);
                                ui.horizontal(|ui| {
                                    if !algo.metadata.eq_type.is_empty() {
                                        ui.colored_label(Color32::from_rgb(120, 170, 255), &algo.metadata.eq_type);
                                        ui.label("|");
                                    }
                                    if !algo.metadata.model.is_empty() {
                                        ui.label(format!("Модель: {}", algo.metadata.model));
                                        ui.label("|");
                                    }
                                    if !algo.metadata.inv_number.is_empty() {
                                        ui.label(format!("Инв. №: {}", algo.metadata.inv_number));
                                        ui.label("|");
                                    }
                                    ui.colored_label(Color32::from_rgb(180, 190, 200), format!("{} {}", algo.steps.len(), lang.card_steps()));
                                });

                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    if !algo.metadata.author.is_empty() {
                                        ui.label(format!("Автор: {}", algo.metadata.author));
                                        ui.label("|");
                                    }
                                    ui.label(format!("{} {}", lang.card_updated(), algo.metadata.updated_at.format("%d.%m.%Y %H:%M")));
                                });
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(lang.btn_delete_archive()).clicked() {
                                    id_to_delete = Some(algo.id);
                                }
                                if ui.button(lang.btn_run_diag()).clicked() {
                                    id_to_load_runner = Some(algo.clone());
                                }
                                if ui.button(lang.btn_open_builder()).clicked() {
                                    id_to_load_builder = Some(algo.clone());
                                }
                            });
                        });
                    });
                ui.add_space(6.0);
            }
        });

        if let Some(algo) = id_to_load_builder {
            self.active_algo = algo;
            self.selected_step_id = self.active_algo.first_step_id;
            self.current_tab = "Конструктор".to_string();
        }

        if let Some(algo) = id_to_load_runner {
            self.active_algo = algo;
            self.reset_runner();
            self.current_tab = "Диагностика".to_string();
        }

        if let Some(del_id) = id_to_delete {
            delete_from_archive(del_id);
            self.refresh_archive();
        }
    }

    fn render_canvas(&mut self, ui: &mut egui::Ui, is_test_mode: bool) {
        let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::hover());
        let canvas_rect = response.rect;
        let lang = self.settings.language;

        let pointer = ui.input(|i| i.pointer.clone());
        let is_in_canvas = pointer.hover_pos().map_or(false, |p| canvas_rect.contains(p));

        let scroll_delta_y = ui.input(|i| i.raw_scroll_delta.y);
        if is_in_canvas && scroll_delta_y.abs() > 0.1 {
            let zoom_factor = if scroll_delta_y > 0.0 { 1.10_f32 } else { 0.90_f32 };
            let new_zoom = (self.canvas_zoom * zoom_factor).clamp(0.35_f32, 2.5_f32);

            if let Some(mouse_pos) = pointer.hover_pos() {
                let mouse_vec = mouse_pos.to_vec2();
                self.canvas_pan = mouse_vec - (mouse_vec - self.canvas_pan) * (new_zoom / self.canvas_zoom);
            }
            self.canvas_zoom = new_zoom;
            ui.ctx().request_repaint();
        }

        if pointer.secondary_down() && is_in_canvas {
            let delta = pointer.delta();
            if delta.length_sq() > 0.0 {
                self.canvas_pan += delta;
                ui.ctx().request_repaint();
            }
            ui.ctx().set_cursor_icon(egui::CursorIcon::AllScroll);
        }

        let node_size = Vec2::new(200.0, 95.0) * self.canvas_zoom;

        if !is_test_mode {
            if pointer.primary_pressed() && is_in_canvas {
                if let Some(pos) = pointer.latest_pos() {
                    let hit_step = self.active_algo.steps.iter().rev().find(|step| {
                        let screen_pos = Pos2::new(
                            step.pos[0] * self.canvas_zoom + self.canvas_pan.x,
                            step.pos[1] * self.canvas_zoom + self.canvas_pan.y,
                        );
                        Rect::from_min_size(screen_pos, node_size).contains(pos)
                    });

                    if let Some(step) = hit_step {
                        self.dragging_node_id = Some(step.id);
                        self.drag_start_mouse = pos;
                        self.drag_start_node_pos = step.pos;
                        self.selected_step_id = Some(step.id);
                    }
                }
            }

            if pointer.primary_down() {
                if let Some(drag_id) = self.dragging_node_id {
                    if let Some(cur_mouse) = pointer.latest_pos() {
                        let total_mouse_delta = cur_mouse - self.drag_start_mouse;
                        if let Some(step) = self.active_algo.steps.iter_mut().find(|s| s.id == drag_id) {
                            step.pos[0] = self.drag_start_node_pos[0] + (total_mouse_delta.x / self.canvas_zoom);
                            step.pos[1] = self.drag_start_node_pos[1] + (total_mouse_delta.y / self.canvas_zoom);
                        }
                        ui.ctx().request_repaint();
                    }
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }
            } else {
                self.dragging_node_id = None;
            }
        }

        let grid_size = 40.0 * self.canvas_zoom;
        let stroke_grid = Stroke::new(1.0_f32, Color32::from_rgb(30, 34, 42));

        let min_k_x = ((canvas_rect.min.x - self.canvas_pan.x) / grid_size).floor() as i32;
        let max_k_x = ((canvas_rect.max.x - self.canvas_pan.x) / grid_size).ceil() as i32;
        for k in min_k_x..=max_k_x {
            let x = (k as f32) * grid_size + self.canvas_pan.x;
            painter.line_segment([Pos2::new(x, canvas_rect.min.y), Pos2::new(x, canvas_rect.max.y)], stroke_grid);
        }

        let min_k_y = ((canvas_rect.min.y - self.canvas_pan.y) / grid_size).floor() as i32;
        let max_k_y = ((canvas_rect.max.y - self.canvas_pan.y) / grid_size).ceil() as i32;
        for k in min_k_y..=max_k_y {
            let y = (k as f32) * grid_size + self.canvas_pan.y;
            painter.line_segment([Pos2::new(canvas_rect.min.x, y), Pos2::new(canvas_rect.max.x, y)], stroke_grid);
        }

        let mut node_rects: Vec<(Uuid, Rect)> = Vec::new();
        for step in &self.active_algo.steps {
            let screen_pos = Pos2::new(
                step.pos[0] * self.canvas_zoom + self.canvas_pan.x,
                step.pos[1] * self.canvas_zoom + self.canvas_pan.y,
            );
            node_rects.push((step.id, Rect::from_min_size(screen_pos, node_size)));
        }

        if !is_test_mode && self.dragging_node_id.is_none() && !pointer.secondary_down() && is_in_canvas {
            if let Some(pos) = pointer.hover_pos() {
                if node_rects.iter().any(|(_, r)| r.contains(pos)) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }
            }
        }

        let mouse_clicked = pointer.primary_clicked() && is_in_canvas;
        let click_pos = pointer.latest_pos();
        let mut link_to_disconnect: Option<(Uuid, usize, u8)> = None;

        let norm_text = match lang { Language::Ru => "Норма", Language::En => "Normal" };
        let abn_text = match lang { Language::Ru => "Отклонение", Language::En => "Abnormal" };
        let safe_text = match lang { Language::Ru => "Ознакомлен", Language::En => "Acknowledged" };

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
                            connections.push((target_id, opt.text.clone(), Color32::from_rgb(120, 160, 255), idx, 0));
                        }
                    }
                }
                StepKind::Measurement { next_if_normal, next_if_abnormal, .. } => {
                    if let Some(nid) = next_if_normal {
                        connections.push((*nid, norm_text.to_string(), Color32::from_rgb(80, 220, 120), 0, 1));
                    }
                    if let Some(aid) = next_if_abnormal {
                        connections.push((*aid, abn_text.to_string(), Color32::from_rgb(255, 90, 90), 0, 2));
                    }
                }
                StepKind::SafetyWarning { next_step_id, .. } => {
                    if let Some(nid) = next_step_id {
                        connections.push((*nid, safe_text.to_string(), Color32::from_rgb(255, 190, 40), 0, 3));
                    }
                }
            }

            for (target_id, label, color, conn_idx, conn_type) in connections {
                if let Some((_, to_rect)) = node_rects.iter().find(|(id, _)| *id == target_id) {
                    let end_pt = Pos2::new(to_rect.center().x, to_rect.min.y);

                    let line_color = if is_test_mode {
                        let is_traversed = self.runner.history.contains(&step.id)
                            && (self.runner.history.contains(&target_id) || self.runner.current_step_id == Some(target_id));
                        if is_traversed {
                            Color32::from_rgb(70, 220, 120)
                        } else {
                            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 65)
                        }
                    } else {
                        color
                    };

                    let badge_rect = draw_arrow(&painter, start_pt, end_pt, &label, line_color, self.canvas_zoom);

                    if !is_test_mode && mouse_clicked {
                        if let (Some(b_rect), Some(pos)) = (badge_rect, click_pos) {
                            if b_rect.contains(pos) {
                                link_to_disconnect = Some((step.id, conn_idx, conn_type));
                            }
                        }
                    }
                }
            }
        }

        if let Some((step_id, idx, conn_type)) = link_to_disconnect {
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
                        if let StepKind::Measurement { next_if_abnormal, .. } = &mut step.kind {
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

        for (id, rect) in &node_rects {
            let step = match self.active_algo.steps.iter().find(|s| s.id == *id) {
                Some(s) => s,
                None => continue,
            };

            let is_start = self.active_algo.first_step_id == Some(*id);
            let is_current_run = is_test_mode && self.runner.current_step_id == Some(*id);
            let is_passed_run = is_test_mode && self.runner.history.contains(id);
            let is_selected_builder = !is_test_mode && self.selected_step_id == Some(*id);

            let (bg_color, border_stroke) = if is_test_mode {
                if is_current_run {
                    (
                        Color32::from_rgb(32, 36, 48),
                        Stroke::new(3.5 * self.canvas_zoom, Color32::from_rgb(255, 195, 0)),
                    )
                } else if is_passed_run {
                    (
                        Color32::from_rgb(22, 30, 26),
                        Stroke::new(2.0 * self.canvas_zoom, Color32::from_rgb(60, 180, 100)),
                    )
                } else {
                    (
                        Color32::from_rgb(20, 22, 26),
                        Stroke::new(1.0_f32 * self.canvas_zoom, Color32::from_rgb(45, 48, 56)),
                    )
                }
            } else if is_selected_builder {
                (
                    Color32::from_rgb(28, 32, 42),
                    Stroke::new(2.5 * self.canvas_zoom, Color32::from_rgb(90, 160, 255)),
                )
            } else if is_start {
                (
                    Color32::from_rgb(25, 28, 35),
                    Stroke::new(2.0 * self.canvas_zoom, Color32::from_rgb(70, 200, 120)),
                )
            } else {
                (
                    Color32::from_rgb(25, 28, 35),
                    Stroke::new(1.0_f32 * self.canvas_zoom, Color32::from_rgb(60, 65, 75)),
                )
            };

            painter.rect_filled(*rect, 6.0 * self.canvas_zoom, bg_color);
            painter.rect_stroke(*rect, 6.0 * self.canvas_zoom, border_stroke);

            let header_height = 26.0 * self.canvas_zoom;
            let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));

            let (header_bg, badge_text) = if is_current_run {
                (Color32::from_rgb(110, 80, 15), lang.badge_active())
            } else if is_passed_run {
                (Color32::from_rgb(25, 70, 40), lang.badge_done())
            } else {
                match &step.kind {
                    StepKind::Standard => (Color32::from_rgb(45, 65, 95), lang.badge_choice()),
                    StepKind::Measurement { .. } => (Color32::from_rgb(70, 45, 95), lang.badge_measure()),
                    StepKind::SafetyWarning { .. } => (Color32::from_rgb(95, 60, 25), lang.badge_safety()),
                }
            };

            painter.rect_filled(header_rect, 6.0 * self.canvas_zoom, header_bg);

            let title_prefix = if is_start && !is_test_mode { "🚩 " } else { "" };
            let title_font_size = (13.0 * self.canvas_zoom).clamp(8.0, 22.0);
            painter.text(
                header_rect.left_center() + Vec2::new(8.0 * self.canvas_zoom, 0.0),
                egui::Align2::LEFT_CENTER,
                format!("{}{}", title_prefix, step.title),
                egui::FontId::proportional(title_font_size),
                Color32::WHITE,
            );

            let badge_font_size = (11.0 * self.canvas_zoom).clamp(7.0, 18.0);
            painter.text(
                header_rect.right_center() - Vec2::new(8.0 * self.canvas_zoom, 0.0),
                egui::Align2::RIGHT_CENTER,
                badge_text,
                egui::FontId::proportional(badge_font_size),
                Color32::from_rgb(220, 230, 245),
            );

            let desc_font_size = (12.0 * self.canvas_zoom).clamp(7.5, 20.0);
            let preview_text = if step.description.trim().is_empty() {
                lang.no_desc().to_string()
            } else {
                let line = step.description.lines().next().unwrap_or("");
                if line.chars().count() > 22 {
                    format!("{}...", line.chars().take(22).collect::<String>())
                } else {
                    line.to_string()
                }
            };

            painter.text(
                rect.min + Vec2::new(10.0 * self.canvas_zoom, 42.0 * self.canvas_zoom),
                egui::Align2::LEFT_TOP,
                preview_text,
                egui::FontId::proportional(desc_font_size),
                Color32::from_rgb(170, 175, 185),
            );

            if step.image_path.is_some() {
                painter.text(
                    rect.min + Vec2::new(10.0 * self.canvas_zoom, 68.0 * self.canvas_zoom),
                    egui::Align2::LEFT_TOP,
                    lang.photo_attached(),
                    egui::FontId::proportional((11.0 * self.canvas_zoom).clamp(7.0, 17.0)),
                    Color32::from_rgb(100, 180, 255),
                );
            }
        }
    }

    fn delete_step(&mut self, del_id: Uuid) {
        self.active_algo.steps.retain(|s| s.id != del_id);
        for s in &mut self.active_algo.steps {
            for opt in &mut s.options {
                if opt.next_step_id == Some(del_id) {
                    opt.next_step_id = None;
                }
            }
            match &mut s.kind {
                StepKind::Measurement { next_if_normal, next_if_abnormal, .. } => {
                    if *next_if_normal == Some(del_id) { *next_if_normal = None; }
                    if *next_if_abnormal == Some(del_id) { *next_if_abnormal = None; }
                }
                StepKind::SafetyWarning { next_step_id, .. } => {
                    if *next_step_id == Some(del_id) { *next_step_id = None; }
                }
                _ => {}
            }
        }
        if self.active_algo.first_step_id == Some(del_id) {
            self.active_algo.first_step_id = self.active_algo.steps.first().map(|s| s.id);
        }
        self.selected_step_id = self.active_algo.steps.first().map(|s| s.id);
    }

    fn render_runner_controls(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        ui.heading(lang.runner_heading());
        ui.label(format!(
            "{} ({}) | №: {}",
            self.active_algo.metadata.name,
            self.active_algo.metadata.model,
            self.active_algo.metadata.inv_number
        ));
        ui.separator();

        let current_id = match self.runner.current_step_id {
            Some(id) => id,
            None => {
                ui.colored_label(Color32::GREEN, lang.diag_complete());
                ui.add_space(8.0);
                if ui.button(lang.start_over()).clicked() {
                    self.reset_runner();
                }
                return;
            }
        };

        let current_step = self.active_algo.steps.iter().find(|s| s.id == current_id).cloned();

        if let Some(step) = current_step {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.colored_label(Color32::from_rgb(255, 200, 50), format!("{} {}", lang.current_node_label(), step.title));
                ui.add_space(6.0);

                if !step.description.is_empty() {
                    ui.label(&step.description);
                }

                if let Some(path) = &step.image_path {
                    ui.add_space(8.0);
                    ui.add(egui::Image::new(to_image_uri(path)).max_width(ui.available_width()).rounding(6.0));
                }

                ui.add_space(12.0);
                ui.separator();

                match &step.kind {
                    StepKind::Standard => {
                        ui.label(lang.choose_action());
                        if step.options.is_empty() {
                            ui.colored_label(Color32::GREEN, lang.final_point());
                            if ui.button(lang.finish_diag_btn()).clicked() {
                                self.navigate_runner(None);
                            }
                        } else {
                            for opt in &step.options {
                                if ui.button(&opt.text).clicked() {
                                    self.navigate_runner(opt.next_step_id);
                                }
                            }
                        }
                    }

                    StepKind::Measurement { unit, min_val, max_val, next_if_normal, next_if_abnormal } => {
                        let prompt = match lang {
                            Language::Ru => format!("Замер (допуск: {} ... {} {}):", min_val, max_val, unit),
                            Language::En => format!("Measurement (tolerance: {} ... {} {}):", min_val, max_val, unit),
                        };
                        ui.label(prompt);
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.runner.measurement_input);
                            ui.label(unit);
                        });

                        if let Ok(val) = self.runner.measurement_input.trim().parse::<f64>() {
                            let in_range = val >= *min_val && val <= *max_val;
                            if in_range {
                                ui.colored_label(Color32::GREEN, format!("✔ {} {} {}", val, unit, lang.in_range_msg()));
                                if ui.button(lang.accept_normal()).clicked() {
                                    self.navigate_runner(*next_if_normal);
                                }
                            } else {
                                let err_msg = match lang {
                                    Language::Ru => format!("❌ Отклонение! (Норма: {} - {} {})", min_val, max_val, unit),
                                    Language::En => format!("❌ Abnormal! (Tolerance: {} - {} {})", min_val, max_val, unit),
                                };
                                ui.colored_label(Color32::LIGHT_RED, err_msg);
                                if ui.button(lang.accept_abnormal()).clicked() {
                                    self.navigate_runner(*next_if_abnormal);
                                }
                            }
                        }
                    }

                    StepKind::SafetyWarning { ack_text, next_step_id } => {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(60, 40, 10))
                            .rounding(4.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.colored_label(Color32::YELLOW, lang.safety_title());
                                ui.checkbox(&mut self.runner.safety_acknowledged, ack_text);
                            });

                        ui.add_space(8.0);
                        ui.add_enabled_ui(self.runner.safety_acknowledged, |ui| {
                            if ui.button(lang.confirm_and_proceed()).clicked() {
                                self.navigate_runner(*next_step_id);
                            }
                        });
                    }
                }
            });
        }
    }

    fn render_inspector(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.language;
        ui.heading(lang.inspector_heading());
        ui.separator();

        let step_lookup: Vec<(Option<Uuid>, String)> = std::iter::once((None, lang.finish_diag().to_string()))
            .chain(self.active_algo.steps.iter().map(|s| (Some(s.id), s.title.clone())))
            .collect();

        let mut step_to_delete: Option<Uuid> = None;
        let selected_id = self.selected_step_id;

        if let Some(step) = self.active_algo.steps.iter_mut().find(|s| Some(s.id) == selected_id) {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut is_start = Some(step.id) == self.active_algo.first_step_id;
                    if ui.checkbox(&mut is_start, lang.start_node()).changed() && is_start {
                        self.active_algo.first_step_id = Some(step.id);
                    }
                    if ui.button(lang.delete_btn()).clicked() {
                        step_to_delete = Some(step.id);
                    }
                });

                ui.add_space(6.0);
                ui.label(lang.block_title());
                ui.text_edit_singleline(&mut step.title);

                ui.add_space(4.0);
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
                            unit: match lang { Language::Ru => "В".to_string(), Language::En => "V".to_string() },
                            min_val: 200.0,
                            max_val: 240.0,
                            next_if_normal: None,
                            next_if_abnormal: None,
                        },
                        2 => StepKind::SafetyWarning {
                            ack_text: match lang {
                                Language::Ru => "Оборудование обесточено".to_string(),
                                Language::En => "Equipment de-energized".to_string(),
                            },
                            next_step_id: None,
                        },
                        _ => StepKind::Standard,
                    };
                }

                ui.add_space(6.0);
                ui.label(lang.instructions());
                ui.text_edit_multiline(&mut step.description);

                ui.separator();
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
                    ui.add(egui::Image::new(to_image_uri(path)).max_width(240.0).rounding(4.0));
                }

                ui.separator();
                ui.heading(lang.links_heading());

                match &mut step.kind {
                    StepKind::Standard => {
                        if ui.button(lang.add_branch()).clicked() {
                            step.options.push(OptionLink {
                                text: match lang { Language::Ru => "Да".to_string(), Language::En => "Yes".to_string() },
                                next_step_id: None,
                            });
                        }

                        let mut opt_to_del = None;
                        for (idx, opt) in step.options.iter_mut().enumerate() {
                            ui.group(|ui| {
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

                                egui::ComboBox::from_id_source(format!("combo_{}_{}", step.id, idx))
                                    .selected_text(cur_title)
                                    .show_ui(ui, |ui| {
                                        for (id_opt, title) in &step_lookup {
                                            ui.selectable_value(&mut opt.next_step_id, *id_opt, title);
                                        }
                                    });
                            });
                        }
                        if let Some(del) = opt_to_del {
                            step.options.remove(del);
                        }
                    }

                    StepKind::Measurement { unit, min_val, max_val, next_if_normal, next_if_abnormal } => {
                        ui.horizontal(|ui| {
                            ui.label(lang.unit());
                            ui.text_edit_singleline(unit);
                        });
                        ui.horizontal(|ui| {
                            ui.label(lang.min_val());
                            ui.add(egui::DragValue::new(min_val).speed(0.1));
                            ui.label(lang.max_val());
                            ui.add(egui::DragValue::new(max_val).speed(0.1));
                        });

                        ui.label(lang.goto_normal());
                        let norm_title = step_lookup.iter().find(|(id, _)| *id == *next_if_normal).map(|(_, t)| t.as_str()).unwrap_or(lang.not_selected());
                        egui::ComboBox::from_id_source(format!("norm_{}", step.id))
                            .selected_text(norm_title)
                            .show_ui(ui, |ui| {
                                for (id_opt, title) in &step_lookup {
                                    ui.selectable_value(next_if_normal, *id_opt, title);
                                }
                            });

                        ui.label(lang.goto_abnormal());
                        let abn_title = step_lookup.iter().find(|(id, _)| *id == *next_if_abnormal).map(|(_, t)| t.as_str()).unwrap_or(lang.not_selected());
                        egui::ComboBox::from_id_source(format!("abn_{}", step.id))
                            .selected_text(abn_title)
                            .show_ui(ui, |ui| {
                                for (id_opt, title) in &step_lookup {
                                    ui.selectable_value(next_if_abnormal, *id_opt, title);
                                }
                            });
                    }

                    StepKind::SafetyWarning { ack_text, next_step_id } => {
                        ui.label(lang.safety_ack());
                        ui.text_edit_singleline(ack_text);
                        ui.label(lang.goto_after_ack());
                        let next_title = step_lookup.iter().find(|(id, _)| *id == *next_step_id).map(|(_, t)| t.as_str()).unwrap_or(lang.not_selected());
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
        } else {
            ui.label(lang.select_node_hint());
        }

        if let Some(del_id) = step_to_delete {
            self.delete_step(del_id);
        }
    }
}

fn draw_arrow(
    painter: &egui::Painter,
    start: Pos2,
    end: Pos2,
    label: &str,
    color: Color32,
    zoom: f32,
) -> Option<Rect> {
    let stroke_width = (2.0 * zoom).clamp(1.0, 4.0);
    let stroke = Stroke::new(stroke_width, color);

    let mid_y = start.y + (end.y - start.y) * 0.5;
    let p1 = Pos2::new(start.x, mid_y);
    let p2 = Pos2::new(end.x, mid_y);

    painter.line_segment([start, p1], stroke);
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, end], stroke);

    let dir = if end.y >= p2.y { Vec2::new(0.0, 1.0) } else { Vec2::new(0.0, -1.0) };
    let normal = Vec2::new(-dir.y, dir.x);
    let arrow_size = 7.0 * zoom;
    let arrow_pt1 = end - dir * arrow_size + normal * (arrow_size * 0.7);
    let arrow_pt2 = end - dir * arrow_size - normal * (arrow_size * 0.7);
    painter.line_segment([end, arrow_pt1], stroke);
    painter.line_segment([end, arrow_pt2], stroke);

    if !label.is_empty() {
        let text_pos = Pos2::new((p1.x + p2.x) * 0.5, mid_y);
        let badge_rect = Rect::from_center_size(
            text_pos,
            Vec2::new((label.len() as f32 * 7.5 + 12.0) * zoom, 18.0 * zoom),
        );

        painter.rect_filled(badge_rect, 3.0 * zoom, Color32::from_rgb(20, 24, 30));
        painter.rect_stroke(badge_rect, 3.0 * zoom, Stroke::new(1.0_f32 * zoom, color));
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional((11.0 * zoom).clamp(7.0, 16.0)),
            color,
        );

        Some(badge_rect)
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Equipment Algo Builder"),
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    eframe::run_native(
        "Equipment Algo Builder",
        options,
        Box::new(|cc| Box::new(AlgoApp::new(cc))),
    )
}