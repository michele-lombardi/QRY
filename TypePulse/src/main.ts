import { invoke } from "@tauri-apps/api/core";
import type {
  MonitorStatus,
  MenuBarPreference,
  OverlayPreference,
  PermissionStatus,
  StartupPreference,
} from "./contracts";
import { byId, pulsePath } from "./ui";

const message = byId<HTMLElement>("runtime-status");
const permissionValue = byId<HTMLElement>("permission-value");
const accessibilityValue = byId<HTMLElement>("accessibility-value");
const monitorDetail = byId<HTMLElement>("monitor-detail");
const liveStatus = byId<HTMLElement>("live-status");
const previewWpm = byId<HTMLElement>("preview-wpm");
const startButton = byId<HTMLButtonElement>("start-monitor");
const stopButton = byId<HTMLButtonElement>("stop-monitor");
const autoStart = byId<HTMLInputElement>("auto-start");
const menuBarWpm = byId<HTMLInputElement>("menu-bar-wpm");
const overlayEnabled = byId<HTMLInputElement>("overlay-enabled");
const overlayPosition = byId<HTMLSelectElement>("overlay-position");
const overlaySize = byId<HTMLSelectElement>("overlay-size");
const overlayContent = byId<HTMLSelectElement>("overlay-content");
byId<SVGPathElement>("settings-wave").setAttribute("d", pulsePath);

const setMessage = (value: string, isError = false): void => {
  message.textContent = value;
  message.classList.toggle("error", isError);
};

const renderPermission = (element: HTMLElement, { status }: PermissionStatus): void => {
  element.textContent = status;
  element.dataset.state = status;
};

const renderMonitor = (status: MonitorStatus): void => {
  monitorDetail.textContent =
    status.state === "running"
      ? `${status.totalActivities.toLocaleString()} activities · ${(status.averageCallbackNs / 1_000).toFixed(2)} µs callback`
      : `Monitor is ${status.state}`;
  liveStatus.textContent = `${Math.round(status.displayedWpm)} WPM · ${status.animationBand}`;
  previewWpm.textContent = Math.round(status.displayedWpm || 82).toString();
  startButton.disabled = status.state === "running" || status.state === "starting";
  stopButton.disabled = status.state === "stopped";
  if (status.lastError) setMessage(status.lastError, true);
};

const renderStartup = (preference: StartupPreference): void => {
  autoStart.checked = preference.autoStartEnabled;
  autoStart.dataset.registered = String(preference.loginItemRegistered);
};

const renderMenuBar = (preference: MenuBarPreference): void => {
  menuBarWpm.checked = preference.wpmEnabled;
};

const renderOverlay = (preference: OverlayPreference): void => {
  overlayEnabled.checked = preference.enabled;
  overlayPosition.value = preference.position;
  overlaySize.value = preference.size;
  overlayContent.value = preference.content;
};

const refresh = async (): Promise<void> => {
  const [input, accessibility, monitor, startup, menuBar, overlay] = await Promise.all([
    invoke<PermissionStatus>("input_permission_status"),
    invoke<PermissionStatus>("accessibility_permission_status"),
    invoke<MonitorStatus>("monitor_status"),
    invoke<StartupPreference>("startup_preference"),
    invoke<MenuBarPreference>("menu_bar_preference"),
    invoke<OverlayPreference>("overlay_preference"),
  ]);
  renderPermission(permissionValue, input);
  renderPermission(accessibilityValue, accessibility);
  renderMonitor(monitor);
  renderStartup(startup);
  renderMenuBar(menuBar);
  renderOverlay(overlay);
};

const selectSection = (name: string): void => {
  document
    .querySelectorAll<HTMLElement>("[data-section]")
    .forEach((button) =>
      button.classList.toggle("active", button.dataset.section === name),
    );
  document
    .querySelectorAll<HTMLElement>("[data-panel]")
    .forEach((panel) => panel.classList.toggle("active", panel.dataset.panel === name));
  const activeButton = document.querySelector<HTMLElement>(`[data-section="${name}"]`);
  byId<HTMLElement>("section-title").textContent =
    activeButton?.textContent?.trim() ?? "Settings";
};

document.querySelectorAll<HTMLButtonElement>("[data-section]").forEach((button) => {
  button.addEventListener("click", () =>
    selectSection(button.dataset.section ?? "general"),
  );
});

const permissionAction = async (
  command: string,
  target: HTMLElement | null,
  success: string,
): Promise<void> => {
  try {
    const response = await invoke<PermissionStatus | void>(command);
    if (target && response) renderPermission(target, response);
    setMessage(success);
  } catch (error) {
    setMessage(String(error), true);
  }
};

byId<HTMLButtonElement>("check-permission").addEventListener(
  "click",
  () =>
    void permissionAction(
      "input_permission_status",
      permissionValue,
      "Input Monitoring refreshed.",
    ),
);
byId<HTMLButtonElement>("request-permission").addEventListener(
  "click",
  () =>
    void permissionAction(
      "request_input_permission",
      permissionValue,
      "Complete the macOS permission flow, then restart QRY if requested.",
    ),
);
byId<HTMLButtonElement>("open-input-settings").addEventListener(
  "click",
  () =>
    void permissionAction(
      "open_input_settings",
      null,
      "Opened Input Monitoring settings.",
    ),
);
byId<HTMLButtonElement>("check-accessibility").addEventListener(
  "click",
  () =>
    void permissionAction(
      "accessibility_permission_status",
      accessibilityValue,
      "Accessibility refreshed.",
    ),
);
byId<HTMLButtonElement>("request-accessibility").addEventListener(
  "click",
  () =>
    void permissionAction(
      "request_accessibility_permission",
      accessibilityValue,
      "Complete the macOS permission flow, then restart QRY if requested.",
    ),
);
byId<HTMLButtonElement>("open-accessibility-settings").addEventListener(
  "click",
  () =>
    void permissionAction(
      "open_accessibility_permission_settings",
      null,
      "Opened Accessibility settings.",
    ),
);

startButton.addEventListener("click", async () => {
  try {
    renderMonitor(await invoke<MonitorStatus>("start_input_monitoring"));
    setMessage("QRY is listening to rhythm only.");
  } catch (error) {
    setMessage(String(error), true);
  }
});
stopButton.addEventListener("click", async () => {
  try {
    renderMonitor(await invoke<MonitorStatus>("stop_input_monitoring"));
    setMessage("Monitoring paused.");
  } catch (error) {
    setMessage(String(error), true);
  }
});

autoStart.addEventListener("change", async () => {
  autoStart.disabled = true;
  try {
    renderStartup(
      await invoke<StartupPreference>("set_auto_start_enabled", {
        enabled: autoStart.checked,
      }),
    );
    setMessage(
      autoStart.checked ? "QRY will start at login." : "Automatic startup is off.",
    );
  } catch (error) {
    setMessage(String(error), true);
    await refresh();
  } finally {
    autoStart.disabled = false;
  }
});

menuBarWpm.addEventListener("change", async () => {
  menuBarWpm.disabled = true;
  try {
    renderMenuBar(
      await invoke<MenuBarPreference>("set_menu_bar_wpm_enabled", {
        enabled: menuBarWpm.checked,
      }),
    );
    setMessage(
      menuBarWpm.checked
        ? "Live WPM is visible in the menu bar."
        : "Live WPM stays in the panel and Pip only.",
    );
  } catch (error) {
    setMessage(String(error), true);
    await refresh();
  } finally {
    menuBarWpm.disabled = false;
  }
});

const saveOverlay = async (): Promise<void> => {
  const controls = [overlayEnabled, overlayPosition, overlaySize, overlayContent];
  controls.forEach((control) => (control.disabled = true));
  try {
    renderOverlay(
      await invoke<OverlayPreference>("set_overlay_preference", {
        preference: {
          enabled: overlayEnabled.checked,
          position: overlayPosition.value,
          size: overlaySize.value,
          content: overlayContent.value,
        },
      }),
    );
    setMessage("Appearance updated.");
  } catch (error) {
    setMessage(String(error), true);
    await refresh();
  } finally {
    controls.forEach((control) => (control.disabled = false));
  }
};

[overlayEnabled, overlayPosition, overlaySize, overlayContent].forEach((control) =>
  control.addEventListener("change", () => void saveOverlay()),
);
byId<HTMLButtonElement>("open-full-statistics").addEventListener(
  "click",
  () => void invoke("open_statistics_window"),
);
byId<HTMLButtonElement>("open-today-panel").addEventListener(
  "click",
  () => void invoke("open_today_window"),
);

window.addEventListener("DOMContentLoaded", () => {
  void refresh()
    .then(() => setMessage("Settings are up to date."))
    .catch((error) => setMessage(String(error), true));
  window.setInterval(() => void refresh().catch(() => undefined), 1_000);
});
