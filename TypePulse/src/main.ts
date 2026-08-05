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
  startButton.disabled = status.state === "running" || status.state === "starting";
  stopButton.disabled = status.state === "stopped";
};

const call = async <T>(command: string): Promise<T> => invoke<T>(command);

const refresh = async (): Promise<void> => {
  const [permission, monitor] = await Promise.all([
    call<PermissionStatus>("input_permission_status"),
    call<MonitorStatus>("monitor_status"),
  ]);
  renderPermission(permission);
  renderMonitor(monitor);
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

window.addEventListener("DOMContentLoaded", async () => {
  try {
    await refresh();
    setMessage(
      "Phase B diagnostics initialized. The interface receives one aggregate snapshot per second.",
    );
    window.setInterval(() => void refresh(), 1_000);
  } catch (error) {
    setMessage(`Tauri diagnostics unavailable: ${String(error)}`, true);
  }
});
