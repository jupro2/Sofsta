mod autostart;
mod commands;
mod config;
mod installed_apps;
mod launcher;
mod processes;
mod runtime;

use std::env;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 检查是否是静默模式（开机自启模式）
    let silent = env::args().any(|arg| arg == "--silent");

    if silent {
        // 静默模式：启动所有自启模式的程序，然后定时退出
        launcher::run_silent_mode();
        return;
    }

    // 正常模式：显示 GUI
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // 配置
            commands::get_config,
            commands::set_exit_delay,
            commands::set_auto_start,
            commands::set_silent_launch,
            // 模式管理
            commands::add_mode,
            commands::update_mode,
            commands::remove_mode,
            commands::set_mode_auto_start,
            // 程序管理
            commands::add_app_to_mode,
            commands::get_installed_apps,
            commands::remove_app,
            commands::toggle_app,
            commands::update_delay,
            // 启动
            commands::launch_mode,
            commands::switch_mode,
            commands::get_runtime_status,
            commands::launch_app_now,
            commands::pick_executable,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
