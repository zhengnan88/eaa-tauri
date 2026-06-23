use crate::core::config::{Config, AppSettings, MonitorFolder};
use crate::core::{archiver, file_scanner, history_cleaner};
use crate::core::logger::Logger;
use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> Option<String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path.map(|fp| fp.to_string()));
    });
    rx.await.ok().flatten()
}

#[tauri::command]
pub fn get_settings() -> AppSettings {
    Config::load().settings
}

#[tauri::command]
pub fn save_settings(settings_data: AppSettings) -> bool {
    let mut config = Config::load();
    config.settings = settings_data;
    config.save().is_ok()
}

#[tauri::command]
pub fn get_data_center() -> String {
    let config = Config::load();
    if config.data_center.is_empty() {
        Config::get_default_data_center()
    } else {
        config.data_center
    }
}

#[tauri::command]
pub fn set_data_center(path: String) -> serde_json::Value {
    let mut config = Config::load();
    let trimmed = path.trim().replace('\u{200c}', "").replace('\u{200b}', "").replace('\u{200d}', "");
    config.data_center = trimmed;
    let init_error = archiver::init_data_center(&config.data_center);
    let _ = config.save();

    match init_error {
        Some(err) => serde_json::json!({ "success": false, "message": err }),
        None => serde_json::json!({ "success": true, "message": "" }),
    }
}

#[tauri::command]
pub fn get_monitor_folders() -> Vec<MonitorFolder> {
    Config::load().monitor_folders
}

#[tauri::command]
pub fn add_monitor_folder(folder_data: MonitorFolder) -> MonitorFolder {
    let mut config = Config::load();
    let mut folder = folder_data;
    folder.path = folder.path.trim().replace('\u{200c}', "").replace('\u{200b}', "").replace('\u{200d}', "");
    config.monitor_folders.push(folder.clone());
    let _ = config.save();
    folder
}

#[tauri::command]
pub fn update_monitor_folder(index: usize, folder_data: MonitorFolder) -> Result<MonitorFolder, String> {
    let mut config = Config::load();
    if index < config.monitor_folders.len() {
        config.monitor_folders[index] = folder_data.clone();
        let _ = config.save();
        return Ok(folder_data);
    }
    Err("Index out of range".to_string())
}

#[tauri::command]
pub fn delete_monitor_folder(index: usize) -> bool {
    let mut config = Config::load();
    if index < config.monitor_folders.len() {
        config.monitor_folders.remove(index);
        let _ = config.save();
        return true;
    }
    false
}

#[tauri::command]
pub fn check_path_access(path: String) -> serde_json::Value {
    let test_file = Path::new(&path).join(".eaa_write_test");
    match std::fs::create_dir_all(&path) {
        Ok(_) => {
            match std::fs::write(&test_file, "test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                    serde_json::json!({ "accessible": true, "message": "" })
                }
                Err(e) => {
                    serde_json::json!({ "accessible": false, "message": e.to_string() })
                }
            }
        }
        Err(e) => {
            serde_json::json!({ "accessible": false, "message": e.to_string() })
        }
    }
}

#[tauri::command]
pub async fn scan_all_monitors() -> serde_json::Value {
    tokio::task::spawn_blocking(|| {
        let config = Config::load();
        let logger = Logger::new(Logger::get_default_log_dir(), config.settings.log_retention_days);
        let mut all_files = Vec::new();

        for mf in &config.monitor_folders {
            let result = file_scanner::scan_folder(mf, true);
            all_files.extend(result.files.clone());
            logger.log_scan(&mf.path, result.scanned_count);
        }

        let total_scanned = all_files.len() as u32;
        let process_results = if !all_files.is_empty() {
            super::archive::process_files_sync_pub(all_files)
        } else {
            Vec::new()
        };

        let clean_result = history_cleaner::clean(&config.data_center);
        if clean_result.moved_folders > 0 {
            logger.log_clean(clean_result.moved_folders, clean_result.moved_files);
        }

        serde_json::json!({
            "scanned_files": total_scanned,
            "processed_results": process_results,
            "cleaned_folders": clean_result.moved_folders,
            "cleaned_files": clean_result.moved_files,
        })
    })
    .await
    .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() }))
}

#[tauri::command]
pub async fn scan_today_monitors() -> serde_json::Value {
    tokio::task::spawn_blocking(|| {
        let config = Config::load();
        let logger = Logger::new(Logger::get_default_log_dir(), config.settings.log_retention_days);
        let today = chrono::Local::now().date_naive();
        let mut today_files = Vec::new();

        for mf in &config.monitor_folders {
            let result = file_scanner::scan_folder(mf, true);
            for fp in &result.files {
                if let Ok(metadata) = std::fs::metadata(fp) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                            let file_date = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                                .map(|dt| dt.date_naive());
                            if file_date == Some(today) {
                                today_files.push(fp.clone());
                            }
                        }
                    }
                }
            }
            logger.log_scan(&mf.path, result.scanned_count);
        }

        let total_scanned = today_files.len() as u32;
        let process_results = if !today_files.is_empty() {
            super::archive::process_files_sync_pub(today_files)
        } else {
            Vec::new()
        };

        serde_json::json!({
            "scanned_files": total_scanned,
            "processed_results": process_results,
        })
    })
    .await
    .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() }))
}
