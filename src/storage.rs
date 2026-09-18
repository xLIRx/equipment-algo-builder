use crate::models::Algorithm;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const ARCHIVE_DIR: &str = "archive";

/// Сохраняет регламент в папку архива с копированием внешних изображений в проект
pub fn save_algorithm_to_archive(algo: &mut Algorithm) -> Result<(), String> {
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

/// Загружает все сохраненные регламенты из архива и сортирует по дате изменения
pub fn load_all_from_archive() -> Vec<Algorithm> {
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

/// Удаляет регламент и его файлы из папки архива
pub fn delete_from_archive(id: Uuid) {
    let project_dir = PathBuf::from(ARCHIVE_DIR).join(id.to_string());
    if project_dir.exists() {
        let _ = fs::remove_dir_all(project_dir);
    }
}

/// Экспортирует регламент в указанный внешний JSON-файл
pub fn export_algorithm_to_file(algo: &Algorithm, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(algo).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

/// Загружает регламент из внешнего JSON-файла
pub fn import_algorithm_from_file(path: &Path) -> Result<Algorithm, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str::<Algorithm>(&content).map_err(|e| e.to_string())
}

/// Преобразует путь к файлу в безопасный URI для рендерера изображений egui
pub fn to_image_uri(path_str: &str) -> String {
    if let Ok(abs) = fs::canonicalize(path_str) {
        let clean = abs.to_string_lossy().replace('\\', "/");
        let formatted = clean.trim_start_matches("//?/").trim_start_matches("/?");
        format!("file://{}", formatted)
    } else {
        format!("file://{}", path_str.replace('\\', "/"))
    }
}
