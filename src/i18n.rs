use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Language {
    Ru,
    En,
}

impl Language {
    // Вкладки
    pub fn tab_builder(&self) -> &'static str { match self { Language::Ru => "🕸 Схема алгоритма", Language::En => "🕸 Flowchart Scheme" } }
    pub fn tab_runner(&self) -> &'static str { match self { Language::Ru => "▶ Тест алгоритма", Language::En => "▶ Diagnostics Test" } }
    pub fn tab_archive(&self) -> &'static str { match self { Language::Ru => "📚 Архив", Language::En => "📚 Archive" } }

    // Тулбар конструктора
    pub fn btn_new(&self) -> &'static str { match self { Language::Ru => "📄 Новый", Language::En => "📄 New" } }
    pub fn btn_save(&self) -> &'static str { match self { Language::Ru => "💾 Сохранить", Language::En => "💾 Save" } }
    pub fn btn_passport(&self) -> &'static str { match self { Language::Ru => "📋 Паспорт", Language::En => "📋 Passport" } }
    pub fn add_block(&self) -> &'static str { match self { Language::Ru => "➕ Добавить блок", Language::En => "➕ Add Step" } }
    pub fn fit_view(&self) -> &'static str { match self { Language::Ru => "🔍 Вписать всё", Language::En => "🔍 Fit to View" } }
    pub fn zoom_label(&self) -> &'static str { match self { Language::Ru => "Масштаб:", Language::En => "Zoom:" } }
    pub fn toolbar_hint(&self) -> &'static str {
        match self {
            Language::Ru => "🖱 ЛКМ: перемещение | ПКМ: панорама | Колесико: масштаб",
            Language::En => "🖱 LMB: drag node | RMB: pan canvas | Wheel: zoom",
        }
    }

    // Окно паспорта оборудования
    pub fn passport_window_title(&self) -> &'static str { match self { Language::Ru => "Паспорт оборудования (Метаданные)", Language::En => "Equipment Passport (Metadata)" } }
    pub fn field_name(&self) -> &'static str { match self { Language::Ru => "Название узла / станка:", Language::En => "Equipment Name:" } }
    pub fn field_type(&self) -> &'static str { match self { Language::Ru => "Тип оборудования:", Language::En => "Equipment Type:" } }
    pub fn field_model(&self) -> &'static str { match self { Language::Ru => "Модель:", Language::En => "Model:" } }
    pub fn field_inv(&self) -> &'static str { match self { Language::Ru => "Инвентарный №:", Language::En => "Inventory #:" } }
    pub fn field_author(&self) -> &'static str { match self { Language::Ru => "Автор регламента:", Language::En => "Author:" } }
    pub fn btn_close(&self) -> &'static str { match self { Language::Ru => "Закрыть", Language::En => "Close" } }

    // Уведомления
    pub fn save_success(&self) -> &'static str { match self { Language::Ru => "✔ Сохранено в архив!", Language::En => "✔ Saved to archive!" } }

    // Архив
    pub fn search_placeholder(&self) -> &'static str { match self { Language::Ru => "🔍 Поиск по названию, модели, номеру, автору...", Language::En => "🔍 Search by name, model, inv number, author..." } }
    pub fn btn_refresh(&self) -> &'static str { match self { Language::Ru => "🔄 Обновить", Language::En => "🔄 Refresh" } }
    pub fn empty_archive(&self) -> &'static str { match self { Language::Ru => "В архиве пока нет регламентов. Создайте новый в конструкторе!", Language::En => "No procedures in archive yet. Create one in the builder!" } }
    pub fn not_found(&self) -> &'static str { match self { Language::Ru => "По запросу ничего не найдено.", Language::En => "Nothing found matching your query." } }
    pub fn card_steps(&self) -> &'static str { match self { Language::Ru => "узлов", Language::En => "steps" } }
    pub fn card_updated(&self) -> &'static str { match self { Language::Ru => "Изменен:", Language::En => "Updated:" } }
    pub fn btn_open_builder(&self) -> &'static str { match self { Language::Ru => "🛠 Редактировать", Language::En => "🛠 Edit" } }
    pub fn btn_run_diag(&self) -> &'static str { match self { Language::Ru => "▶ Диагностика", Language::En => "▶ Run" } }
    pub fn btn_delete_archive(&self) -> &'static str { match self { Language::Ru => "🗑 Удалить", Language::En => "🗑 Delete" } }

    // Бейджи блоков
    pub fn badge_choice(&self) -> &'static str { match self { Language::Ru => "Выбор", Language::En => "Choice" } }
    pub fn badge_measure(&self) -> &'static str { match self { Language::Ru => "Замер", Language::En => "Measure" } }
    pub fn badge_safety(&self) -> &'static str { match self { Language::Ru => "ТБ ⚠", Language::En => "Safety ⚠" } }
    pub fn badge_active(&self) -> &'static str { match self { Language::Ru => "АКТИВЕН ▶", Language::En => "ACTIVE ▶" } }
    pub fn badge_done(&self) -> &'static str { match self { Language::Ru => "ВЫПОЛНЕНО ✔", Language::En => "DONE ✔" } }
    pub fn no_desc(&self) -> &'static str { match self { Language::Ru => "Описание отсутствует", Language::En => "No description" } }
    pub fn photo_attached(&self) -> &'static str { match self { Language::Ru => "🖼 фото прикреплено", Language::En => "🖼 photo attached" } }

    // Инспектор
    pub fn inspector_heading(&self) -> &'static str { match self { Language::Ru => "Параметры шага", Language::En => "Step Properties" } }
    pub fn select_node_hint(&self) -> &'static str { match self { Language::Ru => "Нажмите на любой блок для настройки.", Language::En => "Click any block on the canvas to inspect." } }
    pub fn start_node(&self) -> &'static str { match self { Language::Ru => "🚩 Стартовый узел", Language::En => "🚩 Start Node" } }
    pub fn delete_btn(&self) -> &'static str { match self { Language::Ru => "🗑 Удалить (Del)", Language::En => "🗑 Delete (Del)" } }
    pub fn block_title(&self) -> &'static str { match self { Language::Ru => "Название блока:", Language::En => "Block Title:" } }
    pub fn behavior_type(&self) -> &'static str { match self { Language::Ru => "Тип поведения:", Language::En => "Behavior Type:" } }
    pub fn kind_standard(&self) -> &'static str { match self { Language::Ru => "Обычный выбор действий", Language::En => "Standard Branching" } }
    pub fn kind_measurement(&self) -> &'static str { match self { Language::Ru => "Замер параметра", Language::En => "Measurement" } }
    pub fn kind_safety(&self) -> &'static str { match self { Language::Ru => "Техника безопасности", Language::En => "Safety Warning" } }
    pub fn instructions(&self) -> &'static str { match self { Language::Ru => "Инструкция специалисту:", Language::En => "Operator Instructions:" } }
    pub fn photo_btn(&self) -> &'static str { match self { Language::Ru => "📷 Фото", Language::En => "📷 Photo" } }
    pub fn links_heading(&self) -> &'static str { match self { Language::Ru => "Связи (Стрелки)", Language::En => "Connections (Arrows)" } }
    pub fn add_branch(&self) -> &'static str { match self { Language::Ru => "➕ Добавить ветку", Language::En => "➕ Add Branch" } }
    pub fn branch_text(&self) -> &'static str { match self { Language::Ru => "Текст:", Language::En => "Text:" } }
    pub fn finish_diag(&self) -> &'static str { match self { Language::Ru => "🏁 Завершить диагностику", Language::En => "🏁 Finish Diagnostics" } }
    pub fn not_selected(&self) -> &'static str { match self { Language::Ru => "Не выбрано", Language::En => "Not Selected" } }
    pub fn unit(&self) -> &'static str { match self { Language::Ru => "Ед. изм:", Language::En => "Unit:" } }
    pub fn min_val(&self) -> &'static str { match self { Language::Ru => "Мин:", Language::En => "Min:" } }
    pub fn max_val(&self) -> &'static str { match self { Language::Ru => "Макс:", Language::En => "Max:" } }
    pub fn goto_normal(&self) -> &'static str { match self { Language::Ru => "➔ В норме перейти на:", Language::En => "➔ If normal go to:" } }
    pub fn goto_abnormal(&self) -> &'static str { match self { Language::Ru => "➔ При отклонении перейти на:", Language::En => "➔ If abnormal go to:" } }
    pub fn safety_ack(&self) -> &'static str { match self { Language::Ru => "Подтверждение ТБ:", Language::En => "Safety Check Text:" } }
    pub fn goto_after_ack(&self) -> &'static str { match self { Language::Ru => "➔ После подтверждения переход:", Language::En => "➔ After confirm go to:" } }

    // Пульт диагностики
    pub fn runner_heading(&self) -> &'static str { match self { Language::Ru => "Пульт диагностики", Language::En => "Diagnostics Console" } }
    pub fn step_back(&self) -> &'static str { match self { Language::Ru => "⬅ Шаг назад", Language::En => "⬅ Step Back" } }
    pub fn reset_test(&self) -> &'static str { match self { Language::Ru => "🔄 Сброс теста", Language::En => "🔄 Reset Test" } }
    pub fn focus_step(&self) -> &'static str { match self { Language::Ru => "🎯 Фокус на шаге", Language::En => "🎯 Focus on Step" } }
    pub fn diag_complete(&self) -> &'static str { match self { Language::Ru => "🏁 Диагностика успешно завершена!", Language::En => "🏁 Diagnostics completed successfully!" } }
    pub fn start_over(&self) -> &'static str { match self { Language::Ru => "🔄 Начать сначала", Language::En => "🔄 Start Over" } }
    pub fn current_node_label(&self) -> &'static str { match self { Language::Ru => "Текущий узел:", Language::En => "Current Node:" } }
    pub fn choose_action(&self) -> &'static str { match self { Language::Ru => "Выберите вариант перехода:", Language::En => "Choose action:" } }
    pub fn final_point(&self) -> &'static str { match self { Language::Ru => "✔ Финальная точка алгоритма.", Language::En => "✔ Final algorithm endpoint." } }
    pub fn finish_diag_btn(&self) -> &'static str { match self { Language::Ru => "Завершить диагностику", Language::En => "Complete Diagnostics" } }
    pub fn in_range_msg(&self) -> &'static str { match self { Language::Ru => "в допуске.", Language::En => "within tolerance." } }
    pub fn accept_normal(&self) -> &'static str { match self { Language::Ru => "Принять: В норме ➔", Language::En => "Accept: Normal ➔" } }
    pub fn accept_abnormal(&self) -> &'static str { match self { Language::Ru => "Принять: Отклонение ➔", Language::En => "Accept: Abnormal ➔" } }
    pub fn safety_title(&self) -> &'static str { match self { Language::Ru => "⚠ ТРЕБОВАНИЕ БЕЗОПАСНОСТИ", Language::En => "⚠ SAFETY REQUIREMENT" } }
    pub fn confirm_and_proceed(&self) -> &'static str { match self { Language::Ru => "Подтвердить и продолжить ➔", Language::En => "Confirm & Proceed ➔" } }
}