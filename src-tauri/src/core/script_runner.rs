use std::fs;
use std::path::Path;
use std::process::Command;
use crate::core::config::Script;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScriptResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub returncode: i32,
}

pub fn run_script(script: &Script, param: &str, data_center: &str) -> ScriptResult {
    let script_full_path = Path::new(data_center).join("scripts").join(&script.script_path);

    if !script_full_path.exists() {
        return ScriptResult {
            success: false,
            stdout: String::new(),
            stderr: format!("脚本文件不存在: {}", script_full_path.display()),
            returncode: -1,
        };
    }

    let date_folder = parse_param_to_folder(param);
    let output_dir = Path::new(data_center).join("output").join(&date_folder);
    let _ = fs::create_dir_all(&output_dir);

    let python_exe = find_python();

    let output = Command::new(&python_exe)
        .arg(&script_full_path)
        .arg(param)
        .env("EAA_DATA_CENTER", data_center)
        .env("EAA_DATE_PARAM", param)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = if stdout.len() > 2000 {
                stdout[stdout.len()-2000..].to_string()
            } else {
                stdout.to_string()
            };
            let stderr = if stderr.len() > 2000 {
                stderr[stderr.len()-2000..].to_string()
            } else {
                stderr.to_string()
            };
            ScriptResult {
                success: output.status.success(),
                stdout,
                stderr,
                returncode: output.status.code().unwrap_or(-1),
            }
        }
        Err(e) => ScriptResult {
            success: false,
            stdout: String::new(),
            stderr: e.to_string(),
            returncode: -1,
        },
    }
}

fn parse_param_to_folder(param: &str) -> String {
    let clean = param.replace('-', "");
    if clean.len() == 8 {
        format!("{}-{}-{}", &clean[0..4], &clean[4..6], &clean[6..8])
    } else if clean.len() == 6 {
        format!("{}-{}", &clean[0..4], &clean[4..6])
    } else {
        param.to_string()
    }
}

fn find_python() -> String {
    if let Ok(python3) = which::which("python3") {
        return python3.to_string_lossy().to_string();
    }
    if let Ok(python) = which::which("python") {
        return python.to_string_lossy().to_string();
    }
    for path in &["/usr/bin/python3", "/usr/local/bin/python3", "/opt/homebrew/bin/python3"] {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }
    "python3".to_string()
}
