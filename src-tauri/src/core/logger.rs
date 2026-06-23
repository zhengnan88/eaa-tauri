use std::fs;
use std::path::PathBuf;
use chrono::{Local, NaiveDate, Duration};

const MAX_LOG_ENTRIES: usize = 100;

pub struct Logger {
    log_dir: PathBuf,
    retention_days: u32,
}

impl Logger {
    pub fn new(log_dir: PathBuf, retention_days: u32) -> Self {
        let _ = fs::create_dir_all(&log_dir);
        let logger = Self { log_dir, retention_days };
        logger.clean_old_logs();
        logger
    }

    pub fn get_default_log_dir() -> PathBuf {
        let exe = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe.parent().unwrap_or(&exe);
        let base = if exe_dir.ends_with("MacOS") {
            exe_dir.parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| exe_dir.to_path_buf())
        } else {
            exe_dir.to_path_buf()
        };
        base.join("logs")
    }

    pub fn log_file_processed(&self, file_path: &str, rule: &str, date_str: &str, result: &str, note: &str) {
        let msg = format!("{} | {} | {} | {} | {}", file_path, rule, date_str, result, note);
        self.write_log("INFO", &msg);
    }

    pub fn log_data_ready(&self, rule_name: &str, date_str: &str, file_name: &str) {
        let msg = format!("数据就绪 | {} | {} | {}", rule_name, date_str, file_name);
        self.write_log("INFO", &msg);
    }

    pub fn log_scan(&self, folder_path: &str, file_count: u32) {
        let msg = format!("{} | 扫描完成 | 文件数: {}", folder_path, file_count);
        self.write_log("INFO", &msg);
    }

    pub fn log_clean(&self, moved_count: u32, file_count: u32) {
        let msg = format!("历史清理 | 迁移文件夹: {} | 文件: {}", moved_count, file_count);
        self.write_log("INFO", &msg);
    }

    pub fn get_logs(&self, date_filter: &str, search: &str) -> Vec<serde_json::Value> {
        let mut logs = Vec::new();

        if !date_filter.is_empty() {
            let log_file = self.log_dir.join(format!("app_{}.log", date_filter));
            if log_file.exists() {
                logs.extend(self.read_log_file(&log_file, search));
            }
        } else {
            if let Ok(entries) = fs::read_dir(&self.log_dir) {
                let mut files: Vec<_> = entries.flatten()
                    .filter(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        name.starts_with("app_") && name.ends_with(".log")
                    })
                    .collect();
                files.sort_by_key(|e| e.file_name());
                for entry in files {
                    logs.extend(self.read_log_file(&entry.path(), search));
                }
            }
        }

        logs.reverse();
        logs.truncate(50);
        logs
    }

    pub fn get_data_ready_status(&self, date_str: &str, rules: &[serde_json::Value]) -> Vec<serde_json::Value> {
        let config = crate::core::config::Config::load();
        let sources_dir = std::path::Path::new(&config.data_center).join("sources").join(date_str);

        let mut result = Vec::new();
        for rule in rules {
            let rule_name = rule.get("name").and_then(|v| v.as_str()).unwrap_or("");

            let mut is_ready = false;
            let mut file_name = String::new();

            if sources_dir.exists() {
                if let Ok(entries) = fs::read_dir(&sources_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let lower = name.to_lowercase();
                        if lower.contains(&rule_name.to_lowercase()) {
                            is_ready = true;
                            file_name = name;
                            break;
                        }
                    }
                }
            }

            result.push(serde_json::json!({
                "rule_name": rule_name,
                "date": date_str,
                "file_name": if file_name.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(file_name) },
                "is_ready": is_ready,
            }));
        }

        result
    }

    fn write_log(&self, level: &str, msg: &str) {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let time = Local::now().format("%H:%M:%S").to_string();
        let log_file = self.log_dir.join(format!("app_{}.log", today));
        let line = format!("[{}] [{}] {}\n", time, level, msg);

        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&log_file) {
            use std::io::Write;
            let _ = file.write_all(line.as_bytes());
        }

        self.trim_log_entries();
    }

    fn read_log_file(&self, log_file: &PathBuf, search: &str) -> Vec<serde_json::Value> {
        let mut entries = Vec::new();
        if let Ok(content) = fs::read_to_string(log_file) {
            for line in content.lines() {
                if !search.is_empty() && !line.to_lowercase().contains(&search.to_lowercase()) {
                    continue;
                }
                if let Some(entry) = self.parse_log_line(line) {
                    entries.push(entry);
                }
            }
        }
        entries
    }

    fn parse_log_line(&self, line: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = line.splitn(3, ']').collect();
        if parts.len() < 3 {
            return Some(serde_json::json!({
                "timestamp": "",
                "level": "",
                "file_path": "",
                "detail": line,
            }));
        }

        let timestamp = parts[0].trim_start_matches('[').trim().to_string();
        let level = parts[1].trim_start_matches('[').trim().to_string();
        let detail = parts[2].trim().to_string();

        let detail_parts: Vec<&str> = detail.splitn(2, " | ").collect();
        let file_path = detail_parts[0].to_string();
        let detail_text = if detail_parts.len() > 1 { detail_parts[1].to_string() } else { detail };

        Some(serde_json::json!({
            "timestamp": timestamp,
            "level": level,
            "file_path": file_path,
            "detail": detail_text,
        }))
    }

    fn clean_old_logs(&self) {
        let threshold = Local::now().date_naive() - Duration::days(self.retention_days as i64);
        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("app_") && name.ends_with(".log") {
                    let date_str = &name[4..14];
                    if let Ok(log_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        if log_date < threshold {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }

    fn trim_log_entries(&self) {
        let mut all_lines = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            let mut files: Vec<_> = entries.flatten()
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.starts_with("app_") && name.ends_with(".log")
                })
                .collect();
            files.sort_by_key(|e| e.file_name());

            for entry in files {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    for line in content.lines() {
                        let trimmed = line.trim().to_string();
                        if !trimmed.is_empty() {
                            all_lines.push(trimmed);
                        }
                    }
                }
            }
        }

        if all_lines.len() <= MAX_LOG_ENTRIES {
            return;
        }

        let kept: Vec<_> = all_lines[all_lines.len()-MAX_LOG_ENTRIES..].to_vec();

        // Remove old log files
        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("app_") && name.ends_with(".log") {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        // Write kept lines to today's file
        let today = Local::now().format("%Y-%m-%d").to_string();
        let today_file = self.log_dir.join(format!("app_{}.log", today));
        if let Ok(mut file) = fs::OpenOptions::new().create(true).write(true).truncate(true).open(&today_file) {
            use std::io::Write;
            for line in &kept {
                let _ = writeln!(file, "{}", line);
            }
        }
    }
}
