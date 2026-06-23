use std::fs;
use std::path::Path;
use chrono::{Local, Duration, NaiveDate};

#[derive(Debug, Clone, serde::Serialize)]
pub struct CleanResult {
    pub moved_folders: u32,
    pub moved_files: u32,
}

const THRESHOLD_DAYS: i64 = 31;

pub fn clean(data_center: &str) -> CleanResult {
    let mut result = CleanResult {
        moved_folders: 0,
        moved_files: 0,
    };

    if data_center.is_empty() {
        return result;
    }

    let sources_dir = Path::new(data_center).join("sources");
    let history_dir = Path::new(data_center).join("history");

    if !sources_dir.exists() {
        return result;
    }

    let _ = fs::create_dir_all(&history_dir);

    let threshold = Local::now().date_naive() - Duration::days(THRESHOLD_DAYS);

    if let Ok(entries) = fs::read_dir(&sources_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let folder_name = entry.file_name().to_string_lossy().to_string();
            if let Some(folder_date) = parse_folder_date(&folder_name) {
                if folder_date <= threshold {
                    let file_count = count_files(&path);
                    let target_path = history_dir.join(&folder_name);

                    if target_path.exists() {
                        let _ = merge_folders(&path, &target_path);
                        let _ = fs::remove_dir_all(&path);
                    } else {
                        let _ = fs::rename(&path, &target_path);
                    }
                    result.moved_folders += 1;
                    result.moved_files += file_count;
                }
            }
        }
    }

    result
}

fn parse_folder_date(name: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = name.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (
            parts[0].parse::<i32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            return NaiveDate::from_ymd_opt(y, m, d);
        }
    }
    if parts.len() == 2 {
        if let (Ok(y), Ok(m)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
            return NaiveDate::from_ymd_opt(y, m, 1);
        }
    }
    None
}

fn count_files(dir: &Path) -> u32 {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                count += 1;
            } else if entry.path().is_dir() {
                count += count_files(&entry.path());
            }
        }
    }
    count
}

fn merge_folders(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_item = entry.path();
            let dst_item = dst.join(entry.file_name());
            if src_item.is_dir() {
                if dst_item.exists() {
                    merge_folders(&src_item, &dst_item)?;
                } else {
                    fs::rename(&src_item, &dst_item)?;
                }
            } else if !dst_item.exists() {
                fs::rename(&src_item, &dst_item)?;
            }
        }
    }
    Ok(())
}
