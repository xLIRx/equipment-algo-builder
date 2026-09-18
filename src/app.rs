use crate::i18n::Language;
use crate::models::{Algorithm, Step, StepKind, WireStyle};
use crate::settings::{configure_app_visuals, AppSettings, AppTheme, GRID_SNAP_STEP};
use crate::storage::{
    export_algorithm_to_file, import_algorithm_from_file, save_algorithm_to_archive,
};
use eframe::egui::{self, epaint::CubicBezierShape, Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

const MAX_UNDO_DEPTH: usize = 40;

#[derive(Default)]
pub struct RunnerState {
    pub current_step_id: Option<Uuid>,
    pub history: Vec<Uuid>,
    pub measurement_input: String,
    pub safety_acknowledged: bool,
}

#[derive(Clone, Copy)]
pub struct ContextMenuState {
    pub pos: Pos2,
    pub node_id: Option<Uuid>,
}

pub struct AlgoApp {
    pub current_tab: String,
    pub settings: AppSettings,
    pub active_algo: Algorithm,
    pub last_saved_algo: Option<Algorithm>,

    pub undo_stack: Vec<Algorithm>,
    pub redo_stack: Vec<Algorithm>,

    pub selected_step_ids: HashSet<Uuid>,
    pub is_dragging_nodes: bool,
    pub drag_start_mouse: Pos2,
    pub drag_start_positions: HashMap<Uuid, [f32; 2]>,
    pub dragged_any_distance: bool,
    pub selection_box_start: Option<Pos2>,
    pub wire_drag_source_id: Option<Uuid>,
    pub hovered_connector: Option<String>,

    pub show_search_bar: bool,
    pub search_query: String,
    pub search_results: Vec<Uuid>,
    pub current_search_idx: usize,

    pub pending_png_path: Option<PathBuf>,
    pub last_canvas_rect: Option<Rect>,

    pub canvas_pan: Vec2,
    pub target_pan: Option<Vec2>,
    pub canvas_zoom: f32,

    pub runner: RunnerState,
    pub context_menu: Option<ContextMenuState>,
    pub context_menu_rect: Option<Rect>,
    pub show_passport_modal: bool,
    pub show_close_modal: bool,
    pub force_close: bool,
    pub status_msg: Option<(String, Instant)>,

    pub archive_list: Vec<Algorithm>,
    pub archive_search: String,
    pub archive_loaded: bool,
    pub archive_selected_mfg: Option<String>,
    pub archive_selected_model: Option<String>,
    pub confirm_delete_target: Option<(Uuid, String)>,
}

impl AlgoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let settings = AppSettings::load();
        configure_app_visuals(&cc.egui_ctx, settings.theme);

        let mut default_algo = Algorithm::default();
        default_algo.metadata.name = settings.language.default_equipment_name().to_string();

        Self {
            current_tab: "Конструктор".to_owned(),
            settings,
            active_algo: default_algo.clone(),
            last_saved_algo: Some(default_algo),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selected_step_ids: HashSet::new(),
            is_dragging_nodes: false,
            drag_start_mouse: Pos2::ZERO,
            drag_start_positions: HashMap::new(),
            dragged_any_distance: false,
            selection_box_start: None,
            wire_drag_source_id: None,
            hovered_connector: None,
            show_search_bar: false,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_idx: 0,
            pending_png_path: None,
            last_canvas_rect: None,
            canvas_pan: Vec2::new(120.0_f32, 100.0_f32),
            target_pan: None,
            canvas_zoom: 1.0_f32,
            runner: RunnerState::default(),
            context_menu: None,
            context_menu_rect: None,
            show_passport_modal: false,
            show_close_modal: false,
            force_close: false,
            status_msg: None,
            archive_list: Vec::new(),
            archive_search: String::new(),
            archive_loaded: false,
            archive_selected_mfg: None,
            archive_selected_model: None,
            confirm_delete_target: None,
        }
    }

    pub fn zoom_by_step(&mut self, zoom_in: bool, focal_point: Vec2) {
        let current_pct = (self.canvas_zoom * 100.0_f32).round() as i32;
        let next_pct = if zoom_in {
            if current_pct % 10 == 0 {
                current_pct + 10
            } else {
                ((current_pct / 10) + 1) * 10
            }
        } else if current_pct % 10 == 0 {
            current_pct - 10
        } else {
            (current_pct / 10) * 10
        }
        .clamp(30, 250);

        let new_zoom = (next_pct as f32) / 100.0_f32;
        if (new_zoom - self.canvas_zoom).abs() > 0.001_f32 {
            self.canvas_pan =
                focal_point - (focal_point - self.canvas_pan) * (new_zoom / self.canvas_zoom);
            self.canvas_zoom = new_zoom;
        }
    }

    pub fn zoom_reset(&mut self, focal_point: Vec2) {
        let new_zoom = 1.0_f32;
        self.canvas_pan =
            focal_point - (focal_point - self.canvas_pan) * (new_zoom / self.canvas_zoom);
        self.canvas_zoom = new_zoom;
    }

    pub fn snapshot_for_undo(&mut self) {
        if self.undo_stack.last() != Some(&self.active_algo) {
            self.undo_stack.push(self.active_algo.clone());
            if self.undo_stack.len() > MAX_UNDO_DEPTH {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.active_algo.clone());
            self.active_algo = prev;
            self.selected_step_ids
                .retain(|id| self.active_algo.steps.iter().any(|s| s.id == *id));
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.active_algo.clone());
            self.active_algo = next;
            self.selected_step_ids
                .retain(|id| self.active_algo.steps.iter().any(|s| s.id == *id));
        }
    }

    pub fn update_search_query(&mut self) {
        let q = self.search_query.trim().to_lowercase();
        if q.is_empty() {
            self.search_results.clear();
            self.current_search_idx = 0;
            return;
        }

        self.search_results = self
            .active_algo
            .steps
            .iter()
            .filter(|s| {
                s.title.to_lowercase().contains(&q) || s.description.to_lowercase().contains(&q)
            })
            .map(|s| s.id)
            .collect();

        if self.current_search_idx >= self.search_results.len() {
            self.current_search_idx = 0;
        }
    }

    pub fn jump_to_current_search_match(&mut self, view_size: Vec2) {
        if let Some(&step_id) = self.search_results.get(self.current_search_idx) {
            self.selected_step_ids.clear();
            self.selected_step_ids.insert(step_id);
            self.focus_step_on_canvas(step_id, view_size);
        }
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.last_saved_algo.as_ref() != Some(&self.active_algo)
    }

    pub fn set_status(&mut self, text: String) {
        self.status_msg = Some((text, Instant::now()));
    }

    pub fn focus_step_on_canvas(&mut self, step_id: Uuid, view_size: Vec2) {
        if let Some(step) = self.active_algo.steps.iter().find(|s| s.id == step_id) {
            let target_center_x = (step.pos[0] + 105.0_f32) * self.canvas_zoom;
            let target_center_y = (step.pos[1] + 50.0_f32) * self.canvas_zoom;
            let dest_x = (view_size.x * 0.5_f32) - target_center_x;
            let dest_y = (view_size.y * 0.5_f32) - target_center_y;
            self.target_pan = Some(Vec2::new(dest_x, dest_y));
        }
    }

    pub fn fit_to_view(&mut self, view_size: Vec2) {
        if self.active_algo.steps.is_empty() {
            return;
        }

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

        let content_w = (max_x - min_x) + 120.0_f32;
        let content_h = (max_y - min_y) + 120.0_f32;

        let zoom_x = (view_size.x / content_w).clamp(0.35_f32, 1.5_f32);
        let zoom_y = (view_size.y / content_h).clamp(0.35_f32, 1.5_f32);
        self.canvas_zoom = zoom_x.min(zoom_y);

        let center_x = ((min_x + max_x) * 0.5_f32) * self.canvas_zoom;
        let center_y = ((min_y + max_y) * 0.5_f32) * self.canvas_zoom;

        self.canvas_pan.x = (view_size.x * 0.5_f32) - center_x;
        self.canvas_pan.y = (view_size.y * 0.5_f32) - center_y;
        self.target_pan = None;
    }

    pub fn duplicate_selected(&mut self) {
        if self.selected_step_ids.is_empty() {
            return;
        }
        self.snapshot_for_undo();

        let mut new_selection = HashSet::new();
        let old_steps: Vec<Step> = self
            .active_algo
            .steps
            .iter()
            .filter(|s| self.selected_step_ids.contains(&s.id))
            .cloned()
            .collect();

        for source in old_steps {
            let new_title = self.settings.language.copy_suffix(&source.title);
            let mut new_pos = [source.pos[0] + 35.0_f32, source.pos[1] + 35.0_f32];
            if self.settings.snap_to_grid {
                new_pos[0] = (new_pos[0] / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                new_pos[1] = (new_pos[1] / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
            }

            let mut new_step = Step::new(new_title, new_pos);
            new_step.description = source.description;
            new_step.kind = source.kind;
            new_step.options = source.options;
            new_step.image_path = source.image_path;

            let new_id = new_step.id;
            new_selection.insert(new_id);
            self.active_algo.steps.push(new_step);
        }

        self.selected_step_ids = new_selection;
    }

    pub fn delete_selected(&mut self) {
        if self.selected_step_ids.is_empty() {
            return;
        }
        self.snapshot_for_undo();

        let to_delete = self.selected_step_ids.clone();
        self.active_algo
            .steps
            .retain(|s| !to_delete.contains(&s.id));

        for s in &mut self.active_algo.steps {
            for opt in &mut s.options {
                if opt.next_step_id.map_or(false, |id| to_delete.contains(&id)) {
                    opt.next_step_id = None;
                }
            }
            match &mut s.kind {
                StepKind::Measurement {
                    next_if_normal,
                    next_if_abnormal,
                    ..
                } => {
                    if next_if_normal.map_or(false, |id| to_delete.contains(&id)) {
                        *next_if_normal = None;
                    }
                    if next_if_abnormal.map_or(false, |id| to_delete.contains(&id)) {
                        *next_if_abnormal = None;
                    }
                }
                StepKind::SafetyWarning { next_step_id, .. } => {
                    if next_step_id.map_or(false, |id| to_delete.contains(&id)) {
                        *next_step_id = None;
                    }
                }
                _ => {}
            }
        }

        if self
            .active_algo
            .first_step_id
            .map_or(false, |id| to_delete.contains(&id))
        {
            self.active_algo.first_step_id = self.active_algo.steps.first().map(|s| s.id);
        }

        self.selected_step_ids.clear();
    }

    pub fn connect_nodes(&mut self, from_id: Uuid, to_id: Uuid) {
        if from_id == to_id {
            return;
        }
        self.snapshot_for_undo();
        let lang = self.settings.language;
        if let Some(source) = self.active_algo.steps.iter_mut().find(|s| s.id == from_id) {
            match &mut source.kind {
                StepKind::Standard => {
                    if let Some(opt) = source.options.iter_mut().find(|o| o.next_step_id.is_none())
                    {
                        opt.next_step_id = Some(to_id);
                    } else {
                        let opt_text = lang.default_option_link_text(source.options.len() + 1);
                        source.options.push(crate::models::OptionLink {
                            text: opt_text,
                            next_step_id: Some(to_id),
                        });
                    }
                }
                StepKind::Measurement {
                    next_if_normal,
                    next_if_abnormal,
                    ..
                } => {
                    if next_if_normal.is_none() {
                        *next_if_normal = Some(to_id);
                    } else if next_if_abnormal.is_none() {
                        *next_if_abnormal = Some(to_id);
                    } else {
                        *next_if_normal = Some(to_id);
                    }
                }
                StepKind::SafetyWarning { next_step_id, .. } => {
                    *next_step_id = Some(to_id);
                }
            }
        }
    }

    pub fn render_builder_mode(&mut self, ui: &mut egui::Ui) {
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

            // Проект и База
            island_frame.show(ui, |ui| {
                if ui.button(lang.btn_new()).clicked() {
                    self.snapshot_for_undo();
                    let mut new_algo = Algorithm::default();
                    new_algo.metadata.name = lang.default_equipment_name().to_string();
                    self.active_algo = new_algo.clone();
                    self.last_saved_algo = Some(new_algo);
                    self.selected_step_ids.clear();
                    self.canvas_pan = Vec2::new(120.0_f32, 100.0_f32);
                    self.target_pan = None;
                    self.canvas_zoom = 1.0_f32;
                }

                let save_btn_text = if self.has_unsaved_changes() {
                    format!("{} •", lang.btn_save())
                } else {
                    lang.btn_save().to_string()
                };

                if ui.button(save_btn_text).clicked() {
                    if save_algorithm_to_archive(&mut self.active_algo).is_ok() {
                        self.last_saved_algo = Some(self.active_algo.clone());
                        self.set_status(lang.save_success().to_string());
                        self.refresh_archive();
                    }
                }

                if ui.button(lang.btn_passport()).clicked() {
                    self.show_passport_modal = true;
                }
            });

            // Автономный Экспорт / Импорт файла JSON
            island_frame.show(ui, |ui| {
                if ui.button(lang.btn_import_json()).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON Procedure", &["json"])
                        .pick_file()
                    {
                        if let Ok(mut algo) = import_algorithm_from_file(&path) {
                            self.snapshot_for_undo();
                            if self.archive_list.iter().any(|a| a.id == algo.id) {
                                algo.id = Uuid::new_v4();
                            }
                            let _ = save_algorithm_to_archive(&mut algo);
                            self.active_algo = algo.clone();
                            self.last_saved_algo = Some(algo);
                            self.selected_step_ids.clear();
                            self.fit_to_view(view_size);
                            self.set_status(lang.import_json_success().to_string());
                            self.refresh_archive();
                        } else {
                            self.set_status(lang.import_json_err().to_string());
                        }
                    }
                }

                if ui.button(lang.btn_export_json()).clicked() {
                    let default_fname = format!(
                        "{}_{}.json",
                        self.active_algo.metadata.manufacturer.trim(),
                        self.active_algo.metadata.name.trim()
                    )
                    .replace(' ', "_");

                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON Procedure", &["json"])
                        .set_file_name(&default_fname)
                        .save_file()
                    {
                        if export_algorithm_to_file(&self.active_algo, &path).is_ok() {
                            self.set_status(lang.export_json_success().to_string());
                        }
                    }
                }
            });

            // История отмены/повтора
            island_frame.show(ui, |ui| {
                ui.add_enabled_ui(!self.undo_stack.is_empty(), |ui| {
                    if ui.button("↩").on_hover_text(lang.btn_undo()).clicked() {
                        self.undo();
                    }
                });
                ui.add_enabled_ui(!self.redo_stack.is_empty(), |ui| {
                    if ui.button("↪").on_hover_text(lang.btn_redo()).clicked() {
                        self.redo();
                    }
                });
            });

            // Поиск и Экспорт PNG
            island_frame.show(ui, |ui| {
                if ui.button(lang.btn_search()).clicked() {
                    self.show_search_bar = !self.show_search_bar;
                    if self.show_search_bar {
                        ui.ctx()
                            .memory_mut(|m| m.request_focus(egui::Id::new("canvas_search_input")));
                    }
                }

                if ui.button(lang.btn_export_png()).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PNG Image", &["png"])
                        .set_file_name("flowchart.png")
                        .save_file()
                    {
                        self.pending_png_path = Some(path);
                        self.fit_to_view(view_size);
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Screenshot);
                    }
                }
            });

            // Кнопка добавления шага
            let add_btn = egui::Button::new(
                egui::RichText::new(lang.add_block())
                    .color(Color32::WHITE)
                    .strong(),
            )
            .fill(Color32::from_rgb(37, 99, 235))
            .rounding(6.0_f32)
            .min_size(Vec2::new(0.0_f32, 28.0_f32));

            if ui.add(add_btn).clicked() {
                self.snapshot_for_undo();
                let count = self.active_algo.steps.len() + 1;
                let mut new_pos = [
                    (-self.canvas_pan.x + 220.0_f32 + (count as f32 * 30.0_f32)) / self.canvas_zoom,
                    (-self.canvas_pan.y + 120.0_f32 + (count as f32 * 30.0_f32)) / self.canvas_zoom,
                ];
                if self.settings.snap_to_grid {
                    new_pos[0] = (new_pos[0] / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                    new_pos[1] = (new_pos[1] / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
                }

                let title = lang.step_default_name(count);
                let step = Step::new(title, new_pos);
                let id = step.id;
                if self.active_algo.first_step_id.is_none() {
                    self.active_algo.first_step_id = Some(id);
                }
                self.active_algo.steps.push(step);
                self.selected_step_ids.clear();
                self.selected_step_ids.insert(id);
            }

            // Масштабирование, Сетка и Векторный стиль связей
            island_frame.show(ui, |ui| {
                let btn_size = Vec2::splat(22.0_f32);

                // Векторная иконка сетки (3x3 пина)
                let snap_active = self.settings.snap_to_grid;
                let (snap_rect, snap_resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());
                let snap_hovered = snap_resp.hovered();

                let snap_bg = if snap_active {
                    if is_dark {
                        Color32::from_rgb(30, 48, 76)
                    } else {
                        Color32::from_rgb(220, 235, 255)
                    }
                } else if snap_hovered {
                    if is_dark {
                        Color32::from_rgb(34, 42, 56)
                    } else {
                        Color32::from_rgb(238, 242, 248)
                    }
                } else {
                    Color32::TRANSPARENT
                };

                ui.painter().rect_filled(snap_rect, 4.0_f32, snap_bg);
                if snap_active {
                    ui.painter().rect_stroke(
                        snap_rect,
                        4.0_f32,
                        Stroke::new(1.0_f32, Color32::from_rgb(37, 99, 235)),
                    );
                }

                let dot_c = if snap_active {
                    Color32::from_rgb(37, 99, 235)
                } else if is_dark {
                    Color32::from_rgb(130, 142, 160)
                } else {
                    Color32::from_rgb(100, 116, 139)
                };

                let center = snap_rect.center();
                let pin_spacing = 4.5_f32;
                for dx in &[-1.0_f32, 0.0_f32, 1.0_f32] {
                    for dy in &[-1.0_f32, 0.0_f32, 1.0_f32] {
                        let pos = center + Vec2::new(*dx * pin_spacing, *dy * pin_spacing);
                        ui.painter().circle_filled(pos, 1.1_f32, dot_c);
                    }
                }

                let snap_tip = if snap_active {
                    lang.snap_to_grid_on()
                } else {
                    lang.snap_to_grid_off()
                };
                if snap_resp.on_hover_text(snap_tip).clicked() {
                    self.settings.snap_to_grid = !self.settings.snap_to_grid;
                    self.settings.save();
                }

                ui.add_space(2.0_f32);

                // Векторная иконка стиля связи
                let (wire_rect, wire_resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());
                let wire_hovered = wire_resp.hovered();

                let wire_bg = if wire_hovered {
                    if is_dark {
                        Color32::from_rgb(34, 42, 56)
                    } else {
                        Color32::from_rgb(238, 242, 248)
                    }
                } else {
                    Color32::TRANSPARENT
                };

                ui.painter().rect_filled(wire_rect, 4.0_f32, wire_bg);
                let wire_color = Color32::from_rgb(37, 99, 235);
                let wire_stroke = Stroke::new(1.6_f32, wire_color);

                match self.settings.default_wire_style {
                    WireStyle::Orthogonal => {
                        let p_start = wire_rect.left_top() + Vec2::new(5.0_f32, 5.0_f32);
                        let p_corner = Pos2::new(p_start.x, wire_rect.bottom() - 5.0_f32);
                        let p_end = Pos2::new(wire_rect.right() - 5.0_f32, p_corner.y);

                        ui.painter().line_segment([p_start, p_corner], wire_stroke);
                        ui.painter().line_segment([p_corner, p_end], wire_stroke);
                        ui.painter().circle_filled(p_start, 2.0_f32, wire_color);
                        ui.painter().line_segment(
                            [p_end, p_end + Vec2::new(-3.0_f32, -3.0_f32)],
                            wire_stroke,
                        );
                        ui.painter().line_segment(
                            [p_end, p_end + Vec2::new(-3.0_f32, 3.0_f32)],
                            wire_stroke,
                        );
                    }
                    WireStyle::Curved => {
                        let p_start = wire_rect.left_top() + Vec2::new(5.0_f32, 6.0_f32);
                        let p_end = wire_rect.right_bottom() - Vec2::new(5.0_f32, 6.0_f32);
                        let cp1 = Pos2::new(p_start.x, p_end.y);
                        let cp2 = Pos2::new(p_end.x, p_start.y);

                        let b_shape = CubicBezierShape::from_points_stroke(
                            [p_start, cp1, cp2, p_end],
                            false,
                            Color32::TRANSPARENT,
                            wire_stroke,
                        );
                        ui.painter().add(b_shape);
                        ui.painter().circle_filled(p_start, 2.0_f32, wire_color);
                    }
                }

                if wire_resp
                    .on_hover_text(lang.wire_style_toggle_tip())
                    .clicked()
                {
                    self.settings.default_wire_style = match self.settings.default_wire_style {
                        WireStyle::Curved => WireStyle::Orthogonal,
                        WireStyle::Orthogonal => WireStyle::Curved,
                    };
                    self.settings.save();
                }

                ui.separator();

                let center_focal =
                    Pos2::new(view_size.x * 0.5_f32, view_size.y * 0.5_f32).to_vec2();

                if ui.button("➖").on_hover_text(lang.zoom_out_tip()).clicked() {
                    self.zoom_by_step(false, center_focal);
                }

                if ui
                    .button(format!("{:.0}%", (self.canvas_zoom * 100.0_f32).round()))
                    .on_hover_text(lang.zoom_reset_tip())
                    .clicked()
                {
                    self.zoom_reset(center_focal);
                }

                if ui.button("➕").on_hover_text(lang.zoom_in_tip()).clicked() {
                    self.zoom_by_step(true, center_focal);
                }

                if ui.button(lang.fit_view()).clicked() {
                    self.fit_to_view(view_size);
                }
            });

            if let Some((msg, time)) = &self.status_msg {
                if time.elapsed().as_secs() < 3 {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(16, 185, 129), msg);
                }
            }
        });
        ui.separator();

        let wants_kb = ui.ctx().wants_keyboard_input();
        if !wants_kb && ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.delete_selected();
        }

        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(340.0_f32)
            .width_range(290.0_f32..=480.0_f32)
            .show_inside(ui, |ui| {
                self.render_inspector(ui);
            });

        self.render_canvas(ui, false);
    }
}

impl eframe::App for AlgoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.settings.language;
        let is_dark = self.settings.theme == AppTheme::Dark;

        // Плавная интерполяция панорамирования камеры
        if let Some(target) = self.target_pan {
            let dt = ctx.input(|i| i.stable_dt).min(0.05_f32);
            let t = 1.0_f32 - (-14.0_f32 * dt).exp();
            self.canvas_pan += (target - self.canvas_pan) * t;

            if (self.canvas_pan - target).length_sq() < 0.4_f32 {
                self.canvas_pan = target;
                self.target_pan = None;
            }
            ctx.request_repaint();
        }

        // Сохранение скриншота холста в файл PNG
        for event in &ctx.input(|i| i.raw.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Some(path) = self.pending_png_path.take() {
                    let ppp = ctx.pixels_per_point();
                    let full_w = image.width();
                    let full_h = image.height();

                    let (crop_x, crop_y, crop_w, crop_h) = if let Some(rect) = self.last_canvas_rect
                    {
                        let min_x = ((rect.min.x * ppp).round() as usize).min(full_w);
                        let min_y = ((rect.min.y * ppp).round() as usize).min(full_h);
                        let max_x = ((rect.max.x * ppp).round() as usize).min(full_w);
                        let max_y = ((rect.max.y * ppp).round() as usize).min(full_h);
                        let w = max_x.saturating_sub(min_x);
                        let h = max_y.saturating_sub(min_y);
                        if w > 20 && h > 20 {
                            (min_x, min_y, w, h)
                        } else {
                            (0, 0, full_w, full_h)
                        }
                    } else {
                        (0, 0, full_w, full_h)
                    };

                    let mut raw_bytes = Vec::with_capacity(crop_w * crop_h * 4);
                    for y in crop_y..(crop_y + crop_h) {
                        for x in crop_x..(crop_x + crop_w) {
                            let color = image[(x, y)];
                            raw_bytes.extend_from_slice(&color.to_array());
                        }
                    }

                    if image::save_buffer_with_format(
                        &path,
                        &raw_bytes,
                        crop_w as u32,
                        crop_h as u32,
                        image::ColorType::Rgba8,
                        image::ImageFormat::Png,
                    )
                    .is_ok()
                    {
                        self.set_status(lang.export_png_success().to_string());
                    }
                }
            }
        }

        // Глобальные горячие клавиши
        if !ctx.wants_keyboard_input() && self.current_tab == "Конструктор" {
            let ctrl = ctx.input(|i| i.modifiers.command);
            let shift = ctx.input(|i| i.modifiers.shift);

            if ctrl && ctx.input(|i| i.key_pressed(egui::Key::Z)) {
                if shift {
                    self.redo();
                } else {
                    self.undo();
                }
            } else if ctrl && ctx.input(|i| i.key_pressed(egui::Key::Y)) {
                self.redo();
            } else if ctrl && ctx.input(|i| i.key_pressed(egui::Key::F)) {
                self.show_search_bar = true;
                ctx.memory_mut(|m| m.request_focus(egui::Id::new("canvas_search_input")));
            }
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.force_close && self.has_unsaved_changes() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_close_modal = true;
            }
        }

        // Верхняя панель переключения режимов
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(2.0_f32);
            egui::menu::bar(ui, |ui| {
                let tabs_bg = if is_dark {
                    Color32::from_rgb(18, 22, 29)
                } else {
                    Color32::from_rgb(226, 232, 240)
                };
                let tabs_frame = egui::Frame::none()
                    .fill(tabs_bg)
                    .rounding(6.0_f32)
                    .inner_margin(egui::Margin::symmetric(3.0_f32, 3.0_f32));

                tabs_frame.show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.0_f32, 0.0_f32);

                    let draw_tab =
                        |ui: &mut egui::Ui, is_active: bool, text: &str| -> egui::Response {
                            let text_color = if is_active {
                                Color32::WHITE
                            } else if is_dark {
                                Color32::from_rgb(150, 160, 175)
                            } else {
                                Color32::from_rgb(71, 85, 105)
                            };
                            let fill = if is_active {
                                Color32::from_rgb(37, 99, 235)
                            } else {
                                Color32::TRANSPARENT
                            };
                            let btn = egui::Button::new(
                                egui::RichText::new(text).color(text_color).strong(),
                            )
                            .fill(fill)
                            .rounding(5.0_f32)
                            .min_size(Vec2::new(0.0_f32, 26.0_f32));
                            ui.add(btn)
                        };

                    if draw_tab(ui, self.current_tab == "Конструктор", lang.tab_builder()).clicked()
                    {
                        self.current_tab = "Конструктор".to_string();
                    }
                    if draw_tab(ui, self.current_tab == "Диагностика", lang.tab_runner()).clicked()
                    {
                        self.reset_runner();
                        self.current_tab = "Диагностика".to_string();
                    }
                    if draw_tab(ui, self.current_tab == "Архив", lang.tab_archive()).clicked()
                    {
                        self.refresh_archive();
                        self.current_tab = "Архив".to_string();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut current_lang = self.settings.language;
                    let label = match current_lang {
                        Language::Ru => "🇷🇺 RU",
                        Language::En => "🇬🇧 EN",
                    };

                    egui::ComboBox::from_id_source("lang_selector")
                        .selected_text(label)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(&mut current_lang, Language::Ru, "🇷🇺 Русский")
                                .clicked()
                                || ui
                                    .selectable_value(&mut current_lang, Language::En, "🇬🇧 English")
                                    .clicked()
                            {
                                self.settings.language = current_lang;
                                self.settings.save();
                            }
                        });

                    ui.separator();

                    let (theme_icon, next_theme, tooltip) = match self.settings.theme {
                        AppTheme::Dark => ("🌙", AppTheme::Light, lang.theme_tooltip_to_light()),
                        AppTheme::Light => ("☀", AppTheme::Dark, lang.theme_tooltip_to_dark()),
                    };

                    if ui.button(theme_icon).on_hover_text(tooltip).clicked() {
                        self.settings.theme = next_theme;
                        configure_app_visuals(ctx, next_theme);
                        self.settings.save();
                    }
                });
            });
            ui.add_space(2.0_f32);
        });

        // Отрисовка модальных окон
        self.render_modals(ctx);

        // Центральная панель выбранного режима
        egui::CentralPanel::default().show(ctx, |ui| match self.current_tab.as_str() {
            "Конструктор" => self.render_builder_mode(ui),
            "Диагностика" => self.render_runner_mode(ui),
            "Архив" => self.render_archive_mode(ui),
            _ => {}
        });
    }
}
