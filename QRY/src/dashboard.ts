import { invoke } from "@tauri-apps/api/core";
import type { DailySummary, MonitorStatus } from "./contracts";
import { byId, formatClock, formatNumber, pulsePath } from "./ui";

const wpm = byId<HTMLElement>("dashboard-wpm");
const words = byId<HTMLElement>("dashboard-words");
const best = byId<HTMLElement>("dashboard-best");
const best30 = byId<HTMLElement>("dashboard-best-30");
const best60 = byId<HTMLElement>("dashboard-best-60");
const streak = byId<HTMLElement>("dashboard-streak");
const quiet = byId<HTMLElement>("dashboard-quiet");
const state = byId<HTMLElement>("dashboard-state");
const message = byId<HTMLElement>("dashboard-message");
byId<SVGPathElement>("dashboard-wave").setAttribute("d", pulsePath);

const countStreak = (days: DailySummary[], hasLiveWords: boolean): number => {
  let count = 0;
  for (let index = days.length - 1; index >= 0; index -= 1) {
    const day = days[index];
    const isToday = index === days.length - 1;
    if (day.estimatedWordCount > 0 || (isToday && hasLiveWords)) count += 1;
    else if (isToday) continue;
    else break;
  }
  return count;
};

const refresh = async (): Promise<void> => {
  const [monitor, today, days] = await Promise.all([
    invoke<MonitorStatus>("monitor_status"),
    invoke<DailySummary>("today_summary"),
    invoke<DailySummary[]>("recent_daily_summaries", { days: 366 }),
  ]);
  const liveWords = monitor.currentSessionCharacters / 5;
  const historicalBest = days.reduce((value, day) => Math.max(value, day.peakWpm), 0);
  wpm.textContent = Math.round(monitor.displayedWpm).toString();
  words.textContent = formatNumber(today.estimatedWordCount + liveWords, 1);
  best.textContent = Math.round(
    Math.max(historicalBest, monitor.personalBestWpm),
  ).toString();
  best30.textContent = Math.round(monitor.sustained30BestWpm).toString();
  best60.textContent = Math.round(monitor.sustained60BestWpm).toString();
  streak.textContent = countStreak(days, liveWords > 0).toString();
  quiet.textContent = formatClock(monitor.lastActivityUnixMs);
  state.textContent = monitor.state;
  state.dataset.state = monitor.state;
  message.textContent = monitor.lastError ?? "";
};

const openWindow = async (command: string): Promise<void> => {
  try {
    await invoke(command);
  } catch (error) {
    message.textContent = String(error);
  }
};

byId<HTMLButtonElement>("open-settings").addEventListener(
  "click",
  () => void openWindow("open_settings_window"),
);
byId<HTMLButtonElement>("open-statistics").addEventListener(
  "click",
  () => void openWindow("open_statistics_window"),
);
window.addEventListener("keydown", (event) => {
  if (event.key === "Escape") void invoke("hide_dashboard_window");
});
window.addEventListener("DOMContentLoaded", () => {
  void refresh().catch((error) => (message.textContent = String(error)));
  window.setInterval(() => void refresh().catch(() => undefined), 1_000);
});
