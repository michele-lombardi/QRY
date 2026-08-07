//! Lifecycle and aggregate diagnostics for the passive desktop input monitor.

use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        mpsc::Receiver,
        Arc,
    },
    thread::JoinHandle,
    time::Duration,
};

use typepulse_core::TypingActivity;

/// Bounded receiver for privacy-safe typing activity signals.
pub type ActivityReceiver = Receiver<TypingActivity>;

/// Runtime configuration for the event-tap worker.
#[derive(Clone, Copy, Debug)]
pub struct MonitorConfig {
    /// Maximum number of unconsumed activities buffered in memory.
    pub channel_capacity: usize,
    /// Maximum interval between stop and re-enable checks.
    pub run_loop_slice: Duration,
    /// Interval between permission-revocation checks.
    pub permission_poll_interval: Duration,
    /// Maximum time allowed for event-tap startup.
    pub startup_timeout: Duration,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 1_024,
            run_loop_slice: Duration::from_millis(25),
            permission_poll_interval: Duration::from_secs(1),
            startup_timeout: Duration::from_secs(3),
        }
    }
}

/// Observable lifecycle state of the input monitor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonitorRunState {
    /// The worker thread is installing its event tap.
    Starting,
    /// The passive event tap is processing events.
    Running,
    /// Monitoring was stopped normally.
    Stopped,
    /// Input Monitoring access disappeared while the worker was running.
    PermissionRevoked,
    /// The worker encountered an unrecoverable error.
    Failed,
    /// The current operating system is not supported by this adapter.
    Unsupported,
}

impl MonitorRunState {
    /// Stable string used by diagnostic DTOs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::PermissionRevoked => "permission-revoked",
            Self::Failed => "failed",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Snapshot of low-level health and callback-cost counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MonitorMetricsSnapshot {
    /// Keyboard events observed by the callback.
    pub events_seen: u64,
    /// Privacy-safe activity signals placed on the bounded channel.
    pub activities_emitted: u64,
    /// Signals dropped because the consumer was temporarily behind.
    pub activities_dropped: u64,
    /// Total callback invocations, including event-tap notifications.
    pub callback_count: u64,
    /// Mean time spent inside the event callback, in nanoseconds.
    pub average_callback_ns: u64,
    /// Longest callback observed, in nanoseconds.
    pub max_callback_ns: u64,
    /// Number of event-tap re-enable attempts.
    pub reenable_attempts: u64,
}

/// Failure while creating or managing the native monitor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MonitorError {
    /// Input Monitoring access has not been granted.
    PermissionDenied,
    /// Core Graphics refused to create the passive event tap.
    TapCreationFailed,
    /// Core Foundation refused to create the run-loop source.
    RunLoopSourceFailed,
    /// The operating system refused to create the private input window.
    NativeWindowFailed,
    /// The operating system refused the passive input registration.
    NativeRegistrationFailed,
    /// The native input message loop failed unexpectedly.
    MessageLoopFailed,
    /// The worker did not finish startup within the configured timeout.
    StartupTimedOut,
    /// The operating-system thread could not be created.
    ThreadSpawn(String),
    /// The worker thread panicked during shutdown.
    ThreadPanicked,
    /// This crate was built for a platform without a desktop adapter.
    UnsupportedPlatform,
}

impl fmt::Display for MonitorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => write!(formatter, "Input Monitoring permission is required"),
            Self::TapCreationFailed => write!(formatter, "macOS refused to create the event tap"),
            Self::RunLoopSourceFailed => {
                write!(formatter, "failed to create the event-tap run-loop source")
            }
            Self::NativeWindowFailed => {
                write!(formatter, "failed to create the native input window")
            }
            Self::NativeRegistrationFailed => {
                write!(formatter, "failed to register passive keyboard input")
            }
            Self::MessageLoopFailed => write!(formatter, "native input message loop failed"),
            Self::StartupTimedOut => write!(formatter, "input monitor startup timed out"),
            Self::ThreadSpawn(error) => write!(formatter, "failed to start input worker: {error}"),
            Self::ThreadPanicked => write!(formatter, "input worker panicked"),
            Self::UnsupportedPlatform => write!(formatter, "input monitoring is unsupported"),
        }
    }
}

impl std::error::Error for MonitorError {}

#[derive(Default)]
struct MonitorMetrics {
    events_seen: AtomicU64,
    activities_emitted: AtomicU64,
    activities_dropped: AtomicU64,
    callback_count: AtomicU64,
    callback_total_ns: AtomicU64,
    callback_max_ns: AtomicU64,
    reenable_attempts: AtomicU64,
}

impl MonitorMetrics {
    fn snapshot(&self) -> MonitorMetricsSnapshot {
        let callback_count = self.callback_count.load(Ordering::Relaxed);
        let callback_total_ns = self.callback_total_ns.load(Ordering::Relaxed);

        MonitorMetricsSnapshot {
            events_seen: self.events_seen.load(Ordering::Relaxed),
            activities_emitted: self.activities_emitted.load(Ordering::Relaxed),
            activities_dropped: self.activities_dropped.load(Ordering::Relaxed),
            callback_count,
            average_callback_ns: callback_total_ns.checked_div(callback_count).unwrap_or(0),
            max_callback_ns: self.callback_max_ns.load(Ordering::Relaxed),
            reenable_attempts: self.reenable_attempts.load(Ordering::Relaxed),
        }
    }
}

struct SharedMonitor {
    state: AtomicU8,
    stop_requested: AtomicBool,
    #[cfg(windows)]
    worker_thread_id: std::sync::atomic::AtomicU32,
    metrics: MonitorMetrics,
}

impl SharedMonitor {
    fn new() -> Self {
        Self {
            state: AtomicU8::new(state_to_u8(MonitorRunState::Starting)),
            stop_requested: AtomicBool::new(false),
            #[cfg(windows)]
            worker_thread_id: std::sync::atomic::AtomicU32::new(0),
            metrics: MonitorMetrics::default(),
        }
    }

    fn set_state(&self, state: MonitorRunState) {
        self.state.store(state_to_u8(state), Ordering::Release);
    }

    fn state(&self) -> MonitorRunState {
        state_from_u8(self.state.load(Ordering::Acquire))
    }
}

const fn state_to_u8(state: MonitorRunState) -> u8 {
    match state {
        MonitorRunState::Starting => 0,
        MonitorRunState::Running => 1,
        MonitorRunState::Stopped => 2,
        MonitorRunState::PermissionRevoked => 3,
        MonitorRunState::Failed => 4,
        MonitorRunState::Unsupported => 5,
    }
}

const fn state_from_u8(value: u8) -> MonitorRunState {
    match value {
        0 => MonitorRunState::Starting,
        1 => MonitorRunState::Running,
        2 => MonitorRunState::Stopped,
        3 => MonitorRunState::PermissionRevoked,
        4 => MonitorRunState::Failed,
        _ => MonitorRunState::Unsupported,
    }
}

/// Owned handle for one native keyboard monitor worker.
pub struct KeyboardMonitor {
    shared: Arc<SharedMonitor>,
    worker: Option<JoinHandle<()>>,
}

impl KeyboardMonitor {
    /// Starts the passive event tap and returns its bounded activity receiver.
    pub fn start(config: MonitorConfig) -> Result<(Self, ActivityReceiver), MonitorError> {
        platform::start(config)
    }

    /// Returns the current worker state.
    #[must_use]
    pub fn state(&self) -> MonitorRunState {
        self.shared.state()
    }

    /// Returns a consistent-enough, lock-free diagnostics snapshot.
    #[must_use]
    pub fn metrics(&self) -> MonitorMetricsSnapshot {
        self.shared.metrics.snapshot()
    }

    /// Requests shutdown and joins the worker thread.
    pub fn stop(&mut self) -> Result<(), MonitorError> {
        self.shared.stop_requested.store(true, Ordering::Release);
        platform::request_stop(&self.shared);
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| MonitorError::ThreadPanicked)?;
        }
        if matches!(
            self.state(),
            MonitorRunState::Starting | MonitorRunState::Running
        ) {
            self.shared.set_state(MonitorRunState::Stopped);
        }
        Ok(())
    }
}

impl Drop for KeyboardMonitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc::{self, TrySendError},
            Arc,
        },
        thread,
        time::{Duration, Instant},
    };

    use core_foundation::runloop::{kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFRunLoop};
    use core_graphics::event::{
        CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
        CGEventType, CallbackResult, EventField,
    };

    use super::{
        super::{
            event_filter::{counts_as_typing, RepetitionGuard},
            input_permission_status, PermissionStatus,
        },
        KeyboardMonitor, MonitorConfig, MonitorError, MonitorRunState, SharedMonitor,
        TypingActivity,
    };

    pub(super) fn start(
        config: MonitorConfig,
    ) -> Result<(KeyboardMonitor, mpsc::Receiver<TypingActivity>), MonitorError> {
        if input_permission_status() != PermissionStatus::Granted {
            return Err(MonitorError::PermissionDenied);
        }

        let capacity = config.channel_capacity.max(1);
        let (activity_sender, activity_receiver) = mpsc::sync_channel(capacity);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let shared = Arc::new(SharedMonitor::new());
        let worker_shared = Arc::clone(&shared);

        let worker = thread::Builder::new()
            .name("typepulse-input-monitor".into())
            .spawn(move || {
                if let Err(error) =
                    run_worker(config, activity_sender, &worker_shared, &ready_sender)
                {
                    worker_shared.set_state(MonitorRunState::Failed);
                    let _ = ready_sender.try_send(Err(error));
                }
            })
            .map_err(|error| MonitorError::ThreadSpawn(error.to_string()))?;

        match ready_receiver.recv_timeout(config.startup_timeout) {
            Ok(Ok(())) => Ok((
                KeyboardMonitor {
                    shared,
                    worker: Some(worker),
                },
                activity_receiver,
            )),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => {
                shared.stop_requested.store(true, Ordering::Release);
                request_stop(&shared);
                let _ = worker.join();
                Err(MonitorError::StartupTimedOut)
            }
        }
    }

    pub(super) const fn request_stop(_shared: &SharedMonitor) {}

    fn run_worker(
        config: MonitorConfig,
        activity_sender: mpsc::SyncSender<TypingActivity>,
        shared: &Arc<SharedMonitor>,
        ready_sender: &mpsc::SyncSender<Result<(), MonitorError>>,
    ) -> Result<(), MonitorError> {
        let reenable_requested = Arc::new(AtomicBool::new(false));
        let callback_reenable = Arc::clone(&reenable_requested);
        let callback_shared = Arc::clone(shared);
        let repetition_guard = RepetitionGuard::new();
        let repetition_origin = Instant::now();

        let event_tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown],
            move |_proxy, event_type, event| {
                let started = Instant::now();
                callback_shared
                    .metrics
                    .callback_count
                    .fetch_add(1, Ordering::Relaxed);

                match event_type {
                    CGEventType::KeyDown => {
                        let key_code = event
                            .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                            as u16;
                        let is_auto_repeat = event
                            .get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT)
                            != 0;
                        let occurred_at = Instant::now();
                        let elapsed_ms = occurred_at
                            .saturating_duration_since(repetition_origin)
                            .as_millis()
                            .min(u128::from(u64::MAX))
                            as u64;
                        process_key_event(
                            KeyEventMetadata {
                                key_code,
                                flags: event.get_flags(),
                                is_auto_repeat,
                                occurred_at,
                                elapsed_ms,
                            },
                            &repetition_guard,
                            &activity_sender,
                            &callback_shared,
                        );
                    }
                    CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                        callback_reenable.store(true, Ordering::Release);
                    }
                    _ => {}
                }

                record_callback_duration(&callback_shared, started.elapsed());
                CallbackResult::Keep
            },
        )
        .map_err(|()| MonitorError::TapCreationFailed)?;

        let run_loop_source = event_tap
            .mach_port()
            .create_runloop_source(0)
            .map_err(|()| MonitorError::RunLoopSourceFailed)?;
        CFRunLoop::get_current().add_source(&run_loop_source, unsafe { kCFRunLoopCommonModes });
        event_tap.enable();
        shared.set_state(MonitorRunState::Running);
        let _ = ready_sender.try_send(Ok(()));

        let run_loop_slice = config.run_loop_slice.max(Duration::from_millis(1));
        let permission_poll = config
            .permission_poll_interval
            .max(Duration::from_millis(100));
        let mut last_permission_poll = Instant::now();

        while !shared.stop_requested.load(Ordering::Acquire) {
            CFRunLoop::run_in_mode(unsafe { kCFRunLoopDefaultMode }, run_loop_slice, false);

            if reenable_requested.swap(false, Ordering::AcqRel) {
                shared
                    .metrics
                    .reenable_attempts
                    .fetch_add(1, Ordering::Relaxed);
                event_tap.enable();
            }

            if last_permission_poll.elapsed() >= permission_poll {
                if input_permission_status() != PermissionStatus::Granted {
                    shared.set_state(MonitorRunState::PermissionRevoked);
                    return Ok(());
                }
                last_permission_poll = Instant::now();
            }
        }

        shared.set_state(MonitorRunState::Stopped);
        Ok(())
    }

    fn record_callback_duration(shared: &SharedMonitor, duration: Duration) {
        let elapsed_ns = duration.as_nanos().min(u128::from(u64::MAX)) as u64;
        shared
            .metrics
            .callback_total_ns
            .fetch_add(elapsed_ns, Ordering::Relaxed);
        shared
            .metrics
            .callback_max_ns
            .fetch_max(elapsed_ns, Ordering::Relaxed);
    }

    struct KeyEventMetadata {
        key_code: u16,
        flags: CGEventFlags,
        is_auto_repeat: bool,
        occurred_at: Instant,
        elapsed_ms: u64,
    }

    fn process_key_event(
        event: KeyEventMetadata,
        repetition_guard: &RepetitionGuard,
        activity_sender: &mpsc::SyncSender<TypingActivity>,
        shared: &SharedMonitor,
    ) {
        shared.metrics.events_seen.fetch_add(1, Ordering::Relaxed);
        if !counts_as_typing(event.key_code, event.flags, event.is_auto_repeat)
            || !repetition_guard.accepts(event.key_code, event.elapsed_ms)
        {
            return;
        }

        match activity_sender.try_send(TypingActivity::at(event.occurred_at)) {
            Ok(()) => {
                shared
                    .metrics
                    .activities_emitted
                    .fetch_add(1, Ordering::Relaxed);
            }
            Err(TrySendError::Full(_)) => {
                shared
                    .metrics
                    .activities_dropped
                    .fetch_add(1, Ordering::Relaxed);
            }
            Err(TrySendError::Disconnected(_)) => {
                shared.stop_requested.store(true, Ordering::Release);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::{sync::Arc, time::Instant};

        use core_graphics::event::{CGEventFlags, KeyCode};

        use crate::event_filter::RepetitionGuard;

        use super::{mpsc, process_key_event, KeyEventMetadata, SharedMonitor};

        #[test]
        fn callback_filter_drops_auto_repeat_and_third_identical_key() {
            let shared = Arc::new(SharedMonitor::new());
            let (sender, receiver) = mpsc::sync_channel(8);
            let guard = RepetitionGuard::new();
            let now = Instant::now();
            for (elapsed_ms, is_auto_repeat) in
                [(0, false), (100, false), (200, false), (300, true)]
            {
                process_key_event(
                    KeyEventMetadata {
                        key_code: KeyCode::ANSI_A,
                        flags: CGEventFlags::empty(),
                        is_auto_repeat,
                        occurred_at: now,
                        elapsed_ms,
                    },
                    &guard,
                    &sender,
                    &shared,
                );
            }
            assert_eq!(receiver.try_iter().count(), 2);
        }

        #[test]
        #[ignore = "manual release-mode performance reference"]
        fn typing_callback_hot_path_reference() {
            const SAMPLES: usize = 250_000;
            let shared = Arc::new(SharedMonitor::new());
            let (sender, _receiver) = mpsc::sync_channel(SAMPLES);
            let started = Instant::now();
            let guard = RepetitionGuard::new();

            for index in 0..SAMPLES {
                let key_code = if index % 2 == 0 {
                    KeyCode::ANSI_A
                } else {
                    KeyCode::ANSI_S
                };
                process_key_event(
                    KeyEventMetadata {
                        key_code,
                        flags: CGEventFlags::empty(),
                        is_auto_repeat: false,
                        occurred_at: started,
                        elapsed_ms: index as u64,
                    },
                    &guard,
                    &sender,
                    &shared,
                );
            }

            let average_ns = started.elapsed().as_nanos() / SAMPLES as u128;
            eprintln!("typing callback hot-path reference: {average_ns} ns/activity");
            assert_eq!(
                shared
                    .metrics
                    .activities_emitted
                    .load(std::sync::atomic::Ordering::Relaxed),
                SAMPLES as u64
            );
        }
    }
}

#[cfg(windows)]
mod platform {
    use std::{
        mem::{size_of, MaybeUninit},
        ptr,
        sync::{
            atomic::Ordering,
            mpsc::{self, TrySendError},
            Arc,
        },
        thread,
        time::{Duration, Instant},
    };

    use typepulse_core::TypingActivity;
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::{
            Input::{
                GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE,
                RAWINPUTHEADER, RIDEV_INPUTSINK, RIDEV_REMOVE, RID_INPUT, RIM_TYPEKEYBOARD,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
                PostThreadMessageW, RegisterClassW, TranslateMessage, HWND_MESSAGE, MSG, WM_APP,
                WM_INPUT, WNDCLASSW,
            },
        },
    };

    use crate::event_filter_windows::WindowsEventFilter;

    use super::{KeyboardMonitor, MonitorConfig, MonitorError, MonitorRunState, SharedMonitor};

    const RAW_INPUT_USAGE_PAGE_GENERIC_DESKTOP: u16 = 0x01;
    const RAW_INPUT_USAGE_KEYBOARD: u16 = 0x06;
    const STOP_MESSAGE: u32 = WM_APP + 0x51;
    const WINDOW_CLASS_NAME: [u16; 18] = [
        81, 82, 89, 82, 97, 119, 73, 110, 112, 117, 116, 87, 105, 110, 100, 111, 119, 0,
    ];

    pub(super) fn start(
        config: MonitorConfig,
    ) -> Result<(KeyboardMonitor, mpsc::Receiver<TypingActivity>), MonitorError> {
        let capacity = config.channel_capacity.max(1);
        let (activity_sender, activity_receiver) = mpsc::sync_channel(capacity);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let shared = Arc::new(SharedMonitor::new());
        let worker_shared = Arc::clone(&shared);

        let worker = thread::Builder::new()
            .name("typepulse-input-monitor".into())
            .spawn(move || {
                if let Err(error) = run_worker(activity_sender, &worker_shared, &ready_sender) {
                    worker_shared.set_state(MonitorRunState::Failed);
                    let _ = ready_sender.try_send(Err(error));
                }
                worker_shared.worker_thread_id.store(0, Ordering::Release);
            })
            .map_err(|error| MonitorError::ThreadSpawn(error.to_string()))?;

        match ready_receiver.recv_timeout(config.startup_timeout) {
            Ok(Ok(())) => Ok((
                KeyboardMonitor {
                    shared,
                    worker: Some(worker),
                },
                activity_receiver,
            )),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => {
                shared.stop_requested.store(true, Ordering::Release);
                request_stop(&shared);
                let _ = worker.join();
                Err(MonitorError::StartupTimedOut)
            }
        }
    }

    pub(super) fn request_stop(shared: &SharedMonitor) {
        let thread_id = shared.worker_thread_id.load(Ordering::Acquire);
        if thread_id != 0 {
            // SAFETY: the identifier belongs to the live worker. The message
            // contains no pointers and is consumed only as a shutdown signal.
            let _ = unsafe { PostThreadMessageW(thread_id, STOP_MESSAGE, 0, 0) };
        }
    }

    fn run_worker(
        activity_sender: mpsc::SyncSender<TypingActivity>,
        shared: &Arc<SharedMonitor>,
        ready_sender: &mpsc::SyncSender<Result<(), MonitorError>>,
    ) -> Result<(), MonitorError> {
        // SAFETY: called on the worker and used only to post its private stop message.
        let thread_id = unsafe { GetCurrentThreadId() };
        shared.worker_thread_id.store(thread_id, Ordering::Release);

        let window = create_message_window()?;
        if let Err(error) = register_keyboard(window) {
            // SAFETY: `window` was created on this worker and has not been destroyed.
            let _ = unsafe { DestroyWindow(window) };
            return Err(error);
        }

        shared.set_state(MonitorRunState::Running);
        let _ = ready_sender.try_send(Ok(()));

        let origin = Instant::now();
        let mut filter = WindowsEventFilter::default();
        let result = run_message_loop(window, origin, &mut filter, &activity_sender, shared);

        unregister_keyboard();
        // SAFETY: `window` remains owned by this worker until this call.
        let _ = unsafe { DestroyWindow(window) };
        if result.is_ok() {
            shared.set_state(MonitorRunState::Stopped);
        }
        result
    }

    fn create_message_window() -> Result<HWND, MonitorError> {
        // SAFETY: null requests the module containing the current executable.
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        if instance.is_null() {
            return Err(MonitorError::NativeWindowFailed);
        }

        let window_class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: WINDOW_CLASS_NAME.as_ptr(),
            ..Default::default()
        };
        // SAFETY: the class definition and static UTF-16 name remain valid.
        let atom = unsafe { RegisterClassW(&window_class) };
        if atom == 0 {
            // SAFETY: reads the calling thread's last-error value.
            let error = unsafe { GetLastError() };
            if error != ERROR_CLASS_ALREADY_EXISTS {
                return Err(MonitorError::NativeWindowFailed);
            }
        }

        // SAFETY: creates a private message-only window with no visual surface
        // and no user data. All handles belong to this worker thread.
        let window = unsafe {
            CreateWindowExW(
                0,
                WINDOW_CLASS_NAME.as_ptr(),
                ptr::null(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                ptr::null_mut(),
                instance,
                ptr::null(),
            )
        };
        (!window.is_null())
            .then_some(window)
            .ok_or(MonitorError::NativeWindowFailed)
    }

    fn register_keyboard(window: HWND) -> Result<(), MonitorError> {
        let device = RAWINPUTDEVICE {
            usUsagePage: RAW_INPUT_USAGE_PAGE_GENERIC_DESKTOP,
            usUsage: RAW_INPUT_USAGE_KEYBOARD,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: window,
        };
        // SAFETY: the array contains one initialized registration descriptor.
        let registered =
            unsafe { RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32) };
        (registered != 0)
            .then_some(())
            .ok_or(MonitorError::NativeRegistrationFailed)
    }

    fn unregister_keyboard() {
        let device = RAWINPUTDEVICE {
            usUsagePage: RAW_INPUT_USAGE_PAGE_GENERIC_DESKTOP,
            usUsage: RAW_INPUT_USAGE_KEYBOARD,
            dwFlags: RIDEV_REMOVE,
            hwndTarget: ptr::null_mut(),
        };
        // SAFETY: Windows requires a null target when removing this process's registration.
        let _ = unsafe { RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32) };
    }

    fn run_message_loop(
        window: HWND,
        origin: Instant,
        filter: &mut WindowsEventFilter,
        activity_sender: &mpsc::SyncSender<TypingActivity>,
        shared: &SharedMonitor,
    ) -> Result<(), MonitorError> {
        let mut message = MSG::default();
        loop {
            // SAFETY: `message` is a valid out pointer and this worker owns the queue.
            let status = unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) };
            if status == -1 {
                return Err(MonitorError::MessageLoopFailed);
            }
            if status == 0 || message.message == STOP_MESSAGE {
                return Ok(());
            }

            if message.message == WM_INPUT {
                process_raw_input(
                    message.lParam as HRAWINPUT,
                    origin,
                    filter,
                    activity_sender,
                    shared,
                );
            }

            // SAFETY: the message came from this thread's queue. The private
            // window procedure delegates every message to DefWindowProcW.
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }

            if shared.stop_requested.load(Ordering::Acquire) {
                return Ok(());
            }
            if message.hwnd == window && message.message == STOP_MESSAGE {
                return Ok(());
            }
        }
    }

    fn process_raw_input(
        handle: HRAWINPUT,
        origin: Instant,
        filter: &mut WindowsEventFilter,
        activity_sender: &mpsc::SyncSender<TypingActivity>,
        shared: &SharedMonitor,
    ) {
        let started = Instant::now();
        shared
            .metrics
            .callback_count
            .fetch_add(1, Ordering::Relaxed);

        if let Some(raw) = read_raw_input(handle) {
            if raw.header.dwType == RIM_TYPEKEYBOARD {
                shared.metrics.events_seen.fetch_add(1, Ordering::Relaxed);
                // SAFETY: `dwType` identifies the initialized keyboard union member.
                let keyboard = unsafe { raw.data.keyboard };
                let elapsed_ms = started
                    .saturating_duration_since(origin)
                    .as_millis()
                    .min(u128::from(u64::MAX)) as u64;
                if filter.accepts(keyboard.MakeCode, keyboard.Flags, keyboard.VKey, elapsed_ms) {
                    match activity_sender.try_send(TypingActivity::at(started)) {
                        Ok(()) => {
                            shared
                                .metrics
                                .activities_emitted
                                .fetch_add(1, Ordering::Relaxed);
                        }
                        Err(TrySendError::Full(_)) => {
                            shared
                                .metrics
                                .activities_dropped
                                .fetch_add(1, Ordering::Relaxed);
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            shared.stop_requested.store(true, Ordering::Release);
                        }
                    }
                }
            }
        }

        record_callback_duration(shared, started.elapsed());
    }

    fn read_raw_input(handle: HRAWINPUT) -> Option<RAWINPUT> {
        let mut value = MaybeUninit::<RAWINPUT>::zeroed();
        let mut size = size_of::<RAWINPUT>() as u32;
        // SAFETY: `value` is aligned and large enough for one RAWINPUT value;
        // `size` and the header size describe that initialized destination.
        let read = unsafe {
            GetRawInputData(
                handle,
                RID_INPUT,
                value.as_mut_ptr().cast(),
                &mut size,
                size_of::<RAWINPUTHEADER>() as u32,
            )
        };
        if read == u32::MAX || read < size_of::<RAWINPUTHEADER>() as u32 {
            return None;
        }
        // SAFETY: a successful read initialized the RAWINPUT header and union.
        Some(unsafe { value.assume_init() })
    }

    fn record_callback_duration(shared: &SharedMonitor, duration: Duration) {
        let elapsed_ns = duration.as_nanos().min(u128::from(u64::MAX)) as u64;
        shared
            .metrics
            .callback_total_ns
            .fetch_add(elapsed_ns, Ordering::Relaxed);
        shared
            .metrics
            .callback_max_ns
            .fetch_max(elapsed_ns, Ordering::Relaxed);
    }

    unsafe extern "system" fn window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // SAFETY: this is the registered procedure for `window`; forwarding is
        // required because QRY does not consume or suppress native messages.
        unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    use std::sync::{mpsc, Arc};

    use super::{
        KeyboardMonitor, MonitorConfig, MonitorError, MonitorRunState, SharedMonitor,
        TypingActivity,
    };

    pub(super) fn start(
        _config: MonitorConfig,
    ) -> Result<(KeyboardMonitor, mpsc::Receiver<TypingActivity>), MonitorError> {
        let shared = Arc::new(SharedMonitor::new());
        shared.set_state(MonitorRunState::Unsupported);
        Err(MonitorError::UnsupportedPlatform)
    }

    pub(super) const fn request_stop(_shared: &SharedMonitor) {}
}

#[cfg(test)]
mod tests {
    use super::{state_from_u8, state_to_u8, MonitorMetrics, MonitorRunState};

    #[test]
    fn lifecycle_states_round_trip_through_the_atomic_representation() {
        for state in [
            MonitorRunState::Starting,
            MonitorRunState::Running,
            MonitorRunState::Stopped,
            MonitorRunState::PermissionRevoked,
            MonitorRunState::Failed,
            MonitorRunState::Unsupported,
        ] {
            assert_eq!(state_from_u8(state_to_u8(state)), state);
        }
    }

    #[test]
    fn empty_metrics_have_a_zero_average() {
        assert_eq!(MonitorMetrics::default().snapshot().average_callback_ns, 0);
    }
}
