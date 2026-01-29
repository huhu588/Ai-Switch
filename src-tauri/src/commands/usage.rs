// 使用统计相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc, Timelike, Datelike};

/// 单条使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: String,
    pub timestamp: i64,  // Unix timestamp in milliseconds
    pub provider_name: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost: f64,  // in USD
    pub request_type: String,  // "chat", "completion", etc.
}

/// 使用统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_requests: u64,
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
}

/// 按时间段的使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrend {
    pub label: String,  // 时间标签，如 "00:00", "01:00" 或 "Mon", "Tue"
    pub timestamp: i64,
    pub requests: u64,
    pub cost: f64,
    pub tokens: u64,
}

/// 使用统计数据存储
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageData {
    pub records: Vec<UsageRecord>,
}

impl UsageData {
    /// 获取存储路径
    fn get_storage_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("无法获取用户目录")?;
        let config_dir = home.join(".config").join("opencode");
        fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
        Ok(config_dir.join("usage_stats.json"))
    }

    /// 从文件加载
    pub fn load() -> Result<Self, String> {
        let path = Self::get_storage_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    /// 保存到文件
    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_storage_path()?;
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// 添加记录
    pub fn add_record(&mut self, record: UsageRecord) {
        self.records.push(record);
    }

    /// 获取时间范围内的记录
    pub fn get_records_in_range(&self, start: i64, end: i64) -> Vec<&UsageRecord> {
        self.records
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect()
    }

    /// 计算统计摘要
    pub fn calculate_summary(&self, start: i64, end: i64) -> UsageSummary {
        let records = self.get_records_in_range(start, end);
        
        UsageSummary {
            total_requests: records.len() as u64,
            total_cost: records.iter().map(|r| r.cost).sum(),
            total_input_tokens: records.iter().map(|r| r.input_tokens).sum(),
            total_output_tokens: records.iter().map(|r| r.output_tokens).sum(),
            total_cache_creation_tokens: records.iter().map(|r| r.cache_creation_tokens).sum(),
            total_cache_read_tokens: records.iter().map(|r| r.cache_read_tokens).sum(),
        }
    }

    /// 清理旧数据（保留最近 30 天）
    pub fn cleanup_old_records(&mut self) {
        let thirty_days_ago = Utc::now().timestamp_millis() - 30 * 24 * 60 * 60 * 1000;
        self.records.retain(|r| r.timestamp >= thirty_days_ago);
    }
}

/// 获取使用统计摘要
#[tauri::command]
pub async fn get_usage_summary(period: String) -> Result<UsageSummary, String> {
    let data = UsageData::load()?;
    let now = Utc::now().timestamp_millis();
    
    let start = match period.as_str() {
        "24h" => now - 24 * 60 * 60 * 1000,
        "7d" => now - 7 * 24 * 60 * 60 * 1000,
        "30d" => now - 30 * 24 * 60 * 60 * 1000,
        _ => now - 24 * 60 * 60 * 1000,
    };
    
    Ok(data.calculate_summary(start, now))
}

/// 获取使用趋势数据
#[tauri::command]
pub async fn get_usage_trend(period: String) -> Result<Vec<UsageTrend>, String> {
    let data = UsageData::load()?;
    let now = Utc::now();
    let now_millis = now.timestamp_millis();
    
    let (start_millis, interval_millis, format_fn): (i64, i64, Box<dyn Fn(DateTime<Utc>) -> String>) = match period.as_str() {
        "24h" => {
            // 按小时分组，过去24小时
            let start = now_millis - 24 * 60 * 60 * 1000;
            let interval = 60 * 60 * 1000; // 1 hour
            let format_fn: Box<dyn Fn(DateTime<Utc>) -> String> = Box::new(|dt: DateTime<Utc>| format!("{:02}:00", dt.hour()));
            (start, interval, format_fn)
        },
        "7d" => {
            // 按天分组，过去7天
            let start = now_millis - 7 * 24 * 60 * 60 * 1000;
            let interval = 24 * 60 * 60 * 1000; // 1 day
            let format_fn: Box<dyn Fn(DateTime<Utc>) -> String> = Box::new(|dt: DateTime<Utc>| {
                let weekday = dt.weekday();
                match weekday {
                    chrono::Weekday::Mon => "周一".to_string(),
                    chrono::Weekday::Tue => "周二".to_string(),
                    chrono::Weekday::Wed => "周三".to_string(),
                    chrono::Weekday::Thu => "周四".to_string(),
                    chrono::Weekday::Fri => "周五".to_string(),
                    chrono::Weekday::Sat => "周六".to_string(),
                    chrono::Weekday::Sun => "周日".to_string(),
                }
            });
            (start, interval, format_fn)
        },
        "30d" => {
            // 按天分组，过去30天
            let start = now_millis - 30 * 24 * 60 * 60 * 1000;
            let interval = 24 * 60 * 60 * 1000; // 1 day
            let format_fn: Box<dyn Fn(DateTime<Utc>) -> String> = Box::new(|dt: DateTime<Utc>| format!("{}/{}", dt.format("%m"), dt.format("%d")));
            (start, interval, format_fn)
        },
        _ => {
            let start = now_millis - 24 * 60 * 60 * 1000;
            let interval = 60 * 60 * 1000;
            let format_fn: Box<dyn Fn(DateTime<Utc>) -> String> = Box::new(|dt: DateTime<Utc>| format!("{:02}:00", dt.hour()));
            (start, interval, format_fn)
        }
    };

    // 创建时间桶
    let mut buckets: Vec<UsageTrend> = Vec::new();
    let mut current = start_millis;
    
    while current < now_millis {
        let dt = DateTime::from_timestamp_millis(current).unwrap_or(now);
        buckets.push(UsageTrend {
            label: format_fn(dt),
            timestamp: current,
            requests: 0,
            cost: 0.0,
            tokens: 0,
        });
        current += interval_millis;
    }

    // 将记录分配到时间桶
    let records = data.get_records_in_range(start_millis, now_millis);
    for record in records {
        // 找到对应的时间桶
        for bucket in &mut buckets {
            if record.timestamp >= bucket.timestamp && record.timestamp < bucket.timestamp + interval_millis {
                bucket.requests += 1;
                bucket.cost += record.cost;
                bucket.tokens += record.input_tokens + record.output_tokens;
                break;
            }
        }
    }

    Ok(buckets)
}

/// 添加使用记录
#[tauri::command]
pub async fn add_usage_record(
    provider_name: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: Option<u64>,
    cache_read_tokens: Option<u64>,
    cost: Option<f64>,
    request_type: Option<String>,
) -> Result<(), String> {
    let mut data = UsageData::load()?;
    
    let record = UsageRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now().timestamp_millis(),
        provider_name,
        model,
        input_tokens,
        output_tokens,
        cache_creation_tokens: cache_creation_tokens.unwrap_or(0),
        cache_read_tokens: cache_read_tokens.unwrap_or(0),
        cost: cost.unwrap_or(0.0),
        request_type: request_type.unwrap_or_else(|| "chat".to_string()),
    };
    
    data.add_record(record);
    data.cleanup_old_records();  // 清理旧数据
    data.save()?;
    
    Ok(())
}

/// 清除所有使用统计
#[tauri::command]
pub async fn clear_usage_stats() -> Result<(), String> {
    let data = UsageData::default();
    data.save()
}

/// 获取按服务商分组的统计
#[tauri::command]
pub async fn get_usage_by_provider(period: String) -> Result<HashMap<String, UsageSummary>, String> {
    let data = UsageData::load()?;
    let now = Utc::now().timestamp_millis();
    
    let start = match period.as_str() {
        "24h" => now - 24 * 60 * 60 * 1000,
        "7d" => now - 7 * 24 * 60 * 60 * 1000,
        "30d" => now - 30 * 24 * 60 * 60 * 1000,
        _ => now - 24 * 60 * 60 * 1000,
    };
    
    let records = data.get_records_in_range(start, now);
    let mut by_provider: HashMap<String, Vec<&UsageRecord>> = HashMap::new();
    
    for record in records {
        by_provider
            .entry(record.provider_name.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    let mut result: HashMap<String, UsageSummary> = HashMap::new();
    for (provider, records) in by_provider {
        result.insert(provider, UsageSummary {
            total_requests: records.len() as u64,
            total_cost: records.iter().map(|r| r.cost).sum(),
            total_input_tokens: records.iter().map(|r| r.input_tokens).sum(),
            total_output_tokens: records.iter().map(|r| r.output_tokens).sum(),
            total_cache_creation_tokens: records.iter().map(|r| r.cache_creation_tokens).sum(),
            total_cache_read_tokens: records.iter().map(|r| r.cache_read_tokens).sum(),
        });
    }
    
    Ok(result)
}
