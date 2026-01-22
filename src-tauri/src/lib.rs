// Open Switch Tauri 库入口

pub mod commands;
pub mod config;
pub mod error;

use std::sync::Mutex;
use config::ConfigManager;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_store::StoreExt;

/// 托盘图标状态包装器（用于在应用生命周期内保持托盘图标存活）
pub struct TrayState(pub TrayIcon);

/// 获取当前关闭行为设置
fn get_close_action_from_store(app: &tauri::AppHandle) -> commands::CloseAction {
    if let Ok(store) = app.store("settings.json") {
        if let Some(value) = store.get("app_settings") {
            if let Ok(settings) = serde_json::from_value::<commands::AppSettings>(value.clone()) {
                return settings.close_action;
            }
        }
    }
    commands::CloseAction::Ask
}

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
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(Mutex::new(config_manager))
        .setup(|app| {
            // 创建托盘菜单
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            
            // 创建托盘图标
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Open Switch")
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键点击托盘图标时显示窗口
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            
            // 将托盘图标存储到应用状态中，防止被释放
            app.manage(TrayState(tray));
            
            Ok(())
        })
        .on_window_event(|window, event| {
            // 处理窗口关闭事件
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let close_action = get_close_action_from_store(app);
                
                match close_action {
                    commands::CloseAction::Tray => {
                        // 隐藏窗口而不是关闭
                        let _ = window.hide();
                        api.prevent_close();
                    }
                    commands::CloseAction::Ask => {
                        // 发送事件到前端让用户选择
                        let _ = window.emit("close-requested", ());
                        api.prevent_close();
                    }
                    commands::CloseAction::Quit => {
                        // 直接退出，不阻止关闭
                    }
                }
            }
        })
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
            // Settings commands
            commands::get_app_settings,
            commands::save_app_settings,
            commands::get_close_action,
            commands::set_close_action,
            commands::handle_close_choice,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
