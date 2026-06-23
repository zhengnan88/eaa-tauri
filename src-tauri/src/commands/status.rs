use crate::core::config::Config;

#[tauri::command]
pub fn get_data_ready_status(date_str: String) -> Vec<serde_json::Value> {
    let config = Config::load();
    let rules_json: Vec<serde_json::Value> = config.rules.iter().map(|r| {
        serde_json::json!({
            "name": r.name,
        })
    }).collect();

    let log_dir = crate::core::logger::Logger::get_default_log_dir();
    let logger = crate::core::logger::Logger::new(log_dir, config.settings.log_retention_days);
    logger.get_data_ready_status(&date_str, &rules_json)
}
