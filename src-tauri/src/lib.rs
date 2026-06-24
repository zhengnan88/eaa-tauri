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

    #[test]
    fn test_read_order_detail_headers() {
        let files = [
            "/Volumes/WenshuSpace/下载/订单详情_20260416_103708_8.xls",
            "/Volumes/WenshuSpace/下载/订单详情_20260109_112137_6.xls",
        ];
        for file in &files {
            println!("\n=== {} ===", file);
            let result = crate::excel::reader::read_headers(file, 1);
            match result {
                Some((headers, row)) => {
                    println!("Detected row: {}", row);
                    println!("Headers ({}): {:?}", headers.len(), headers);
                    
                    let rule_cols = vec!["订单号", "运单号", "商家流水号", "商家名称", "商家ID", "城市", "骑手", "骑手ID", "站点", "站点ID"];
                    for col in &rule_cols {
                        if headers.iter().any(|h| h == col) {
                            println!("  ✓ {}", col);
                        } else {
                            println!("  ✗ {} - MISSING", col);
                        }
                    }
                }
                None => println!("Failed to read headers"),
            }
        }
    }
}

    #[test]
    fn test_order_detail_rule_match() {
        use crate::core::config::{Rule, AppSettings};
        
        let rule_cols = vec![
            "订单号", "运单号", "商家流水号", "商家名称", "商家ID", "城市", "骑手", "骑手ID", "站点", "站点ID",
            "区域", "重量", "预订单", "状态", "组织类型", "众包类型", "配送时效", "等待时长", "送达时长", "连击时长",
            "导航距离", "折线距离", "商家地址", "商家配送评分", "商家配送评价", "订单原价", "订单金额", "付商家款", "实际付款", "收用户款",
            "实际收款", "配送费", "下单时间", "支付时间", "期望送达时间", "商家推单时间", "调度时间", "接单时间", "到店时间", "取货时间",
            "送达时间", "取消时间", "取消原因", "取消操作人", "申请退款原因", "申请退款操作人", "驻点订单类型", "业务类型", "是否跨区单", "是否乐跑单",
            "商户点击出餐时间", "最早考核时间", "最晚考核时间", "增值服务产品", "骑手签约站点", "取件AOI名称", "送件AOI名称",
        ];
        
        let rule = Rule {
            id: "test".to_string(),
            name: "订单详情".to_string(),
            feature_columns: rule_cols.into_iter().map(String::from).collect(),
            feature_row: 1,
            date_column: String::new(),
            force_precision: "day".to_string(),
            naming_pattern: String::new(),
            auto_archive: true,
        };
        
        let settings = AppSettings {
            ignore_case: true,
            ignore_space: true,
            log_retention_days: 30,
        };
        
        let file = "/Volumes/WenshuSpace/下载/订单详情_20260416_103708_8.xls";
        let (headers, _) = crate::excel::reader::read_headers(file, 1).unwrap();
        
        let matched = crate::core::rule_engine::rule_matches(&rule, &headers, &settings);
        println!("Rule matches: {}", matched);
        
        if !matched {
            let normalized_headers: Vec<String> = headers.iter()
                .map(|h| crate::core::rule_engine::normalize_column(h, true, true))
                .collect();
            for col in &rule.feature_columns {
                let normalized_col = crate::core::rule_engine::normalize_column(col, true, true);
                if !normalized_headers.contains(&normalized_col) {
                    println!("  MISSING: '{}' (normalized: '{}')", col, normalized_col);
                }
            }
        }
    }
