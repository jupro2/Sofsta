use crate::config::AppConfig;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// 静默模式：启动所有 auto_start 模式的程序，然后定时退出
pub fn run_silent_mode() {
    let config = AppConfig::load();

    // 收集所有 auto_start 模式中的已启用程序
    let apps_to_launch: Vec<_> = config
        .modes
        .iter()
        .filter(|m| m.auto_start)
        .flat_map(|m| m.apps.iter())
        .filter(|a| a.enabled)
        .collect();

    if apps_to_launch.is_empty() {
        return;
    }

    // 逐个启动程序
    for app in &apps_to_launch {
        if app.delay_seconds > 0 {
            thread::sleep(Duration::from_secs(app.delay_seconds as u64));
        }

        let mut cmd = Command::new(&app.path);
        if !app.args.is_empty() {
            cmd.args(app.args.split_whitespace().collect::<Vec<_>>());
        }

        match cmd.spawn() {
            Ok(_) => println!("[AutoStartManager] 已启动: {}", app.name),
            Err(e) => eprintln!("[AutoStartManager] 启动 {} 失败: {}", app.name, e),
        }
    }

    // 等待指定时间后退出
    let exit_delay = Duration::from_secs(config.exit_delay_minutes as u64 * 60);
    println!(
        "[AutoStartManager] 所有程序已启动，将在 {} 分钟后退出",
        config.exit_delay_minutes
    );

    thread::sleep(exit_delay);
    println!("[AutoStartManager] 自动退出");
}
