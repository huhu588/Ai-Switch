// Open Switch Tauri 库入口

pub mod commands;
pub mod config;
pub mod error;

use std::sync::Mutex;
use config::ConfigManager;

/// 运行 Tauri 应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化配置管理器
    let config_manager = ConfigManager::new()
        .expect("初始化配置管理器失败");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(Mutex::new(config_manager))
        .invoke_handler(tauri::generate_handler![
            // Provider commands
            commands::get_providers,
            commands::get_provider,
            commands::add_provider,
            commands::update_provider,
            commands::delete_provider,
            commands::toggle_provider,
            commands::check_provider_applied,
            commands::apply_config,
            // Model commands
            commands::get_models,
            commands::add_model,
            commands::delete_model,
            commands::fetch_site_models,
            commands::add_models_batch,
            // MCP commands
            commands::get_mcp_servers,
            commands::get_mcp_server,
            commands::add_mcp_server,
            commands::update_mcp_server,
            commands::delete_mcp_server,
            commands::toggle_mcp_server,
            commands::sync_mcp_config,
            commands::get_recommended_mcp_servers,
            commands::add_recommended_mcp_servers,
            commands::check_mcp_server_health,
            // skills commands
            commands::get_installed_skills,
            commands::get_recommended_skills,
            commands::install_skills,
            commands::delete_skills,
            commands::read_skills_content,
            // skills repository commands
            commands::get_skills_repos,
            commands::add_skills_repo,
            commands::delete_skills_repo,
            commands::toggle_skills_repo,
            commands::toggle_skills_repo_enabled,
            commands::fetch_skills_from_repo,
            commands::discover_skills,
            // Rule commands
            commands::get_installed_rules,
            commands::get_recommended_rules,
            commands::install_rule,
            commands::delete_rule,
            commands::read_rule_content,
            commands::save_rule_content,
            commands::toggle_rule_enabled,
            // Status commands
            commands::get_status,
            commands::get_version,
            commands::get_local_ip,
            // Backup commands
            commands::create_backup,
            commands::export_backup,
            commands::preview_backup,
            commands::import_backup,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
