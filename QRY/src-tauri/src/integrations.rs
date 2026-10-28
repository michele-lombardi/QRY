//! Opt-in, aggregate-only local bridge for the capsule and Vorssaint.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{App, AppHandle, Manager, State};
use typepulse_core::AppPreferences;
use typepulse_platform_desktop::MonitorRunState;

use crate::{app_state::DiagnosticState, permission_flow::PermissionFlowRuntime};

const STATE_FILE: &str = "state-v1.json";

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct IntegrationPreference {
    capsule_enabled: bool,
    external_enabled: bool,
    notch_pip_enabled: bool,
}

impl From<AppPreferences> for IntegrationPreference {
    fn from(value: AppPreferences) -> Self {
        Self {
            capsule_enabled: value.capsule_enabled,
            external_enabled: value.integrations_enabled,
            notch_pip_enabled: value.notch_pip_enabled,
        }
    }
}

impl IntegrationPreference {
    fn publishes(self) -> bool {
        self.capsule_enabled || self.external_enabled
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IntegrationStatus {
    preference: IntegrationPreference,
    snapshot_path: String,
    vorssaint_installed: bool,
    capsule_installed: bool,
    last_error: Option<String>,
}

#[derive(Clone)]
pub(crate) struct IntegrationRuntime {
    preference: Arc<Mutex<IntegrationPreference>>,
    stopped: Arc<AtomicBool>,
    write_lock: Arc<Mutex<()>>,
    last_error: Arc<Mutex<Option<String>>>,
    path: PathBuf,
}

impl IntegrationRuntime {
    pub(crate) fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
        let _guard = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeSnapshot {
    schema_version: u8,
    generated_at_unix_ms: u64,
    instance_id: String,
    sequence: u64,
    status: &'static str,
    displayed_wpm: u64,
    animation_band: &'static str,
    celebration_sequence: u64,
    capsule_enabled: bool,
    external_enabled: bool,
    notch_pip_enabled: bool,
    presentation: Presentation,
    today: Today,
    #[serde(skip_serializing_if = "Option::is_none")]
    statistics: Option<Statistics>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Statistics {
    records: Records,
    days: Vec<Day>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Records {
    peak_wpm: f64,
    sustained_30_wpm: f64,
    sustained_60_wpm: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Day {
    date: String,
    estimated_words: f64,
    average_wpm: f64,
    peak_wpm: f64,
}

impl From<typepulse_core::DailySummary> for Day {
    fn from(value: typepulse_core::DailySummary) -> Self {
        Self {
            date: value.date.to_string(),
            estimated_words: value.estimated_word_count,
            average_wpm: value.average_wpm,
            peak_wpm: value.peak_wpm,
        }
    }
}

struct StatisticsCache {
    generated_at: u64,
    through: String,
    value: Statistics,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Presentation {
    visible: bool,
    behavior: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Today {
    date: String,
    estimated_words: f64,
    peak_wpm: f64,
    active_typing_seconds: f64,
}

pub(crate) fn configure(app: &mut App, preferences: AppPreferences) -> tauri::Result<()> {
    if preferences.capsule_enabled {
        launch_capsule();
    }
    let runtime = IntegrationRuntime {
        preference: Arc::new(Mutex::new(preferences.into())),
        stopped: Arc::new(AtomicBool::new(false)),
        write_lock: Arc::new(Mutex::new(())),
        last_error: Arc::new(Mutex::new(None)),
        path: app
            .path()
            .app_cache_dir()?
            .join("integrations")
            .join(STATE_FILE),
    };
    if atomic_write_value(
        &runtime.path.with_file_name("preferences-v1.json"),
        &IntegrationPreference::from(preferences),
        unix_ms(),
    )
    .is_err()
    {
        *runtime.last_error.lock().unwrap_or_else(|e| e.into_inner()) =
            Some("Cannot save capsule visibility".into());
    }
    app.manage(runtime.clone());
    let handle = app.handle().clone();
    std::thread::Builder::new()
        .name("qry-local-bridge".into())
        .spawn(move || run(handle, runtime))
        .map_err(tauri::Error::Io)?;
    Ok(())
}

fn run(app: AppHandle, runtime: IntegrationRuntime) {
    let instance_id = format!("{}-{}", std::process::id(), unix_ms());
    let mut sequence = 0_u64;
    let mut statistics = None;
    while !runtime.stopped.load(Ordering::Acquire) {
        {
            let _guard = runtime.write_lock.lock().unwrap_or_else(|e| e.into_inner());
            if runtime.stopped.load(Ordering::Acquire) {
                break;
            }
            let preference = *runtime.preference.lock().unwrap_or_else(|e| e.into_inner());
            let result = if preference.publishes() {
                sequence = sequence.saturating_add(1);
                make_snapshot(&app, preference, &instance_id, sequence, &mut statistics).and_then(
                    |snapshot| {
                        atomic_write(&runtime.path, &snapshot)
                            .map_err(|_| "Cannot publish local status".to_string())
                    },
                )
            } else {
                match fs::remove_file(&runtime.path) {
                    Ok(()) => Ok(()),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(_) => Err("Cannot remove local status".to_string()),
                }
            };
            *runtime.last_error.lock().unwrap_or_else(|e| e.into_inner()) = result.err();
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn make_snapshot(
    app: &AppHandle,
    preference: IntegrationPreference,
    instance: &str,
    sequence: u64,
    cache: &mut Option<StatisticsCache>,
) -> Result<BridgeSnapshot, String> {
    let state = app.state::<DiagnosticState>();
    let status = state.snapshot();
    let ready = app.state::<PermissionFlowRuntime>().permits_normal_start();
    let running = ready && status.run_state == MonitorRunState::Running;
    let (live, today) = state.integration_metrics()?;
    let now = unix_ms();
    let statistics = if preference.external_enabled {
        let through = today.date.to_string();
        if cache.as_ref().is_none_or(|cached| {
            cached.through != through || now.saturating_sub(cached.generated_at) >= 10_000
        }) {
            let (days, records) = state.integration_history()?;
            *cache = Some(StatisticsCache {
                generated_at: now,
                through,
                value: Statistics {
                    records: Records {
                        peak_wpm: records.peak_wpm.unwrap_or(0.0),
                        sustained_30_wpm: records.sustained_30_wpm.unwrap_or(0.0),
                        sustained_60_wpm: records.sustained_60_wpm.unwrap_or(0.0),
                    },
                    days: days.into_iter().map(Day::from).collect(),
                },
            });
        }
        let mut statistics = cache
            .as_ref()
            .expect("statistics cache initialized")
            .value
            .clone();
        if let Some(last) = statistics.days.last_mut() {
            *last = today.into();
        }
        statistics.records.peak_wpm = statistics.records.peak_wpm.max(live.personal_best_wpm);
        statistics.records.sustained_30_wpm = statistics
            .records
            .sustained_30_wpm
            .max(live.sustained_30_best_wpm);
        statistics.records.sustained_60_wpm = statistics
            .records
            .sustained_60_wpm
            .max(live.sustained_60_best_wpm);
        Some(statistics)
    } else {
        *cache = None;
        None
    };
    let preferences = state.load_preferences()?;
    let quiet_ms = now.saturating_sub(status.last_activity_unix_ms.max(0) as u64);
    let visible = presentation_visible(
        running,
        live.current_session_characters,
        quiet_ms,
        preferences.overlay_hide_delay_seconds,
    );
    Ok(BridgeSnapshot {
        schema_version: 1,
        generated_at_unix_ms: now,
        instance_id: instance.to_string(),
        sequence,
        status: if !ready {
            "permission_required"
        } else if running {
            "monitoring"
        } else {
            "paused"
        },
        displayed_wpm: if running {
            live.displayed_wpm.round().clamp(0.0, 999.0) as u64
        } else {
            0
        },
        animation_band: if running {
            live.animation_band.as_str()
        } else {
            "still"
        },
        celebration_sequence: live.celebration_sequence,
        capsule_enabled: preference.capsule_enabled,
        external_enabled: preference.external_enabled,
        notch_pip_enabled: preference.notch_pip_enabled && preference.external_enabled,
        presentation: Presentation {
            visible,
            behavior: if !visible || quiet_ms >= 350 {
                "breathe"
            } else {
                crate::overlay::pip_behavior(live.displayed_wpm, live.active_typing_seconds)
            },
        },
        today: Today {
            date: today.date.to_string(),
            estimated_words: today.estimated_word_count,
            peak_wpm: today.peak_wpm,
            active_typing_seconds: today.active_typing_duration.as_secs_f64(),
        },
        statistics,
    })
}

fn presentation_visible(running: bool, characters: u64, quiet_ms: u64, delay_seconds: u32) -> bool {
    running && characters >= 3 && quiet_ms < u64::from(delay_seconds) * 1_000
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

fn atomic_write(path: &Path, snapshot: &BridgeSnapshot) -> Result<(), std::io::Error> {
    atomic_write_value(path, snapshot, snapshot.sequence)
}

fn atomic_write_value<T: Serialize>(
    path: &Path,
    value: &T,
    sequence: u64,
) -> Result<(), std::io::Error> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("missing status directory"))?;
    fs::create_dir_all(parent)?;
    if fs::symlink_metadata(parent)?.file_type().is_symlink() {
        return Err(std::io::Error::other(
            "status directory must not be a symlink",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
    }
    let temp = parent.join(format!(".state-{}-{}.tmp", std::process::id(), sequence));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        let data = serde_json::to_vec(value).map_err(std::io::Error::other)?;
        file.write_all(&data)?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[tauri::command]
pub(crate) fn integration_status(
    app: AppHandle,
    runtime: State<'_, IntegrationRuntime>,
) -> IntegrationStatus {
    let home = app.path().home_dir().unwrap_or_default();
    IntegrationStatus {
        preference: *runtime.preference.lock().unwrap_or_else(|e| e.into_inner()),
        snapshot_path: runtime.path.to_string_lossy().into_owned(),
        vorssaint_installed: Path::new("/Applications/Vorssaint.app").is_dir()
            || home.join("Applications/Vorssaint.app").is_dir(),
        capsule_installed: capsule_executable().is_some_and(|path| path.is_file()),
        last_error: runtime
            .last_error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone(),
    }
}

#[tauri::command]
pub(crate) fn set_integration_preference(
    preference: IntegrationPreference,
    app: AppHandle,
    state: State<'_, DiagnosticState>,
    runtime: State<'_, IntegrationRuntime>,
) -> Result<IntegrationStatus, String> {
    let mut preferences = state.load_preferences()?;
    preferences.capsule_enabled = preference.capsule_enabled;
    preferences.integrations_enabled = preference.external_enabled;
    preferences.notch_pip_enabled = preference.notch_pip_enabled;
    state.save_preferences(preferences)?;
    let _guard = runtime
        .write_lock
        .lock()
        .map_err(|_| "Local integration state unavailable")?;
    atomic_write_value(
        &runtime.path.with_file_name("preferences-v1.json"),
        &preference,
        unix_ms(),
    )
    .map_err(|_| "Cannot save capsule visibility")?;
    *runtime
        .preference
        .lock()
        .map_err(|_| "Local integration state unavailable")? = preference;
    drop(_guard);
    if preference.capsule_enabled {
        launch_capsule();
    }
    Ok(integration_status(app, runtime))
}

fn capsule_executable() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    Some(
        executable
            .parent()?
            .parent()?
            .join("Helpers/QRY Capsule.app/Contents/MacOS/QRYCapsule"),
    )
}

fn launch_capsule() {
    #[cfg(target_os = "macos")]
    if let Some(executable) = capsule_executable().filter(|path| path.is_file()) {
        // The helper has its own single-instance guard and never requests input access.
        let _ = std::thread::Builder::new()
            .name("qry-capsule".into())
            .spawn(move || {
                let _ = std::process::Command::new(executable)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_requires_live_qualified_typing_and_respects_delay() {
        assert!(!presentation_visible(false, 10, 0, 5));
        assert!(!presentation_visible(true, 2, 0, 5));
        assert!(presentation_visible(true, 3, 4_999, 5));
        assert!(!presentation_visible(true, 3, 5_000, 5));
    }

    #[test]
    fn external_sharing_is_opt_in() {
        let preference = IntegrationPreference::from(AppPreferences::default());
        assert!(!preference.publishes());
        assert!(!preference.notch_pip_enabled);
    }

    #[test]
    fn bridge_dto_contains_only_bounded_aggregate_presentation() {
        let snapshot = BridgeSnapshot {
            schema_version: 1,
            generated_at_unix_ms: 1,
            instance_id: "test".into(),
            sequence: 1,
            status: "monitoring",
            displayed_wpm: 60,
            animation_band: "fast",
            celebration_sequence: 0,
            capsule_enabled: true,
            external_enabled: true,
            notch_pip_enabled: true,
            presentation: Presentation {
                visible: true,
                behavior: "walk",
            },
            today: Today {
                date: "2026-10-28".into(),
                estimated_words: 10.0,
                peak_wpm: 80.0,
                active_typing_seconds: 30.0,
            },
            statistics: Some(Statistics {
                records: Records {
                    peak_wpm: 90.0,
                    sustained_30_wpm: 70.0,
                    sustained_60_wpm: 60.0,
                },
                days: vec![Day {
                    date: "2026-10-28".into(),
                    estimated_words: 10.0,
                    average_wpm: 60.0,
                    peak_wpm: 80.0,
                }],
            }),
        };
        let value = serde_json::to_value(&snapshot).unwrap();
        let fields: std::collections::BTreeSet<_> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            fields,
            [
                "schemaVersion",
                "generatedAtUnixMs",
                "instanceId",
                "sequence",
                "status",
                "displayedWpm",
                "animationBand",
                "celebrationSequence",
                "capsuleEnabled",
                "externalEnabled",
                "notchPipEnabled",
                "presentation",
                "today",
                "statistics"
            ]
            .into_iter()
            .collect()
        );
        assert!(serde_json::to_vec(&snapshot).unwrap().len() < 1_024);
        let directory = std::env::temp_dir().join(format!(
            "qry-bridge-test-{}-{}",
            std::process::id(),
            unix_ms()
        ));
        let path = directory.join(STATE_FILE);
        atomic_write(&path, &snapshot).unwrap();
        atomic_write(&path, &snapshot).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&fs::read(&path).unwrap()).unwrap(),
            value
        );
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
