use super::*;
use crate::processes::{ProcessInfo, Spawned};
use std::cell::{Cell, RefCell};

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!("Sofsta-tests-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }
    fn mode(&self, name: &str) -> Mode {
        let path = self.0.join(format!("{}.exe", name));
        fs::write(&path, []).unwrap();
        Mode {
            id: name.into(),
            name: name.into(),
            icon_emoji: "📋".into(),
            color: "#3B82F6".into(),
            auto_start: false,
            apps: vec![AppEntry {
                id: name.into(),
                name: name.into(),
                path: path.to_string_lossy().into_owned(),
                args: String::new(),
                working_dir: None,
                enabled: true,
                delay_seconds: 0,
            }],
        }
    }
    fn store(&self) -> RuntimeStore {
        RuntimeStore::open_at(&self.0).unwrap()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        // This fixture owns only flat, uniquely named files; no recursive deletion.
        for entry in fs::read_dir(&self.0).unwrap() {
            let _ = fs::remove_file(entry.unwrap().path());
        }
        let _ = fs::remove_dir(&self.0);
    }
}

#[derive(Default)]
struct FakeBackend {
    processes: RefCell<Vec<ProcessInfo>>,
    events: RefCell<Vec<String>>,
    next: Cell<u32>,
    refuse_close: Cell<bool>,
    fail_spawn: RefCell<HashSet<String>>,
    unreadable: RefCell<HashSet<u32>>,
}
impl ProcessBackend for FakeBackend {
    fn snapshot(&self) -> Result<Snapshot, String> {
        Ok(Snapshot {
            processes: self.processes.borrow().clone(),
            unreadable_pids: self.unreadable.borrow().clone(),
            ..Snapshot::default()
        })
    }
    fn spawn(&self, app: &AppEntry) -> Result<Spawned, String> {
        self.events.borrow_mut().push(format!("start:{}", app.name));
        if self.fail_spawn.borrow().contains(&app.name) {
            return Err("simulated spawn failure".into());
        }
        let pid = self.next.get() + 10000;
        self.next.set(pid);
        let identity = ProcessIdentity {
            pid,
            created_at: u64::from(pid),
        };
        self.processes.borrow_mut().push(ProcessInfo {
            identity,
            parent_pid: 0,
            path: app.path.clone(),
        });
        Ok(Spawned {
            identity,
            _child: None,
            #[cfg(windows)]
            _shell_handle: None,
        })
    }
    fn close(&self, ids: &[ProcessIdentity], force: bool) -> Result<(), String> {
        self.events
            .borrow_mut()
            .push(if force { "force-close" } else { "close" }.into());
        if force || !self.refuse_close.get() {
            self.processes
                .borrow_mut()
                .retain(|p| !ids.contains(&p.identity));
        }
        Ok(())
    }
}

fn start(store: &mut RuntimeStore, mode: &Mode, backend: &FakeBackend) -> OperationReport {
    store
        .run(mode, false, false, false, Duration::ZERO, backend)
        .unwrap()
}
fn switch(
    store: &mut RuntimeStore,
    mode: &Mode,
    force: bool,
    backend: &FakeBackend,
) -> OperationReport {
    store
        .run(mode, true, force, false, Duration::ZERO, backend)
        .unwrap()
}

#[test]
fn closes_old_mode_before_starting_target_and_restores_tracking_after_restart() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let game = sandbox.mode("game");
    let backend = FakeBackend::default();
    let mut store = sandbox.store();
    assert_eq!(start(&mut store, &office, &backend).started, 1);
    drop(store);
    let mut reopened = sandbox.store();
    let report = switch(&mut reopened, &game, false, &backend);
    assert_eq!(
        backend.events.borrow().as_slice(),
        ["start:office", "close", "start:game"]
    );
    assert_eq!(report.closed, 1);
    assert_eq!(report.started, 1);
    assert_eq!(report.status.modes[0].id, "game");
    assert_eq!(report.status.managed_app_count, 1);
}

#[test]
fn blocked_close_never_launches_target_then_explicit_force_can_finish() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let game = sandbox.mode("game");
    let backend = FakeBackend::default();
    backend.refuse_close.set(true);
    let mut store = sandbox.store();
    start(&mut store, &office, &backend);
    let blocked = switch(&mut store, &game, false, &backend);
    assert_eq!(blocked.unclosed, ["office"]);
    assert_eq!(blocked.started, 0);
    assert_eq!(blocked.status.modes[0].id, "office");
    assert!(!backend
        .events
        .borrow()
        .iter()
        .any(|event| event == "start:game"));
    let forced = switch(&mut store, &game, true, &backend);
    assert!(forced.unclosed.is_empty());
    assert_eq!(forced.started, 1);
}

#[test]
fn preexisting_software_in_selected_mode_is_adopted_and_closed_on_switch() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let game = sandbox.mode("game");
    let backend = FakeBackend::default();
    let foreign = ProcessIdentity {
        pid: 1234,
        created_at: 42,
    };
    backend.processes.borrow_mut().push(ProcessInfo {
        identity: foreign,
        parent_pid: 0,
        path: office.apps[0].path.clone(),
    });
    let mut store = sandbox.store();
    let initial = start(&mut store, &office, &backend);
    assert_eq!(initial.already_running, ["office"]);
    assert_eq!(initial.status.managed_app_count, 1);
    switch(&mut store, &game, false, &backend);
    assert!(!backend
        .processes
        .borrow()
        .iter()
        .any(|p| p.identity == foreign));
    assert!(backend
        .events
        .borrow()
        .iter()
        .any(|event| event.contains("close")));
}

#[test]
fn reused_pid_is_pruned_without_closing_the_new_process() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let game = sandbox.mode("game");
    let backend = FakeBackend::default();
    let mut store = sandbox.store();
    start(&mut store, &office, &backend);
    let reused = {
        let mut processes = backend.processes.borrow_mut();
        processes[0].identity.created_at += 100;
        processes[0].path = "foreign.exe".into();
        processes[0].identity
    };
    switch(&mut store, &game, false, &backend);
    assert!(backend
        .processes
        .borrow()
        .iter()
        .any(|p| p.identity == reused));
    assert!(!backend
        .events
        .borrow()
        .iter()
        .any(|event| event.contains("close")));
}

#[test]
fn invalid_or_empty_target_is_rejected_before_closing_current_apps() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let mut game = sandbox.mode("game");
    let backend = FakeBackend::default();
    let mut store = sandbox.store();
    start(&mut store, &office, &backend);
    game.apps[0].path = sandbox.0.join("missing.exe").to_string_lossy().into_owned();
    assert!(store
        .run(&game, true, false, false, Duration::ZERO, &backend)
        .is_err());
    game.apps.clear();
    assert!(store
        .run(&game, true, false, false, Duration::ZERO, &backend)
        .is_err());
    assert_eq!(backend.events.borrow().as_slice(), ["start:office"]);
}

#[test]
fn repeats_are_idempotent_and_launch_failures_are_reported() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let game = sandbox.mode("game");
    let backend = FakeBackend::default();
    let mut store = sandbox.store();
    start(&mut store, &office, &backend);
    let repeated = start(&mut store, &office, &backend);
    assert_eq!(repeated.started, 0);
    assert_eq!(repeated.already_running, ["office"]);
    backend.fail_spawn.borrow_mut().insert("game".into());
    let failed = switch(&mut store, &game, false, &backend);
    assert_eq!(failed.closed, 1);
    assert_eq!(failed.started, 0);
    assert_eq!(failed.failures.len(), 1);
    assert_eq!(failed.failures[0].name, "game");
}

#[test]
fn verified_descendants_are_tracked_but_older_or_reused_parent_relationships_are_not() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let backend = FakeBackend::default();
    let mut store = sandbox.store();
    start(&mut store, &office, &backend);
    let parent = backend.processes.borrow()[0].identity;
    let child = ProcessIdentity {
        pid: parent.pid + 1,
        created_at: parent.created_at + 1,
    };
    let unrelated = ProcessIdentity {
        pid: parent.pid + 2,
        created_at: parent.created_at - 1,
    };
    let linked_browser = ProcessIdentity {
        pid: parent.pid + 3,
        created_at: parent.created_at + 2,
    };
    backend.processes.borrow_mut().extend([
        ProcessInfo {
            identity: child,
            parent_pid: parent.pid,
            path: office.apps[0].path.clone(),
        },
        ProcessInfo {
            identity: unrelated,
            parent_pid: parent.pid,
            path: office.apps[0].path.clone(),
        },
        ProcessInfo {
            identity: linked_browser,
            parent_pid: parent.pid,
            path: "browser-opened-from-an-office-link.exe".into(),
        },
    ]);
    store.refresh(&backend.snapshot().unwrap(), &[]);
    assert!(store.identities().contains(&child));
    assert!(!store.identities().contains(&unrelated));
    assert!(!store.identities().contains(&linked_browser));
    backend
        .processes
        .borrow_mut()
        .retain(|p| p.identity != parent);
    store.refresh(&backend.snapshot().unwrap(), &[]);
    assert_eq!(store.identities(), [child]);
}

#[test]
fn inaccessible_owned_processes_block_switch_instead_of_being_assumed_exited() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let game = sandbox.mode("game");
    let backend = FakeBackend::default();
    let mut store = sandbox.store();
    start(&mut store, &office, &backend);
    let pid = backend.processes.borrow()[0].identity.pid;
    backend.processes.borrow_mut().clear();
    backend.unreadable.borrow_mut().insert(pid);
    let report = switch(&mut store, &game, false, &backend);
    assert_eq!(report.unclosed, ["office"]);
    assert_eq!(report.started, 0);
}

#[cfg(windows)]
#[test]
fn concurrent_instances_cannot_mutate_the_runtime_file_together() {
    let sandbox = Sandbox::new();
    let first = sandbox.store();
    assert!(RuntimeStore::open_at(&sandbox.0).is_err());
    drop(first);
    assert!(RuntimeStore::open_at(&sandbox.0).is_ok());
}

#[test]
fn malformed_runtime_record_is_not_replaced_with_an_empty_session() {
    let sandbox = Sandbox::new();
    fs::write(sandbox.0.join("runtime.json"), b"{broken").unwrap();
    assert!(RuntimeStore::open_at(&sandbox.0).is_err());
    assert_eq!(
        fs::read_to_string(sandbox.0.join("runtime.json")).unwrap(),
        "{broken"
    );
}

#[test]
fn synchronizing_a_legacy_mode_recovers_preexisting_apps_without_matching_other_paths() {
    let sandbox = Sandbox::new();
    let office = sandbox.mode("office");
    let game = sandbox.mode("game");
    let backend = FakeBackend::default();
    let matching = ProcessIdentity {
        pid: 7777,
        created_at: 10,
    };
    let unrelated = ProcessIdentity {
        pid: 7778,
        created_at: 11,
    };
    backend.processes.borrow_mut().extend([
        ProcessInfo {
            identity: matching,
            parent_pid: 0,
            path: office.apps[0].path.clone(),
        },
        ProcessInfo {
            identity: unrelated,
            parent_pid: 0,
            path: sandbox
                .0
                .join("other-folder")
                .join("office.exe")
                .to_string_lossy()
                .into_owned(),
        },
    ]);
    let mut store = sandbox.store();
    store.state.modes.push(RunningMode {
        id: office.id.clone(),
        name: office.name.clone(),
    });
    store
        .sync_from_snapshot(&[office, game.clone()], &backend.snapshot().unwrap())
        .unwrap();
    assert_eq!(store.identities(), [matching]);
    let report = switch(&mut store, &game, false, &backend);
    assert_eq!(report.closed, 1);
    assert!(backend
        .processes
        .borrow()
        .iter()
        .any(|p| p.identity == unrelated));
}
