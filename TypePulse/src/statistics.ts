import { invoke } from "@tauri-apps/api/core";
import type { DailySummary, MetricBucket, MonitorStatus } from "./contracts";
import { byId, formatDuration, formatNumber } from "./ui";

type ChartPoint = {
  label: string;
  words: number;
  average: number;
  peak: number;
  active: number;
  sessions: number;
};
const svgNs = "http://www.w3.org/2000/svg";
let selectedDays = 1;
let refreshing = false;

const message = byId<HTMLElement>("statistics-message");
const chart = byId<SVGSVGElement>("rhythm-chart");
const chartLabels = byId<HTMLElement>("chart-labels");
const detailBody = byId<HTMLTableSectionElement>("detail-body");

const aggregate = (points: ChartPoint[]) => {
  const words = points.reduce((sum, point) => sum + point.words, 0);
  const weighted = points.reduce((sum, point) => sum + point.average * point.words, 0);
  return {
    words,
    average: words > 0 ? weighted / words : 0,
    peak: points.reduce((value, point) => Math.max(value, point.peak), 0),
    active: points.reduce((sum, point) => sum + point.active, 0),
    sessions: points.reduce((sum, point) => sum + point.sessions, 0),
  };
};

const dayLabel = (date: string): string =>
  new Intl.DateTimeFormat(undefined, { weekday: "short", day: "numeric" }).format(
    new Date(`${date}T12:00:00`),
  );

const bucketPoints = (buckets: MetricBucket[]): ChartPoint[] =>
  buckets.map((bucket) => ({
    label: new Intl.DateTimeFormat(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date(bucket.intervalStartUnixMs)),
    words: bucket.estimatedCharacterCount / 5,
    average: bucket.averageWpm,
    peak: bucket.peakWpm,
    active:
      bucket.averageWpm > 0
        ? (bucket.estimatedCharacterCount / 5 / bucket.averageWpm) * 60
        : 0,
    sessions: 0,
  }));

const dailyPoints = (days: DailySummary[]): ChartPoint[] =>
  days.map((day) => ({
    label: dayLabel(day.date),
    words: day.estimatedWordCount,
    average: day.averageWpm,
    peak: day.peakWpm,
    active: day.activeTypingSeconds,
    sessions: day.sessionCount,
  }));

const mergeCurrentSession = (
  points: ChartPoint[],
  monitor: MonitorStatus,
): ChartPoint[] => {
  const liveWords = monitor.currentSessionCharacters / 5;
  const last = points[points.length - 1];
  if (!last || liveWords <= 0) return points;
  const totalWords = last.words + liveWords;
  last.average =
    totalWords > 0
      ? (last.average * last.words + monitor.currentSessionAverageWpm * liveWords) /
        totalWords
      : 0;
  last.words = totalWords;
  last.peak = Math.max(last.peak, monitor.currentSessionPeakWpm);
  last.active += monitor.currentSessionActiveTypingSeconds;
  last.sessions += 1;
  return points;
};

const currentDaySummary = (
  today: DailySummary,
  monitor: MonitorStatus,
): ReturnType<typeof aggregate> => {
  const points = dailyPoints([today]);
  return aggregate(mergeCurrentSession(points, monitor));
};

const svgElement = (name: string, attributes: Record<string, string>): SVGElement => {
  const element = document.createElementNS(svgNs, name);
  Object.entries(attributes).forEach(([key, value]) =>
    element.setAttribute(key, value),
  );
  return element;
};

const renderChart = (points: ChartPoint[]): void => {
  chart.replaceChildren();
  chartLabels.replaceChildren();
  byId<HTMLElement>("empty-chart").hidden = points.some(
    (point) => point.words > 0 || point.peak > 0,
  );
  const width = 880;
  const height = 270;
  const padding = 18;
  const maxWpm = Math.max(120, ...points.map((point) => point.peak));
  const maxWords = Math.max(1, ...points.map((point) => point.words));
  for (let line = 0; line <= 4; line += 1) {
    const y = padding + ((height - padding * 2) * line) / 4;
    chart.append(
      svgElement("line", {
        x1: "0",
        y1: String(y),
        x2: String(width),
        y2: String(y),
        stroke: "currentColor",
        "stroke-opacity": "0.08",
        "stroke-width": "1",
      }),
    );
  }
  if (points.length === 0) return;
  const step = points.length > 1 ? (width - padding * 2) / (points.length - 1) : 0;
  const barWidth = Math.max(
    3,
    Math.min(24, (width - padding * 2) / Math.max(points.length, 1) - 4),
  );
  const coordinate = (value: number, index: number): [number, number] => [
    padding + step * index,
    height - padding - (value / maxWpm) * (height - padding * 2),
  ];
  points.forEach((point, index) => {
    const x = points.length === 1 ? width / 2 : padding + step * index;
    const barHeight = (point.words / maxWords) * (height - padding * 2) * 0.34;
    chart.append(
      svgElement("rect", {
        x: String(x - barWidth / 2),
        y: String(height - padding - barHeight),
        width: String(barWidth),
        height: String(barHeight),
        rx: "3",
        fill: "currentColor",
        "fill-opacity": "0.09",
      }),
    );
  });
  const path = points
    .map(
      (point, index) =>
        `${index === 0 ? "M" : "L"}${coordinate(point.average, index).join(" ")}`,
    )
    .join(" ");
  chart.append(
    svgElement("path", {
      d: path,
      fill: "none",
      stroke: "#3cefff",
      "stroke-width": "3",
      "stroke-linecap": "round",
      "stroke-linejoin": "round",
    }),
  );
  points.forEach((point, index) => {
    const [x, y] = coordinate(point.peak, index);
    chart.append(
      svgElement("circle", { cx: String(x), cy: String(y), r: "3.5", fill: "#30d158" }),
    );
  });
  const labelIndexes = new Set([
    0,
    Math.floor((points.length - 1) / 2),
    points.length - 1,
  ]);
  labelIndexes.forEach((index) => {
    const span = document.createElement("span");
    span.textContent = points[index]?.label ?? "";
    chartLabels.append(span);
  });
};

const renderTable = (points: ChartPoint[]): void => {
  detailBody.replaceChildren();
  [...points].reverse().forEach((point) => {
    const row = document.createElement("tr");
    [
      point.label,
      formatNumber(point.words, 1),
      `${point.average.toFixed(1)} WPM`,
      `${point.peak.toFixed(1)} WPM`,
      formatDuration(point.active),
    ].forEach((value) => {
      const cell = document.createElement("td");
      cell.textContent = value;
      row.append(cell);
    });
    detailBody.append(row);
  });
  byId<HTMLElement>("detail-count").textContent =
    `${points.length} ${points.length === 1 ? "point" : "points"}`;
};

const renderSummary = (summary: ReturnType<typeof aggregate>): void => {
  byId<HTMLElement>("summary-words").textContent = formatNumber(summary.words, 1);
  byId<HTMLElement>("summary-average").textContent = summary.average.toFixed(1);
  byId<HTMLElement>("summary-peak").textContent = summary.peak.toFixed(1);
  byId<HTMLElement>("summary-active").textContent = formatDuration(summary.active);
  byId<HTMLElement>("summary-sessions").textContent = `${summary.sessions} sessions`;
  if (summary.words === 0) {
    byId<HTMLElement>("insight-title").textContent = "Start typing";
    byId<HTMLElement>("insight-copy").textContent =
      "Completed sessions will appear here as a private local rhythm history.";
  } else {
    byId<HTMLElement>("insight-title").textContent =
      summary.peak >= 90 ? "A fast rhythm" : "A steady rhythm";
    byId<HTMLElement>("insight-copy").textContent =
      `You averaged ${summary.average.toFixed(1)} WPM across ${formatNumber(summary.words, 1)} estimated words, with a ${summary.peak.toFixed(1)} WPM peak.`;
  }
};

const refresh = async (): Promise<void> => {
  if (refreshing) return;
  refreshing = true;
  let points: ChartPoint[];
  try {
    if (selectedDays === 1) {
      const [buckets, today, monitor] = await Promise.all([
        invoke<MetricBucket[]>("today_metric_buckets"),
        invoke<DailySummary>("today_summary"),
        invoke<MonitorStatus>("monitor_status"),
      ]);
      points = bucketPoints(buckets);
      renderSummary(currentDaySummary(today, monitor));
      byId<HTMLElement>("chart-title").textContent = "Today's rhythm";
      byId<HTMLElement>("period-caption").textContent = new Intl.DateTimeFormat(
        undefined,
        { dateStyle: "full" },
      ).format(new Date());
    } else {
      const [days, monitor] = await Promise.all([
        invoke<DailySummary[]>("recent_daily_summaries", { days: selectedDays }),
        invoke<MonitorStatus>("monitor_status"),
      ]);
      points = mergeCurrentSession(dailyPoints(days), monitor);
      renderSummary(aggregate(points));
      byId<HTMLElement>("chart-title").textContent =
        selectedDays === 366 ? "Yearly rhythm" : `${selectedDays}-day rhythm`;
      byId<HTMLElement>("period-caption").textContent =
        selectedDays === 366 ? "Last 12 months" : `Last ${selectedDays} days`;
    }
    renderChart(points);
    renderTable(points);
    message.textContent = "";
  } finally {
    refreshing = false;
  }
};

document.querySelectorAll<HTMLButtonElement>("[data-range]").forEach((button) => {
  button.addEventListener("click", () => {
    selectedDays = Number(button.dataset.range ?? 1);
    document
      .querySelectorAll("[data-range]")
      .forEach((item) => item.classList.toggle("active", item === button));
    void refresh().catch((error) => (message.textContent = String(error)));
  });
});
byId<HTMLButtonElement>("open-settings").addEventListener(
  "click",
  () => void invoke("open_settings_window"),
);
byId<HTMLButtonElement>("copy-csv").addEventListener("click", async () => {
  try {
    const csv = await invoke<string>("export_daily_statistics_csv", {
      days: selectedDays,
    });
    await navigator.clipboard.writeText(csv);
    message.textContent = "CSV copied to the clipboard.";
  } catch (error) {
    message.textContent = String(error);
  }
});
byId<HTMLButtonElement>("reset-today").addEventListener("click", async () => {
  if (!window.confirm("Reset all of today's QRY statistics? This cannot be undone."))
    return;
  try {
    await invoke("reset_today_statistics");
    await refresh();
    message.textContent = "Today's completed statistics were reset.";
  } catch (error) {
    message.textContent = String(error);
  }
});
window.addEventListener("DOMContentLoaded", () => {
  void refresh().catch((error) => (message.textContent = String(error)));
  window.setInterval(
    () => void refresh().catch((error) => (message.textContent = String(error))),
    5_000,
  );
});
window.addEventListener("focus", () => {
  void refresh().catch((error) => (message.textContent = String(error)));
});
