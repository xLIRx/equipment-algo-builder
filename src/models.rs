use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum WireStyle {
    Curved,
    Orthogonal,
}

impl Default for WireStyle {
    fn default() -> Self {
        WireStyle::Curved
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EquipmentInfo {
    #[serde(default)]
    pub manufacturer: String,
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
            manufacturer: String::new(),
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    // Вложенный подпроцесс с раздельным выходом "Успех" / "Дефект"
    Subprocess {
        sub_algo: Box<Algorithm>,
        next_if_success: Option<Uuid>,
        next_if_failure: Option<Uuid>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Algorithm {
    pub id: Uuid,
    pub metadata: EquipmentInfo,
    pub steps: Vec<Step>,
    pub first_step_id: Option<Uuid>,
    #[serde(default)]
    pub wire_styles: HashMap<String, WireStyle>, // ключ "from_id:to_id"
}

impl Default for Algorithm {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: EquipmentInfo::default(),
            steps: Vec::new(),
            first_step_id: None,
            wire_styles: HashMap::new(),
        }
    }
}
