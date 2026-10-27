use super::*;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
        RegisterClassW, TranslateMessage, MSG, WM_CLOSE, WM_DESTROY, WNDCLASSW,
        WS_OVERLAPPEDWINDOW,
    },
};

#[test]
fn window_fixture() {
    let Some(marker) = std::env::var_os("SOFSTA_TEST_WINDOW_MARKER") else {
        return;
    };
    unsafe extern "system" fn window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_DESTROY {
            PostQuitMessage(0);
            return 0;
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
    let name: Vec<u16> = format!("SofstaTest{}\0", std::process::id())
        .encode_utf16()
        .collect();
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            ..std::mem::zeroed()
        };
        assert_ne!(RegisterClassW(&class), 0);
        // A hidden test window: no installed app, document, visible UI, or user configuration.
        let window = CreateWindowExW(
            0,
            name.as_ptr(),
            name.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            100,
            100,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );
        assert!(!window.is_null());
        std::fs::write(marker, b"ready").unwrap();
        let mut message: MSG = std::mem::zeroed();
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

struct Fixture {
    child: std::process::Child,
    marker: PathBuf,
}
impl Fixture {
    fn start() -> Self {
        use std::os::windows::process::CommandExt;
        let marker =
            std::env::temp_dir().join(format!("Sofsta-window-{}.ready", uuid::Uuid::new_v4()));
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "processes::tests::window_fixture", "--nocapture"])
            .env("SOFSTA_TEST_WINDOW_MARKER", &marker)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x0800_0000)
            .spawn()
            .unwrap();
        let mut result = Self { child, marker };
        let deadline = Instant::now() + Duration::from_secs(5);
        while !result.marker.exists() && Instant::now() < deadline {
            assert!(
                result.child.try_wait().unwrap().is_none(),
                "test window process exited before readiness"
            );
            thread::sleep(Duration::from_millis(20));
        }
        assert!(result.marker.exists());
        result
    }
    fn identity(&self) -> ProcessIdentity {
        NativeBackend
            .snapshot()
            .unwrap()
            .processes
            .iter()
            .find(|p| p.identity.pid == self.child.id())
            .unwrap()
            .identity
    }
    fn wait_for_exit(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while self.child.try_wait().unwrap().is_none() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            self.child.try_wait().unwrap().is_some(),
            "fixture did not exit"
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        // Only terminate exact Child handles created by this test, including during assertion failure.
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.marker);
    }
}

#[test]
fn windows_close_and_force_respect_creation_identity_and_leave_other_instances_alive() {
    let mut managed = Fixture::start();
    let mut foreign = Fixture::start();
    let identity = managed.identity();
    NativeBackend
        .close(
            &[ProcessIdentity {
                created_at: identity.created_at + 1,
                ..identity
            }],
            true,
        )
        .unwrap();
    assert!(
        managed.child.try_wait().unwrap().is_none(),
        "a reused/mismatched PID must not be terminated"
    );
    NativeBackend.close(&[identity], false).unwrap();
    managed.wait_for_exit();
    assert!(
        foreign.child.try_wait().unwrap().is_none(),
        "another instance of the same executable must stay alive"
    );
    NativeBackend.close(&[foreign.identity()], true).unwrap();
    foreign.wait_for_exit();
    // Posting WM_CLOSE to an already exited identity is harmless.
    NativeBackend.close(&[identity], false).unwrap();
    assert_eq!(WM_CLOSE, 0x0010);
}
