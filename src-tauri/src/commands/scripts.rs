use crate::core::config::{Config, Script};
use crate::core::script_runner;
use uuid::Uuid;

#[tauri::command]
pub fn get_scripts() -> Vec<Script> {
    Config::load().scripts
}

#[tauri::command]
pub fn add_script(script_data: Script) -> Script {
    let mut config = Config::load();
    let mut script = script_data;
    if script.id.is_empty() {
        script.id = Uuid::new_v4().to_string();
    }
    let max_order = config.scripts.iter().map(|s| s.order).max().unwrap_or(0);
    if script.order == 0 {
        script.order = max_order + 1;
    }
    config.scripts.push(script.clone());
    let _ = config.save();
    script
}

#[tauri::command]
pub fn update_script(script_id: String, script_data: Script) -> Result<Script, String> {
    let mut config = Config::load();
    for i in 0..config.scripts.len() {
        if config.scripts[i].id == script_id {
            let mut script = script_data;
            script.id = script_id;
            config.scripts[i] = script.clone();
            let _ = config.save();
            return Ok(script);
        }
    }
    Err("Script not found".to_string())
}

#[tauri::command]
pub fn delete_script(script_id: String) -> bool {
    let mut config = Config::load();
    let original_len = config.scripts.len();
    config.scripts.retain(|s| s.id != script_id);
    if config.scripts.len() < original_len {
        let _ = config.save();
        return true;
    }
    false
}

#[tauri::command]
pub fn move_script(script_id: String, direction: String) -> bool {
    let mut config = Config::load();
    let index = config.scripts.iter().position(|s| s.id == script_id);
    if let Some(idx) = index {
        if direction == "up" && idx > 0 {
            config.scripts.swap(idx, idx - 1);
            let _ = config.save();
            return true;
        } else if direction == "down" && idx < config.scripts.len() - 1 {
            config.scripts.swap(idx, idx + 1);
            let _ = config.save();
            return true;
        }
    }
    false
}

#[tauri::command]
pub async fn run_script(script_id: String, param: String) -> script_runner::ScriptResult {
    let config = Config::load();
    let script = config.scripts.iter().find(|s| s.id == script_id).cloned();
    match script {
        Some(s) => {
            // 在阻塞线程中运行，避免阻塞 UI
            tokio::task::spawn_blocking(move || {
                script_runner::run_script(&s, &param, &config.data_center)
            })
            .await
            .unwrap_or(script_runner::ScriptResult {
                success: false,
                stdout: String::new(),
                stderr: "执行异常".to_string(),
                returncode: -1,
            })
        }
        None => script_runner::ScriptResult {
            success: false,
            stdout: String::new(),
            stderr: "脚本不存在".to_string(),
            returncode: -1,
        },
    }
}
