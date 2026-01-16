// Provider 相关的 Tauri commands

use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

use crate::config::{ConfigManager, OpenCodeProvider};
use crate::error::AppError;

/// Provider 列表项（传递给前端）
#[derive(Debug, Clone, Serialize)]
pub struct ProviderItem {
    pub name: String,
    pub base_url: String,
    pub model_count: usize,
    pub description: Option<String>,
}

/// 添加/编辑 Provider 的参数
#[derive(Debug, Deserialize)]
pub struct ProviderInput {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub npm: Option<String>,
    pub description: Option<String>,
}

/// 应用配置的参数
#[derive(Debug, Deserialize)]
pub struct ApplyConfigInput {
    pub provider_names: Vec<String>,
    pub apply_to_global: bool,
    pub apply_to_project: bool,
}

/// 获取所有 Provider 列表
#[tauri::command]
pub fn get_providers(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<ProviderItem>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let providers = manager.opencode().get_all_providers()?;
    
    let mut items: Vec<ProviderItem> = providers
        .iter()
        .map(|(name, provider)| ProviderItem {
            name: name.clone(),
            base_url: provider.options.base_url.clone(),
            model_count: provider.models.len(),
            description: provider.metadata.description.clone(),
        })
        .collect();
    
    // 按名称排序
    items.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(items)
}

/// 获取单个 Provider 详情
#[tauri::command]
pub fn get_provider(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Option<OpenCodeProvider>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    Ok(manager.opencode().get_provider(&name)?)
}

/// 添加新 Provider
#[tauri::command]
pub fn add_provider(
    input: ProviderInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode_mut().add_provider(
        input.name,
        input.base_url,
        input.api_key,
        input.npm,
        input.description,
    )?;
    Ok(())
}

/// 更新 Provider
#[tauri::command]
pub fn update_provider(
    name: String,
    input: ProviderInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode_mut().update_provider_metadata(
        &name,
        Some(input.base_url),
        Some(input.api_key),
        input.npm,
        input.description,
    )?;
    Ok(())
}

/// 删除 Provider
#[tauri::command]
pub fn delete_provider(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode_mut().delete_provider(&name)?;
    Ok(())
}

/// 应用配置到全局/项目
#[tauri::command]
pub fn apply_config(
    input: ApplyConfigInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    if input.apply_to_global {
        manager.apply_multiple_opencode_to_global(&input.provider_names)?;
    }
    
    if input.apply_to_project {
        manager.apply_multiple_opencode_to_project(&input.provider_names)?;
    }
    
    Ok(())
}
