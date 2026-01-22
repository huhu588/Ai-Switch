// 应用设置命令模块
// 处理关闭窗口行为等应用设置

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_store::StoreExt;
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
        serde_json::from_value::<AppSettings>(value.clone()).unwrap_or_default()
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
        serde_json::from_value::<AppSettings>(value.clone()).unwrap_or_default()
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
