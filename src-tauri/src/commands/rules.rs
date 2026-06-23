use crate::core::config::{Config, Rule};
use crate::excel::reader;
use uuid::Uuid;

#[tauri::command]
pub fn get_rules() -> Vec<Rule> {
    Config::load().rules
}

#[tauri::command]
pub fn add_rule(rule_data: Rule) -> Rule {
    let mut config = Config::load();
    let mut rule = rule_data;
    if rule.id.is_empty() {
        rule.id = Uuid::new_v4().to_string();
    }
    config.rules.push(rule.clone());
    let _ = config.save();
    rule
}

#[tauri::command]
pub fn update_rule(rule_id: String, rule_data: Rule) -> Result<Rule, String> {
    let mut config = Config::load();
    for i in 0..config.rules.len() {
        if config.rules[i].id == rule_id {
            let mut rule = rule_data;
            rule.id = rule_id;
            config.rules[i] = rule.clone();
            let _ = config.save();
            return Ok(rule);
        }
    }
    Err("Rule not found".to_string())
}

#[tauri::command]
pub fn delete_rule(rule_id: String) -> bool {
    let mut config = Config::load();
    let original_len = config.rules.len();
    config.rules.retain(|r| r.id != rule_id);
    if config.rules.len() < original_len {
        let _ = config.save();
        return true;
    }
    false
}

#[tauri::command]
pub fn move_rule(rule_id: String, direction: String) -> bool {
    let mut config = Config::load();
    let index = config.rules.iter().position(|r| r.id == rule_id);
    if let Some(idx) = index {
        if direction == "up" && idx > 0 {
            config.rules.swap(idx, idx - 1);
            let _ = config.save();
            return true;
        } else if direction == "down" && idx < config.rules.len() - 1 {
            config.rules.swap(idx, idx + 1);
            let _ = config.save();
            return true;
        }
    }
    false
}

#[tauri::command]
pub fn load_excel_headers(file_path: String, feature_row: u32) -> serde_json::Value {
    match reader::read_headers(&file_path, feature_row) {
        Some((headers, detected_row)) => serde_json::json!({
            "headers": headers,
            "detected_row": detected_row,
        }),
        None => serde_json::json!({
            "headers": [],
            "detected_row": feature_row,
        }),
    }
}
