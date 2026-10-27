use crate::config::AppConfig;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub(crate) fn program_command(path: &str, args: &str, working_dir: Option<&str>) -> Command {
    let mut command = Command::new(path);
    if !args.is_empty() {
        // Windows 快捷方式保存的是完整参数字符串，保留引号和包含空格的参数。
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.raw_arg(args);
        }
        #[cfg(not(target_os = "windows"))]
        command.args(args.split_whitespace());
    }
    if let Some(directory) = working_dir.filter(|d| !d.is_empty()) {
        command.current_dir(directory);
    }
    command
}

/// 静默模式：启动所有 auto_start 模式的程序，然后定时退出
pub fn run_silent_mode() {
    let config = AppConfig::load();

    if !config.auto_start_enabled {
        return;
    }
    let modes: Vec<_> = config
        .modes
        .iter()
        .filter(|m| m.auto_start && m.apps.iter().any(|a| a.enabled))
        .collect();
    if modes.is_empty() {
        return;
    }
    for mode in modes {
        let result = crate::runtime::RuntimeStore::acquire()
            .and_then(|mut store| store.launch_mode(mode, true));
        match result {
            Ok(report) => {
                for failure in report.failures {
                    eprintln!("[Sofsta] {}: {}", failure.name, failure.message);
                }
            }
            Err(error) => eprintln!("[Sofsta] {}", error),
        }
    }

    // 退出前持续记录子进程；运行记录保存在磁盘，之后打开 GUI 仍能执行切换。
    let deadline =
        std::time::Instant::now() + Duration::from_secs(u64::from(config.exit_delay_minutes) * 60);
    while std::time::Instant::now() < deadline {
        if let Ok(mut store) = crate::runtime::RuntimeStore::acquire() {
            let _ = store.sync_current_modes(&config.modes);
        }
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn launch_preserves_quoted_arguments_and_working_directory() {
        // Only run a console expression; do not start installed software or touch user config.
        let powershell = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let directory = std::env::temp_dir();
        let output = program_command(
            powershell.to_str().unwrap(),
            r#"-NoProfile -NonInteractive -Command "[Console]::Write('hello world|' + (Get-Location).Path)""#,
            directory.to_str(),
        )
        .output()
        .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        let actual = String::from_utf8(output.stdout).unwrap();
        let (argument, actual_directory) = actual.split_once('|').unwrap();
        assert_eq!(argument, "hello world");
        assert_eq!(
            std::path::Path::new(actual_directory)
                .canonicalize()
                .unwrap(),
            directory.canonicalize().unwrap()
        );
    }
}
