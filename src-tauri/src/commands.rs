use crate::autostart;
use crate::config::{AppConfig, AppEntry, Mode};
use crate::{installed_apps, runtime};
use tauri::command;

// ───────────────────────── 配置 ─────────────────────────

/// 获取完整配置
#[command]
pub fn get_config() -> Result<AppConfig, String> {
    Ok(AppConfig::load())
}

/// 设置自动退出延迟时间（分钟）
#[command]
pub fn set_exit_delay(minutes: u32) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.exit_delay_minutes = minutes;
    config.save()
}

/// 设置是否开机自启动（总开关）
#[command]
pub fn set_auto_start(enabled: bool) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.auto_start_enabled = enabled;

    if enabled {
        autostart::register_autostart()?;
    } else {
        autostart::unregister_autostart()?;
    }

    config.save()
}

/// 设置是否静默启动
#[command]
pub fn set_silent_launch(enabled: bool) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.silent_launch = enabled;
    config.save()
}

// ───────────────────────── 模式管理 ─────────────────────────

/// 创建新模式
#[command]
pub fn add_mode(name: String, icon_emoji: String, color: String) -> Result<Mode, String> {
    let mut config = AppConfig::load();

    let mode = Mode {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        icon_emoji,
        color,
        auto_start: false,
        apps: Vec::new(),
    };

    config.modes.push(mode.clone());
    config.save()?;

    Ok(mode)
}

/// 编辑模式信息
#[command]
pub fn update_mode(
    id: String,
    name: String,
    icon_emoji: String,
    color: String,
) -> Result<(), String> {
    let mut config = AppConfig::load();
    let mode = config.find_mode_mut(&id).ok_or("未找到该模式")?;
    mode.name = name;
    mode.icon_emoji = icon_emoji;
    mode.color = color;
    config.save()
}

/// 删除模式（至少保留一个模式）
#[command]
pub fn remove_mode(id: String) -> Result<(), String> {
    let mut config = AppConfig::load();

    if config.modes.len() <= 1 {
        return Err("至少需要保留一个模式".to_string());
    }

    config.modes.retain(|m| m.id != id);
    config.save()
}

/// 设置某模式是否开机自启
#[command]
pub fn set_mode_auto_start(mode_id: String, enabled: bool) -> Result<(), String> {
    let mut config = AppConfig::load();
    let mode = config.find_mode_mut(&mode_id).ok_or("未找到该模式")?;
    mode.auto_start = enabled;
    config.save()
}

// ───────────────────────── 程序管理（按模式） ─────────────────────────

/// 向指定模式添加程序
#[command]
pub fn add_app_to_mode(
    mode_id: String,
    path: String,
    name: Option<String>,
    args: Option<String>,
    working_dir: Option<String>,
) -> Result<AppEntry, String> {
    let mut config = AppConfig::load();
    let args = args.unwrap_or_default();

    // 检查该模式中是否已存在
    let mode = config.find_mode(&mode_id).ok_or("未找到该模式")?;
    if mode
        .apps
        .iter()
        .any(|a| a.path.to_lowercase() == path.to_lowercase() && a.args == args)
    {
        return Err("该程序已在此模式中".to_string());
    }

    let app = AppEntry {
        id: uuid::Uuid::new_v4().to_string(),
        name: name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| AppConfig::extract_name(&path)),
        path,
        args,
        working_dir,
        enabled: true,
        delay_seconds: 5,
    };

    let mode = config.find_mode_mut(&mode_id).ok_or("未找到该模式")?;
    mode.apps.push(app.clone());
    config.save()?;

    Ok(app)
}

/// 只读扫描在工作线程中执行，避免阻塞界面。
#[command]
pub async fn get_installed_apps() -> Result<installed_apps::DiscoveryResult, String> {
    tauri::async_runtime::spawn_blocking(installed_apps::discover)
        .await
        .map_err(|e| format!("扫描软件失败: {}", e))?
}

/// 从指定模式删除程序
#[command]
pub fn remove_app(mode_id: String, app_id: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    let mode = config.find_mode_mut(&mode_id).ok_or("未找到该模式")?;
    mode.apps.retain(|a| a.id != app_id);
    config.save()
}

/// 切换程序启用状态
#[command]
pub fn toggle_app(mode_id: String, app_id: String, enabled: bool) -> Result<(), String> {
    let mut config = AppConfig::load();
    let mode = config.find_mode_mut(&mode_id).ok_or("未找到该模式")?;
    let app = mode
        .apps
        .iter_mut()
        .find(|a| a.id == app_id)
        .ok_or("未找到该程序")?;
    app.enabled = enabled;
    config.save()
}

/// 更新启动延迟
#[command]
pub fn update_delay(mode_id: String, app_id: String, seconds: u32) -> Result<(), String> {
    let mut config = AppConfig::load();
    let mode = config.find_mode_mut(&mode_id).ok_or("未找到该模式")?;
    let app = mode
        .apps
        .iter_mut()
        .find(|a| a.id == app_id)
        .ok_or("未找到该程序")?;
    app.delay_seconds = seconds;
    config.save()
}

// ───────────────────────── 启动 ─────────────────────────

async fn runtime_command<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|e| format!("运行操作失败: {}", e))?
}

/// 一键启动模式，重复执行只补充尚未启动的程序。
#[command]
pub async fn launch_mode(mode_id: String) -> Result<runtime::OperationReport, String> {
    runtime_command(move || {
        let config = AppConfig::load();
        let mode = config.find_mode(&mode_id).ok_or("未找到该模式")?;
        runtime::RuntimeStore::acquire()?.launch_mode(mode, false)
    })
    .await
}

/// 先关闭已接管的旧程序，确认退出后才启动目标模式。
#[command]
pub async fn switch_mode(
    mode_id: String,
    force: Option<bool>,
) -> Result<runtime::OperationReport, String> {
    runtime_command(move || {
        let config = AppConfig::load();
        let mode = config.find_mode(&mode_id).ok_or("未找到该模式")?;
        let mut store = runtime::RuntimeStore::acquire()?;
        store.sync_current_modes(&config.modes)?;
        store.switch_mode(mode, force.unwrap_or(false))
    })
    .await
}

#[command]
pub async fn get_runtime_status() -> Result<runtime::RuntimeStatus, String> {
    runtime_command(|| {
        let config = AppConfig::load();
        runtime::RuntimeStore::acquire()?.sync_current_modes(&config.modes)
    })
    .await
}

/// 选择可执行文件（打开文件对话框）
#[command]
pub fn pick_executable(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::{DialogExt, FilePath};

    let result = app
        .dialog()
        .file()
        .set_title("选择要自启动的程序")
        .add_filter("可执行文件", &["exe"])
        .add_filter("所有文件", &["*"])
        .blocking_pick_file();

    match result {
        Some(file_path) => match file_path {
            FilePath::Path(p) => Ok(Some(p.to_string_lossy().to_string())),
            _ => Ok(None),
        },
        None => Ok(None),
    }
}

/// 立即测试启动一个程序
#[command]
pub async fn launch_app_now(
    mode_id: String,
    app_id: String,
) -> Result<runtime::OperationReport, String> {
    runtime_command(move || {
        let config = AppConfig::load();
        let mut mode = config.find_mode(&mode_id).ok_or("未找到该模式")?.clone();
        let mut app = mode
            .apps
            .iter()
            .find(|a| a.id == app_id)
            .ok_or("未找到该程序")?
            .clone();
        app.enabled = true;
        app.delay_seconds = 0;
        mode.apps = vec![app];
        runtime::RuntimeStore::acquire()?.launch_mode(&mode, true)
    })
    .await
}
