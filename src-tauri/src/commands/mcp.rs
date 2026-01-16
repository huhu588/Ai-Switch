// MCP 服务器相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

use crate::config::{ConfigManager, McpServer, McpServerType, McpOAuthConfig};
use crate::error::AppError;

/// MCP 服务器列表项
#[derive(Debug, Clone, Serialize)]
pub struct McpServerItem {
    pub name: String,
    pub server_type: String,
    pub enabled: bool,
    pub url: Option<String>,
    pub command: Option<Vec<String>>,
}

/// MCP 服务器输入
#[derive(Debug, Deserialize)]
pub struct McpServerInput {
    pub name: String,
    pub server_type: String, // "local" | "remote"
    pub enabled: bool,
    pub timeout: Option<u32>,
    // Local
    pub command: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
    // Remote
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub oauth: Option<OAuthInput>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthInput {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scope: Option<String>,
}

/// 同步配置参数
#[derive(Debug, Deserialize)]
pub struct SyncMcpInput {
    pub server_names: Vec<String>,
    pub sync_to_global: bool,
    pub sync_to_project: bool,
}

/// 获取所有 MCP 服务器列表
#[tauri::command]
pub fn get_mcp_servers(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<McpServerItem>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mcp_config = manager.mcp().read_config()?;
    
    let mut items: Vec<McpServerItem> = mcp_config
        .servers
        .iter()
        .map(|(name, server)| McpServerItem {
            name: name.clone(),
            server_type: server.server_type.to_string(),
            enabled: server.enabled,
            url: server.url.clone(),
            command: server.command.clone(),
        })
        .collect();
    
    items.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(items)
}

/// 获取单个 MCP 服务器详情
#[tauri::command]
pub fn get_mcp_server(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Option<McpServer>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    Ok(manager.mcp().get_server(&name)?)
}

/// 添加 MCP 服务器
#[tauri::command]
pub fn add_mcp_server(
    input: McpServerInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let server = build_mcp_server(&input)?;
    manager.mcp_mut().save_server(&input.name, server)?;
    
    Ok(())
}

/// 更新 MCP 服务器
#[tauri::command]
pub fn update_mcp_server(
    old_name: String,
    input: McpServerInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 如果名称变了，需要重命名
    if old_name != input.name {
        manager.mcp_mut().rename_server(&old_name, &input.name)?;
    }
    
    let server = build_mcp_server(&input)?;
    manager.mcp_mut().save_server(&input.name, server)?;
    
    Ok(())
}

/// 删除 MCP 服务器
#[tauri::command]
pub fn delete_mcp_server(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.mcp_mut().delete_server(&name)?;
    Ok(())
}

/// 切换 MCP 服务器启用状态
#[tauri::command]
pub fn toggle_mcp_server(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<bool, AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let new_state = manager.mcp_mut().toggle_server_enabled(&name)?;
    Ok(new_state)
}

/// 同步 MCP 配置到 opencode.json
#[tauri::command]
pub fn sync_mcp_config(
    input: SyncMcpInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let names = if input.server_names.is_empty() {
        None
    } else {
        Some(input.server_names.as_slice())
    };
    
    if input.sync_to_global {
        manager.mcp().sync_to_opencode(names)?;
    }
    
    if input.sync_to_project {
        manager.mcp().sync_to_project(names)?;
    }
    
    Ok(())
}

/// 构建 McpServer 对象
fn build_mcp_server(input: &McpServerInput) -> Result<McpServer, AppError> {
    let server_type = match input.server_type.as_str() {
        "local" => McpServerType::Local,
        "remote" => McpServerType::Remote,
        _ => return Err(AppError::Custom("无效的服务器类型".to_string())),
    };
    
    let oauth = input.oauth.as_ref().map(|o| McpOAuthConfig {
        client_id: o.client_id.clone(),
        client_secret: o.client_secret.clone(),
        scope: o.scope.clone(),
    });
    
    Ok(McpServer {
        server_type,
        enabled: input.enabled,
        timeout: input.timeout,
        command: input.command.clone(),
        environment: input.environment.clone().unwrap_or_default(),
        url: input.url.clone(),
        headers: input.headers.clone().unwrap_or_default(),
        oauth,
        metadata: Default::default(),
    })
}
