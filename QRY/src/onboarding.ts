import { invoke } from "@tauri-apps/api/core";

import { byId } from "./ui";

type PermissionValue = "granted" | "denied" | "unknown";

interface PermissionFlowStatus {
  state:
    | "checking"
    | "permission-required"
    | "waiting"
    | "ready"
    | "restarting"
    | "running"
    | "exiting";
  platform: "macos" | "windows" | "unsupported";
  inputStatus: PermissionValue;
  accessibilityStatus: PermissionValue;
  inputPermissionRequired: boolean;
  inputSettingsAvailable: boolean;
  accessibilityPermissionRequired: boolean;
  accessibilitySettingsAvailable: boolean;
  restartRequired: boolean;
  secondsRemaining: number | null;
  onboardingCompleted: boolean;
}

interface PermissionStatus {
  status: PermissionValue;
}

const inputStatus = byId<HTMLElement>("input-status");
const accessibilityStatus = byId<HTMLElement>("accessibility-status");
const inputMessage = byId<HTMLElement>("input-message");
const accessibilityMessage = byId<HTMLElement>("accessibility-message");
const requestInput = byId<HTMLButtonElement>("request-input");
const openInputSettings = byId<HTMLButtonElement>("open-input-settings");
const inputContinue = byId<HTMLButtonElement>("input-continue");
const requestAccessibility = byId<HTMLButtonElement>("request-accessibility");
const openAccessibilitySettings = byId<HTMLButtonElement>(
  "open-accessibility-settings",
);
const finishSetup = byId<HTMLButtonElement>("finish-setup");
const onboardingAutoStart = byId<HTMLInputElement>("onboarding-auto-start");

let currentStep = 1;
let latestStatus: PermissionFlowStatus | null = null;
let busy = false;

const setStep = (step: number): void => {
  currentStep = Math.min(3, Math.max(1, step));
  document.querySelectorAll<HTMLElement>("[data-step]").forEach((panel) => {
    panel.classList.toggle("active", Number(panel.dataset.step) === currentStep);
  });
  document.querySelectorAll<HTMLElement>("[data-dot]").forEach((dot) => {
    const dotStep = Number(dot.dataset.dot);
    dot.classList.toggle("active", dotStep === currentStep);
    dot.classList.toggle("complete", dotStep < currentStep);
  });
};

const renderPermission = (
  target: HTMLElement,
  value: PermissionValue,
  permissionRequired = true,
): void => {
  target.dataset.status = value;
  target.textContent =
    value === "granted"
      ? permissionRequired
        ? "Allowed"
        : "Ready"
      : value === "denied"
        ? "Not allowed"
        : "Unavailable";
};

const render = (status: PermissionFlowStatus): void => {
  latestStatus = status;
  const windows = status.platform === "windows";
  document.documentElement.dataset.platform = status.platform;
  renderPermission(inputStatus, status.inputStatus, status.inputPermissionRequired);
  renderPermission(
    accessibilityStatus,
    status.accessibilityStatus,
    status.accessibilityPermissionRequired,
  );

  byId<HTMLElement>("local-storage-title").textContent = windows
    ? "Stays on your PC"
    : "Stays on your Mac";
  byId<HTMLElement>("input-eyebrow").textContent = status.inputPermissionRequired
    ? "Required permission"
    : "System check";
  byId<HTMLElement>("input-title").textContent = status.inputPermissionRequired
    ? "Allow Input Monitoring"
    : "Global typing access is ready";
  byId<HTMLElement>("input-description").textContent = status.inputPermissionRequired
    ? "macOS requires this permission before QRY can observe global typing activity. QRY immediately discards key identity and keeps only anonymous rhythm."
    : "Windows does not require a separate input permission. QRY uses a passive native listener and immediately discards key identity.";
  byId<HTMLElement>("input-name").textContent = status.inputPermissionRequired
    ? "Input Monitoring"
    : "Global typing access";
  byId<HTMLElement>("input-detail").textContent = status.inputPermissionRequired
    ? "Required for live WPM"
    : "No Windows permission prompt required";

  byId<HTMLElement>("accessibility-eyebrow").textContent =
    status.accessibilityPermissionRequired
      ? "Optional permission"
      : "Display placement";
  byId<HTMLElement>("accessibility-description").textContent =
    status.accessibilityPermissionRequired
      ? "Accessibility lets Pip appear on the display containing your focused window. QRY never reads the app name, window title or content. Without it, Pip uses your main display."
      : "Windows allows QRY to place Pip on the display containing your foreground window without an extra permission. QRY never reads the app name, window title or content.";
  byId<HTMLElement>("accessibility-name").textContent =
    status.accessibilityPermissionRequired
      ? "Accessibility"
      : "Focused-display placement";
  byId<HTMLElement>("accessibility-detail").textContent =
    status.accessibilityPermissionRequired
      ? "Optional focused-display placement"
      : "Available without additional access";

  const inputGranted = status.inputStatus === "granted";
  const terminal = status.state === "restarting" || status.state === "exiting";
  requestInput.classList.toggle(
    "hidden",
    inputGranted || !status.inputPermissionRequired,
  );
  openInputSettings.classList.toggle("hidden", !status.inputSettingsAvailable);
  inputContinue.classList.toggle("hidden", !inputGranted);
  inputContinue.disabled = !inputGranted || terminal;
  finishSetup.disabled = !inputGranted || terminal;
  if (terminal) {
    document.querySelectorAll<HTMLButtonElement>("button").forEach((button) => {
      button.disabled = true;
    });
  }

  if (status.onboardingCompleted && !inputGranted && currentStep === 1) {
    setStep(2);
  }

  if (!status.inputPermissionRequired && inputGranted) {
    inputMessage.textContent =
      "Windows is ready. Continue to focused-display placement and startup options.";
  } else if (status.state === "waiting") {
    const remaining = status.secondsRemaining ?? 0;
    inputMessage.textContent = `Allow QRY in System Settings. This window closes in ${remaining}s if access is not granted.`;
  } else if (inputGranted) {
    inputMessage.textContent =
      "Input Monitoring is allowed. Continue to the optional step.";
  } else if (status.state === "exiting") {
    inputMessage.textContent = "Permission was not granted. QRY is closing.";
  } else {
    inputMessage.textContent = "QRY cannot run until Input Monitoring is allowed.";
  }

  requestAccessibility.classList.toggle(
    "hidden",
    !status.accessibilityPermissionRequired,
  );
  openAccessibilitySettings.classList.toggle(
    "hidden",
    !status.accessibilitySettingsAvailable,
  );
  accessibilityMessage.textContent = !status.accessibilityPermissionRequired
    ? "Focused-display placement is ready without a Windows permission prompt."
    : status.accessibilityStatus === "granted"
      ? "Accessibility is allowed. Pip can follow the focused display."
      : "Optional — Pip will use your main display if you skip this.";
  finishSetup.textContent = status.restartRequired
    ? "Finish and restart QRY"
    : "Finish setup";
  byId<HTMLElement>("restart-note").textContent = status.restartRequired
    ? "QRY restarts once to activate the new permission."
    : "QRY starts monitoring immediately after setup.";
};

const refresh = async (): Promise<void> => {
  render(await invoke<PermissionFlowStatus>("permission_flow_status"));
};

const runBusy = async (action: () => Promise<void>): Promise<void> => {
  if (busy) return;
  busy = true;
  document.querySelectorAll<HTMLButtonElement>("button").forEach((button) => {
    button.disabled = true;
  });
  try {
    await action();
  } catch (error) {
    const message = String(error);
    if (currentStep === 2) inputMessage.textContent = message;
    if (currentStep === 3) accessibilityMessage.textContent = message;
  } finally {
    busy = false;
    document.querySelectorAll<HTMLButtonElement>("button").forEach((button) => {
      button.disabled = false;
    });
    if (latestStatus) render(latestStatus);
  }
};

byId<HTMLButtonElement>("privacy-continue").addEventListener("click", () => {
  setStep(2);
  void refresh();
});

requestInput.addEventListener("click", () => {
  void runBusy(async () => {
    const status = await invoke<PermissionFlowStatus>("begin_permission_flow");
    render(status);
    if (status.inputStatus !== "granted") {
      await invoke("open_input_settings");
    }
  });
});

openInputSettings.addEventListener("click", () => {
  void runBusy(async () => {
    render(await invoke<PermissionFlowStatus>("wait_for_input_permission"));
    await invoke("open_input_settings");
    await refresh();
  });
});

inputContinue.addEventListener("click", () => setStep(3));

requestAccessibility.addEventListener("click", () => {
  void runBusy(async () => {
    const status = await invoke<PermissionStatus>("request_accessibility_permission");
    renderPermission(accessibilityStatus, status.status);
    await refresh();
  });
});

openAccessibilitySettings.addEventListener("click", () => {
  void runBusy(async () => {
    await invoke("open_accessibility_permission_settings");
    await refresh();
  });
});

finishSetup.addEventListener("click", () => {
  void runBusy(async () => {
    accessibilityMessage.textContent = latestStatus?.restartRequired
      ? "Restarting QRY…"
      : "Starting QRY…";
    render(
      await invoke<PermissionFlowStatus>("complete_permission_flow", {
        autoStartEnabled: onboardingAutoStart.checked,
      }),
    );
  });
});

document.querySelectorAll<HTMLButtonElement>(".exit-button").forEach((button) => {
  button.addEventListener("click", () => void invoke("exit_permission_flow"));
});

window.addEventListener("focus", () => void refresh());
window.addEventListener("DOMContentLoaded", () => {
  setStep(1);
  void refresh();
  window.setInterval(() => void refresh().catch(() => undefined), 750);
});
