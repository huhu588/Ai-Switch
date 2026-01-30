// 应用设置命令模块
// 处理关闭窗口行为、自动启动等应用设置

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_store::StoreExt;
use tauri_plugin_autostart::ManagerExt;
use std::sync::Mutex;

use crate::config::ConfigManager;

/// 关闭窗口时的行为
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloseAction {
    /// 每次询问
    Ask,
    /// 最小化到托盘
    Tray,
    /// 直接退出
    Quit,
}

impl Default for CloseAction {
    fn default() -> Self {
        CloseAction::Ask
    }
}

impl std::fmt::Display for CloseAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloseAction::Ask => write!(f, "ask"),
            CloseAction::Tray => write!(f, "tray"),
            CloseAction::Quit => write!(f, "quit"),
        }
    }
}

impl std::str::FromStr for CloseAction {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ask" => Ok(CloseAction::Ask),
            "tray" => Ok(CloseAction::Tray),
            "quit" => Ok(CloseAction::Quit),
            _ => Err(format!("Unknown close action: {}", s)),
        }
    }
}

/// 应用设置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 关闭窗口时的行为
    pub close_action: CloseAction,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            close_action: CloseAction::Ask,
        }
    }
}

const SETTINGS_STORE_KEY: &str = "app_settings";

/// 获取应用设置
#[tauri::command]
pub async fn get_app_settings(
    app: tauri::AppHandle,
    _config: State<'_, Mutex<ConfigManager>>,
) -> Result<AppSettings, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value(value.clone())
            .map_err(|e| e.to_string())
    } else {
        Ok(AppSettings::default())
    }
}

/// 保存应用设置
#[tauri::command]
pub async fn save_app_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
    _config: State<'_, Mutex<ConfigManager>>,
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    store.set(SETTINGS_STORE_KEY, value);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 获取关闭行为设置
#[tauri::command]
pub async fn get_close_action(
    app: tauri::AppHandle,
    _config: State<'_, Mutex<ConfigManager>>,
) -> Result<String, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    let settings = if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value::<AppSettings>(value.clone()).map_err(|e| {
            eprintln!("读取 settings.json 失败: {}", e);
            format!("读取 settings.json 失败: {}", e)
        })?
    } else {
        AppSettings::default()
    };
    
    Ok(settings.close_action.to_string())
}

/// 设置关闭行为
#[tauri::command]
pub async fn set_close_action(
    app: tauri::AppHandle,
    action: String,
    _config: State<'_, Mutex<ConfigManager>>,
) -> Result<(), String> {
    let close_action: CloseAction = action.parse()?;
    
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    // 读取现有设置或创建默认设置
    let mut settings = if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value::<AppSettings>(value.clone()).map_err(|e| {
            eprintln!("读取 settings.json 失败: {}", e);
            format!("读取 settings.json 失败: {}", e)
        })?
    } else {
        AppSettings::default()
    };
    
    settings.close_action = close_action;
    
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    store.set(SETTINGS_STORE_KEY, value);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 处理用户的关闭选择（统一由后端处理窗口操作）
#[tauri::command]
pub async fn handle_close_choice(
    app: tauri::AppHandle,
    choice: String,
) -> Result<(), String> {
    match choice.as_str() {
        "tray" => {
            // 隐藏窗口到托盘
            if let Some(window) = app.get_webview_window("main") {
                window.hide().map_err(|e| e.to_string())?;
            }
        }
        "quit" => {
            // 退出应用
            app.exit(0);
        }
        _ => {
            return Err(format!("Unknown choice: {}", choice));
        }
    }
    Ok(())
}

// ============== 自动启动设置 ==============

/// 获取自动启动状态
#[tauri::command]
pub async fn get_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app.autolaunch();
    autostart_manager.is_enabled().map_err(|e| e.to_string())
}

/// 设置自动启动
#[tauri::command]
pub async fn set_autostart_enabled(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autostart_manager = app.autolaunch();
    
    if enabled {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// ============== 环境变量冲突检测 ==============

/// 冲突来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSource {
    pub app: String,         // "claude", "codex", "gemini"
    pub value: String,       // 脱敏后的值
    pub config_path: String, // 配置文件路径
}

/// 环境变量冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConflict {
    pub variable: String,
    pub sources: Vec<ConflictSource>,
}

/// 脱敏处理：只显示前4个字符和后4个字符
fn mask_value(value: &str) -> String {
    if value.len() <= 12 {
        "*".repeat(value.len().min(8))
    } else {
        format!("{}...{}", &value[..4], &value[value.len()-4..])
    }
}

/// 检测环境变量冲突
#[tauri::command]
pub async fn detect_env_conflicts() -> Result<Vec<EnvConflict>, String> {
    use std::collections::HashMap;
    
    let mut env_map: HashMap<String, Vec<ConflictSource>> = HashMap::new();
    
    // 1. 读取 Claude Code 配置
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let claude_path = std::path::Path::new(&home).join(".claude").join("settings.json");
        if claude_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&claude_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(env) = json.get("env").and_then(|v| v.as_object()) {
                        for (key, value) in env {
                            if let Some(val_str) = value.as_str() {
                                env_map.entry(key.clone()).or_default().push(ConflictSource {
                                    app: "Claude Code".to_string(),
                                    value: mask_value(val_str),
                                    config_path: claude_path.display().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 2. 读取 Codex 配置
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let codex_auth_path = std::path::Path::new(&home).join(".codex").join("auth.json");
        if codex_auth_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&codex_auth_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(obj) = json.as_object() {
                        for (key, value) in obj {
                            if let Some(val_str) = value.as_str() {
                                env_map.entry(key.clone()).or_default().push(ConflictSource {
                                    app: "Codex".to_string(),
                                    value: mask_value(val_str),
                                    config_path: codex_auth_path.display().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // 读取 config.toml 中的 env_key
        let codex_config_path = std::path::Path::new(&home).join(".codex").join("config.toml");
        if codex_config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&codex_config_path) {
                // 简单解析 env_key = "XXX" 行
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("env_key") {
                        if let Some(value) = line.split('=').nth(1) {
                            let key = value.trim().trim_matches('"').trim_matches('\'');
                            if !key.is_empty() {
                                env_map.entry(key.to_string()).or_default().push(ConflictSource {
                                    app: "Codex".to_string(),
                                    value: "(env_key reference)".to_string(),
                                    config_path: codex_config_path.display().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 3. 读取 Gemini 配置
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let gemini_env_path = std::path::Path::new(&home).join(".gemini").join(".env");
        if gemini_env_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&gemini_env_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"').trim_matches('\'');
                        env_map.entry(key.to_string()).or_default().push(ConflictSource {
                            app: "Gemini".to_string(),
                            value: mask_value(value),
                            config_path: gemini_env_path.display().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // 4. 筛选出有冲突的变量（被多个工具使用）
    let conflicts: Vec<EnvConflict> = env_map
        .into_iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|(variable, sources)| EnvConflict { variable, sources })
        .collect();
    
    Ok(conflicts)
}
