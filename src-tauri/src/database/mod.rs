//! 数据库模块
//!
//! 提供 SQLite 数据库连接和操作

pub mod schema;

use crate::error::AppError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 数据库版本号
pub const SCHEMA_VERSION: i32 = 1;

/// 数据库连接封装
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

/// 获取数据库锁的宏
#[macro_export]
macro_rules! lock_conn {
    ($conn:expr) => {
        $conn.lock().map_err(|e| AppError::Database(format!("获取数据库锁失败: {e}")))?
    };
}

pub use lock_conn;

impl Database {
    /// 获取数据库文件路径
    fn get_db_path() -> Result<PathBuf, AppError> {
        let home = dirs::home_dir().ok_or_else(|| AppError::Database("无法获取用户目录".to_string()))?;
        let config_dir = home.join(".config").join("opencode");
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| AppError::Database(format!("创建配置目录失败: {e}")))?;
        Ok(config_dir.join("ai-switch.db"))
    }

    /// 打开或创建数据库
    pub fn open() -> Result<Self, AppError> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::Database(format!("打开数据库失败: {e}")))?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        // 创建表和应用迁移
        db.create_tables()?;
        db.apply_migrations()?;
        db.ensure_model_pricing_seeded()?;
        
        Ok(db)
    }

    /// 创建内存数据库（用于测试）
    #[allow(dead_code)]
    pub fn memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| AppError::Database(format!("创建内存数据库失败: {e}")))?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.create_tables()?;
        db.ensure_model_pricing_seeded()?;
        
        Ok(db)
    }
}
