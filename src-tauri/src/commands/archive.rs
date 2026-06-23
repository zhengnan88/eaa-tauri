use crate::core::config::Config;
use crate::core::{archiver, rule_engine, date_extractor};
use crate::excel::reader;
use crate::core::logger::Logger;
use std::path::Path;

fn process_files_sync(file_paths: Vec<String>) -> Vec<archiver::ArchiveResult> {
    let config = Config::load();
    let logger = Logger::new(Logger::get_default_log_dir(), config.settings.log_retention_days);
    let mut results = Vec::new();

    for file_path in &file_paths {
        let file_name = Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if !reader::is_excel_file(Path::new(file_path)) {
            results.push(archiver::ArchiveResult {
                success: false,
                file_name,
                rule_name: String::new(),
                date: String::new(),
                target_path: String::new(),
                status: "skipped".to_string(),
                message: "非Excel文件".to_string(),
            });
            continue;
        }

        let header_result = reader::read_headers(file_path, 1);
        let headers = match header_result {
            Some((h, _)) if !h.is_empty() => h,
            _ => {
                results.push(archiver::ArchiveResult {
                    success: false,
                    file_name,
                    rule_name: String::new(),
                    date: String::new(),
                    target_path: String::new(),
                    status: "failed".to_string(),
                    message: "无法读取表头".to_string(),
                });
                logger.log_file_processed(file_path, "", "", "无法读取表头", "错误");
                continue;
            }
        };

        let matched_rule = rule_engine::match_all(&headers, &config.rules, &config.settings);
        let matched_rule = match matched_rule {
            Some(r) => r,
            None => {
                results.push(archiver::ArchiveResult {
                    success: false,
                    file_name,
                    rule_name: String::new(),
                    date: String::new(),
                    target_path: String::new(),
                    status: "no_rule".to_string(),
                    message: "无匹配规则".to_string(),
                });
                logger.log_file_processed(file_path, "", "", "无匹配规则", "跳过");
                continue;
            }
        };

        let _headers = if matched_rule.feature_row > 1 {
            match reader::read_headers(file_path, matched_rule.feature_row) {
                Some((h, _)) if !h.is_empty() => h,
                _ => headers,
            }
        } else {
            headers
        };

        if !matched_rule.auto_archive {
            let extracted = date_extractor::extract_date(file_path, &matched_rule.date_column, &matched_rule.force_precision);
            results.push(archiver::ArchiveResult {
                success: false,
                file_name,
                rule_name: matched_rule.name.clone(),
                date: extracted.date_value.format("%Y-%m-%d").to_string(),
                target_path: String::new(),
                status: "skipped".to_string(),
                message: "识别成功但未转存".to_string(),
            });
            logger.log_file_processed(
                file_path,
                &matched_rule.name,
                &extracted.date_value.format("%Y-%m-%d").to_string(),
                "识别成功但未转存",
                "",
            );
            continue;
        }

        let result = archiver::archive(file_path, matched_rule, &config);
        logger.log_file_processed(
            file_path,
            &result.rule_name,
            &result.date,
            &result.status,
            &result.message,
        );
        if result.success {
            logger.log_data_ready(&result.rule_name, &result.date, &result.file_name);
        }
        results.push(result);
    }

    results
}

#[tauri::command]
pub async fn process_files(file_paths: Vec<String>) -> Vec<archiver::ArchiveResult> {
    tokio::task::spawn_blocking(move || process_files_sync(file_paths))
        .await
        .unwrap_or_default()
}

pub fn process_files_sync_pub(file_paths: Vec<String>) -> Vec<archiver::ArchiveResult> {
    process_files_sync(file_paths)
}
