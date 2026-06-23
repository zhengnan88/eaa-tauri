import { invoke } from '@tauri-apps/api/core';

export interface Rule {
  id: string;
  name: string;
  feature_columns: string[];
  feature_row: number;
  date_column: string;
  force_precision: string;
  naming_pattern: string;
  auto_archive: boolean;
}

export interface Script {
  id: string;
  name: string;
  script_path: string;
  param_type: string;
  order: number;
}

export interface MonitorFolder {
  path: string;
  include_sub: boolean;
  scan_interval_min: number;
  last_scan_time: string;
}

export interface AppSettings {
  ignore_case: boolean;
  ignore_space: boolean;
  log_retention_days: number;
}

export interface ArchiveResult {
  success: boolean;
  file_name: string;
  rule_name: string;
  date: string;
  target_path: string;
  status: string;
  message: string;
}

export interface ScriptResult {
  success: boolean;
  stdout: string;
  stderr: string;
  returncode: number;
}

export interface DataReadyStatus {
  rule_name: string;
  date: string;
  file_name: string;
  is_ready: boolean;
}

export interface HeaderResult {
  headers: string[];
  detected_row: number;
}

// Archive commands
export async function processFiles(filePaths: string[]): Promise<ArchiveResult[]> {
  return invoke<ArchiveResult[]>('process_files', { filePaths });
}

// Rules commands
export async function getRules(): Promise<Rule[]> {
  return invoke<Rule[]>('get_rules');
}

export async function addRule(ruleData: Rule): Promise<Rule> {
  return invoke<Rule>('add_rule', { ruleData });
}

export async function updateRule(ruleId: string, ruleData: Rule): Promise<Rule> {
  return invoke<Rule>('update_rule', { ruleId, ruleData });
}

export async function deleteRule(ruleId: string): Promise<boolean> {
  return invoke<boolean>('delete_rule', { ruleId });
}

export async function moveRule(ruleId: string, direction: string): Promise<boolean> {
  return invoke<boolean>('move_rule', { ruleId, direction });
}

export async function loadExcelHeaders(filePath: string, featureRow: number): Promise<HeaderResult> {
  return invoke<HeaderResult>('load_excel_headers', { filePath, featureRow });
}

// Scripts commands
export async function getScripts(): Promise<Script[]> {
  return invoke<Script[]>('get_scripts');
}

export async function addScript(scriptData: Script): Promise<Script> {
  return invoke<Script>('add_script', { scriptData });
}

export async function updateScript(scriptId: string, scriptData: Script): Promise<Script> {
  return invoke<Script>('update_script', { scriptId, scriptData });
}

export async function deleteScript(scriptId: string): Promise<boolean> {
  return invoke<boolean>('delete_script', { scriptId });
}

export async function moveScript(scriptId: string, direction: string): Promise<boolean> {
  return invoke<boolean>('move_script', { scriptId, direction });
}

export async function runScript(scriptId: string, param: string): Promise<ScriptResult> {
  return invoke<ScriptResult>('run_script', { scriptId, param });
}

// Settings commands
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings');
}

export async function saveSettings(settingsData: AppSettings): Promise<boolean> {
  return invoke<boolean>('save_settings', { settingsData });
}

export async function getDataCenter(): Promise<string> {
  return invoke<string>('get_data_center');
}

export async function setDataCenter(path: string): Promise<{ success: boolean; message: string }> {
  return invoke<{ success: boolean; message: string }>('set_data_center', { path });
}

export async function getMonitorFolders(): Promise<MonitorFolder[]> {
  return invoke<MonitorFolder[]>('get_monitor_folders');
}

export async function addMonitorFolder(folderData: MonitorFolder): Promise<MonitorFolder> {
  return invoke<MonitorFolder>('add_monitor_folder', { folderData });
}

export async function updateMonitorFolder(index: number, folderData: MonitorFolder): Promise<MonitorFolder> {
  return invoke<MonitorFolder>('update_monitor_folder', { index, folderData });
}

export async function deleteMonitorFolder(index: number): Promise<boolean> {
  return invoke<boolean>('delete_monitor_folder', { index });
}

export async function checkPathAccess(path: string): Promise<{ accessible: boolean; message: string }> {
  return invoke<{ accessible: boolean; message: string }>('check_path_access', { path });
}

export async function scanAllMonitors(): Promise<any> {
  return invoke('scan_all_monitors');
}

export async function scanTodayMonitors(): Promise<any> {
  return invoke('scan_today_monitors');
}

export async function pickFolder(): Promise<string | null> {
  return invoke<string | null>('pick_folder');
}

// Status commands
export async function getDataReadyStatus(dateStr: string): Promise<DataReadyStatus[]> {
  return invoke<DataReadyStatus[]>('get_data_ready_status', { dateStr });
}
