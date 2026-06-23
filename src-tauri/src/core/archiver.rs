use std::fs;
use std::path::{Path, PathBuf};
use chrono::Datelike;
use crate::core::config::{Config, Rule};
use crate::core::date_extractor::extract_date;
use crate::core::date_utils::{format_date, ExtractedDate};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveResult {
    pub success: bool,
    pub file_name: String,
    pub rule_name: String,
    pub date: String,
    pub target_path: String,
    pub status: String,
    pub message: String,
}

pub fn init_data_center(data_center: &str) -> Option<String> {
    if data_center.is_empty() {
        return Some("未设置数据中心目录".to_string());
    }
    for subdir in &["sources", "history", "output"] {
        let path = Path::new(data_center).join(subdir);
        if let Err(e) = fs::create_dir_all(&path) {
            return Some(format!("无法创建目录: {}", e));
        }
    }
    None
}

pub fn archive(file_path: &str, rule: &Rule, config: &Config) -> ArchiveResult {
    let file_name = Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if config.data_center.is_empty() {
        return ArchiveResult {
            success: false,
            file_name,
            rule_name: rule.name.clone(),
            date: String::new(),
            target_path: String::new(),
            status: "failed".to_string(),
            message: "未设置数据中心目录".to_string(),
        };
    }

    if let Some(init_error) = init_data_center(&config.data_center) {
        return ArchiveResult {
            success: false,
            file_name,
            rule_name: rule.name.clone(),
            date: String::new(),
            target_path: String::new(),
            status: "failed".to_string(),
            message: init_error,
        };
    }

    let extracted = extract_date(file_path, &rule.date_column, &rule.force_precision);
    let date_folder = get_date_folder_name(&extracted);
    let target_dir = Path::new(&config.data_center).join("sources").join(&date_folder);

    if let Err(e) = fs::create_dir_all(&target_dir) {
        return ArchiveResult {
            success: false,
            file_name,
            rule_name: rule.name.clone(),
            date: String::new(),
            target_path: String::new(),
            status: "failed".to_string(),
            message: format!("创建目标目录失败: {}", e),
        };
    }

    let ext = Path::new(file_path)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let stem = Path::new(file_path)
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let new_filename = if rule.naming_pattern.is_empty() {
        format!("{}{}", stem, ext)
    } else {
        let name = format_date(&extracted, &rule.naming_pattern);
        if name.trim().is_empty() {
            file_name.clone()
        } else {
            format!("{}{}", name, ext)
        }
    };

    let target_path = target_dir.join(&new_filename);
    let target_path_str = target_path.to_string_lossy().to_string();

    // Handle conflict
    let final_path = handle_conflict(&target_path, Path::new(file_path));

    if let Err(e) = fs::copy(file_path, &final_path) {
        return ArchiveResult {
            success: false,
            file_name,
            rule_name: rule.name.clone(),
            date: extracted.date_value.format("%Y-%m-%d").to_string(),
            target_path: target_path_str,
            status: "failed".to_string(),
            message: format!("复制文件失败: {}", e),
        };
    }

    ArchiveResult {
        success: true,
        file_name,
        rule_name: rule.name.clone(),
        date: extracted.date_value.format("%Y-%m-%d").to_string(),
        target_path: target_path_str,
        status: "success".to_string(),
        message: "归档成功".to_string(),
    }
}

fn get_date_folder_name(extracted: &ExtractedDate) -> String {
    let d = &extracted.date_value;
    if extracted.precision == "month" {
        format!("{:04}-{:02}", d.year(), d.month())
    } else {
        format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
    }
}

fn handle_conflict(target_path: &Path, source_path: &Path) -> PathBuf {
    if !target_path.exists() {
        return target_path.to_path_buf();
    }

    let existing_size = fs::metadata(target_path).map(|m| m.len()).unwrap_or(0);
    let new_size = fs::metadata(source_path).map(|m| m.len()).unwrap_or(0);

    if new_size > existing_size {
        let _ = fs::remove_file(target_path);
    }

    target_path.to_path_buf()
}
