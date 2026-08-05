//! Shared diagnostic state for the Phase B monitor spike.

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

use typepulse_platform_macos::{
    ActivityReceiver, KeyboardMonitor, MonitorConfig, MonitorError, MonitorMetricsSnapshot,
    MonitorRunState,
};

/// Process-wide owner of the optional native monitor.
#[derive(Default)]
pub(crate) struct DiagnosticState {
    active: Mutex<Option<ActiveMonitor>>,
    total_activities: Arc<AtomicU64>,
}

impl DiagnosticState {
    pub(crate) fn start(&self) -> Result<(), String> {
        let mut active = self.active.lock().map_err(lock_error)?;
        if active.is_some() {
            return Err("input monitoring is already active".into());
        }

        self.total_activities.store(0, Ordering::Relaxed);
        let (mut monitor, receiver) =
            KeyboardMonitor::start(MonitorConfig::default()).map_err(|error| error.to_string())?;
        let relay_total = Arc::clone(&self.total_activities);

        let relay_result = std::thread::Builder::new()
            .name("typepulse-diagnostic-relay".into())
            .spawn(move || relay_activity(receiver, relay_total));
        let relay = match relay_result {
            Ok(relay) => relay,
            Err(error) => {
                let _ = monitor.stop();
                return Err(format!("failed to start diagnostic relay: {error}"));
            }
        };

        *active = Some(ActiveMonitor {
            monitor,
            relay: Some(relay),
        });
        Ok(())
    }

    pub(crate) fn stop(&self) -> Result<(), String> {
        let active = self.active.lock().map_err(lock_error)?.take();
        if let Some(mut active) = active {
            active.stop().map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> (MonitorRunState, MonitorMetricsSnapshot, u64) {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (run_state, metrics) = active.as_ref().map_or(
            (MonitorRunState::Stopped, MonitorMetricsSnapshot::default()),
            |active| (active.monitor.state(), active.monitor.metrics()),
        );
        (
            run_state,
            metrics,
            self.total_activities.load(Ordering::Relaxed),
        )
    }
}

struct ActiveMonitor {
    monitor: KeyboardMonitor,
    relay: Option<JoinHandle<()>>,
}

impl ActiveMonitor {
    fn stop(&mut self) -> Result<(), MonitorError> {
        self.monitor.stop()?;
        if let Some(relay) = self.relay.take() {
            relay.join().map_err(|_| MonitorError::ThreadPanicked)?;
        }
        Ok(())
    }
}

fn relay_activity(receiver: ActivityReceiver, total: Arc<AtomicU64>) {
    for _activity in receiver {
        total.fetch_add(1, Ordering::Relaxed);
    }
}

fn lock_error<T>(_error: std::sync::PoisonError<T>) -> String {
    "diagnostic monitor state is unavailable".into()
}
