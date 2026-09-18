use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Language {
    Ru,
    En,
}

impl Language {
    // Вкладки
    pub fn tab_builder(&self) -> &'static str {
        match self {
            Language::Ru => "🕸 Конструктор",
            Language::En => "🕸 Flowchart",
        }
    }
    pub fn tab_runner(&self) -> &'static str {
        match self {
            Language::Ru => "▶ Диагностика",
            Language::En => "▶ Diagnostics",
        }
    }
    pub fn tab_archive(&self) -> &'static str {
        match self {
            Language::Ru => "📚 База регламентов",
            Language::En => "📚 Procedures",
        }
    }

    // Тулбар
    pub fn btn_new(&self) -> &'static str {
        match self {
            Language::Ru => "📄 Новый",
            Language::En => "📄 New",
        }
    }
    pub fn btn_save(&self) -> &'static str {
        match self {
            Language::Ru => "💾 Сохранить",
            Language::En => "💾 Save",
        }
    }
    pub fn btn_passport(&self) -> &'static str {
        match self {
            Language::Ru => "📋 Паспорт",
            Language::En => "📋 Passport",
        }
    }
    pub fn btn_undo(&self) -> &'static str {
        match self {
            Language::Ru => "Отменить (Ctrl+Z)",
            Language::En => "Undo (Ctrl+Z)",
        }
    }
    pub fn btn_redo(&self) -> &'static str {
        match self {
            Language::Ru => "Повторить (Ctrl+Y)",
            Language::En => "Redo (Ctrl+Y)",
        }
    }
    pub fn btn_search(&self) -> &'static str {
        match self {
            Language::Ru => "🔍 Поиск (Ctrl+F)",
            Language::En => "🔍 Search (Ctrl+F)",
        }
    }
    pub fn btn_export_png(&self) -> &'static str {
        match self {
            Language::Ru => "📷 Экспорт PNG",
            Language::En => "📷 Export PNG",
        }
    }
    pub fn add_block(&self) -> &'static str {
        match self {
            Language::Ru => "➕ Добавить блок",
            Language::En => "➕ Add Step",
        }
    }
    pub fn fit_view(&self) -> &'static str {
        match self {
            Language::Ru => "🔍 Вписать всё",
            Language::En => "🔍 Fit View",
        }
    }
    pub fn zoom_in_tip(&self) -> &'static str {
        match self {
            Language::Ru => "Приблизить",
            Language::En => "Zoom In",
        }
    }
    pub fn zoom_out_tip(&self) -> &'static str {
        match self {
            Language::Ru => "Отдалить",
            Language::En => "Zoom Out",
        }
    }
    pub fn zoom_reset_tip(&self) -> &'static str {
        match self {
            Language::Ru => "Сбросить масштаб на 100%",
            Language::En => "Reset zoom to 100%",
        }
    }

    // Префиксы и общие слова
    pub fn tolerance_prefix(&self) -> &'static str {
        match self {
            Language::Ru => "Допуск:",
            Language::En => "Tolerance:",
        }
    }
    pub fn untitled(&self) -> &'static str {
        match self {
            Language::Ru => "Без названия",
            Language::En => "Untitled",
        }
    }
    pub fn equipment_catalog(&self) -> &'static str {
        match self {
            Language::Ru => "Каталог оборудования:",
            Language::En => "Equipment Catalog:",
        }
    }
    pub fn model_prefix(&self) -> &'static str {
        match self {
            Language::Ru => "Модель:",
            Language::En => "Model:",
        }
    }
    pub fn inv_prefix(&self) -> &'static str {
        match self {
            Language::Ru => "Инв. №:",
            Language::En => "Inv. #:",
        }
    }
    pub fn author_prefix(&self) -> &'static str {
        match self {
            Language::Ru => "Автор:",
            Language::En => "Author:",
        }
    }
    pub fn default_equipment_name(&self) -> &'static str {
        match self {
            Language::Ru => "Новое оборудование",
            Language::En => "New Equipment",
        }
    }
    pub fn default_unit(&self) -> &'static str {
        match self {
            Language::Ru => "В",
            Language::En => "V",
        }
    }
    pub fn default_safety_ack(&self) -> &'static str {
        match self {
            Language::Ru => "Оборудование обесточено",
            Language::En => "Equipment de-energized",
        }
    }
    pub fn default_branch_yes(&self) -> &'static str {
        match self {
            Language::Ru => "Да",
            Language::En => "Yes",
        }
    }

    pub fn step_default_name(&self, count: usize) -> String {
        match self {
            Language::Ru => format!("Шаг {}", count),
            Language::En => format!("Step {}", count),
        }
    }

    pub fn copy_suffix(&self, title: &str) -> String {
        match self {
            Language::Ru => format!("{} (копия)", title),
            Language::En => format!("{} (copy)", title),
        }
    }

    pub fn default_option_link_text(&self, index: usize) -> String {
        match self {
            Language::Ru => format!("Переход {}", index),
            Language::En => format!("Option {}", index),
        }
    }

    // Метки стрелок связей
    pub fn link_normal(&self) -> &'static str {
        match self {
            Language::Ru => "Норма",
            Language::En => "Normal",
        }
    }
    pub fn link_abnormal(&self) -> &'static str {
        match self {
            Language::Ru => "Отклонение",
            Language::En => "Abnormal",
        }
    }
    pub fn link_safety(&self) -> &'static str {
        match self {
            Language::Ru => "Ознакомлен",
            Language::En => "Acknowledged",
        }
    }

    // Секции инспектора
    pub fn section_general(&self) -> &'static str {
        match self {
            Language::Ru => "Основные параметры",
            Language::En => "General Properties",
        }
    }
    pub fn section_instructions(&self) -> &'static str {
        match self {
            Language::Ru => "Инструкция специалисту",
            Language::En => "Operator Instructions",
        }
    }
    pub fn section_branching(&self) -> &'static str {
        match self {
            Language::Ru => "Ветвления и логика",
            Language::En => "Branching & Logic",
        }
    }

    // Empty State холста
    pub fn empty_canvas_title(&self) -> &'static str {
        match self {
            Language::Ru => "Схема алгоритма пуста",
            Language::En => "Algorithm Flowchart is Empty",
        }
    }
    pub fn empty_canvas_hint(&self) -> &'static str {
        match self {
            Language::Ru => "Дважды кликните по холсту или нажмите кнопку ниже",
            Language::En => "Double-click on the canvas or press the button below",
        }
    }
    pub fn empty_canvas_btn(&self) -> &'static str {
        match self {
            Language::Ru => "➕ Создать первый блок",
            Language::En => "➕ Create Initial Step",
        }
    }

    // Справка инспектора
    pub fn inspector_empty_title(&self) -> &'static str {
        match self {
            Language::Ru => "Блок не выбран",
            Language::En => "No Block Selected",
        }
    }
    pub fn inspector_empty_desc(&self) -> &'static str {
        match self {
            Language::Ru => "Кликните по любому блоку на схеме для настройки инструкций, замеров и логики переходов.",
            Language::En => "Click any block on the canvas to configure instructions, measurements, and branching.",
        }
    }
    pub fn cheat_sheet_title(&self) -> &'static str {
        match self {
            Language::Ru => "Горячие клавиши и управление:",
            Language::En => "Shortcuts & Navigation:",
        }
    }

    // Клавиши для шпаргалки
    pub fn key_lmb(&self) -> &'static str {
        match self {
            Language::Ru => "ЛКМ",
            Language::En => "LMB",
        }
    }
    pub fn key_rmb(&self) -> &'static str {
        match self {
            Language::Ru => "ПКМ",
            Language::En => "RMB",
        }
    }
    pub fn key_mmb(&self) -> &'static str {
        match self {
            Language::Ru => "СКМ",
            Language::En => "MMB",
        }
    }
    pub fn key_dot(&self) -> &'static str {
        match self {
            Language::Ru => "Точка",
            Language::En => "Dot",
        }
    }
    pub fn key_badge(&self) -> &'static str {
        match self {
            Language::Ru => "Плашка",
            Language::En => "Badge",
        }
    }
    pub fn key_wire(&self) -> &'static str {
        match self {
            Language::Ru => "Связь",
            Language::En => "Wire",
        }
    }

    pub fn cheat_create_node(&self) -> &'static str {
        match self {
            Language::Ru => "Создать блок",
            Language::En => "Create step",
        }
    }
    pub fn cheat_drag_wire(&self) -> &'static str {
        match self {
            Language::Ru => "Тянуть связь",
            Language::En => "Connect wire",
        }
    }
    pub fn cheat_cut_wire(&self) -> &'static str {
        match self {
            Language::Ru => "Разорвать связь",
            Language::En => "Disconnect wire",
        }
    }
    pub fn cheat_toggle_wire(&self) -> &'static str {
        match self {
            Language::Ru => "Сменить стиль (90° / кривая)",
            Language::En => "Toggle style (90° / curved)",
        }
    }
    pub fn cheat_multiselect(&self) -> &'static str {
        match self {
            Language::Ru => "Группа блоков",
            Language::En => "Multi-select",
        }
    }
    pub fn cheat_del_selected(&self) -> &'static str {
        match self {
            Language::Ru => "Удалить выделенное",
            Language::En => "Delete selected",
        }
    }
    pub fn cheat_undo_act(&self) -> &'static str {
        match self {
            Language::Ru => "Отмена действия",
            Language::En => "Undo action",
        }
    }
    pub fn cheat_redo_act(&self) -> &'static str {
        match self {
            Language::Ru => "Повтор действия",
            Language::En => "Redo action",
        }
    }
    pub fn cheat_search_act(&self) -> &'static str {
        match self {
            Language::Ru => "Быстрый поиск",
            Language::En => "Quick search",
        }
    }
    pub fn cheat_pan_act(&self) -> &'static str {
        match self {
            Language::Ru => "Панорама холста",
            Language::En => "Pan canvas",
        }
    }

    // Магнитная сетка и автономный JSON
    pub fn snap_to_grid_on(&self) -> &'static str {
        match self {
            Language::Ru => "Магнитная сетка: Вкл (клик для выкл)",
            Language::En => "Snap to grid: On (click to disable)",
        }
    }
    pub fn snap_to_grid_off(&self) -> &'static str {
        match self {
            Language::Ru => "Магнитная сетка: Выкл (клик для вкл)",
            Language::En => "Snap to grid: Off (click to enable)",
        }
    }
    pub fn btn_export_json(&self) -> &'static str {
        match self {
            Language::Ru => "📤 Экспорт файла",
            Language::En => "📤 Export File",
        }
    }
    pub fn btn_import_json(&self) -> &'static str {
        match self {
            Language::Ru => "📥 Импорт файла",
            Language::En => "📥 Import File",
        }
    }
    pub fn export_json_success(&self) -> &'static str {
        match self {
            Language::Ru => "✔ Регламент выгружен в JSON!",
            Language::En => "✔ Procedure exported to JSON!",
        }
    }
    pub fn import_json_success(&self) -> &'static str {
        match self {
            Language::Ru => "✔ Регламент успешно импортирован!",
            Language::En => "✔ Procedure imported successfully!",
        }
    }
    pub fn import_json_err(&self) -> &'static str {
        match self {
            Language::Ru => "❌ Ошибка: повреждённый файл JSON",
            Language::En => "❌ Error: Invalid procedure JSON file",
        }
    }

    // Стиль связей
    pub fn wire_style_toggle_tip(&self) -> &'static str {
        match self {
            Language::Ru => {
                "Стиль связей по умолчанию (ПКМ по стрелке переключает конкретную связь)"
            }
            Language::En => "Default wire routing (RMB on a wire toggles individual style)",
        }
    }
    pub fn status_wire_style_curved(&self) -> &'static str {
        match self {
            Language::Ru => "✔ Стиль связи: Плавная кривая",
            Language::En => "✔ Wire style: Curved",
        }
    }
    pub fn status_wire_style_ortho(&self) -> &'static str {
        match self {
            Language::Ru => "✔ Стиль связи: Прямой угол (90°)",
            Language::En => "✔ Wire style: Orthogonal (90°)",
        }
    }

    // Миникарта
    pub fn minimap_links_on(&self) -> &'static str {
        match self {
            Language::Ru => "Связи на миникарте: Вкл (клик для выкл)",
            Language::En => "Minimap links: On (click to disable)",
        }
    }
    pub fn minimap_links_off(&self) -> &'static str {
        match self {
            Language::Ru => "Связи на миникарте: Выкл (клик для вкл)",
            Language::En => "Minimap links: Off (click to enable)",
        }
    }

    // Поиск
    pub fn search_nodes_placeholder(&self) -> &'static str {
        match self {
            Language::Ru => "Поиск по узлам...",
            Language::En => "Search steps...",
        }
    }
    pub fn search_no_matches(&self) -> &'static str {
        match self {
            Language::Ru => "Не найдено",
            Language::En => "No matches",
        }
    }
    pub fn export_png_success(&self) -> &'static str {
        match self {
            Language::Ru => "✔ Схема сохранена в PNG!",
            Language::En => "✔ Flowchart exported to PNG!",
        }
    }

    // Темы
    pub fn theme_tooltip_to_light(&self) -> &'static str {
        match self {
            Language::Ru => "Переключить на светлую тему",
            Language::En => "Switch to Light Theme",
        }
    }
    pub fn theme_tooltip_to_dark(&self) -> &'static str {
        match self {
            Language::Ru => "Переключить на темную тему",
            Language::En => "Switch to Dark Theme",
        }
    }

    // Линтер
    pub fn lint_unreachable(&self) -> &'static str {
        match self {
            Language::Ru => "⚠ Узел недостижим (нет входящих связей)",
            Language::En => "⚠ Unreachable step (no incoming connections)",
        }
    }
    pub fn lint_incomplete(&self) -> &'static str {
        match self {
            Language::Ru => "⚠ Не все ветвления соединены с продолжением",
            Language::En => "⚠ Incomplete branches (missing target node)",
        }
    }

    // Контекстное меню ПКМ
    pub fn ctx_make_start(&self) -> &'static str {
        match self {
            Language::Ru => "🚩 Сделать стартовым",
            Language::En => "🚩 Set as Start Node",
        }
    }
    pub fn ctx_duplicate(&self) -> &'static str {
        match self {
            Language::Ru => "📋 Дублировать",
            Language::En => "📋 Duplicate",
        }
    }
    pub fn ctx_delete(&self) -> &'static str {
        match self {
            Language::Ru => "🗑 Удалить",
            Language::En => "🗑 Delete",
        }
    }
    pub fn ctx_add_here(&self) -> &'static str {
        match self {
            Language::Ru => "➕ Добавить блок здесь",
            Language::En => "➕ Add Step Here",
        }
    }

    // Окно подтверждения выхода
    pub fn confirm_close_title(&self) -> &'static str {
        match self {
            Language::Ru => "⚠ Несохраненные изменения",
            Language::En => "⚠ Unsaved Changes",
        }
    }
    pub fn confirm_close_msg(&self) -> &'static str {
        match self {
            Language::Ru => {
                "В текущем регламенте есть несохраненные изменения.\nСохранить их перед выходом?"
            }
            Language::En => {
                "There are unsaved changes in the current procedure.\nSave before exiting?"
            }
        }
    }
    pub fn btn_save_and_exit(&self) -> &'static str {
        match self {
            Language::Ru => "💾 Сохранить и выйти",
            Language::En => "💾 Save & Exit",
        }
    }
    pub fn btn_discard_and_exit(&self) -> &'static str {
        match self {
            Language::Ru => "Выйти без сохранения",
            Language::En => "Exit without Saving",
        }
    }
    pub fn btn_cancel(&self) -> &'static str {
        match self {
            Language::Ru => "Отмена",
            Language::En => "Cancel",
        }
    }

    // Подтверждение удаления
    pub fn confirm_delete_title(&self) -> &'static str {
        match self {
            Language::Ru => "🗑 Подтверждение удаления",
            Language::En => "🗑 Confirm Deletion",
        }
    }
    pub fn confirm_delete_msg(&self) -> &'static str {
        match self {
            Language::Ru => "Вы действительно хотите удалить этот регламент?",
            Language::En => "Are you sure you want to delete this procedure?",
        }
    }

    // Паспорт оборудования
    pub fn passport_window_title(&self) -> &'static str {
        match self {
            Language::Ru => "Паспорт оборудования (Метаданные)",
            Language::En => "Equipment Passport (Metadata)",
        }
    }
    pub fn passport_desc(&self) -> &'static str {
        match self {
            Language::Ru => "Идентификационные данные станка и технологической карты",
            Language::En => "Machine identification and procedure metadata",
        }
    }
    pub fn passport_sec_equipment(&self) -> &'static str {
        match self {
            Language::Ru => "⚙ Данные оборудования",
            Language::En => "⚙ Equipment Details",
        }
    }
    pub fn passport_sec_doc(&self) -> &'static str {
        match self {
            Language::Ru => "📄 Сведения о регламенте",
            Language::En => "📄 Procedure Details",
        }
    }
    pub fn field_name(&self) -> &'static str {
        match self {
            Language::Ru => "Название регламента / узла:",
            Language::En => "Procedure / Node Name:",
        }
    }
    pub fn field_manufacturer(&self) -> &'static str {
        match self {
            Language::Ru => "Производитель / Бренд:",
            Language::En => "Manufacturer / Brand:",
        }
    }
    pub fn field_model(&self) -> &'static str {
        match self {
            Language::Ru => "Модель / Станок:",
            Language::En => "Model / Machine:",
        }
    }
    pub fn field_type(&self) -> &'static str {
        match self {
            Language::Ru => "Тип оборудования:",
            Language::En => "Equipment Type:",
        }
    }
    pub fn field_inv(&self) -> &'static str {
        match self {
            Language::Ru => "Инвентарный №:",
            Language::En => "Inventory #:",
        }
    }
    pub fn field_author(&self) -> &'static str {
        match self {
            Language::Ru => "Автор регламента:",
            Language::En => "Author:",
        }
    }
    pub fn field_updated_date(&self) -> &'static str {
        match self {
            Language::Ru => "Дата изменения:",
            Language::En => "Last Updated:",
        }
    }
    pub fn btn_apply(&self) -> &'static str {
        match self {
            Language::Ru => "✔ Применить",
            Language::En => "✔ Apply",
        }
    }
    pub fn save_success(&self) -> &'static str {
        match self {
            Language::Ru => "✔ Сохранено в архив!",
            Language::En => "✔ Saved to archive!",
        }
    }

    // Плейсхолдеры паспорта
    pub fn hint_manufacturer(&self) -> &'static str {
        match self {
            Language::Ru => "Haas, Siemens, DMG Mori...",
            Language::En => "Haas, Siemens, DMG Mori...",
        }
    }
    pub fn hint_model(&self) -> &'static str {
        match self {
            Language::Ru => "VF-2, CTX 310...",
            Language::En => "VF-2, CTX 310...",
        }
    }
    pub fn hint_type(&self) -> &'static str {
        match self {
            Language::Ru => "Фрезерный ЧПУ, Токарный...",
            Language::En => "CNC Milling, Lathe...",
        }
    }
    pub fn hint_inv(&self) -> &'static str {
        match self {
            Language::Ru => "ИНВ-04821...",
            Language::En => "INV-04821...",
        }
    }
    pub fn hint_name(&self) -> &'static str {
        match self {
            Language::Ru => "ТО шпиндельного узла...",
            Language::En => "Spindle Maintenance...",
        }
    }
    pub fn hint_author(&self) -> &'static str {
        match self {
            Language::Ru => "ФИО инженера...",
            Language::En => "Engineer Name...",
        }
    }

    // Каталог базы
    pub fn search_placeholder(&self) -> &'static str {
        match self {
            Language::Ru => "🔍 Поиск по названию, модели, номеру...",
            Language::En => "🔍 Search by name, model, number...",
        }
    }
    pub fn btn_refresh(&self) -> &'static str {
        match self {
            Language::Ru => "🔄 Обновить",
            Language::En => "🔄 Refresh",
        }
    }
    pub fn empty_archive(&self) -> &'static str {
        match self {
            Language::Ru => "В архиве пока нет регламентов. Создайте первый в конструкторе!",
            Language::En => "No procedures in library yet. Create one in the builder!",
        }
    }
    pub fn not_found(&self) -> &'static str {
        match self {
            Language::Ru => "В выбранной категории ничего не найдено.",
            Language::En => "Nothing found in selected category.",
        }
    }
    pub fn all_equipment(&self) -> &'static str {
        match self {
            Language::Ru => "Все производители",
            Language::En => "All Manufacturers",
        }
    }
    pub fn no_manufacturer(&self) -> &'static str {
        match self {
            Language::Ru => "Без бренда",
            Language::En => "Unspecified Brand",
        }
    }
    pub fn no_model(&self) -> &'static str {
        match self {
            Language::Ru => "Общее",
            Language::En => "General",
        }
    }
    pub fn card_steps(&self) -> &'static str {
        match self {
            Language::Ru => "шагов",
            Language::En => "steps",
        }
    }
    pub fn card_updated(&self) -> &'static str {
        match self {
            Language::Ru => "Изменен:",
            Language::En => "Updated:",
        }
    }
    pub fn btn_open_builder(&self) -> &'static str {
        match self {
            Language::Ru => "🛠 Редактировать",
            Language::En => "🛠 Edit",
        }
    }
    pub fn btn_run_diag(&self) -> &'static str {
        match self {
            Language::Ru => "▶ Диагностика",
            Language::En => "▶ Run",
        }
    }
    pub fn btn_delete_archive(&self) -> &'static str {
        match self {
            Language::Ru => "🗑 Удалить",
            Language::En => "🗑 Delete",
        }
    }

    // Бейджи блоков
    pub fn badge_choice(&self) -> &'static str {
        match self {
            Language::Ru => "ВЫБОР",
            Language::En => "CHOICE",
        }
    }
    pub fn badge_measure(&self) -> &'static str {
        match self {
            Language::Ru => "ЗАМЕР",
            Language::En => "MEASURE",
        }
    }
    pub fn badge_safety(&self) -> &'static str {
        match self {
            Language::Ru => "ТБ ⚠",
            Language::En => "SAFETY ⚠",
        }
    }
    pub fn badge_active(&self) -> &'static str {
        match self {
            Language::Ru => "АКТИВЕН ▶",
            Language::En => "ACTIVE ▶",
        }
    }
    pub fn badge_done(&self) -> &'static str {
        match self {
            Language::Ru => "ВЫПОЛНЕНО ✔",
            Language::En => "DONE ✔",
        }
    }
    pub fn no_desc(&self) -> &'static str {
        match self {
            Language::Ru => "Инструкция не задана",
            Language::En => "No instruction given",
        }
    }
    pub fn photo_attached(&self) -> &'static str {
        match self {
            Language::Ru => "🖼 Фото прикреплено",
            Language::En => "🖼 Photo attached",
        }
    }

    // Инспектор
    pub fn inspector_heading(&self) -> &'static str {
        match self {
            Language::Ru => "Параметры шага",
            Language::En => "Step Inspector",
        }
    }
    pub fn multi_select_hint(&self) -> &'static str {
        match self {
            Language::Ru => "Выделено блоков: ",
            Language::En => "Selected blocks: ",
        }
    }
    pub fn start_node(&self) -> &'static str {
        match self {
            Language::Ru => "🚩 Стартовый узел",
            Language::En => "🚩 Start Step",
        }
    }
    pub fn delete_btn(&self) -> &'static str {
        match self {
            Language::Ru => "🗑 Удалить блок",
            Language::En => "🗑 Delete Step",
        }
    }
    pub fn block_title(&self) -> &'static str {
        match self {
            Language::Ru => "Название шага:",
            Language::En => "Step Title:",
        }
    }
    pub fn behavior_type(&self) -> &'static str {
        match self {
            Language::Ru => "Тип операции:",
            Language::En => "Operation Type:",
        }
    }
    pub fn kind_standard(&self) -> &'static str {
        match self {
            Language::Ru => "Выбор вариантов действий",
            Language::En => "Standard Branching",
        }
    }
    pub fn kind_measurement(&self) -> &'static str {
        match self {
            Language::Ru => "Замер параметра (с допуском)",
            Language::En => "Measurement Check",
        }
    }
    pub fn kind_safety(&self) -> &'static str {
        match self {
            Language::Ru => "Требование безопасности (ТБ)",
            Language::En => "Safety Requirement",
        }
    }
    pub fn photo_btn(&self) -> &'static str {
        match self {
            Language::Ru => "📷 Прикрепить фото",
            Language::En => "📷 Attach Photo",
        }
    }
    pub fn add_branch(&self) -> &'static str {
        match self {
            Language::Ru => "➕ Добавить вариант",
            Language::En => "➕ Add Option",
        }
    }
    pub fn branch_text(&self) -> &'static str {
        match self {
            Language::Ru => "Текст:",
            Language::En => "Text:",
        }
    }
    pub fn finish_diag(&self) -> &'static str {
        match self {
            Language::Ru => "🏁 Завершить регламент",
            Language::En => "🏁 Finish Procedure",
        }
    }
    pub fn not_selected(&self) -> &'static str {
        match self {
            Language::Ru => "Не выбрано",
            Language::En => "Not Selected",
        }
    }
    pub fn unit(&self) -> &'static str {
        match self {
            Language::Ru => "Ед. изм:",
            Language::En => "Unit:",
        }
    }
    pub fn min_val(&self) -> &'static str {
        match self {
            Language::Ru => "Мин:",
            Language::En => "Min:",
        }
    }
    pub fn max_val(&self) -> &'static str {
        match self {
            Language::Ru => "Макс:",
            Language::En => "Max:",
        }
    }
    pub fn goto_normal(&self) -> &'static str {
        match self {
            Language::Ru => "➔ В норме перейти к:",
            Language::En => "➔ If normal go to:",
        }
    }
    pub fn goto_abnormal(&self) -> &'static str {
        match self {
            Language::Ru => "➔ При отклонении к:",
            Language::En => "➔ If abnormal go to:",
        }
    }
    pub fn safety_ack(&self) -> &'static str {
        match self {
            Language::Ru => "Текст чекбокса подтверждения:",
            Language::En => "Confirmation check text:",
        }
    }
    pub fn goto_after_ack(&self) -> &'static str {
        match self {
            Language::Ru => "➔ После подтверждения к:",
            Language::En => "➔ After confirmation go to:",
        }
    }

    // Пульт диагностики
    pub fn runner_heading(&self) -> &'static str {
        match self {
            Language::Ru => "Пульт диагностики",
            Language::En => "Diagnostic Console",
        }
    }
    pub fn step_back(&self) -> &'static str {
        match self {
            Language::Ru => "⬅ Назад",
            Language::En => "⬅ Back",
        }
    }
    pub fn reset_test(&self) -> &'static str {
        match self {
            Language::Ru => "🔄 Сброс",
            Language::En => "🔄 Reset",
        }
    }
    pub fn focus_step(&self) -> &'static str {
        match self {
            Language::Ru => "🎯 Центрировать",
            Language::En => "🎯 Center",
        }
    }
    pub fn route_history(&self) -> &'static str {
        match self {
            Language::Ru => "Маршрут проверки:",
            Language::En => "Audit Trail:",
        }
    }
    pub fn crumb_step(&self) -> &'static str {
        match self {
            Language::Ru => "Шаг",
            Language::En => "Step",
        }
    }
    pub fn crumb_current(&self) -> &'static str {
        match self {
            Language::Ru => "Текущий",
            Language::En => "Current",
        }
    }
    pub fn crumb_rollback_tip(&self) -> &'static str {
        match self {
            Language::Ru => "Кликните для отката к этому шагу",
            Language::En => "Click to roll back to this step",
        }
    }

    pub fn measurement_prompt(&self, min: f64, max: f64, unit: &str) -> String {
        match self {
            Language::Ru => format!("Замер (допуск: {} ... {} {}):", min, max, unit),
            Language::En => format!("Measurement (tolerance: {} ... {} {}):", min, max, unit),
        }
    }

    pub fn abnormal_err(&self, min: f64, max: f64, unit: &str) -> String {
        match self {
            Language::Ru => format!("❌ Отклонение! (Норма: {} - {} {})", min, max, unit),
            Language::En => format!("❌ Out of spec! (Normal: {} - {} {})", min, max, unit),
        }
    }

    pub fn diag_complete(&self) -> &'static str {
        match self {
            Language::Ru => "🏁 Регламент проверки завершен!",
            Language::En => "🏁 Diagnostic procedure completed!",
        }
    }
    pub fn start_over(&self) -> &'static str {
        match self {
            Language::Ru => "🔄 Пройти заново",
            Language::En => "🔄 Run Again",
        }
    }
    pub fn current_node_label(&self) -> &'static str {
        match self {
            Language::Ru => "Текущий шаг:",
            Language::En => "Current Step:",
        }
    }
    pub fn choose_action(&self) -> &'static str {
        match self {
            Language::Ru => "Выберите результат проверки:",
            Language::En => "Select verification result:",
        }
    }
    pub fn final_point(&self) -> &'static str {
        match self {
            Language::Ru => "✔ Финальная точка регламента.",
            Language::En => "✔ Terminal step reached.",
        }
    }
    pub fn finish_diag_btn(&self) -> &'static str {
        match self {
            Language::Ru => "🏁 Завершить регламент",
            Language::En => "🏁 Finish Procedure",
        }
    }
    pub fn in_range_msg(&self) -> &'static str {
        match self {
            Language::Ru => "в допуске.",
            Language::En => "within tolerance.",
        }
    }
    pub fn accept_normal(&self) -> &'static str {
        match self {
            Language::Ru => "Принять: В норме ➔",
            Language::En => "Accept: Normal ➔",
        }
    }
    pub fn accept_abnormal(&self) -> &'static str {
        match self {
            Language::Ru => "Принять: Отклонение ➔",
            Language::En => "Accept: Abnormal ➔",
        }
    }
    pub fn safety_title(&self) -> &'static str {
        match self {
            Language::Ru => "⚠ ТРЕБОВАНИЕ БЕЗОПАСНОСТИ",
            Language::En => "⚠ SAFETY REQUIREMENT",
        }
    }
    pub fn confirm_and_proceed(&self) -> &'static str {
        match self {
            Language::Ru => "Подтвердить и продолжить ➔",
            Language::En => "Confirm & Proceed ➔",
        }
    }
}
