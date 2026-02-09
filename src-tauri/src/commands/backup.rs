// Backup and Import module
// Supports exporting and importing providers, MCP, rules, and skills

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

use crate::config::{ConfigManager, McpServer, McpServerType};
use crate::config::codex_manager::CodexConfigManager;
use crate::config::gemini_manager::GeminiConfigManager;
use crate::error::AppError;
use super::model::build_variants;

/// Backup file format version
const BACKUP_VERSION: &str = "1.2.0";

/// Exported Provider data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedProvider {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub npm: Option<String>,
    pub description: Option<String>,
    pub model_type: Option<String>,
    pub enabled: bool,
    pub models: Vec<ExportedModel>,
}

/// Exported Model data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedModel {
    pub id: String,
    pub name: String,
    pub reasoning_effort: Option<String>,
}

/// Exported OAuth config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedOAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scope: Option<String>,
}

/// Exported MCP server data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedMcpServer {
    pub name: String,
    pub server_type: String,
    pub enabled: bool,
    pub timeout: Option<u32>,
    pub command: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    /// OAuth 配置（用于远程服务器认证）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<ExportedOAuthConfig>,
}

/// Exported Rule data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedRule {
    pub name: String,
    pub location: String,
    pub rule_type: String,
    pub content: String,
    /// 文件扩展名 (md 或 mdc)，用于导入时正确恢复
    #[serde(default = "default_file_ext")]
    pub file_ext: String,
}

fn default_file_ext() -> String {
    "md".to_string()
}

/// Exported skills data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSkills {
    pub name: String,
    pub location: String,
    pub content: String,
}

// ==================== Codex CLI 配置导出结构 ====================

/// Exported Codex model provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedCodexProvider {
    pub name: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_openai_auth: Option<bool>,
}

/// Exported Codex MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedCodexMcpServer {
    pub name: String,
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// Exported Codex configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportedCodexConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_providers: Vec<ExportedCodexProvider>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<ExportedCodexMcpServer>,
}

// ==================== Gemini CLI 配置导出结构 ====================

/// Exported Gemini environment configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportedGeminiEnv {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_gemini_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_gemini_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini_model: Option<String>,
}

/// Exported Gemini MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedGeminiMcpServer {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Exported Gemini configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportedGeminiConfig {
    #[serde(default)]
    pub env: ExportedGeminiEnv,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<ExportedGeminiMcpServer>,
}

/// Exported usage stats record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedUsageRecord {
    pub session_id: String,
    pub timestamp: i64,
    pub model: String,
    pub source: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_read_tokens: u32,
    #[serde(default)]
    pub cache_creation_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
}

/// 导出的对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use: Option<serde_json::Value>,
}

/// 导出的对话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedChatConversation {
    pub messages: Vec<ExportedChatMessage>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<f64>,
}

/// 导出的开发环境（仅保存版本号，跨设备重装）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedDevEnv {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Complete backup data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub version: String,
    pub created_at: String,
    pub app_name: String,
    pub providers: Vec<ExportedProvider>,
    pub mcp_servers: Vec<ExportedMcpServer>,
    pub rules: Vec<ExportedRule>,
    pub skills: Vec<ExportedSkills>,
    /// Codex CLI 配置（v1.1.0 新增）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_config: Option<ExportedCodexConfig>,
    /// Gemini CLI 配置（v1.1.0 新增）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini_config: Option<ExportedGeminiConfig>,
    /// 使用统计数据
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_stats: Option<Vec<ExportedUsageRecord>>,
    /// 对话记录
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_conversations: Option<Vec<ExportedChatConversation>>,
    /// 开发环境（仅版本号）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dev_envs: Option<Vec<ExportedDevEnv>>,
}

/// Export statistics
#[derive(Debug, Clone, Serialize)]
pub struct ExportStats {
    pub providers: usize,
    pub models: usize,
    pub mcp_servers: usize,
    pub rules: usize,
    pub skills: usize,
    /// Codex model providers 数量
    pub codex_providers: usize,
    /// Codex MCP servers 数量
    pub codex_mcp_servers: usize,
    /// Gemini 配置是否存在
    pub gemini_configured: bool,
    /// Gemini MCP servers 数量
    pub gemini_mcp_servers: usize,
    /// 使用统计记录数
    #[serde(default)]
    pub usage_records: usize,
    /// 对话记录数量
    #[serde(default)]
    pub chat_conversations: usize,
}

/// Export options - 选择性导出
#[derive(Debug, Clone, Deserialize)]
pub struct ExportOptions {
    #[serde(default = "default_true")]
    pub include_providers: bool,
    #[serde(default = "default_true")]
    pub include_mcp: bool,
    #[serde(default = "default_true")]
    pub include_rules: bool,
    #[serde(default = "default_true")]
    pub include_skills: bool,
    #[serde(default = "default_true")]
    pub include_codex: bool,
    #[serde(default = "default_true")]
    pub include_gemini: bool,
    /// 是否导出使用统计数据
    #[serde(default)]
    pub include_usage_stats: bool,
}

fn default_true() -> bool { true }

/// 精细化导出选项 - 按名称级别过滤
#[derive(Debug, Clone, Deserialize)]
pub struct FilteredExportOptions {
    /// 选中的 OpenCode provider 名称
    #[serde(default)]
    pub provider_names: Vec<String>,
    /// 选中的 OpenCode MCP 服务器名称
    #[serde(default)]
    pub mcp_names: Vec<String>,
    /// 选中的规则标识 "name|location"
    #[serde(default)]
    pub rule_ids: Vec<String>,
    /// 选中的 skill 标识 "name|location"
    #[serde(default)]
    pub skill_ids: Vec<String>,
    /// 选中的 Codex CLI provider 名称
    #[serde(default)]
    pub codex_provider_names: Vec<String>,
    /// 选中的 Codex CLI MCP 服务器名称
    #[serde(default)]
    pub codex_mcp_names: Vec<String>,
    /// 是否包含 Gemini 环境配置
    #[serde(default)]
    pub include_gemini_env: bool,
    /// 选中的 Gemini CLI MCP 服务器名称
    #[serde(default)]
    pub gemini_mcp_names: Vec<String>,
    /// 使用统计来源列表 (claude/codex/gemini/opencode/cursor)，空数组则不导出
    #[serde(default)]
    pub usage_sources: Vec<String>,
    /// 需要备份的开发环境（仅版本号）
    #[serde(default)]
    pub dev_envs: Vec<ExportedDevEnv>,
}

/// Import options
#[derive(Debug, Clone, Deserialize)]
pub struct ImportOptions {
    pub import_providers: bool,
    pub import_mcp: bool,
    pub import_rules: bool,
    pub import_skills: bool,
    pub overwrite_existing: bool,
    /// 是否导入 Codex CLI 配置
    #[serde(default)]
    pub import_codex: bool,
    /// 是否导入 Gemini CLI 配置
    #[serde(default)]
    pub import_gemini: bool,
    /// 是否导入使用统计数据
    #[serde(default)]
    pub import_usage_stats: bool,
}

/// Import result
#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub providers_imported: usize,
    pub providers_skipped: usize,
    pub mcp_imported: usize,
    pub mcp_skipped: usize,
    pub rules_imported: usize,
    pub rules_skipped: usize,
    pub skills_imported: usize,
    pub skills_skipped: usize,
    /// Codex 配置导入数量
    pub codex_imported: usize,
    /// Codex 配置跳过数量
    pub codex_skipped: usize,
    /// Gemini 配置导入数量
    pub gemini_imported: usize,
    /// Gemini 配置跳过数量
    pub gemini_skipped: usize,
    /// 使用统计导入数量
    #[serde(default)]
    pub usage_imported: usize,
    /// 使用统计跳过数量
    #[serde(default)]
    pub usage_skipped: usize,
    pub errors: Vec<String>,
}

fn get_skills_paths() -> Vec<(PathBuf, String)> {
    let mut paths = Vec::new();
    // 与 opencode CLI 保持一致，所有平台使用 ~/.config/opencode
    if let Some(home_dir) = dirs::home_dir() {
        paths.push((
            home_dir.join(".config").join("opencode").join("skills"),
            "global_opencode".to_string(),
        ));
        paths.push((
            home_dir.join(".claude").join("skills"),
            "global_claude".to_string(),
        ));
        // Codex CLI skills 路径
        paths.push((
            home_dir.join(".codex").join("skills"),
            "global_codex".to_string(),
        ));
        // Gemini CLI skills 路径
        paths.push((
            home_dir.join(".gemini").join("skills"),
            "global_gemini".to_string(),
        ));
    }
    paths
}

fn get_rule_paths() -> HashMap<String, PathBuf> {
    let mut paths = HashMap::new();
    if let Some(home) = dirs::home_dir() {
        paths.insert("global_opencode".to_string(), home.join(".config").join("opencode"));
        paths.insert("global_claude".to_string(), home.join(".claude"));
        // Codex CLI rules 路径
        paths.insert("global_codex".to_string(), home.join(".codex"));
        // Gemini CLI rules 路径
        paths.insert("global_gemini".to_string(), home.join(".gemini"));
    }
    paths
}

fn create_backup_internal(manager: &ConfigManager) -> Result<BackupData, AppError> {
    let providers_map = manager.opencode().get_all_providers()?;
    let mut providers: Vec<ExportedProvider> = Vec::new();
    
    for (name, provider) in providers_map {
        let models: Vec<ExportedModel> = provider.models
            .iter()
            .map(|(id, info)| ExportedModel {
                id: id.clone(),
                name: info.name.clone(),
                reasoning_effort: info.reasoning_effort.clone(),
            })
            .collect();
        
        providers.push(ExportedProvider {
            name,
            base_url: provider.options.base_url.clone(),
            api_key: provider.options.api_key.clone(),
            npm: provider.npm.clone(),
            description: provider.metadata.description.clone(),
            model_type: provider.model_type.clone(),
            enabled: provider.enabled,
            models,
        });
    }
    
    let mcp_config = manager.mcp().read_config()?;
    let mcp_servers: Vec<ExportedMcpServer> = mcp_config.servers
        .iter()
        .map(|(name, server)| {
            // 转换 OAuth 配置
            let oauth = server.oauth.as_ref().map(|o| ExportedOAuthConfig {
                client_id: o.client_id.clone(),
                client_secret: o.client_secret.clone(),
                scope: o.scope.clone(),
            });
            
            ExportedMcpServer {
                name: name.clone(),
                server_type: server.server_type.to_string(),
                enabled: server.enabled,
                timeout: server.timeout,
                command: server.command.clone(),
                environment: if server.environment.is_empty() { None } else { Some(server.environment.clone()) },
                url: server.url.clone(),
                headers: if server.headers.is_empty() { None } else { Some(server.headers.clone()) },
                oauth,
            }
        })
        .collect();
    
    let mut rules: Vec<ExportedRule> = Vec::new();
    let rule_paths = get_rule_paths();
    
    for (location_key, base_path) in &rule_paths {
        // 读取 AGENTS.md (OpenCode 和 Codex)
        if location_key == "global_opencode" || location_key == "global_codex" {
            let agents_path = base_path.join("AGENTS.md");
            if agents_path.exists() {
                if let Ok(content) = fs::read_to_string(&agents_path) {
                    rules.push(ExportedRule {
                        name: "AGENTS.md".to_string(),
                        location: location_key.clone(),
                        rule_type: "agents_md".to_string(),
                        content,
                        file_ext: "md".to_string(),
                    });
                }
            }
        }
        
        // 读取 GEMINI.md (Gemini CLI)
        if location_key == "global_gemini" {
            let gemini_md_path = base_path.join("GEMINI.md");
            if gemini_md_path.exists() {
                if let Ok(content) = fs::read_to_string(&gemini_md_path) {
                    rules.push(ExportedRule {
                        name: "GEMINI.md".to_string(),
                        location: location_key.clone(),
                        rule_type: "gemini_md".to_string(),
                        content,
                        file_ext: "md".to_string(),
                    });
                }
            }
        }
        
        let rules_dir = base_path.join("rules");
        if rules_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&rules_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("md");
                        if ext == "md" || ext == "mdc" {
                            if let Ok(content) = fs::read_to_string(&path) {
                                let name = path.file_stem()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                rules.push(ExportedRule {
                                    name,
                                    location: location_key.clone(),
                                    rule_type: "rules_dir".to_string(),
                                    content,
                                    file_ext: ext.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut skills: Vec<ExportedSkills> = Vec::new();
    for (base_path, location) in get_skills_paths() {
        if !base_path.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skills_file = path.join("SKILL.md");
                    if skills_file.exists() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        if let Ok(content) = fs::read_to_string(&skills_file) {
                            skills.push(ExportedSkills {
                                name,
                                location: location.clone(),
                                content,
                            });
                        }
                    }
                }
            }
        }
    }
    
    // 读取 Codex CLI 配置
    let codex_config = read_codex_config();
    
    // 读取 Gemini CLI 配置
    let gemini_config = read_gemini_config();
    
    Ok(BackupData {
        version: BACKUP_VERSION.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        app_name: "Ai Switch".to_string(),
        providers,
        mcp_servers,
        rules,
        skills,
        codex_config,
        gemini_config,
        usage_stats: None,
        chat_conversations: None,
        dev_envs: None,
    })
}

/// 读取 Codex CLI 配置
fn read_codex_config() -> Option<ExportedCodexConfig> {
    let codex_manager = CodexConfigManager::new().ok()?;
    
    // 读取 model_providers
    let model_providers: Vec<ExportedCodexProvider> = codex_manager
        .get_model_providers()
        .ok()
        .map(|providers| {
            providers
                .into_iter()
                .map(|(name, provider)| ExportedCodexProvider {
                    name,
                    base_url: provider.base_url,
                    env_key: provider.env_key,
                    requires_openai_auth: provider.requires_openai_auth,
                })
                .collect()
        })
        .unwrap_or_default();
    
    // 读取 mcp_servers
    let mcp_servers: Vec<ExportedCodexMcpServer> = codex_manager
        .get_mcp_servers()
        .ok()
        .map(|servers| {
            servers
                .into_iter()
                .map(|(name, server)| ExportedCodexMcpServer {
                    name,
                    command: server.command,
                    env: server.env,
                })
                .collect()
        })
        .unwrap_or_default();
    
    // 如果没有任何配置，返回 None
    if model_providers.is_empty() && mcp_servers.is_empty() {
        return None;
    }
    
    Some(ExportedCodexConfig {
        model_providers,
        mcp_servers,
    })
}

/// 读取 Gemini CLI 配置
fn read_gemini_config() -> Option<ExportedGeminiConfig> {
    let gemini_manager = GeminiConfigManager::new().ok()?;
    
    // 读取环境配置（即使失败也继续，因为可能只有 MCP 配置）
    let env = gemini_manager.read_env().ok()
        .map(|env_config| ExportedGeminiEnv {
            gemini_api_key: env_config.gemini_api_key,
            google_gemini_api_key: env_config.google_gemini_api_key,
            google_gemini_base_url: env_config.google_gemini_base_url,
            gemini_model: env_config.gemini_model,
        })
        .unwrap_or_default();
    
    // 读取 MCP servers
    let mcp_servers: Vec<ExportedGeminiMcpServer> = gemini_manager
        .get_mcp_servers()
        .ok()
        .map(|servers| {
            servers
                .into_iter()
                .map(|(name, server)| ExportedGeminiMcpServer {
                    name,
                    command: server.command,
                    args: server.args,
                    env: server.env,
                    url: server.url,
                })
                .collect()
        })
        .unwrap_or_default();
    
    // 检查是否有任何配置
    let has_env = env.gemini_api_key.is_some() 
        || env.google_gemini_api_key.is_some() 
        || env.google_gemini_base_url.is_some()
        || env.gemini_model.is_some();
    
    if !has_env && mcp_servers.is_empty() {
        return None;
    }
    
    Some(ExportedGeminiConfig {
        env,
        mcp_servers,
    })
}

#[tauri::command]
pub fn create_backup(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<BackupData, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    create_backup_internal(&manager)
}

#[tauri::command]
pub fn export_backup(
    file_path: String,
    options: Option<ExportOptions>,
    config_manager: State<'_, Mutex<ConfigManager>>,
    db: State<'_, std::sync::Arc<crate::database::Database>>,
) -> Result<ExportStats, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut backup = create_backup_internal(&manager)?;

    let opts = options.unwrap_or(ExportOptions {
        include_providers: true, include_mcp: true, include_rules: true,
        include_skills: true, include_codex: true, include_gemini: true, include_usage_stats: false,
    });

    // 按选项过滤
    if !opts.include_providers { backup.providers.clear(); }
    if !opts.include_mcp { backup.mcp_servers.clear(); }
    if !opts.include_rules { backup.rules.clear(); }
    if !opts.include_skills { backup.skills.clear(); }
    if !opts.include_codex { backup.codex_config = None; }
    if !opts.include_gemini { backup.gemini_config = None; }

    // 导出使用统计
    if opts.include_usage_stats {
        let conn = db.conn.lock().map_err(|e| AppError::Custom(format!("DB lock failed: {}", e)))?;
        let mut usage_records = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT session_id, created_at, model, app_type, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost FROM proxy_request_logs ORDER BY created_at"
        ) {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok(ExportedUsageRecord {
                    session_id: row.get::<_, String>(0).unwrap_or_default(),
                    timestamp: row.get::<_, i64>(1).unwrap_or(0),
                    model: row.get::<_, String>(2).unwrap_or_default(),
                    source: row.get::<_, String>(3).unwrap_or_default(),
                    input_tokens: row.get::<_, u32>(4).unwrap_or(0),
                    output_tokens: row.get::<_, u32>(5).unwrap_or(0),
                    cache_read_tokens: row.get::<_, u32>(6).unwrap_or(0),
                    cache_creation_tokens: row.get::<_, u32>(7).unwrap_or(0),
                    cost: row.get::<_, f64>(8).ok(),
                })
            }) {
                for r in rows.flatten() { usage_records.push(r); }
            }
        }
        backup.usage_stats = Some(usage_records);
    }

    let stats = ExportStats {
        providers: backup.providers.len(),
        models: backup.providers.iter().map(|p| p.models.len()).sum(),
        mcp_servers: backup.mcp_servers.len(),
        rules: backup.rules.len(),
        skills: backup.skills.len(),
        codex_providers: backup.codex_config.as_ref().map(|c| c.model_providers.len()).unwrap_or(0),
        codex_mcp_servers: backup.codex_config.as_ref().map(|c| c.mcp_servers.len()).unwrap_or(0),
        gemini_configured: backup.gemini_config.is_some(),
        gemini_mcp_servers: backup.gemini_config.as_ref().map(|c| c.mcp_servers.len()).unwrap_or(0),
        usage_records: backup.usage_stats.as_ref().map(|r| r.len()).unwrap_or(0),
        chat_conversations: 0,
    };
    
    let content = serde_json::to_string_pretty(&backup)
        .map_err(|e| AppError::Custom(format!("Failed to serialize: {}", e)))?;
    
    fs::write(&file_path, content)
        .map_err(|e| AppError::Custom(format!("Failed to write file: {}", e)))?;
    
    Ok(stats)
}

#[tauri::command]
pub fn export_backup_filtered(
    file_path: String,
    options: FilteredExportOptions,
    chat_conversations: Option<Vec<ExportedChatConversation>>,
    config_manager: State<'_, Mutex<ConfigManager>>,
    db: State<'_, std::sync::Arc<crate::database::Database>>,
) -> Result<ExportStats, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut backup = create_backup_internal(&manager)?;

    // 按名称过滤 providers
    backup.providers.retain(|p| options.provider_names.contains(&p.name));
    
    // 按名称过滤 MCP
    backup.mcp_servers.retain(|m| options.mcp_names.contains(&m.name));
    
    // 按 "name|location" 过滤 rules
    backup.rules.retain(|r| {
        let id = format!("{}|{}", r.name, r.location);
        options.rule_ids.contains(&id)
    });
    
    // 按 "name|location" 过滤 skills
    backup.skills.retain(|s| {
        let id = format!("{}|{}", s.name, s.location);
        options.skill_ids.contains(&id)
    });
    
    // Codex CLI 过滤：按名称分别过滤 providers 和 MCP
    if let Some(ref mut codex) = backup.codex_config {
        codex.model_providers.retain(|p| options.codex_provider_names.contains(&p.name));
        codex.mcp_servers.retain(|m| options.codex_mcp_names.contains(&m.name));
        if codex.model_providers.is_empty() && codex.mcp_servers.is_empty() {
            backup.codex_config = None;
        }
    }
    // Gemini CLI 过滤：env 和 MCP 分别控制
    if let Some(ref mut gemini) = backup.gemini_config {
        if !options.include_gemini_env {
            gemini.env = ExportedGeminiEnv {
                gemini_api_key: None,
                google_gemini_api_key: None,
                google_gemini_base_url: None,
                gemini_model: None,
            };
        }
        gemini.mcp_servers.retain(|m| options.gemini_mcp_names.contains(&m.name));
        let has_env = options.include_gemini_env;
        if !has_env && gemini.mcp_servers.is_empty() {
            backup.gemini_config = None;
        }
    }
    
    // 开发环境（直接使用前端传入的版本号）
    if !options.dev_envs.is_empty() {
        backup.dev_envs = Some(options.dev_envs.clone());
    }

    // 按来源导出使用统计
    if !options.usage_sources.is_empty() {
        let conn = db.conn.lock().map_err(|e| AppError::Custom(format!("DB lock failed: {}", e)))?;
        let mut usage_records = Vec::new();
        let placeholders: Vec<&str> = options.usage_sources.iter().map(|_| "?").collect();
        let sql = format!(
            "SELECT session_id, created_at, model, app_type, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost FROM proxy_request_logs WHERE app_type IN ({}) ORDER BY created_at",
            placeholders.join(", ")
        );
        if let Ok(mut stmt) = conn.prepare(&sql) {
            if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(options.usage_sources.iter()), |row| {
                Ok(ExportedUsageRecord {
                    session_id: row.get::<_, String>(0).unwrap_or_default(),
                    timestamp: row.get::<_, i64>(1).unwrap_or(0),
                    model: row.get::<_, String>(2).unwrap_or_default(),
                    source: row.get::<_, String>(3).unwrap_or_default(),
                    input_tokens: row.get::<_, u32>(4).unwrap_or(0),
                    output_tokens: row.get::<_, u32>(5).unwrap_or(0),
                    cache_read_tokens: row.get::<_, u32>(6).unwrap_or(0),
                    cache_creation_tokens: row.get::<_, u32>(7).unwrap_or(0),
                    cost: row.get::<_, f64>(8).ok(),
                })
            }) {
                for r in rows.flatten() { usage_records.push(r); }
            }
        }
        backup.usage_stats = Some(usage_records);
    }
    
    // 添加对话记录
    backup.chat_conversations = chat_conversations;
    
    let stats = ExportStats {
        providers: backup.providers.len(),
        models: backup.providers.iter().map(|p| p.models.len()).sum(),
        mcp_servers: backup.mcp_servers.len(),
        rules: backup.rules.len(),
        skills: backup.skills.len(),
        codex_providers: backup.codex_config.as_ref().map(|c| c.model_providers.len()).unwrap_or(0),
        codex_mcp_servers: backup.codex_config.as_ref().map(|c| c.mcp_servers.len()).unwrap_or(0),
        gemini_configured: backup.gemini_config.is_some(),
        gemini_mcp_servers: backup.gemini_config.as_ref().map(|c| c.mcp_servers.len()).unwrap_or(0),
        usage_records: backup.usage_stats.as_ref().map(|r| r.len()).unwrap_or(0),
        chat_conversations: backup.chat_conversations.as_ref().map(|c| c.len()).unwrap_or(0),
    };
    
    let content = serde_json::to_string_pretty(&backup)
        .map_err(|e| AppError::Custom(format!("Failed to serialize: {}", e)))?;
    
    fs::write(&file_path, content)
        .map_err(|e| AppError::Custom(format!("Failed to write file: {}", e)))?;
    
    Ok(stats)
}

#[tauri::command]
pub fn preview_backup(file_path: String) -> Result<BackupData, AppError> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| AppError::Custom(format!("Failed to read file: {}", e)))?;
    
    let backup: BackupData = serde_json::from_str(&content)
        .map_err(|e| AppError::Custom(format!("Failed to parse file: {}", e)))?;
    
    Ok(backup)
}

#[tauri::command]
pub fn import_backup(
    file_path: String,
    options: ImportOptions,
    config_manager: State<'_, Mutex<ConfigManager>>,
    db: State<'_, std::sync::Arc<crate::database::Database>>,
) -> Result<ImportResult, AppError> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| AppError::Custom(format!("Failed to read file: {}", e)))?;
    
    let backup: BackupData = serde_json::from_str(&content)
        .map_err(|e| AppError::Custom(format!("Failed to parse file: {}", e)))?;
    
    let mut result = ImportResult {
        success: true,
        providers_imported: 0,
        providers_skipped: 0,
        mcp_imported: 0,
        mcp_skipped: 0,
        rules_imported: 0,
        rules_skipped: 0,
        skills_imported: 0,
        skills_skipped: 0,
        codex_imported: 0,
        codex_skipped: 0,
        gemini_imported: 0,
        gemini_skipped: 0,
        usage_imported: 0,
        usage_skipped: 0,
        errors: Vec::new(),
    };
    
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    if options.import_providers {
        let existing = match manager.opencode().get_all_providers() {
            Ok(map) => Some(map),
            Err(e) => {
                result.errors.push(format!("读取现有 Provider 失败: {}", e));
                None
            }
        };
        
        if let Some(existing) = existing {
            for provider in &backup.providers {
                let exists = existing.contains_key(&provider.name);
            
                if exists && !options.overwrite_existing {
                    result.providers_skipped += 1;
                    continue;
                }
            
                if exists && options.overwrite_existing {
                    if let Err(e) = manager.opencode_mut().delete_provider(&provider.name) {
                        result.errors.push(format!("删除 Provider '{}' 失败: {}", provider.name, e));
                        continue;
                    }
                }
            
                // 根据 model_type 生成 variants
                let model_type = provider.model_type.clone().unwrap_or_else(|| "claude".to_string());
                let variants = build_variants(&model_type);
            
                match manager.opencode_mut().add_provider(
                    provider.name.clone(),
                    provider.base_url.clone(),
                    provider.api_key.clone(),
                    provider.npm.clone(),
                    provider.description.clone(),
                    provider.model_type.clone(),
                    true,
                ) {
                    Ok(_) => {
                        for model in &provider.models {
                            let model_info = crate::config::OpenCodeModelInfo {
                                id: model.id.clone(),
                                name: model.name.clone(),
                                limit: None,
                                reasoning: Some(true),  // 启用 opencode 思考强度切换 (ctrl+t)
                                variants: Some(variants.clone()),
                                options: None,
                                reasoning_effort: model.reasoning_effort.clone(),
                                thinking_budget: None,
                                model_detection: None,
                            };
                            if let Err(e) = manager.opencode_mut().add_model(&provider.name, model.id.clone(), model_info) {
                                result.errors.push(format!(
                                    "Provider '{}' 添加模型 '{}' 失败: {}",
                                    provider.name, model.id, e
                                ));
                            }
                        }
                        if let Err(e) = manager.opencode_mut().toggle_provider(&provider.name, provider.enabled) {
                            result.errors.push(format!(
                                "Provider '{}' 更新启用状态失败: {}",
                                provider.name, e
                            ));
                        }
                        result.providers_imported += 1;
                    }
                    Err(e) => {
                        result.errors.push(format!("Provider '{}': {}", provider.name, e));
                    }
                }
            }
        }
    }
    
    if options.import_mcp {
        let existing = match manager.mcp().read_config().map(|c| c.servers) {
            Ok(map) => Some(map),
            Err(e) => {
                result.errors.push(format!("读取现有 MCP 配置失败: {}", e));
                None
            }
        };
        
        if let Some(existing) = existing {
            for mcp in &backup.mcp_servers {
                let exists = existing.contains_key(&mcp.name);
            
                if exists && !options.overwrite_existing {
                    result.mcp_skipped += 1;
                    continue;
                }
            
                if exists && options.overwrite_existing {
                    if let Err(e) = manager.mcp_mut().delete_server(&mcp.name) {
                        result.errors.push(format!("删除 MCP '{}' 失败: {}", mcp.name, e));
                        continue;
                    }
                }
            
                // 转换 OAuth 配置
                let oauth = mcp.oauth.as_ref().map(|o| crate::config::McpOAuthConfig {
                    client_id: o.client_id.clone(),
                    client_secret: o.client_secret.clone(),
                    scope: o.scope.clone(),
                });
            
                let server = if mcp.server_type == "local" {
                    McpServer {
                        server_type: McpServerType::Local,
                        enabled: mcp.enabled,
                        timeout: mcp.timeout,
                        command: mcp.command.clone(),
                        environment: mcp.environment.clone().unwrap_or_default(),
                        url: None,
                        headers: HashMap::new(),
                        oauth: None, // 本地服务器不需要 OAuth
                        metadata: Default::default(),
                    }
                } else {
                    McpServer {
                        server_type: McpServerType::Remote,
                        enabled: mcp.enabled,
                        timeout: mcp.timeout,
                        command: None,
                        environment: HashMap::new(),
                        url: mcp.url.clone(),
                        headers: mcp.headers.clone().unwrap_or_default(),
                        oauth, // 恢复 OAuth 配置
                        metadata: Default::default(),
                    }
                };
            
                match manager.mcp_mut().save_server(&mcp.name, server) {
                    Ok(_) => result.mcp_imported += 1,
                    Err(e) => result.errors.push(format!("MCP '{}': {}", mcp.name, e)),
                }
            }
        }
        if let Err(e) = manager.mcp().sync_to_opencode(None) {
            result.errors.push(format!("同步 MCP 配置失败: {}", e));
        }
    }
    
    if options.import_rules {
        let rule_paths = get_rule_paths();
        
        for rule in &backup.rules {
            if let Some(base_path) = rule_paths.get(&rule.location) {
                // 确保基础目录存在
                if let Err(e) = fs::create_dir_all(base_path) {
                    result.errors.push(format!("创建目录失败: {}", e));
                    continue;
                }
                
                let target_path = if rule.rule_type == "agents_md" {
                    // AGENTS.md (OpenCode 和 Codex)
                    base_path.join("AGENTS.md")
                } else if rule.rule_type == "gemini_md" {
                    // GEMINI.md (Gemini CLI)
                    base_path.join("GEMINI.md")
                } else {
                    // rules 目录下的规则文件
                    let rules_dir = base_path.join("rules");
                    if let Err(e) = fs::create_dir_all(&rules_dir) {
                        result.errors.push(format!("Create dir failed: {}", e));
                        continue;
                    }
                    // 使用保存的扩展名，保持 .md 或 .mdc 一致
                    let ext = if rule.file_ext.is_empty() { "md" } else { &rule.file_ext };
                    rules_dir.join(format!("{}.{}", rule.name, ext))
                };
                
                if target_path.exists() && !options.overwrite_existing {
                    result.rules_skipped += 1;
                    continue;
                }
                
                match fs::write(&target_path, &rule.content) {
                    Ok(_) => result.rules_imported += 1,
                    Err(e) => result.errors.push(format!("Rule '{}': {}", rule.name, e)),
                }
            }
        }
    }
    
    if options.import_skills {
        for skills in &backup.skills {
            // 支持 OpenCode、Claude、Codex、Gemini 的 skills 路径
            let base_path = match skills.location.as_str() {
                "global_opencode" => dirs::home_dir().map(|d| d.join(".config").join("opencode").join("skills")),
                "global_claude" => dirs::home_dir().map(|d| d.join(".claude").join("skills")),
                "global_codex" => dirs::home_dir().map(|d| d.join(".codex").join("skills")),
                "global_gemini" => dirs::home_dir().map(|d| d.join(".gemini").join("skills")),
                _ => None,
            };
            
            if let Some(base) = base_path {
                let skills_dir = base.join(&skills.name);
                if let Err(e) = fs::create_dir_all(&skills_dir) {
                    result.errors.push(format!("Create dir failed: {}", e));
                    continue;
                }
                
                let skills_file = skills_dir.join("SKILL.md");
                if skills_file.exists() && !options.overwrite_existing {
                    result.skills_skipped += 1;
                    continue;
                }
                
                match fs::write(&skills_file, &skills.content) {
                    Ok(_) => result.skills_imported += 1,
                    Err(e) => result.errors.push(format!("skills '{}': {}", skills.name, e)),
                }
            }
        }
    }
    
    // 导入 Codex CLI 配置
    if options.import_codex {
        if let Some(ref codex_config) = backup.codex_config {
            import_codex_config(codex_config, &options, &mut result);
        }
    }
    
    // 导入 Gemini CLI 配置
    if options.import_gemini {
        if let Some(ref gemini_config) = backup.gemini_config {
            import_gemini_config(gemini_config, &options, &mut result);
        }
    }
    
    // 导入使用统计数据
    if options.import_usage_stats {
        if let Some(records) = &backup.usage_stats {
            if let Ok(conn) = db.conn.lock() {
                // 收集已有的 session_id+timestamp 用于去重
                let mut existing_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
                if let Ok(mut stmt) = conn.prepare("SELECT session_id, created_at FROM proxy_request_logs") {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        let sid: String = row.get(0)?;
                        let ts: i64 = row.get(1)?;
                        Ok(format!("{}|{}", sid, ts))
                    }) {
                        for key in rows.flatten() { existing_keys.insert(key); }
                    }
                }

                let _ = conn.execute_batch("BEGIN IMMEDIATE");
                for record in records {
                    let dedup_key = format!("{}|{}", record.session_id, record.timestamp);
                    if existing_keys.contains(&dedup_key) {
                        result.usage_skipped += 1;
                        continue;
                    }
                    let insert_result = conn.execute(
                        "INSERT INTO proxy_request_logs (session_id, created_at, model, app_type, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost, success) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1)",
                        rusqlite::params![
                            record.session_id, record.timestamp, record.model, record.source,
                            record.input_tokens, record.output_tokens, record.cache_read_tokens,
                            record.cache_creation_tokens, record.cost.unwrap_or(0.0)
                        ],
                    );
                    match insert_result {
                        Ok(_) => { result.usage_imported += 1; }
                        Err(_) => { result.usage_skipped += 1; }
                    }
                }
                let _ = conn.execute_batch("COMMIT");
            }
        }
    }

    result.success = result.errors.is_empty();
    Ok(result)
}

/// 导入 Codex CLI 配置
fn import_codex_config(
    config: &ExportedCodexConfig,
    options: &ImportOptions,
    result: &mut ImportResult,
) {
    let codex_manager = match CodexConfigManager::new() {
        Ok(m) => m,
        Err(e) => {
            result.errors.push(format!("初始化 Codex 配置管理器失败: {}", e));
            return;
        }
    };
    
    // 导入 model_providers
    for provider in &config.model_providers {
        let existing = codex_manager.get_model_providers().ok();
        let exists = existing
            .as_ref()
            .map(|p| p.contains_key(&provider.name))
            .unwrap_or(false);
        
        if exists && !options.overwrite_existing {
            result.codex_skipped += 1;
            continue;
        }
        
        let codex_provider = crate::config::codex_manager::CodexModelProvider {
            name: provider.name.clone(),
            base_url: provider.base_url.clone(),
            env_key: provider.env_key.clone(),
            requires_openai_auth: provider.requires_openai_auth,
        };
        
        match codex_manager.add_model_provider(&provider.name, codex_provider) {
            Ok(_) => result.codex_imported += 1,
            Err(e) => result.errors.push(format!("Codex Provider '{}': {}", provider.name, e)),
        }
    }
    
    // 导入 MCP servers
    for server in &config.mcp_servers {
        let existing = codex_manager.get_mcp_servers().ok();
        let exists = existing
            .as_ref()
            .map(|s| s.contains_key(&server.name))
            .unwrap_or(false);
        
        if exists && !options.overwrite_existing {
            result.codex_skipped += 1;
            continue;
        }
        
        let codex_server = crate::config::codex_manager::CodexMcpServer {
            command: server.command.clone(),
            env: server.env.clone(),
        };
        
        match codex_manager.add_mcp_server(&server.name, codex_server) {
            Ok(_) => result.codex_imported += 1,
            Err(e) => result.errors.push(format!("Codex MCP '{}': {}", server.name, e)),
        }
    }
}

/// 导入 Gemini CLI 配置
fn import_gemini_config(
    config: &ExportedGeminiConfig,
    options: &ImportOptions,
    result: &mut ImportResult,
) {
    let gemini_manager = match GeminiConfigManager::new() {
        Ok(m) => m,
        Err(e) => {
            result.errors.push(format!("初始化 Gemini 配置管理器失败: {}", e));
            return;
        }
    };
    
    // 导入环境配置
    let has_env = config.env.gemini_api_key.is_some() 
        || config.env.google_gemini_api_key.is_some() 
        || config.env.google_gemini_base_url.is_some()
        || config.env.gemini_model.is_some();
    
    if has_env {
        // 检查是否已存在配置
        let existing_env = gemini_manager.read_env().ok();
        let env_exists = existing_env
            .as_ref()
            .map(|e| e.gemini_api_key.is_some() || e.google_gemini_api_key.is_some())
            .unwrap_or(false);
        
        if env_exists && !options.overwrite_existing {
            result.gemini_skipped += 1;
        } else {
            let env = crate::config::gemini_manager::GeminiEnv {
                gemini_api_key: config.env.gemini_api_key.clone(),
                google_gemini_api_key: config.env.google_gemini_api_key.clone(),
                google_gemini_base_url: config.env.google_gemini_base_url.clone(),
                gemini_model: config.env.gemini_model.clone(),
                other: std::collections::HashMap::new(),
            };
            
            match gemini_manager.write_env(&env) {
                Ok(_) => result.gemini_imported += 1,
                Err(e) => result.errors.push(format!("Gemini ENV: {}", e)),
            }
        }
    }
    
    // 导入 MCP servers
    for server in &config.mcp_servers {
        let existing = gemini_manager.get_mcp_servers().ok();
        let exists = existing
            .as_ref()
            .map(|s| s.contains_key(&server.name))
            .unwrap_or(false);
        
        if exists && !options.overwrite_existing {
            result.gemini_skipped += 1;
            continue;
        }
        
        let gemini_server = crate::config::gemini_manager::GeminiMcpServer {
            command: server.command.clone(),
            args: server.args.clone(),
            env: server.env.clone(),
            url: server.url.clone(),
        };
        
        match gemini_manager.add_mcp_server(&server.name, gemini_server) {
            Ok(_) => result.gemini_imported += 1,
            Err(e) => result.errors.push(format!("Gemini MCP '{}': {}", server.name, e)),
        }
    }
}
