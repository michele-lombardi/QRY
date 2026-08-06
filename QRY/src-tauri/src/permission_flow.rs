//! Required-permission bootstrap and bounded first-run onboarding lifecycle.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{App, AppHandle, Manager, State};
use typepulse_platform_macos::{
    accessibility_permission_status, input_permission_status, request_input_permission,
    MonitorRunState, PermissionStatus,
};

use crate::app_state::DiagnosticState;

pub(crate) const GATE_LABEL: &str = "onboarding";
const WAIT_DURATION: Duration = Duration::from_secs(120);
const WATCH_INTERVAL: Duration = Duration::from_millis(500);

/// Testable high-level state of the permission-dependent application lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LifecycleState {
    Checking,
    PermissionRequired,
    Waiting,
    Ready,
    Restarting,
    Running,
    Exiting,
}

impl LifecycleState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Checking => "checking",
            Self::PermissionRequired => "permission-required",
            Self::Waiting => "waiting",
            Self::Ready => "ready",
            Self::Restarting => "restarting",
            Self::Running => "running",
            Self::Exiting => "exiting",
        }
    }
}

#[derive(Debug)]
struct LifecycleInner {
    state: LifecycleState,
    wait_deadline: Option<Instant>,
    onboarding_completed: bool,
}

/// Shared permission state used by the gate commands and revocation watchdog.
#[derive(Clone, Debug)]
pub(crate) struct PermissionFlowRuntime {
    inner: Arc<Mutex<LifecycleInner>>,
    shutdown: Arc<AtomicBool>,
    exit_scheduled: Arc<AtomicBool>,
}

impl PermissionFlowRuntime {
    fn new(permission: PermissionStatus, onboarding_completed: bool) -> Self {
        Self::new_at(permission, onboarding_completed, Instant::now())
    }

    fn new_at(permission: PermissionStatus, onboarding_completed: bool, now: Instant) -> Self {
        let mut inner = LifecycleInner {
            state: LifecycleState::Checking,
            wait_deadline: None,
            onboarding_completed,
        };
        inner.apply_permission(permission, onboarding_completed, now);
        Self {
            inner: Arc::new(Mutex::new(inner)),
            shutdown: Arc::new(AtomicBool::new(false)),
            exit_scheduled: Arc::new(AtomicBool::new(false)),
        }
    }

    fn state(&self) -> LifecycleState {
        self.inner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .state
    }

    pub(crate) fn permits_normal_start(&self) -> bool {
        self.state() == LifecycleState::Running
    }

    fn begin_request(&self, permission: PermissionStatus, now: Instant) {
        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        if permission == PermissionStatus::Granted {
            inner.state = LifecycleState::Ready;
            inner.wait_deadline = None;
        } else {
            inner.state = LifecycleState::Waiting;
            inner.wait_deadline = Some(now + WAIT_DURATION);
        }
    }

    fn observe(&self, permission: PermissionStatus, now: Instant) -> LifecycleState {
        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        inner.observe(permission, now);
        inner.state
    }

    fn mark_restarting(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        inner.state = LifecycleState::Restarting;
        inner.wait_deadline = None;
        inner.onboarding_completed = true;
        self.shutdown.store(true, Ordering::Release);
    }

    fn mark_exiting(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        inner.state = LifecycleState::Exiting;
        inner.wait_deadline = None;
        self.shutdown.store(true, Ordering::Release);
    }

    fn mark_revoked(&self) -> bool {
        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        if inner.state != LifecycleState::Running {
            return false;
        }
        inner.state = LifecycleState::PermissionRequired;
        inner.wait_deadline = None;
        true
    }

    fn seconds_remaining(&self, now: Instant) -> Option<u64> {
        self.inner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .wait_deadline
            .map(|deadline| deadline.saturating_duration_since(now).as_secs())
    }

    fn onboarding_completed(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .onboarding_completed
    }
}

impl LifecycleInner {
    fn apply_permission(
        &mut self,
        permission: PermissionStatus,
        onboarding_completed: bool,
        _now: Instant,
    ) {
        self.state = if permission == PermissionStatus::Granted && onboarding_completed {
            LifecycleState::Running
        } else {
            LifecycleState::PermissionRequired
        };
        self.wait_deadline = None;
    }

    fn observe(&mut self, permission: PermissionStatus, now: Instant) {
        if permission == PermissionStatus::Granted
            && matches!(
                self.state,
                LifecycleState::Checking
                    | LifecycleState::PermissionRequired
                    | LifecycleState::Waiting
                    | LifecycleState::Ready
            )
        {
            self.state = LifecycleState::Ready;
            self.wait_deadline = None;
            return;
        }

        if permission != PermissionStatus::Granted && self.state == LifecycleState::Ready {
            self.state = LifecycleState::PermissionRequired;
            self.wait_deadline = None;
            return;
        }

        if self.state == LifecycleState::Waiting
            && self.wait_deadline.is_some_and(|deadline| now >= deadline)
        {
            self.state = LifecycleState::Exiting;
            self.wait_deadline = None;
        }
    }
}

/// Aggregate-free state rendered by the onboarding webview.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PermissionFlowDto {
    state: &'static str,
    input_status: &'static str,
    accessibility_status: &'static str,
    seconds_remaining: Option<u64>,
    onboarding_completed: bool,
}

impl PermissionFlowDto {
    fn snapshot(
        runtime: &PermissionFlowRuntime,
        permission: PermissionStatus,
        now: Instant,
    ) -> Self {
        Self {
            state: runtime.state().as_str(),
            input_status: permission.as_str(),
            accessibility_status: accessibility_permission_status().as_str(),
            seconds_remaining: runtime.seconds_remaining(now),
            onboarding_completed: runtime.onboarding_completed(),
        }
    }
}

/// Creates and registers the bootstrap state before any permission-dependent surface.
pub(crate) fn configure(app: &mut App, onboarding_completed: bool) -> PermissionFlowRuntime {
    let runtime = PermissionFlowRuntime::new(input_permission_status(), onboarding_completed);
    app.manage(runtime.clone());
    runtime
}

/// Presents only the permission surface and removes access to normal QRY UI.
pub(crate) fn show_gate<R: tauri::Runtime>(app: &AppHandle<R>) {
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);

    crate::shell::enter_permission_gate(app);
    crate::overlay::enter_permission_gate(app);
    if let Some(window) = app.get_webview_window(GATE_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Focuses the appropriate surface when a second launch is redirected here.
pub(crate) fn focus_primary_surface(app: &AppHandle) {
    if let Some(runtime) = app.try_state::<PermissionFlowRuntime>() {
        if runtime.state() != LifecycleState::Running {
            show_gate(app);
            return;
        }
    }
    crate::shell::open_settings_window(app.clone());
}

/// Watches the already-running monitor for permission revocation.
pub(crate) fn start_revocation_watchdog(app: AppHandle, runtime: PermissionFlowRuntime) {
    std::thread::Builder::new()
        .name("qry-permission-watchdog".into())
        .spawn(move || {
            while !runtime.shutdown.load(Ordering::Acquire) {
                std::thread::sleep(WATCH_INTERVAL);
                if runtime.state() != LifecycleState::Running {
                    continue;
                }
                let monitor_revoked = app.try_state::<DiagnosticState>().is_some_and(|state| {
                    state.snapshot().run_state == MonitorRunState::PermissionRevoked
                });
                if (monitor_revoked || input_permission_status() != PermissionStatus::Granted)
                    && runtime.mark_revoked()
                {
                    if let Some(state) = app.try_state::<DiagnosticState>() {
                        let _ = state.stop();
                        if let Err(error) = crate::commands::preferences::reconcile_auto_start(
                            &app, &state, false,
                        ) {
                            state.record_runtime_error(format!(
                                "automatic login cleanup after permission revocation failed: {error}"
                            ));
                        }
                    }
                    show_gate(&app);
                }
            }
        })
        .expect("permission watchdog thread must start");
}

#[tauri::command]
pub(crate) fn permission_flow_status(
    app: AppHandle,
    runtime: State<'_, PermissionFlowRuntime>,
) -> PermissionFlowDto {
    let now = Instant::now();
    let permission = input_permission_status();
    let state = runtime.observe(permission, now);
    let dto = PermissionFlowDto::snapshot(&runtime, permission, now);
    if state == LifecycleState::Exiting {
        schedule_exit(&app, &runtime);
    }
    dto
}

#[tauri::command]
pub(crate) fn begin_permission_flow(
    runtime: State<'_, PermissionFlowRuntime>,
) -> PermissionFlowDto {
    let now = Instant::now();
    let permission = request_input_permission();
    runtime.begin_request(permission, now);
    PermissionFlowDto::snapshot(&runtime, permission, now)
}

#[tauri::command]
pub(crate) fn wait_for_input_permission(
    runtime: State<'_, PermissionFlowRuntime>,
) -> PermissionFlowDto {
    let now = Instant::now();
    let permission = input_permission_status();
    runtime.begin_request(permission, now);
    PermissionFlowDto::snapshot(&runtime, permission, now)
}

#[tauri::command]
pub(crate) fn complete_permission_flow(
    app: AppHandle,
    state: State<'_, DiagnosticState>,
    runtime: State<'_, PermissionFlowRuntime>,
    auto_start_enabled: bool,
) -> Result<PermissionFlowDto, String> {
    let permission = input_permission_status();
    if permission != PermissionStatus::Granted {
        runtime.observe(permission, Instant::now());
        return Err("Input Monitoring permission is still required".into());
    }

    crate::commands::preferences::complete_onboarding_auto_start(&app, &state, auto_start_enabled)?;
    runtime.mark_restarting();
    let dto = PermissionFlowDto::snapshot(&runtime, permission, Instant::now());
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));
        app.restart();
    });
    Ok(dto)
}

#[tauri::command]
pub(crate) fn exit_permission_flow(app: AppHandle, runtime: State<'_, PermissionFlowRuntime>) {
    runtime.mark_exiting();
    schedule_exit(&app, &runtime);
}

/// Exits from a native gate-window close without routing through the webview.
pub(crate) fn exit_from_gate(app: &AppHandle) {
    if let Some(runtime) = app.try_state::<PermissionFlowRuntime>() {
        schedule_exit(app, &runtime);
    } else {
        app.exit(0);
    }
}

fn schedule_exit(app: &AppHandle, runtime: &PermissionFlowRuntime) {
    if runtime.exit_scheduled.swap(true, Ordering::AcqRel) {
        return;
    }
    runtime.mark_exiting();
    if let Some(overlay) = app.try_state::<crate::overlay::OverlayRuntime>() {
        overlay.stop();
    }
    if let Some(state) = app.try_state::<DiagnosticState>() {
        let _ = state.stop();
    }
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        handle.exit(0);
    });
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use typepulse_platform_macos::PermissionStatus;

    use serde_json::Value;

    use super::{LifecycleState, PermissionFlowDto, PermissionFlowRuntime, WAIT_DURATION};

    #[test]
    fn completed_granted_boot_enters_running() {
        let runtime =
            PermissionFlowRuntime::new_at(PermissionStatus::Granted, true, Instant::now());
        assert_eq!(runtime.state(), LifecycleState::Running);
    }

    #[test]
    fn missing_permission_never_enters_running() {
        let now = Instant::now();
        let runtime = PermissionFlowRuntime::new_at(PermissionStatus::Denied, false, now);
        assert_eq!(runtime.state(), LifecycleState::PermissionRequired);
        runtime.begin_request(PermissionStatus::Denied, now);
        assert_eq!(runtime.state(), LifecycleState::Waiting);
        assert_eq!(
            runtime.observe(PermissionStatus::Denied, now + WAIT_DURATION),
            LifecycleState::Exiting
        );
    }

    #[test]
    fn grant_during_wait_becomes_ready_for_one_clean_restart() {
        let now = Instant::now();
        let runtime = PermissionFlowRuntime::new_at(PermissionStatus::Denied, false, now);
        runtime.begin_request(PermissionStatus::Denied, now);
        assert_eq!(
            runtime.observe(PermissionStatus::Granted, now + Duration::from_secs(1)),
            LifecycleState::Ready
        );
        runtime.mark_restarting();
        assert_eq!(runtime.state(), LifecycleState::Restarting);
    }

    #[test]
    fn runtime_revocation_returns_to_required_gate_once() {
        let runtime =
            PermissionFlowRuntime::new_at(PermissionStatus::Granted, true, Instant::now());
        assert!(runtime.mark_revoked());
        assert!(!runtime.mark_revoked());
        assert_eq!(runtime.state(), LifecycleState::PermissionRequired);
    }

    #[test]
    fn gate_dto_exposes_only_permission_lifecycle_state() {
        let runtime =
            PermissionFlowRuntime::new_at(PermissionStatus::Denied, false, Instant::now());
        let value = serde_json::to_value(PermissionFlowDto::snapshot(
            &runtime,
            PermissionStatus::Denied,
            Instant::now(),
        ))
        .unwrap();
        let Value::Object(object) = value else {
            panic!("permission flow must serialize as an object");
        };
        assert_eq!(object.len(), 5);
        for key in [
            "state",
            "inputStatus",
            "accessibilityStatus",
            "secondsRemaining",
            "onboardingCompleted",
        ] {
            assert!(object.contains_key(key));
        }
    }
}
