use std::fs;
use std::path::Path;
use crate::core::config::MonitorFolder;
use crate::excel::reader::is_excel_file;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanResult {
    pub files: Vec<String>,
    pub scanned_count: u32,
}

pub fn scan_folder(monitor_folder: &MonitorFolder, full_scan: bool) -> ScanResult {
    let path = Path::new(&monitor_folder.path);
    if !path.exists() {
        return ScanResult {
            files: Vec::new(),
            scanned_count: 0,
        };
    }

    let last_scan = if !full_scan && !monitor_folder.last_scan_time.is_empty() {
        chrono::NaiveDateTime::parse_from_str(&monitor_folder.last_scan_time, "%Y-%m-%dT%H:%M:%S%.f")
            .ok()
            .or_else(|| chrono::NaiveDateTime::parse_from_str(&monitor_folder.last_scan_time, "%Y-%m-%dT%H:%M:%S").ok())
    } else {
        None
    };

    let mut excel_files = Vec::new();
    let mut scanned_count = 0;

    if monitor_folder.include_sub {
        scan_recursive(path, &mut excel_files, &mut scanned_count, last_scan);
    } else {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() && is_excel_file(&file_path) {
                    scanned_count += 1;
                    if should_include(&file_path, last_scan) {
                        excel_files.push(file_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    ScanResult {
        files: excel_files,
        scanned_count,
    }
}

fn scan_recursive(dir: &Path, files: &mut Vec<String>, count: &mut u32, last_scan: Option<chrono::NaiveDateTime>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_excel_file(&path) {
                *count += 1;
                if should_include(&path, last_scan) {
                    files.push(path.to_string_lossy().to_string());
                }
            } else if path.is_dir() {
                scan_recursive(&path, files, count, last_scan);
            }
        }
    }
}

fn should_include(path: &Path, last_scan: Option<chrono::NaiveDateTime>) -> bool {
    let Some(scan_time) = last_scan else {
        return true;
    };

    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            let file_time = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .map(|dt| dt.naive_local())
                .unwrap_or_default();
            file_time > scan_time
        })
        .unwrap_or(false)
}
