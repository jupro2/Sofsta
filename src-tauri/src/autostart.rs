#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

const REG_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "Sofsta";
const LEGACY_APP_NAME: &str = "AutoStartManager";

#[cfg(target_os = "windows")]
fn remove_entry(key: &RegKey, name: &str) -> Result<(), String> {
    match key.delete_value(name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("删除启动项 {} 失败: {}", name, error)),
    }
}

/// 注册开机自启动
#[cfg(target_os = "windows")]
pub fn register_autostart() -> Result<(), String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("获取程序路径失败: {}", e))?;

    let value = format!("\"{}\" --silent", exe_path.display());

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(REG_KEY_PATH, KEY_WRITE)
        .map_err(|e| format!("打开注册表失败: {}", e))?;

    run_key
        .set_value(APP_NAME, &value)
        .map_err(|e| format!("设置注册表值失败: {}", e))?;

    // Only remove the legacy entry after the Sofsta entry has been written successfully.
    remove_entry(&run_key, LEGACY_APP_NAME)?;

    Ok(())
}

/// 注销开机自启动
#[cfg(target_os = "windows")]
pub fn unregister_autostart() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(REG_KEY_PATH, KEY_WRITE)
        .map_err(|e| format!("打开注册表失败: {}", e))?;

    remove_entry(&run_key, APP_NAME)?;
    remove_entry(&run_key, LEGACY_APP_NAME)
}

/// 检查是否已注册开机自启动
#[cfg(target_os = "windows")]
pub fn is_autostart_registered() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(REG_KEY_PATH, KEY_READ)
        .map_err(|e| format!("打开注册表失败: {}", e))?;

    Ok([APP_NAME, LEGACY_APP_NAME]
        .iter()
        .any(|name| run_key.get_value::<String, _>(*name).is_ok()))
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
