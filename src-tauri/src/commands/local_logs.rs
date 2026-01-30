//! 本地日志解析和导入模块
//!
//! 支持从 Claude Code、Codex CLI、Gemini CLI 和 Opencode 的本地日志文件中解析使用统计数据

use crate::database::Database;
use crate::error::AppError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tauri::State;

// ============================================================================
// 数据结构
// ============================================================================

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    /// Claude Code 日志文件数
    pub claude_files: u32,
    /// Claude Code 日志条目数（预估）
    pub claude_entries: u32,
    /// Claude Code 日志目录
    pub claude_path: Option<String>,
    /// Codex CLI 日志文件数
    pub codex_files: u32,
    /// Codex CLI 日志条目数（预估）
    pub codex_entries: u32,
    /// Codex CLI 日志目录
    pub codex_path: Option<String>,
    /// Gemini CLI 日志文件数
    pub gemini_files: u32,
    /// Gemini CLI 日志条目数（预估）
    pub gemini_entries: u32,
    /// Gemini CLI 日志目录
    pub gemini_path: Option<String>,
    /// Opencode 日志文件数
    pub opencode_files: u32,
    /// Opencode 日志条目数（预估）
    pub opencode_entries: u32,
    /// Opencode 日志目录
    pub opencode_path: Option<String>,
    /// 数据库中已有的本地导入记录数
    pub existing_records: u32,
}

/// 本地日志导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalLogImportResult {
    /// 新增记录数
    pub imported: u32,
    /// 跳过的重复记录数
    pub skipped: u32,
    /// 解析失败的条目数
    pub failed: u32,
    /// 总处理条目数
    pub total: u32,
}

/// 本地日志条目
#[derive(Debug, Clone)]
pub struct LocalLogEntry {
    /// 来源: "claude" | "codex"
    pub source: String,
    /// Unix 时间戳（秒）
    pub timestamp: i64,
    /// 模型名称
    pub model: String,
    /// 输入 token 数
    pub input_tokens: u32,
    /// 输出 token 数
    pub output_tokens: u32,
    /// 缓存读取 token 数
    pub cache_read_tokens: u32,
    /// 缓存创建 token 数
    pub cache_creation_tokens: u32,
    /// 成本（如果日志中有）
    pub cost_usd: Option<f64>,
    /// 会话 ID（用于去重）
    pub session_id: String,
    /// 项目名称
    pub project_name: Option<String>,
}

// ============================================================================
// Claude Code 日志解析
// ============================================================================

/// 获取 Claude Code 日志目录
fn get_claude_log_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let claude_dir = home.join(".claude").join("projects");
    if claude_dir.exists() {
        Some(claude_dir)
    } else {
        None
    }
}

/// 扫描 Claude Code 日志文件
fn scan_claude_logs() -> (Vec<PathBuf>, u32) {
    let Some(log_dir) = get_claude_log_dir() else {
        return (vec![], 0);
    };

    let mut files = Vec::new();
    let mut entry_count = 0u32;

    // 遍历 projects 目录下的所有子目录
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 查找 .jsonl 文件
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let file_path = sub_entry.path();
                        if file_path.extension().map_or(false, |ext| ext == "jsonl") {
                            // 粗略估算条目数（按行数）
                            if let Ok(content) = fs::read_to_string(&file_path) {
                                entry_count += content.lines().count() as u32;
                            }
                            files.push(file_path);
                        }
                    }
                }
            }
        }
    }

    (files, entry_count)
}

/// 解析 Claude Code 日志文件
fn parse_claude_log_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    // 从文件路径提取项目名称
    let project_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    // 从文件名提取会话 ID
    let session_id = path
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    for line in content.lines() {
        if let Some(entry) = parse_claude_log_line(line, &session_id, &project_name) {
            entries.push(entry);
        }
    }

    entries
}

/// 解析 Claude Code 日志行
fn parse_claude_log_line(
    line: &str,
    session_id: &str,
    project_name: &Option<String>,
) -> Option<LocalLogEntry> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    // Claude Code 日志格式：
    // - type: "assistant" 的消息包含 usage 信息
    // - message.usage 包含 token 统计
    // - costUSD 可能在顶层或 message 中
    
    let msg_type = json.get("type").and_then(|v| v.as_str())?;
    
    // 只处理 assistant 类型的消息（包含使用量）
    if msg_type != "assistant" {
        return None;
    }

    // 尝试从多个位置获取 usage
    let usage = json
        .get("message")
        .and_then(|m| m.get("usage"))
        .or_else(|| json.get("usage"))?;

    let input_tokens = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let output_tokens = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let cache_read_tokens = usage
        .get("cache_read_input_tokens")
        .or_else(|| usage.get("cacheReadInputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let cache_creation_tokens = usage
        .get("cache_creation_input_tokens")
        .or_else(|| usage.get("cacheCreationInputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    // 如果没有任何 token，跳过
    if input_tokens == 0 && output_tokens == 0 {
        return None;
    }

    // 获取模型名称
    let model = json
        .get("message")
        .and_then(|m| m.get("model"))
        .or_else(|| json.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // 获取时间戳
    let timestamp = json
        .get("timestamp")
        .and_then(|v| {
            // 可能是 ISO 格式字符串或 Unix 时间戳
            if let Some(ts) = v.as_i64() {
                // 如果是毫秒，转换为秒
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                // 尝试解析 ISO 格式
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    // 获取成本
    let cost_usd = json
        .get("costUSD")
        .or_else(|| json.get("cost_usd"))
        .and_then(|v| v.as_f64());

    // 生成唯一的条目 ID
    let entry_session_id = format!("{}-{}", session_id, timestamp);

    Some(LocalLogEntry {
        source: "claude".to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        cost_usd,
        session_id: entry_session_id,
        project_name: project_name.clone(),
    })
}

// ============================================================================
// Codex CLI 日志解析
// ============================================================================

/// 获取 Codex CLI 日志目录
fn get_codex_log_dir() -> Option<PathBuf> {
    // 优先使用环境变量
    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        let path = PathBuf::from(codex_home);
        if path.exists() {
            return Some(path);
        }
    }

    // 默认位置
    let home = dirs::home_dir()?;
    let codex_dir = home.join(".codex");
    if codex_dir.exists() {
        Some(codex_dir)
    } else {
        None
    }
}

/// 扫描 Codex CLI 日志文件
fn scan_codex_logs() -> (Vec<PathBuf>, u32) {
    let Some(log_dir) = get_codex_log_dir() else {
        return (vec![], 0);
    };

    let mut files = Vec::new();
    let mut entry_count = 0u32;

    // 递归扫描函数
    fn scan_dir_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>, entry_count: &mut u32) {
        if !dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 递归扫描子目录 (sessions/YYYY/MM/DD/)
                    scan_dir_recursive(&path, files, entry_count);
                } else if path.extension().map_or(false, |ext| ext == "jsonl") {
                    // 找到 .jsonl 文件
                    if let Ok(content) = fs::read_to_string(&path) {
                        *entry_count += content.lines().count() as u32;
                    }
                    files.push(path);
                }
            }
        }
    }

    // 扫描根目录和 sessions 子目录
    scan_dir_recursive(&log_dir, &mut files, &mut entry_count);
    
    // 也扫描 sessions 目录（如果和根目录不同）
    let sessions_dir = log_dir.join("sessions");
    if sessions_dir.exists() && sessions_dir != log_dir {
        scan_dir_recursive(&sessions_dir, &mut files, &mut entry_count);
    }

    (files, entry_count)
}

/// 解析 Codex CLI 日志文件
fn parse_codex_log_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    // 从文件名提取会话 ID
    let session_id = path
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Codex 使用累计 token，需要追踪上一次的值来计算 delta
    let mut last_input_tokens = 0u32;
    let mut last_output_tokens = 0u32;
    let mut last_cached_tokens = 0u32;
    let mut current_model = "gpt-5".to_string();

    for line in content.lines() {
        if let Some(result) =
            parse_codex_log_line(line, &session_id, last_input_tokens, last_output_tokens, last_cached_tokens)
        {
            last_input_tokens = result.new_input;
            last_output_tokens = result.new_output;
            last_cached_tokens = result.new_cached;
            if let Some(m) = result.model {
                current_model = m;
            }

            if let Some(mut entry) = result.entry {
                if entry.model == "unknown" {
                    entry.model = current_model.clone();
                }
                entries.push(entry);
            }
        }
    }

    entries
}

/// Codex 解析结果
struct CodexParseResult {
    entry: Option<LocalLogEntry>,
    new_input: u32,
    new_output: u32,
    new_cached: u32,
    model: Option<String>,
}

/// 解析 Codex CLI 日志行
/// 返回 (条目, 累计输入, 累计输出, 累计缓存, 模型)
fn parse_codex_log_line(
    line: &str,
    session_id: &str,
    last_input: u32,
    last_output: u32,
    last_cached: u32,
) -> Option<CodexParseResult> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    // 处理模型上下文
    if json.get("type").and_then(|v| v.as_str()) == Some("turn_context") {
        let model = json
            .get("payload")
            .and_then(|p| p.get("model"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        return Some(CodexParseResult {
            entry: None,
            new_input: last_input,
            new_output: last_output,
            new_cached: last_cached,
            model,
        });
    }

    // Codex 日志格式：
    // 1) type=event_msg, payload.type=token_count
    // 2) event_msg.payload.type=token_count
    let payload = if json.get("type").and_then(|v| v.as_str()) == Some("event_msg") {
        json.get("payload")?
    } else if let Some(event_msg) = json.get("event_msg") {
        event_msg.get("payload")?
    } else {
        return None;
    };

    let payload_type = payload.get("type").and_then(|v| v.as_str())?;
    if payload_type != "token_count" {
        return None;
    }

    let info = payload.get("info");
    let total_usage = info
        .and_then(|i| i.get("total_token_usage"))
        .or_else(|| payload.get("total_token_usage"))
        .or_else(|| payload.get("token_usage"));

    let last_usage = info.and_then(|i| i.get("last_token_usage"));

    let mut total_input = total_usage
        .and_then(|u| u.get("input_tokens"))
        .or_else(|| payload.get("input_tokens"))
        .or_else(|| payload.get("inputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let mut total_output = total_usage
        .and_then(|u| u.get("output_tokens"))
        .or_else(|| payload.get("output_tokens"))
        .or_else(|| payload.get("outputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let mut total_cached = total_usage
        .and_then(|u| u.get("cached_input_tokens"))
        .or_else(|| payload.get("cached_input_tokens"))
        .or_else(|| payload.get("cachedInputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let (input_delta, output_delta, cached_delta) = if total_input > 0 || total_output > 0 || total_cached > 0 {
        (
            total_input.saturating_sub(last_input),
            total_output.saturating_sub(last_output),
            total_cached.saturating_sub(last_cached),
        )
    } else if let Some(last) = last_usage {
        let input = last
            .get("input_tokens")
            .or_else(|| last.get("inputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let output = last
            .get("output_tokens")
            .or_else(|| last.get("outputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let cached = last
            .get("cached_input_tokens")
            .or_else(|| last.get("cachedInputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        total_input = last_input.saturating_add(input);
        total_output = last_output.saturating_add(output);
        total_cached = last_cached.saturating_add(cached);

        (input, output, cached)
    } else {
        return None;
    };

    // 如果没有变化，跳过
    if input_delta == 0 && output_delta == 0 && cached_delta == 0 {
        return None;
    }

    // 获取时间戳
    let timestamp_value = json
        .get("timestamp")
        .or_else(|| payload.get("timestamp"))
        .or_else(|| json.get("event_msg").and_then(|m| m.get("timestamp")));

    let timestamp = timestamp_value
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let entry_session_id = format!("{}-{}", session_id, timestamp);

    let entry = LocalLogEntry {
        source: "codex".to_string(),
        timestamp,
        model: "unknown".to_string(), // 后续更新
        input_tokens: input_delta,
        output_tokens: output_delta,
        cache_read_tokens: cached_delta,
        cache_creation_tokens: 0,
        cost_usd: None,
        session_id: entry_session_id,
        project_name: None,
    };

    Some(CodexParseResult {
        entry: Some(entry),
        new_input: total_input,
        new_output: total_output,
        new_cached: total_cached,
        model: None,
    })
}

// ============================================================================
// Gemini CLI 日志解析
// ============================================================================

/// 获取 Gemini CLI 日志目录
fn get_gemini_log_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let gemini_dir = home.join(".gemini").join("tmp");
    if gemini_dir.exists() {
        Some(gemini_dir)
    } else {
        None
    }
}

/// 扫描 Gemini CLI 日志文件
fn scan_gemini_logs() -> (Vec<PathBuf>, u32) {
    let Some(log_dir) = get_gemini_log_dir() else {
        return (vec![], 0);
    };

    let mut files = Vec::new();
    let mut entry_count = 0u32;

    // 递归扫描 tmp/<project_hash>/chats/ 目录
    fn scan_gemini_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>, entry_count: &mut u32) {
        if !dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 递归扫描子目录
                    scan_gemini_recursive(&path, files, entry_count);
                } else {
                    // Gemini 日志可能是 .json 或 .jsonl
                    let ext = path.extension().and_then(|e| e.to_str());
                    if ext == Some("json") || ext == Some("jsonl") {
                        // 检查是否在 chats 目录下或者是 session 文件
                        let is_chat_file = path.parent()
                            .and_then(|p| p.file_name())
                            .map_or(false, |n| n == "chats");
                        let is_session_file = path.file_name()
                            .and_then(|n| n.to_str())
                            .map_or(false, |n| n.starts_with("session-"));
                        
                        if is_chat_file || is_session_file {
                            if let Ok(content) = fs::read_to_string(&path) {
                                // 对于 JSONL，按行计数；对于 JSON，算作 1 条
                                if ext == Some("jsonl") {
                                    *entry_count += content.lines().count() as u32;
                                } else {
                                    *entry_count += 1;
                                }
                            }
                            files.push(path);
                        }
                    }
                }
            }
        }
    }

    scan_gemini_recursive(&log_dir, &mut files, &mut entry_count);

    (files, entry_count)
}

/// 解析 Gemini CLI 日志文件
fn parse_gemini_log_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    // 从文件名提取会话 ID
    let session_id = path
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let ext = path.extension().and_then(|e| e.to_str());

    if ext == Some("jsonl") {
        // JSONL 格式：每行一个 JSON 对象
        for line in content.lines() {
            if let Some(entry) = parse_gemini_log_line(line, &session_id) {
                entries.push(entry);
            }
        }
    } else {
        // JSON 格式：整个文件是一个 JSON 对象
        entries.extend(parse_gemini_json_file(&content, &session_id));
    }

    entries
}

/// 解析 Gemini CLI 日志行 (JSONL 格式)
fn parse_gemini_log_line(line: &str, session_id: &str) -> Option<LocalLogEntry> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;
    parse_gemini_usage_from_json(&json, session_id)
}

/// 解析 Gemini CLI JSON 文件
fn parse_gemini_json_file(content: &str, session_id: &str) -> Vec<LocalLogEntry> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(content) else {
        return vec![];
    };

    parse_gemini_entries_from_value(&json, session_id)
}

/// 从 Gemini JSON 中提取使用量（支持 messages 数组）
fn parse_gemini_entries_from_value(json: &serde_json::Value, session_id: &str) -> Vec<LocalLogEntry> {
    if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
        let base_session_id = json
            .get("sessionId")
            .and_then(|v| v.as_str())
            .unwrap_or(session_id);
        return messages
            .iter()
            .filter_map(|m| parse_gemini_message(m, base_session_id))
            .collect();
    }

    if let Some(array) = json.as_array() {
        return array
            .iter()
            .filter_map(|m| parse_gemini_message(m, session_id))
            .collect();
    }

    if let Some(entry) = parse_gemini_usage_from_json(json, session_id) {
        return vec![entry];
    }

    vec![]
}

/// 解析 Gemini messages 中的单条消息
fn parse_gemini_message(message: &serde_json::Value, session_id: &str) -> Option<LocalLogEntry> {
    let tokens = message.get("tokens")?;

    let input_tokens = tokens
        .get("input")
        .or_else(|| tokens.get("prompt"))
        .or_else(|| tokens.get("prompt_tokens"))
        .or_else(|| tokens.get("inputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let output_tokens = tokens
        .get("output")
        .or_else(|| tokens.get("completion"))
        .or_else(|| tokens.get("completion_tokens"))
        .or_else(|| tokens.get("outputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let thoughts_tokens = tokens
        .get("thoughts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let tool_tokens = tokens
        .get("tool")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let cache_read_tokens = tokens
        .get("cached")
        .or_else(|| tokens.get("cache").and_then(|c| c.get("read")))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let cache_creation_tokens = tokens
        .get("cache")
        .and_then(|c| c.get("write"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let output_total = output_tokens
        .saturating_add(thoughts_tokens)
        .saturating_add(tool_tokens);

    if input_tokens == 0 && output_total == 0 && cache_read_tokens == 0 && cache_creation_tokens == 0 {
        return None;
    }

    let model = message
        .get("model")
        .or_else(|| message.get("modelId"))
        .or_else(|| message.get("modelID"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let timestamp_value = message.get("timestamp").or_else(|| message.get("time"));
    let timestamp = timestamp_value
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let msg_id = message
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg");
    let entry_session_id = format!("{}-{}-{}", session_id, timestamp, msg_id);

    Some(LocalLogEntry {
        source: "gemini".to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens: output_total,
        cache_read_tokens,
        cache_creation_tokens,
        cost_usd: None,
        session_id: entry_session_id,
        project_name: None,
    })
}

/// 从 Gemini JSON 中提取使用量
fn parse_gemini_usage_from_json(json: &serde_json::Value, session_id: &str) -> Option<LocalLogEntry> {
    // Gemini CLI 日志格式：
    // - stats 对象包含 token 使用统计
    // - usageMetadata 也可能包含使用信息
    
    // 尝试从 stats 获取
    let stats = json.get("stats")
        .or_else(|| json.get("usageMetadata"));
    
    let (input_tokens, output_tokens, cached_tokens) = if let Some(stats) = stats {
        let input = stats.get("promptTokenCount")
            .or_else(|| stats.get("prompt_tokens"))
            .or_else(|| stats.get("inputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let output = stats.get("candidatesTokenCount")
            .or_else(|| stats.get("completion_tokens"))
            .or_else(|| stats.get("outputTokens"))
            .or_else(|| stats.get("responseTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let cached = stats.get("cachedContentTokenCount")
            .or_else(|| stats.get("cached_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        (input, output, cached)
    } else {
        // 尝试从顶层获取
        let input = json.get("inputTokens")
            .or_else(|| json.get("prompt_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let output = json.get("outputTokens")
            .or_else(|| json.get("completion_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let cached = json.get("cachedTokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        (input, output, cached)
    };

    // 如果没有任何 token，跳过
    if input_tokens == 0 && output_tokens == 0 {
        return None;
    }

    // 获取模型名称
    let model = json.get("model")
        .or_else(|| json.get("modelVersion"))
        .and_then(|v| v.as_str())
        .unwrap_or("gemini-2.5-flash")
        .to_string();

    // 获取时间戳
    let timestamp = json.get("timestamp")
        .or_else(|| json.get("createTime"))
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let entry_session_id = format!("{}-{}", session_id, timestamp);

    Some(LocalLogEntry {
        source: "gemini".to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens: cached_tokens,
        cache_creation_tokens: 0,
        cost_usd: None,
        session_id: entry_session_id,
        project_name: None,
    })
}

// ============================================================================
// Opencode 日志解析
// ============================================================================

/// 获取 Opencode 日志目录
fn get_opencode_log_dir() -> Option<PathBuf> {
    // Opencode 存储位置：~/.local/share/opencode/storage/
    let home = dirs::home_dir()?;
    
    #[cfg(windows)]
    let opencode_dir = home.join(".local").join("share").join("opencode").join("storage");
    #[cfg(not(windows))]
    let opencode_dir = home.join(".local").join("share").join("opencode").join("storage");
    
    if opencode_dir.exists() {
        Some(opencode_dir)
    } else {
        None
    }
}

/// 扫描 Opencode 日志文件
fn scan_opencode_logs() -> (Vec<PathBuf>, u32) {
    let Some(storage_dir) = get_opencode_log_dir() else {
        return (vec![], 0);
    };

    let mut files = Vec::new();
    let mut entry_count = 0u32;

    // 扫描 message/{sessionID}/msg_{messageID}.json 文件
    let message_dir = storage_dir.join("message");
    if message_dir.exists() {
        if let Ok(sessions) = fs::read_dir(&message_dir) {
            for session in sessions.flatten() {
                let session_path = session.path();
                if session_path.is_dir() {
                    if let Ok(messages) = fs::read_dir(&session_path) {
                        for msg in messages.flatten() {
                            let msg_path = msg.path();
                            if msg_path.extension().and_then(|e| e.to_str()) == Some("json") {
                                if let Some(name) = msg_path.file_name().and_then(|n| n.to_str()) {
                                    if name.starts_with("msg_") {
                                        files.push(msg_path);
                                        entry_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (files, entry_count)
}

/// 解析 Opencode 日志文件
fn parse_opencode_log_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return entries;
    };

    // 从路径提取会话 ID
    let session_id = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // 尝试解析 Opencode 消息格式
    if let Some(entry) = parse_opencode_message(&json, &session_id) {
        entries.push(entry);
    }

    entries
}

/// 解析 Opencode 消息
fn parse_opencode_message(json: &serde_json::Value, session_id: &str) -> Option<LocalLogEntry> {
    // Opencode 消息格式：优先读取 tokens 字段
    let (input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens) = if let Some(tokens) = json.get("tokens") {
        let input = tokens
            .get("input")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let output = tokens
            .get("output")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let reasoning = tokens
            .get("reasoning")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let cache_read = tokens
            .get("cache")
            .and_then(|c| c.get("read"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let cache_write = tokens
            .get("cache")
            .and_then(|c| c.get("write"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        (input, output.saturating_add(reasoning), cache_read, cache_write)
    } else if let Some(usage) = json.get("usage") {
        let input = usage
            .get("input_tokens")
            .or_else(|| usage.get("inputTokens"))
            .or_else(|| usage.get("prompt_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let output = usage
            .get("output_tokens")
            .or_else(|| usage.get("outputTokens"))
            .or_else(|| usage.get("completion_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let cache_read = usage
            .get("cache_read_input_tokens")
            .or_else(|| usage.get("cacheReadInputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let cache_creation = usage
            .get("cache_creation_input_tokens")
            .or_else(|| usage.get("cacheCreationInputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        (input, output, cache_read, cache_creation)
    } else {
        return None;
    };

    // 如果没有 token 数据，跳过
    if input_tokens == 0 && output_tokens == 0 && cache_read_tokens == 0 && cache_creation_tokens == 0 {
        return None;
    }

    // 获取模型
    let model = json
        .get("modelID")
        .or_else(|| json.get("modelId"))
        .or_else(|| json.get("model").and_then(|m| m.get("modelID")))
        .or_else(|| json.get("model").and_then(|m| m.get("modelId")))
        .or_else(|| json.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // 获取时间戳
    let timestamp_value = json
        .get("time")
        .and_then(|t| t.get("completed").or_else(|| t.get("created")))
        .or_else(|| json.get("timestamp"))
        .or_else(|| json.get("created_at"));

    let timestamp = timestamp_value
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    // 获取成本
    let cost_usd = json
        .get("cost")
        .or_else(|| json.get("costUSD"))
        .and_then(|v| v.as_f64());

    let msg_id = json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg");
    let entry_session_id = format!("{}-{}-{}", session_id, timestamp, msg_id);

    Some(LocalLogEntry {
        source: "opencode".to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        cost_usd,
        session_id: entry_session_id,
        project_name: None,
    })
}

// ============================================================================
// 数据库操作
// ============================================================================

/// 获取服务商特定的模型定价
fn get_provider_model_pricing(conn: &rusqlite::Connection, provider_id: &str, model_id: &str) -> Option<(Decimal, Decimal, Decimal, Decimal)> {
    let cleaned = clean_model_id(model_id);

    // 先查询服务商特定定价
    let result = conn.query_row(
        "SELECT input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
         FROM provider_model_pricing WHERE provider_id = ?1 AND model_id = ?2",
        [provider_id, &cleaned],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    );

    match result {
        Ok((input, output, cache_read, cache_creation)) => Some((
            Decimal::from_str(&input).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&output).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&cache_read).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&cache_creation).unwrap_or(Decimal::ZERO),
        )),
        // 如果没有服务商特定定价，回退到默认定价
        Err(_) => get_model_pricing_default(conn, &cleaned),
    }
}

/// 获取默认模型定价
fn get_model_pricing_default(conn: &rusqlite::Connection, cleaned_model_id: &str) -> Option<(Decimal, Decimal, Decimal, Decimal)> {
    let result = conn.query_row(
        "SELECT input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
         FROM model_pricing WHERE model_id = ?1",
        [cleaned_model_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    );

    match result {
        Ok((input, output, cache_read, cache_creation)) => Some((
            Decimal::from_str(&input).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&output).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&cache_read).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&cache_creation).unwrap_or(Decimal::ZERO),
        )),
        Err(_) => None,
    }
}

/// 清洗模型 ID
fn clean_model_id(model_id: &str) -> String {
    let without_prefix = model_id.rsplit_once('/').map_or(model_id, |(_, r)| r);
    let without_suffix = without_prefix.split(':').next().unwrap_or(without_prefix);
    without_suffix.trim().replace('@', "-")
}

/// 计算成本
fn calculate_cost(entry: &LocalLogEntry, pricing: Option<(Decimal, Decimal, Decimal, Decimal)>) -> Decimal {
    let Some((input_price, output_price, cache_read_price, cache_creation_price)) = pricing else {
        return Decimal::ZERO;
    };

    let million = Decimal::from(1_000_000u64);
    
    let billable_input = (entry.input_tokens as u64).saturating_sub(entry.cache_read_tokens as u64);
    
    let input_cost = Decimal::from(billable_input) * input_price / million;
    let output_cost = Decimal::from(entry.output_tokens as u64) * output_price / million;
    let cache_read_cost = Decimal::from(entry.cache_read_tokens as u64) * cache_read_price / million;
    let cache_creation_cost = Decimal::from(entry.cache_creation_tokens as u64) * cache_creation_price / million;

    input_cost + output_cost + cache_read_cost + cache_creation_cost
}

/// 检查记录是否已存在
fn record_exists(conn: &rusqlite::Connection, session_id: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM proxy_request_logs WHERE request_id = ?1",
        [session_id],
        |_| Ok(()),
    )
    .is_ok()
}

/// 插入日志条目到数据库
fn insert_log_entry(conn: &rusqlite::Connection, entry: &LocalLogEntry, cost: Decimal) -> Result<(), AppError> {
    let app_type = format!("{}_local", entry.source);
    let provider_id = format!("{}_local", entry.source);
    let provider_name = match entry.source.as_str() {
        "claude" => "Claude Code (Local)",
        "codex" => "Codex CLI (Local)",
        "gemini" => "Gemini CLI (Local)",
        "opencode" => "Opencode (Local)",
        _ => "Local Import",
    };

    let zero = Decimal::ZERO;
    
    conn.execute(
        "INSERT INTO proxy_request_logs (
            request_id, provider_id, provider_name, app_type, model,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
            input_cost_usd, output_cost_usd, cache_read_cost_usd, cache_creation_cost_usd, total_cost_usd,
            latency_ms, status_code, is_streaming, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        rusqlite::params![
            entry.session_id,
            provider_id,
            provider_name,
            app_type,
            entry.model,
            entry.input_tokens,
            entry.output_tokens,
            entry.cache_read_tokens,
            entry.cache_creation_tokens,
            zero.to_string(),
            zero.to_string(),
            zero.to_string(),
            zero.to_string(),
            cost.to_string(),
            0i64, // latency_ms
            200i64, // status_code
            0, // is_streaming
            entry.timestamp,
        ],
    )
    .map_err(|e| AppError::Database(format!("插入日志条目失败: {e}")))?;

    Ok(())
}

/// 获取已导入的本地记录数
fn get_existing_local_records(conn: &rusqlite::Connection) -> u32 {
    conn.query_row(
        "SELECT COUNT(*) FROM proxy_request_logs WHERE app_type LIKE '%_local'",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c as u32)
    .unwrap_or(0)
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// 扫描本地日志文件
#[tauri::command]
pub async fn scan_local_logs(db: State<'_, Arc<Database>>) -> Result<ScanResult, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let (claude_files, claude_entries) = scan_claude_logs();
    let (codex_files, codex_entries) = scan_codex_logs();
    let (gemini_files, gemini_entries) = scan_gemini_logs();
    let (opencode_files, opencode_entries) = scan_opencode_logs();
    
    let existing_records = get_existing_local_records(&conn);
    
    Ok(ScanResult {
        claude_files: claude_files.len() as u32,
        claude_entries,
        claude_path: get_claude_log_dir().map(|p| p.to_string_lossy().to_string()),
        codex_files: codex_files.len() as u32,
        codex_entries,
        codex_path: get_codex_log_dir().map(|p| p.to_string_lossy().to_string()),
        gemini_files: gemini_files.len() as u32,
        gemini_entries,
        gemini_path: get_gemini_log_dir().map(|p| p.to_string_lossy().to_string()),
        opencode_files: opencode_files.len() as u32,
        opencode_entries,
        opencode_path: get_opencode_log_dir().map(|p| p.to_string_lossy().to_string()),
        existing_records,
    })
}

/// 导入本地日志
#[tauri::command]
pub async fn import_local_logs(
    sources: Vec<String>,
    db: State<'_, Arc<Database>>,
) -> Result<LocalLogImportResult, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let mut imported = 0u32;
    let mut skipped = 0u32;
    let mut failed = 0u32;
    let mut total = 0u32;
    
    // 用于去重的集合
    let mut seen_ids: HashSet<String> = HashSet::new();
    
    // 导入 Claude Code 日志
    if sources.contains(&"claude".to_string()) {
        let (files, _) = scan_claude_logs();
        for file in files {
            let entries = parse_claude_log_file(&file);
            for entry in entries {
                total += 1;
                
                // 检查是否已处理过
                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                // 检查数据库中是否已存在
                if record_exists(&conn, &entry.session_id) {
                    skipped += 1;
                    continue;
                }
                
                // 计算成本（优先使用服务商特定定价）
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = entry.cost_usd
                    .map(|c| Decimal::from_str(&c.to_string()).unwrap_or(Decimal::ZERO))
                    .unwrap_or_else(|| calculate_cost(&entry, pricing));
                
                // 插入数据库
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
        }
    }
    
    // 导入 Codex CLI 日志
    if sources.contains(&"codex".to_string()) {
        let (files, _) = scan_codex_logs();
        for file in files {
            let entries = parse_codex_log_file(&file);
            for entry in entries {
                total += 1;
                
                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                if record_exists(&conn, &entry.session_id) {
                    skipped += 1;
                    continue;
                }
                
                // 计算成本（优先使用服务商特定定价）
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = calculate_cost(&entry, pricing);
                
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
        }
    }
    
    // 导入 Gemini CLI 日志
    if sources.contains(&"gemini".to_string()) {
        let (files, _) = scan_gemini_logs();
        for file in files {
            let entries = parse_gemini_log_file(&file);
            for entry in entries {
                total += 1;
                
                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                if record_exists(&conn, &entry.session_id) {
                    skipped += 1;
                    continue;
                }
                
                // 计算成本（优先使用服务商特定定价）
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = calculate_cost(&entry, pricing);
                
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
        }
    }
    
    // 导入 Opencode 日志
    if sources.contains(&"opencode".to_string()) {
        let (files, _) = scan_opencode_logs();
        for file in files {
            let entries = parse_opencode_log_file(&file);
            for entry in entries {
                total += 1;
                
                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                if record_exists(&conn, &entry.session_id) {
                    skipped += 1;
                    continue;
                }
                
                // 计算成本（优先使用服务商特定定价）
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = calculate_cost(&entry, pricing);
                
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
        }
    }
    
    Ok(LocalLogImportResult {
        imported,
        skipped,
        failed,
        total,
    })
}

/// 清除本地导入的日志
#[tauri::command]
pub async fn clear_local_logs(db: State<'_, Arc<Database>>) -> Result<u32, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let deleted = conn
        .execute("DELETE FROM proxy_request_logs WHERE app_type LIKE '%_local'", [])
        .map_err(|e| format!("清除本地日志失败: {e}"))?;
    
    Ok(deleted as u32)
}
