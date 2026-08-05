window.addEventListener("DOMContentLoaded", () => {
  const runtimeStatus = document.querySelector<HTMLElement>("#runtime-status");

  if (runtimeStatus) {
    runtimeStatus.textContent = "Tauri frontend initialized successfully.";
  }
});
