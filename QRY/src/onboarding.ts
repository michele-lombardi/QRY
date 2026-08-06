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
  inputStatus: PermissionValue;
  accessibilityStatus: PermissionValue;
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
const inputContinue = byId<HTMLButtonElement>("input-continue");
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

const renderPermission = (target: HTMLElement, value: PermissionValue): void => {
  target.dataset.status = value;
  target.textContent =
    value === "granted"
      ? "Allowed"
      : value === "denied"
        ? "Not allowed"
        : "Unavailable";
};

const render = (status: PermissionFlowStatus): void => {
  latestStatus = status;
  renderPermission(inputStatus, status.inputStatus);
  renderPermission(accessibilityStatus, status.accessibilityStatus);

  const inputGranted = status.inputStatus === "granted";
  const terminal = status.state === "restarting" || status.state === "exiting";
  requestInput.classList.toggle("hidden", inputGranted);
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

  if (status.state === "waiting") {
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

  accessibilityMessage.textContent =
    status.accessibilityStatus === "granted"
      ? "Accessibility is allowed. Pip can follow the focused display."
      : "Optional — Pip will use your main display if you skip this.";
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

byId<HTMLButtonElement>("open-input-settings").addEventListener("click", () => {
  void runBusy(async () => {
    render(await invoke<PermissionFlowStatus>("wait_for_input_permission"));
    await invoke("open_input_settings");
    await refresh();
  });
});

inputContinue.addEventListener("click", () => setStep(3));

byId<HTMLButtonElement>("request-accessibility").addEventListener("click", () => {
  void runBusy(async () => {
    const status = await invoke<PermissionStatus>("request_accessibility_permission");
    renderPermission(accessibilityStatus, status.status);
    await refresh();
  });
});

byId<HTMLButtonElement>("open-accessibility-settings").addEventListener("click", () => {
  void runBusy(async () => {
    await invoke("open_accessibility_permission_settings");
    await refresh();
  });
});

finishSetup.addEventListener("click", () => {
  void runBusy(async () => {
    accessibilityMessage.textContent = "Restarting QRY…";
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
