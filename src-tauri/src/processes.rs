use crate::config::AppEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Child;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub created_at: u64,
}

#[derive(Clone)]
pub struct ProcessInfo {
    pub identity: ProcessIdentity,
    pub parent_pid: u32,
    pub path: String,
}

#[derive(Default)]
pub struct Snapshot {
    pub processes: Vec<ProcessInfo>,
    // Permission failures are not proof of exit: keep those records and block unsafe switching.
    pub unreadable_pids: HashSet<u32>,
    #[cfg(windows)]
    pub(crate) guards: Vec<native::Handle>,
}

pub struct Spawned {
    pub identity: ProcessIdentity,
    // Keep the original process object alive until its first descendants have been recorded.
    pub _child: Option<Child>,
    #[cfg(windows)]
    pub _shell_handle: Option<std::os::windows::io::OwnedHandle>,
}

pub trait ProcessBackend {
    fn snapshot(&self) -> Result<Snapshot, String>;
    fn spawn(&self, app: &AppEntry) -> Result<Spawned, String>;
    fn close(&self, identities: &[ProcessIdentity], force: bool) -> Result<(), String>;
}

pub struct NativeBackend;

#[cfg(all(test, windows))]
#[path = "process_tests.rs"]
mod tests;

pub fn path_key(path: &str) -> String {
    let canonical = std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string());
    canonical
        .trim_start_matches(r"\\?\")
        .replace('/', "\\")
        .to_lowercase()
}

#[cfg(windows)]
pub(crate) mod native {
    use super::*;
    use std::{
        ffi::c_void,
        mem::size_of,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    };
    use windows_sys::Win32::{
        Foundation::{
            CloseHandle, FILETIME, HANDLE, HWND, INVALID_HANDLE_VALUE, LPARAM, WAIT_TIMEOUT,
        },
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                GetProcessId, GetProcessTimes, OpenProcess, QueryFullProcessImageNameW,
                TerminateProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
                PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
            },
        },
        UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE},
    };

    fn elevation_error(error: &std::io::Error) -> String {
        match error.raw_os_error() {
            Some(1223) => "你取消了 Windows 管理员授权，程序未启动".into(),
            Some(5) => {
                "权限不足：目标软件可能以管理员身份运行，请以管理员身份打开管理器后重试".into()
            }
            _ => format!("Windows 启动失败: {}", error),
        }
    }

    // A separate STA thread avoids inheriting a worker thread's COM apartment.
    fn shell_launch(app: AppEntry, elevate: bool) -> Result<Spawned, String> {
        std::thread::spawn(move || {
            use windows_sys::Win32::{
                System::Com::{
                    CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED,
                    COINIT_DISABLE_OLE1DDE,
                },
                UI::{
                    Shell::{
                        ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC,
                        SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
                    },
                    WindowsAndMessaging::SW_SHOWNORMAL,
                },
            };
            let initialized = unsafe {
                CoInitializeEx(
                    std::ptr::null(),
                    (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
                )
            };
            if initialized < 0 {
                return Err(format!("初始化 Windows 授权启动失败: {:#x}", initialized));
            }
            struct Apartment;
            impl Drop for Apartment {
                fn drop(&mut self) {
                    unsafe {
                        CoUninitialize();
                    }
                }
            }
            let _apartment = Apartment;
            fn wide(value: &str) -> Vec<u16> {
                value.encode_utf16().chain(Some(0)).collect()
            }
            let verb = wide(if elevate { "runas" } else { "open" });
            let file = wide(&app.path);
            let arguments = wide(&app.args);
            let directory = app
                .working_dir
                .as_deref()
                .filter(|v| !v.is_empty())
                .map(wide);
            let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
            info.cbSize = size_of::<SHELLEXECUTEINFOW>() as u32;
            info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI;
            info.lpVerb = verb.as_ptr();
            info.lpFile = file.as_ptr();
            info.lpParameters = if app.args.is_empty() {
                std::ptr::null()
            } else {
                arguments.as_ptr()
            };
            info.lpDirectory = directory.as_ref().map_or(std::ptr::null(), |v| v.as_ptr());
            info.nShow = SW_SHOWNORMAL;
            if unsafe { ShellExecuteExW(&mut info) } == 0 {
                return Err(elevation_error(&std::io::Error::last_os_error()));
            }
            if info.hProcess.is_null() {
                return Err("Windows 已提交启动请求，但未返回进程句柄，无法确认运行状态".into());
            }
            let handle = unsafe { OwnedHandle::from_raw_handle(info.hProcess) };
            let pid = unsafe { GetProcessId(info.hProcess) };
            if pid == 0 {
                return Err("软件已启动，但无法读取其进程标识".into());
            }
            let created_at = creation_time(info.hProcess)
                .map_err(|e| format!("软件已启动，但无法记录其运行信息: {}", e))?;
            Ok(Spawned {
                identity: ProcessIdentity { pid, created_at },
                _child: None,
                _shell_handle: Some(handle),
            })
        })
        .join()
        .map_err(|_| "Windows 授权启动线程异常".to_string())?
    }

    fn spawn_with_fallback(
        app: &AppEntry,
        create: impl FnOnce() -> std::io::Result<Child>,
        elevate: impl FnOnce(AppEntry) -> Result<Spawned, String>,
    ) -> Result<Spawned, String> {
        let child = match create() {
            Ok(child) => child,
            Err(error) if error.raw_os_error() == Some(740) => return elevate(app.clone()),
            Err(error) => return Err(format!("启动失败: {}", error)),
        };
        let identity = ProcessIdentity {
            pid: child.id(),
            created_at: creation_time(child.as_raw_handle() as *mut c_void)?,
        };
        Ok(Spawned {
            identity,
            _child: Some(child),
            _shell_handle: None,
        })
    }

    #[cfg(test)]
    mod elevation_tests {
        use super::*;
        fn entry() -> AppEntry {
            AppEntry {
                id: "elevation-test".into(),
                name: "test".into(),
                path: "example.exe".into(),
                args: "--profile \"two words\"".into(),
                working_dir: Some("C:\\Example Folder".into()),
                enabled: true,
                delay_seconds: 0,
            }
        }

        #[test]
        fn elevation_required_uses_shell_fallback_and_preserves_launch_metadata() {
            let app = entry();
            let called = std::cell::Cell::new(false);
            let result = spawn_with_fallback(
                &app,
                || Err(std::io::Error::from_raw_os_error(740)),
                |received| {
                    called.set(true);
                    assert_eq!(received.path, app.path);
                    assert_eq!(received.args, app.args);
                    assert_eq!(received.working_dir, app.working_dir);
                    Err("test authorization cancelled".into())
                },
            );
            assert!(called.get());
            assert!(matches!(result, Err(message) if message == "test authorization cancelled"));
        }

        #[test]
        fn ordinary_start_errors_do_not_trigger_admin_prompts() {
            let result = spawn_with_fallback(
                &entry(),
                || Err(std::io::Error::from_raw_os_error(2)),
                |_| panic!("file-not-found must not elevate"),
            );
            assert!(result.is_err());
            assert!(elevation_error(&std::io::Error::from_raw_os_error(1223)).contains("取消"));
        }

        #[test]
        #[ignore = "Explicit opt-in: starts SOFSTA_TEST_LAUNCH_EXE and may show UAC; leaves the application running"]
        fn interactive_launch_smoke() {
            let path = std::env::var("SOFSTA_TEST_LAUNCH_EXE")
                .expect("set SOFSTA_TEST_LAUNCH_EXE explicitly");
            assert!(std::path::Path::new(&path).is_file());
            let app = AppEntry {
                path: path.clone(),
                name: "Interactive startup verification".into(),
                args: String::new(),
                working_dir: std::path::Path::new(&path)
                    .parent()
                    .map(|p| p.to_string_lossy().into_owned()),
                ..entry()
            };
            let process = NativeBackend
                .spawn(&app)
                .unwrap_or_else(|e| panic!("{}", e));
            println!(
                "Launch succeeded: PID {}; shell elevation used: {}",
                process.identity.pid,
                process._shell_handle.is_some()
            );
        }

        #[test]
        #[ignore = "Read-only local diagnosis: match configured executables against the process snapshot; never closes or starts software"]
        fn configured_process_match_diagnostic() {
            let text =
                std::fs::read_to_string(crate::config::config_dir().join("config.json")).unwrap();
            let config: crate::config::AppConfig = serde_json::from_str(&text).unwrap();
            let snapshot = NativeBackend.snapshot().unwrap();
            for mode in config.modes {
                for app in mode.apps.into_iter().filter(|app| app.enabled) {
                    let expected = path_key(&app.path);
                    let count = snapshot
                        .processes
                        .iter()
                        .filter(|p| !p.path.is_empty() && path_key(&p.path) == expected)
                        .count();
                    println!(
                        "{} / {}: {} matching processes; executable exists: {}",
                        mode.name,
                        app.name,
                        count,
                        std::path::Path::new(&app.path).is_file()
                    );
                }
            }
        }
    }

    pub struct Handle(pub HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    fn creation_time(handle: HANDLE) -> Result<u64, String> {
        let mut created: FILETIME = unsafe { std::mem::zeroed() };
        let mut exited = created;
        let mut kernel = created;
        let mut user = created;
        if unsafe { GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) }
            == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
    }

    fn open(pid: u32, force: bool) -> Result<Handle, std::io::Error> {
        let rights = PROCESS_QUERY_LIMITED_INFORMATION
            | PROCESS_SYNCHRONIZE
            | if force { PROCESS_TERMINATE } else { 0 };
        let handle = unsafe { OpenProcess(rights, 0, pid) };
        if handle.is_null() {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(Handle(handle))
        }
    }

    fn verified(identity: ProcessIdentity, force: bool) -> Result<Option<Handle>, String> {
        let handle = match open(identity.pid, force) {
            Ok(handle) => handle,
            Err(error) => {
                // ERROR_INVALID_PARAMETER means the process no longer exists.
                if error.raw_os_error() == Some(87) {
                    return Ok(None);
                }
                return Err(error.to_string());
            }
        };
        if creation_time(handle.0)? != identity.created_at
            || unsafe { WaitForSingleObject(handle.0, 0) } != WAIT_TIMEOUT
        {
            return Ok(None);
        }
        Ok(Some(handle))
    }

    impl ProcessBackend for NativeBackend {
        fn snapshot(&self) -> Result<Snapshot, String> {
            let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
            if handle == INVALID_HANDLE_VALUE {
                return Err(format!(
                    "读取运行进程失败: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let snapshot_handle = Handle(handle);
            let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
            entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
            let mut result = Snapshot::default();
            let mut present = unsafe { Process32FirstW(snapshot_handle.0, &mut entry) } != 0;
            if !present && std::io::Error::last_os_error().raw_os_error() != Some(18) {
                return Err(format!(
                    "读取进程快照失败: {}",
                    std::io::Error::last_os_error()
                ));
            }
            while present {
                let pid = entry.th32ProcessID;
                if pid != 0 {
                    match open(pid, false) {
                        Ok(process) => {
                            if unsafe { WaitForSingleObject(process.0, 0) } == WAIT_TIMEOUT {
                                let mut path = vec![0u16; 32768];
                                let mut length = path.len() as u32;
                                match creation_time(process.0) {
                                    Ok(created_at) => {
                                        // A missing image path prevents duplicate detection, but not identity verification.
                                        let path = if unsafe {
                                            QueryFullProcessImageNameW(
                                                process.0,
                                                0,
                                                path.as_mut_ptr(),
                                                &mut length,
                                            )
                                        } != 0
                                        {
                                            String::from_utf16_lossy(&path[..length as usize])
                                        } else {
                                            String::new()
                                        };
                                        result.processes.push(ProcessInfo {
                                            identity: ProcessIdentity { pid, created_at },
                                            parent_pid: entry.th32ParentProcessID,
                                            path,
                                        });
                                        result.guards.push(process);
                                    }
                                    Err(_) => {
                                        result.unreadable_pids.insert(pid);
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            result.unreadable_pids.insert(pid);
                        }
                    }
                }
                present = unsafe { Process32NextW(snapshot_handle.0, &mut entry) } != 0;
            }
            if std::io::Error::last_os_error().raw_os_error() != Some(18) {
                return Err(format!(
                    "进程枚举未完成: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(result)
        }

        fn spawn(&self, app: &AppEntry) -> Result<Spawned, String> {
            spawn_with_fallback(
                app,
                || {
                    crate::launcher::program_command(
                        &app.path,
                        &app.args,
                        app.working_dir.as_deref(),
                    )
                    .spawn()
                },
                |app| shell_launch(app, true),
            )
        }

        fn close(&self, identities: &[ProcessIdentity], force: bool) -> Result<(), String> {
            let mut handles = Vec::new();
            let mut errors = Vec::new();
            for &identity in identities {
                // A running manager must never be closed through an app entry pointing to itself.
                if identity.pid == std::process::id() {
                    errors.push("无法关闭管理器自身".to_string());
                    continue;
                }
                match verified(identity, force) {
                    Ok(Some(handle)) => handles.push((identity.pid, handle)),
                    Ok(None) => {}
                    Err(error) => errors.push(format!(
                        "PID {}: {}。若该软件以管理员身份运行，请以管理员身份重新打开管理器后重试",
                        identity.pid, error
                    )),
                }
            }
            if force {
                for (pid, handle) in &handles {
                    if unsafe { TerminateProcess(handle.0, 1) } == 0
                        && unsafe { WaitForSingleObject(handle.0, 0) } == WAIT_TIMEOUT
                    {
                        errors.push(format!(
                            "结束 PID {} 失败: {}",
                            pid,
                            std::io::Error::last_os_error()
                        ));
                    }
                }
            } else {
                struct Context {
                    pids: HashSet<u32>,
                    errors: Vec<String>,
                }
                unsafe extern "system" fn close_window(window: HWND, parameter: LPARAM) -> i32 {
                    let context = &mut *(parameter as *mut Context);
                    let mut pid = 0;
                    GetWindowThreadProcessId(window, &mut pid);
                    if context.pids.contains(&pid) && PostMessageW(window, WM_CLOSE, 0, 0) == 0 {
                        context
                            .errors
                            .push(format!("向 PID {} 发送关闭请求失败", pid));
                    }
                    1
                }
                let mut context = Context {
                    pids: handles.iter().map(|(pid, _)| *pid).collect(),
                    errors: Vec::new(),
                };
                if unsafe {
                    EnumWindows(Some(close_window), &mut context as *mut Context as LPARAM)
                } == 0
                {
                    errors.push(format!(
                        "枚举应用窗口失败: {}",
                        std::io::Error::last_os_error()
                    ));
                }
                errors.extend(context.errors);
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors.join("；"))
            }
        }
    }
}

#[cfg(not(windows))]
impl ProcessBackend for NativeBackend {
    fn snapshot(&self) -> Result<Snapshot, String> {
        Err("模式切换仅支持 Windows".into())
    }
    fn spawn(&self, _: &AppEntry) -> Result<Spawned, String> {
        Err("模式切换仅支持 Windows".into())
    }
    fn close(&self, _: &[ProcessIdentity], _: bool) -> Result<(), String> {
        Err("模式切换仅支持 Windows".into())
    }
}
