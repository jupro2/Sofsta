use crate::config::{AppEntry, Mode};
use crate::processes::{path_key, NativeBackend, ProcessBackend, ProcessIdentity, Snapshot};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningMode {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManagedApp {
    mode_id: String,
    entry: AppEntry,
    processes: Vec<ProcessIdentity>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RuntimeState {
    version: u32,
    modes: Vec<RunningMode>,
    apps: Vec<ManagedApp>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            version: 1,
            modes: Vec::new(),
            apps: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct RuntimeStatus {
    pub modes: Vec<RunningMode>,
    pub managed_app_count: usize,
    pub process_count: usize,
}

#[derive(Debug, Serialize)]
pub struct LaunchFailure {
    pub name: String,
    pub message: String,
}

#[derive(Debug, Default, Serialize)]
pub struct OperationReport {
    pub status: RuntimeStatus,
    pub started: usize,
    pub closed: usize,
    pub already_running: Vec<String>,
    pub failures: Vec<LaunchFailure>,
    pub unclosed: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct RuntimeStore {
    directory: PathBuf,
    _lock: File,
    state: RuntimeState,
    saved: String,
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;

impl RuntimeStore {
    pub fn acquire() -> Result<Self, String> {
        Self::open_at(&crate::config::config_dir())
    }

    fn open_at(directory: &Path) -> Result<Self, String> {
        fs::create_dir_all(directory).map_err(|e| format!("创建运行记录目录失败: {}", e))?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }
        let lock = options
            .open(directory.join("runtime.lock"))
            .map_err(|_| "另一个窗口正在处理启动或切换操作，请稍候重试".to_string())?;
        let path = directory.join("runtime.json");
        let state = match fs::read_to_string(path) {
            Ok(text) => serde_json::from_str::<RuntimeState>(&text)
                .map_err(|e| format!("运行记录无法读取，为避免关闭错误进程，操作已停止: {}", e))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => RuntimeState::default(),
            Err(e) => return Err(format!("读取运行记录失败: {}", e)),
        };
        if state.version != 1 {
            return Err("运行记录版本不兼容，无法安全切换模式".into());
        }
        let saved = serde_json::to_string(&state).map_err(|e| e.to_string())?;
        Ok(Self {
            directory: directory.to_path_buf(),
            _lock: lock,
            state,
            saved,
        })
    }

    fn save(&mut self) -> Result<(), String> {
        let json = serde_json::to_string(&self.state).map_err(|e| e.to_string())?;
        if json == self.saved {
            return Ok(());
        }
        let temporary = self
            .directory
            .join(format!("runtime-{}.tmp", uuid::Uuid::new_v4()));
        fs::write(&temporary, &json).map_err(|e| format!("保存运行记录失败: {}", e))?;
        fs::rename(&temporary, self.directory.join("runtime.json"))
            .map_err(|e| format!("更新运行记录失败: {}", e))?;
        self.saved = json;
        Ok(())
    }

    fn status(&self) -> RuntimeStatus {
        RuntimeStatus {
            modes: self.state.modes.clone(),
            managed_app_count: self.state.apps.len(),
            process_count: self.state.apps.iter().map(|a| a.processes.len()).sum(),
        }
    }

    fn refresh(&mut self, snapshot: &Snapshot, trusted_new: &[ProcessIdentity]) {
        let live: HashSet<_> = snapshot.processes.iter().map(|p| p.identity).collect();
        for app in &mut self.state.apps {
            let executable = path_key(&app.entry.path);
            let mut owned: HashSet<_> = app
                .processes
                .iter()
                .copied()
                .filter(|id| live.contains(id))
                .collect();
            owned.extend(
                trusted_new
                    .iter()
                    .copied()
                    .filter(|id| app.processes.contains(id)),
            );
            loop {
                let descendants: Vec<_> = snapshot
                    .processes
                    .iter()
                    .filter(|p| {
                        p.identity.pid != std::process::id()
                            && !owned.contains(&p.identity)
                            // Only adopt additional instances of this executable. An app can
                            // also launch unrelated software (for example a browser from a link).
                            && !p.path.is_empty()
                            && path_key(&p.path) == executable
                            && owned.iter().any(|parent| {
                                parent.pid == p.parent_pid
                                    && parent.created_at <= p.identity.created_at
                            })
                    })
                    .map(|p| p.identity)
                    .collect();
                if descendants.is_empty() {
                    break;
                }
                owned.extend(descendants);
            }
            // Retain inaccessible recorded processes; never infer that access denied means exited.
            owned.extend(
                app.processes
                    .iter()
                    .copied()
                    .filter(|id| snapshot.unreadable_pids.contains(&id.pid)),
            );
            owned.retain(|id| live.contains(id) || snapshot.unreadable_pids.contains(&id.pid));
            let mut processes: Vec<_> = owned.into_iter().collect();
            processes.sort_by_key(|id| (id.pid, id.created_at));
            app.processes = processes;
        }
        self.state.apps.retain(|app| !app.processes.is_empty());
    }

    fn adopt_matching(&mut self, mode_id: &str, app: &AppEntry, snapshot: &Snapshot) {
        let key = path_key(&app.path);
        let matching: Vec<_> = snapshot
            .processes
            .iter()
            .filter(|p| {
                p.identity.pid != std::process::id()
                    && !p.path.is_empty()
                    && path_key(&p.path) == key
            })
            .map(|p| p.identity)
            .collect();
        if matching.is_empty() {
            return;
        }
        if let Some(managed) = self
            .state
            .apps
            .iter_mut()
            .find(|a| path_key(&a.entry.path) == key)
        {
            for identity in matching {
                if !managed.processes.contains(&identity) {
                    managed.processes.push(identity);
                }
            }
        } else {
            self.state.apps.push(ManagedApp {
                mode_id: mode_id.into(),
                entry: app.clone(),
                processes: matching,
            });
        }
    }

    pub fn sync_current_modes(&mut self, modes: &[Mode]) -> Result<RuntimeStatus, String> {
        let snapshot = NativeBackend.snapshot()?;
        self.sync_from_snapshot(modes, &snapshot)?;
        Ok(self.status())
    }

    fn sync_from_snapshot(&mut self, modes: &[Mode], snapshot: &Snapshot) -> Result<(), String> {
        self.refresh(snapshot, &[]);
        for mode in modes
            .iter()
            .filter(|mode| self.state.modes.iter().any(|m| m.id == mode.id))
            .cloned()
            .collect::<Vec<_>>()
        {
            for app in mode.apps.iter().filter(|app| app.enabled) {
                self.adopt_matching(&mode.id, app, snapshot);
            }
        }
        self.save()
    }

    fn identities(&self) -> Vec<ProcessIdentity> {
        self.state
            .apps
            .iter()
            .flat_map(|a| a.processes.iter().copied())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn validate(entries: &[AppEntry]) -> Result<(), String> {
        if entries.is_empty() {
            return Err("此模式没有已启用的程序，请先添加或启用程序".into());
        }
        for app in entries {
            if !Path::new(&app.path).is_file() {
                return Err(format!("「{}」的程序文件不存在，请先修正路径", app.name));
            }
            if app.path.contains('\0') || app.args.contains('\0') {
                return Err(format!("「{}」包含无效的启动参数", app.name));
            }
            if app.delay_seconds > 300 {
                return Err(format!("「{}」的启动延迟不能超过 300 秒", app.name));
            }
            if let Some(directory) = app.working_dir.as_deref().filter(|d| !d.is_empty()) {
                if !Path::new(directory).is_dir() {
                    return Err(format!("「{}」的工作目录不存在，请先修正配置", app.name));
                }
            }
        }
        Ok(())
    }

    pub fn launch_mode(
        &mut self,
        mode: &Mode,
        allow_multiple: bool,
    ) -> Result<OperationReport, String> {
        self.run(
            mode,
            false,
            false,
            allow_multiple,
            Duration::from_secs(8),
            &NativeBackend,
        )
    }

    pub fn switch_mode(&mut self, mode: &Mode, force: bool) -> Result<OperationReport, String> {
        self.run(
            mode,
            true,
            force,
            false,
            Duration::from_secs(8),
            &NativeBackend,
        )
    }

    fn run<P: ProcessBackend>(
        &mut self,
        mode: &Mode,
        switching: bool,
        force: bool,
        allow_multiple: bool,
        close_timeout: Duration,
        backend: &P,
    ) -> Result<OperationReport, String> {
        let entries: Vec<_> = mode.apps.iter().filter(|a| a.enabled).cloned().collect();
        // Preflight all target entries before sending any close request to the old mode.
        Self::validate(&entries)?;
        self.refresh(&backend.snapshot()?, &[]);
        self.save()?;
        let same_mode = self.state.modes.len() == 1 && self.state.modes[0].id == mode.id;
        if !switching && !allow_multiple && !same_mode && !self.state.apps.is_empty() {
            return Err("另一个模式仍在运行，请使用「切换模式」关闭旧程序后再启动".into());
        }
        let mut report = OperationReport::default();
        if switching && !same_mode && !self.state.apps.is_empty() {
            let initial_count = self.state.apps.len();
            let deadline = Instant::now() + close_timeout;
            let mut requested = HashSet::new();
            loop {
                let new: Vec<_> = self
                    .identities()
                    .into_iter()
                    .filter(|id| requested.insert(*id))
                    .collect();
                if !new.is_empty() {
                    if let Err(error) = backend.close(&new, force) {
                        report.warnings.push(error);
                    }
                }
                self.refresh(&backend.snapshot()?, &[]);
                self.save()?;
                if self.state.apps.is_empty() || Instant::now() >= deadline {
                    break;
                }
                thread::sleep(Duration::from_millis(150));
            }
            report.closed = initial_count.saturating_sub(self.state.apps.len());
            if !self.state.apps.is_empty() {
                report.unclosed = self
                    .state
                    .apps
                    .iter()
                    .map(|a| a.entry.name.clone())
                    .collect();
                report.status = self.status();
                return Ok(report); // No target process is started while any old process remains.
            }
        }

        if !allow_multiple {
            self.state.modes.clear();
        }
        if !self.state.modes.iter().any(|m| m.id == mode.id) {
            self.state.modes.push(RunningMode {
                id: mode.id.clone(),
                name: mode.name.clone(),
            });
        }
        self.save()?;
        for app in entries {
            let before = backend.snapshot()?;
            self.refresh(&before, &[]);
            let owned: HashSet<_> = self.identities().into_iter().collect();
            let key = path_key(&app.path);
            let tracked = self.state.apps.iter().any(|a| {
                path_key(&a.entry.path) == key
                    && a.entry.args == app.args
                    && a.entry.working_dir == app.working_dir
            });
            let external = before.processes.iter().any(|p| {
                !owned.contains(&p.identity) && !p.path.is_empty() && path_key(&p.path) == key
            });
            if tracked || external {
                // Selecting a mode manages its configured executables, including instances
                // that were already running before the manager was opened.
                self.adopt_matching(&mode.id, &app, &before);
                self.save()?;
                report.already_running.push(app.name.clone());
                continue;
            }
            // Release process-query handles before a user-configured delay.
            drop(before);
            if app.delay_seconds > 0 {
                thread::sleep(Duration::from_secs(u64::from(app.delay_seconds)));
            }
            match backend.spawn(&app) {
                Ok(spawned) => {
                    let app_id = app.id.clone();
                    let app_name = app.name.clone();
                    self.state.apps.push(ManagedApp {
                        mode_id: mode.id.clone(),
                        entry: app,
                        processes: vec![spawned.identity],
                    });
                    self.save()?;
                    report.started += 1;
                    // Capture early child processes while the original Child handle still prevents PID reuse.
                    thread::sleep(Duration::from_millis(150));
                    self.refresh(&backend.snapshot()?, &[spawned.identity]);
                    self.save()?;
                    if !self.state.apps.iter().any(|a| a.entry.id == app_id) {
                        report.warnings.push(format!(
                            "「{}」的启动进程已退出，未确认到可管理的子进程",
                            app_name
                        ));
                    }
                }
                Err(message) => report.failures.push(LaunchFailure {
                    name: app.name,
                    message,
                }),
            }
        }
        self.save()?;
        report.status = self.status();
        Ok(report)
    }
}
