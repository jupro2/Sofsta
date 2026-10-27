use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
    pub args: String,
    pub working_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscoveryResult {
    pub apps: Vec<InstalledApp>,
    pub warnings: Vec<String>,
}

#[cfg(target_os = "windows")]
pub fn discover() -> Result<DiscoveryResult, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let windows_dir = std::env::var_os("SystemRoot").ok_or("无法找到 Windows 系统目录")?;
    let powershell = std::path::PathBuf::from(windows_dir)
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let output = Command::new(powershell)
        .args(["-NoProfile", "-NonInteractive", "-Sta", "-Command"])
        .arg(include_str!("discover_apps.ps1"))
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("读取本机软件失败: {}", e))?;

    if !output.status.success() {
        return Err("读取本机软件失败，请重试或手动选择文件".to_string());
    }
    serde_json::from_slice(&output.stdout).map_err(|e| format!("解析本机软件列表失败: {}", e))
}

#[cfg(not(target_os = "windows"))]
pub fn discover() -> Result<DiscoveryResult, String> {
    Err("软件发现功能仅支持 Windows 系统".to_string())
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Reads the local Start menu and registry; run explicitly for a Windows smoke test"]
    fn discovery_reads_local_software_without_launching_it() {
        let result = discover().unwrap();
        let mut seen = std::collections::HashSet::new();
        for app in &result.apps {
            assert!(!app.name.is_empty());
            assert!(app.path.to_lowercase().ends_with(".exe"));
            assert!(seen.insert((app.path.to_lowercase(), &app.args)));
        }
        println!(
            "Discovered {} launch entries; {} source warnings",
            result.apps.len(),
            result.warnings.len()
        );
    }
}
