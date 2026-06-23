#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            env_logger::init();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::archive::process_files,
            commands::rules::get_rules,
            commands::rules::add_rule,
            commands::rules::update_rule,
            commands::rules::delete_rule,
            commands::rules::move_rule,
            commands::rules::load_excel_headers,
            commands::scripts::get_scripts,
            commands::scripts::add_script,
            commands::scripts::update_script,
            commands::scripts::delete_script,
            commands::scripts::move_script,
            commands::scripts::run_script,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_data_center,
            commands::settings::set_data_center,
            commands::settings::get_monitor_folders,
            commands::settings::add_monitor_folder,
            commands::settings::update_monitor_folder,
            commands::settings::delete_monitor_folder,
            commands::settings::check_path_access,
            commands::settings::pick_folder,
            commands::settings::scan_all_monitors,
            commands::settings::scan_today_monitors,
            commands::status::get_data_ready_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod commands;
pub mod core;
pub mod excel;

#[cfg(test)]
mod tests {
    use super::excel::reader;
    use super::core::date_extractor;
    use super::core::rule_engine;
    use super::core::archiver;
    use super::core::config::{Rule, AppSettings, Config};

    #[test]
    fn test_read_headers商家明细() {
        let file = "/Users/zhengnan/Workspace/eaa-tauri/test_data/商家明细_20260621.xlsx";
        let result = reader::read_headers(file, 1);
        assert!(result.is_some());
        let (headers, row) = result.unwrap();
        println!("商家明细 headers (row {}): {:?}", row, headers);
        assert!(!headers.is_empty());
    }

    #[test]
    fn test_read_headers补贴明细() {
        let file = "/Users/zhengnan/Workspace/eaa-tauri/test_data/补贴明细_20260621.xlsx";
        let result = reader::read_headers(file, 1);
        assert!(result.is_some());
        let (headers, row) = result.unwrap();
        println!("补贴明细 headers (row {}): {:?}", row, headers);
        assert!(!headers.is_empty());
    }

    #[test]
    fn test_extract_date商家明细() {
        let file = "/Users/zhengnan/Workspace/eaa-tauri/test_data/商家明细_20260621.xlsx";
        let headers = reader::read_headers(file, 1).map(|(h, _)| h).unwrap_or_default();
        println!("Headers: {:?}", headers);
        
        // Try to find a date column
        for header in &headers {
            let extracted = date_extractor::extract_date(file, header, "auto");
            println!("Column '{}': date={}, precision={}", 
                header, extracted.date_value, extracted.precision);
        }
    }

    #[test]
    fn test_rule_matching() {
        let headers = vec!["商家名称".to_string(), "交易金额".to_string(), "交易日期".to_string()];
        let rule = Rule {
            id: "test".to_string(),
            name: "商家明细".to_string(),
            feature_columns: vec!["商家名称".to_string(), "交易金额".to_string()],
            feature_row: 1,
            date_column: "交易日期".to_string(),
            force_precision: "auto".to_string(),
            naming_pattern: String::new(),
            auto_archive: true,
        };
        let settings = AppSettings::default();
        let result = rule_engine::rule_matches(&rule, &headers, &settings);
        assert!(result, "Rule should match headers");
    }

    #[test]
    fn test_full_archive_flow() {
        let config_path = std::path::Path::new("/Users/zhengnan/Workspace/eaa-tauri/src-tauri/target/debug/config.json");
        let content = std::fs::read_to_string(config_path).expect("Failed to read config.json");
        let config: Config = serde_json::from_str(&content).expect("Failed to parse config.json");
        assert!(!config.data_center.is_empty(), "Data center should be configured");
        assert!(!config.rules.is_empty(), "Rules should be configured");

        let file = "/Users/zhengnan/Workspace/eaa-tauri/test_data/商家明细_20260621.xlsx";
        let result = archiver::archive(file, &config.rules[0], &config);
        println!("Archive result: {:?}", result);
        
        assert!(result.success, "Archive should succeed: {}", result.message);
        assert!(!result.target_path.is_empty());
        assert!(!result.date.is_empty());
        println!("File archived to: {}", result.target_path);

        let file2 = "/Users/zhengnan/Workspace/eaa-tauri/test_data/补贴明细_20260621.xlsx";
        let result2 = archiver::archive(file2, &config.rules[1], &config);
        println!("Archive result 2: {:?}", result2);
        assert!(result2.success, "Archive should succeed: {}", result2.message);
    }

    #[test]
    fn test_process_files_command() {
        use crate::commands::archive::process_files_sync_pub;
        
        let file_paths = vec![
            "/Users/zhengnan/Workspace/eaa-tauri/test_data/商家明细_20260607.xlsx".to_string(),
            "/Users/zhengnan/Workspace/eaa-tauri/test_data/商家明细_20260621.xlsx".to_string(),
            "/Users/zhengnan/Workspace/eaa-tauri/test_data/补贴明细_20260621.xlsx".to_string(),
        ];
        
        let results = process_files_sync_pub(file_paths);
        println!("process_files results:");
        for r in &results {
            println!("  {} -> {} ({})", r.file_name, r.status, r.message);
        }
        
        assert_eq!(results.len(), 3, "Should have 3 results");
        assert!(results.iter().all(|r| r.success), "All archives should succeed");
    }
}
