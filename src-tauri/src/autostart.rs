#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

const REG_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "AutoStartManager";

/// 注册开机自启动
#[cfg(target_os = "windows")]
pub fn register_autostart() -> Result<(), String> {
    let exe_path =
        std::env::current_exe().map_err(|e| format!("获取程序路径失败: {}", e))?;

    let value = format!("\"{}\" --silent", exe_path.display());

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(REG_KEY_PATH, KEY_WRITE)
        .map_err(|e| format!("打开注册表失败: {}", e))?;

    run_key
        .set_value(APP_NAME, &value)
        .map_err(|e| format!("设置注册表值失败: {}", e))?;

    Ok(())
}

/// 注销开机自启动
#[cfg(target_os = "windows")]
pub fn unregister_autostart() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(REG_KEY_PATH, KEY_WRITE)
        .map_err(|e| format!("打开注册表失败: {}", e))?;

    match run_key.delete_value(APP_NAME) {
        Ok(_) => Ok(()),
        Err(_) => Ok(()), // 值不存在也视为成功
    }
}

/// 检查是否已注册开机自启动
#[cfg(target_os = "windows")]
pub fn is_autostart_registered() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(REG_KEY_PATH, KEY_READ)
        .map_err(|e| format!("打开注册表失败: {}", e))?;

    match run_key.get_value::<String, _>(APP_NAME) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// 非 Windows 平台的空实现
#[cfg(not(target_os = "windows"))]
pub fn register_autostart() -> Result<(), String> {
    Err("仅支持 Windows 系统".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn unregister_autostart() -> Result<(), String> {
    Err("仅支持 Windows 系统".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn is_autostart_registered() -> Result<bool, String> {
    Ok(false)
}
