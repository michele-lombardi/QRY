import { invoke } from "@tauri-apps/api/core";

type PermissionStatus = { status: "granted" | "denied" | "unknown" };

type MonitorStatus = {
  state: string;
  totalActivities: number;
  eventsSeen: number;
  activitiesEmitted: number;
  activitiesDropped: number;
  callbackCount: number;
  averageCallbackNs: number;
  maxCallbackNs: number;
  reenableAttempts: number;
  sessionPhase: string;
  rawWpm: number;
  displayedWpm: number;
  animationBand: string;
  lastError: string | null;
};

type StartupPreference = {
  autoStartEnabled: boolean;
  loginItemRegistered: boolean;
};

type OverlayPreference = {
  enabled: boolean;
  position: "top-left" | "top-right" | "bottom-left" | "bottom-right";
  size: "small" | "medium" | "large";
  content: "wpm" | "animation" | "both";
};

type DailySummary = {
  date: string;
  estimatedCharacterCount: number;
  estimatedWordCount: number;
  averageWpm: number;
  peakWpm: number;
  activeTypingSeconds: number;
  sessionCount: number;
};

const byId = <T extends HTMLElement>(id: string): T => {
  const element = document.querySelector<T>(`#${id}`);
  if (!element) throw new Error(`Missing diagnostic element: ${id}`);
  return element;
};

const permissionValue = byId<HTMLElement>("permission-value");
const monitorValue = byId<HTMLElement>("monitor-value");
const activityValue = byId<HTMLElement>("activity-value");
const callbackValue = byId<HTMLElement>("callback-value");
const droppedValue = byId<HTMLElement>("dropped-value");
const wpmValue = byId<HTMLElement>("wpm-value");
const todayDate = byId<HTMLElement>("today-date");
const todayWords = byId<HTMLElement>("today-words");
const todayWpm = byId<HTMLElement>("today-wpm");
const todayActive = byId<HTMLElement>("today-active");
const todaySessions = byId<HTMLElement>("today-sessions");
const autoStart = byId<HTMLInputElement>("auto-start");
const overlayEnabled = byId<HTMLInputElement>("overlay-enabled");
const overlayPosition = byId<HTMLSelectElement>("overlay-position");
const overlaySize = byId<HTMLSelectElement>("overlay-size");
const overlayContent = byId<HTMLSelectElement>("overlay-content");
const message = byId<HTMLElement>("runtime-status");
const startButton = byId<HTMLButtonElement>("start-monitor");
const stopButton = byId<HTMLButtonElement>("stop-monitor");

const setMessage = (value: string, isError = false): void => {
  message.textContent = value;
  message.classList.toggle("error", isError);
};

const renderPermission = ({ status }: PermissionStatus): void => {
  permissionValue.textContent = status;
  permissionValue.dataset.state = status;
};

const renderMonitor = (status: MonitorStatus): void => {
  monitorValue.textContent = status.state;
  monitorValue.dataset.state = status.state;
  activityValue.textContent = status.totalActivities.toLocaleString();
  callbackValue.textContent = `${(status.averageCallbackNs / 1_000).toFixed(2)} µs avg · ${(status.maxCallbackNs / 1_000).toFixed(2)} µs max`;
  droppedValue.textContent = `${status.activitiesDropped.toLocaleString()} dropped · ${status.reenableAttempts.toLocaleString()} re-enable`;
  wpmValue.textContent = `${status.displayedWpm.toFixed(1)} · ${status.animationBand}`;
  startButton.disabled = status.state === "running" || status.state === "starting";
  stopButton.disabled = status.state === "stopped";
  if (status.lastError) setMessage(status.lastError, true);
};

const renderStartup = (preference: StartupPreference): void => {
  autoStart.checked = preference.autoStartEnabled;
  autoStart.dataset.registered = String(preference.loginItemRegistered);
};

const renderOverlayPreference = (preference: OverlayPreference): void => {
  overlayEnabled.checked = preference.enabled;
  overlayPosition.value = preference.position;
  overlaySize.value = preference.size;
  overlayContent.value = preference.content;
};

const renderToday = (summary: DailySummary): void => {
  todayDate.textContent = summary.date;
  todayWords.textContent = summary.estimatedWordCount.toFixed(2);
  todayWpm.textContent = `${summary.averageWpm.toFixed(1)} / ${summary.peakWpm.toFixed(1)} WPM`;
  const minutes = Math.floor(summary.activeTypingSeconds / 60);
  const seconds = Math.floor(summary.activeTypingSeconds % 60);
  todayActive.textContent = `${minutes}m ${seconds}s`;
  todaySessions.textContent = summary.sessionCount.toLocaleString();
};

const call = async <T>(command: string): Promise<T> => invoke<T>(command);

const refresh = async (): Promise<void> => {
  const [permission, monitor, startup, overlay, today] = await Promise.all([
    call<PermissionStatus>("input_permission_status"),
    call<MonitorStatus>("monitor_status"),
    call<StartupPreference>("startup_preference"),
    call<OverlayPreference>("overlay_preference"),
    call<DailySummary>("today_summary"),
  ]);
  renderPermission(permission);
  renderMonitor(monitor);
  renderStartup(startup);
  renderOverlayPreference(overlay);
  renderToday(today);
};

byId<HTMLButtonElement>("check-permission").addEventListener("click", async () => {
  try {
    renderPermission(await call<PermissionStatus>("input_permission_status"));
    setMessage("Permission status refreshed.");
  } catch (error) {
    setMessage(String(error), true);
  }
});

byId<HTMLButtonElement>("request-permission").addEventListener("click", async () => {
  try {
    renderPermission(await call<PermissionStatus>("request_input_permission"));
    setMessage(
      "Permission request completed. Restart the debug app if macOS asks for it.",
    );
  } catch (error) {
    setMessage(String(error), true);
  }
});

byId<HTMLButtonElement>("open-settings").addEventListener("click", async () => {
  try {
    await call<void>("open_input_settings");
    setMessage("Opened Privacy & Security → Input Monitoring.");
  } catch (error) {
    setMessage(String(error), true);
  }
});

startButton.addEventListener("click", async () => {
  try {
    renderMonitor(await call<MonitorStatus>("start_input_monitoring"));
    setMessage("Passive monitoring started. Type in another application to test it.");
  } catch (error) {
    setMessage(String(error), true);
    await refresh();
  }
});

stopButton.addEventListener("click", async () => {
  try {
    renderMonitor(await call<MonitorStatus>("stop_input_monitoring"));
    setMessage("Monitoring stopped.");
  } catch (error) {
    setMessage(String(error), true);
  }
});

autoStart.addEventListener("change", async () => {
  autoStart.disabled = true;
  try {
    const preference = await invoke<StartupPreference>("set_auto_start_enabled", {
      enabled: autoStart.checked,
    });
    renderStartup(preference);
    setMessage(
      preference.autoStartEnabled
        ? "Automatic login launch and monitoring enabled."
        : "Automatic startup disabled.",
    );
  } catch (error) {
    setMessage(String(error), true);
    await refresh();
  } finally {
    autoStart.disabled = false;
  }
});

const saveOverlayPreference = async (): Promise<void> => {
  const controls = [overlayEnabled, overlayPosition, overlaySize, overlayContent];
  controls.forEach((control) => (control.disabled = true));
  try {
    const preference = await invoke<OverlayPreference>("set_overlay_preference", {
      preference: {
        enabled: overlayEnabled.checked,
        position: overlayPosition.value,
        size: overlaySize.value,
        content: overlayContent.value,
      },
    });
    renderOverlayPreference(preference);
    setMessage("Overlay preferences saved and applied live.");
  } catch (error) {
    setMessage(String(error), true);
    await refresh();
  } finally {
    controls.forEach((control) => (control.disabled = false));
  }
};

[overlayEnabled, overlayPosition, overlaySize, overlayContent].forEach((control) => {
  control.addEventListener("change", () => void saveOverlayPreference());
});

window.addEventListener("DOMContentLoaded", async () => {
  try {
    await refresh();
    setMessage(
      "Local metrics initialized. Daily totals roll over automatically at the local date change.",
    );
    window.setInterval(
      () =>
        void refresh().catch((error) => {
          setMessage(`Refresh failed: ${String(error)}`, true);
        }),
      1_000,
    );
  } catch (error) {
    setMessage(`Tauri diagnostics unavailable: ${String(error)}`, true);
  }
});
