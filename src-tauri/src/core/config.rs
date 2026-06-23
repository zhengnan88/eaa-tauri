use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub ignore_case: bool,
    #[serde(default = "default_ignore_space")]
    pub ignore_space: bool,
    #[serde(default = "default_log_retention")]
    pub log_retention_days: u32,
}

fn default_ignore_space() -> bool {
    true
}
fn default_log_retention() -> u32 {
    30
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ignore_case: false,
            ignore_space: true,
            log_retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorFolder {
    pub path: String,
    #[serde(default = "default_true")]
    pub include_sub: bool,
    #[serde(default = "default_scan_interval")]
    pub scan_interval_min: u32,
    #[serde(default)]
    pub last_scan_time: String,
}

fn default_true() -> bool {
    true
}
fn default_scan_interval() -> u32 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(default = "default_uuid")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub feature_columns: Vec<String>,
    #[serde(default = "default_feature_row")]
    pub feature_row: u32,
    #[serde(default)]
    pub date_column: String,
    #[serde(default = "default_force_precision")]
    pub force_precision: String,
    #[serde(default)]
    pub naming_pattern: String,
    #[serde(default = "default_true")]
    pub auto_archive: bool,
}

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}
fn default_feature_row() -> u32 {
    1
}
fn default_force_precision() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    #[serde(default = "default_uuid")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub script_path: String,
    #[serde(default = "default_param_type")]
    pub param_type: String,
    #[serde(default)]
    pub order: u32,
}

fn default_param_type() -> String {
    "day".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub data_center: String,
    #[serde(default)]
    pub monitor_folders: Vec<MonitorFolder>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub scripts: Vec<Script>,
    #[serde(default)]
    pub settings: AppSettings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_center: String::new(),
            monitor_folders: Vec::new(),
            rules: Vec::new(),
            scripts: Vec::new(),
            settings: AppSettings::default(),
        }
    }
}

fn get_app_dir() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe.parent().unwrap_or(&exe);
    // Inside .app bundle: EAA.app/Contents/MacOS/eaa-tauri
    // Go up to find the directory containing the .app bundle
    if exe_dir.ends_with("MacOS") {
        exe_dir.parent()           // Contents/
            .and_then(|p| p.parent())  // EAA.app/
            .and_then(|p| p.parent())  // Parent of EAA.app
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| exe_dir.to_path_buf())
    } else {
        exe_dir.to_path_buf()
    }
}

pub fn get_config_path() -> PathBuf {
    get_app_dir().join("config.json")
}

impl Config {
    pub fn load() -> Self {
        let path = get_config_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = get_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn get_default_data_center() -> String {
        get_app_dir().join("datacenter").to_string_lossy().to_string()
    }
}
