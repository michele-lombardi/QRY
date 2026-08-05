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
const wpmChart = byId<SVGSVGElement>("wpm-chart");
const wordsChart = byId<SVGSVGElement>("words-chart");
const detailBody = byId<HTMLTableSectionElement>("detail-body");

const chartWidth = 920;
const chartHeight = 280;
const plotLeft = 58;
const plotRight = 900;
const plotTop = 18;
const plotBottom = 238;

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

type AxisScale = { maximum: number; step: number };

const niceScale = (value: number, minimum: number): AxisScale => {
  const target = Math.max(value, minimum);
  const roughStep = target / 4;
  const magnitude = 10 ** Math.floor(Math.log10(roughStep));
  const normalized = roughStep / magnitude;
  const factor =
    normalized <= 1
      ? 1
      : normalized <= 2
        ? 2
        : normalized <= 2.5
          ? 2.5
          : normalized <= 5
            ? 5
            : 10;
  const step = factor * magnitude;
  return { maximum: Math.ceil(target / step) * step, step };
};

const svgText = (content: string, attributes: Record<string, string>): SVGElement => {
  const element = svgElement("text", attributes);
  element.textContent = content;
  return element;
};

const xCoordinate = (index: number, count: number): number =>
  count <= 1
    ? (plotLeft + plotRight) / 2
    : plotLeft + (index / (count - 1)) * (plotRight - plotLeft);

const yCoordinate = (value: number, maximum: number): number =>
  plotBottom - (value / maximum) * (plotBottom - plotTop);

const timeLabelIndexes = (count: number): number[] => {
  if (count <= 0) return [];
  const labelCount = Math.min(count, 6);
  return Array.from(
    new Set(
      Array.from({ length: labelCount }, (_, index) =>
        Math.round((index * (count - 1)) / Math.max(labelCount - 1, 1)),
      ),
    ),
  );
};

const renderAxes = (
  chart: SVGSVGElement,
  points: ChartPoint[],
  scale: AxisScale,
  unit: string,
): void => {
  const tickCount = Math.round(scale.maximum / scale.step);
  for (let tick = 0; tick <= tickCount; tick += 1) {
    const value = tick * scale.step;
    const y = yCoordinate(value, scale.maximum);
    chart.append(
      svgElement("line", {
        x1: String(plotLeft),
        y1: String(y),
        x2: String(plotRight),
        y2: String(y),
        stroke: "currentColor",
        "stroke-opacity": tick === 0 ? "0.18" : "0.08",
        "stroke-width": "1",
      }),
      svgText(value.toLocaleString(undefined, { maximumFractionDigits: 1 }), {
        x: String(plotLeft - 10),
        y: String(y + 4),
        fill: "currentColor",
        "fill-opacity": "0.55",
        "font-size": "11",
        "text-anchor": "end",
      }),
    );
  }

  chart.append(
    svgText(unit, {
      x: "2",
      y: "12",
      fill: "currentColor",
      "fill-opacity": "0.55",
      "font-size": "10",
      "font-weight": "650",
    }),
  );

  timeLabelIndexes(points.length).forEach((index) => {
    const x = xCoordinate(index, points.length);
    chart.append(
      svgElement("line", {
        x1: String(x),
        y1: String(plotBottom),
        x2: String(x),
        y2: String(plotBottom + 5),
        stroke: "currentColor",
        "stroke-opacity": "0.24",
        "stroke-width": "1",
      }),
      svgText(points[index]?.label ?? "", {
        x: String(x),
        y: String(plotBottom + 22),
        fill: "currentColor",
        "fill-opacity": "0.55",
        "font-size": "11",
        "text-anchor": "middle",
      }),
    );
  });

  chart.append(
    svgText("Time", {
      x: String(chartWidth),
      y: String(chartHeight - 2),
      fill: "currentColor",
      "fill-opacity": "0.55",
      "font-size": "10",
      "font-weight": "650",
      "text-anchor": "end",
    }),
  );
};

const appendTooltip = (element: SVGElement, content: string): void => {
  const title = svgElement("title", {});
  title.textContent = content;
  element.append(title);
};

const renderSpeedChart = (points: ChartPoint[]): void => {
  wpmChart.replaceChildren();
  const hasData = points.some((point) => point.average > 0 || point.peak > 0);
  byId<HTMLElement>("empty-wpm-chart").hidden = hasData;
  const scale = niceScale(
    Math.max(0, ...points.map((point) => Math.max(point.average, point.peak))),
    100,
  );
  renderAxes(wpmChart, points, scale, "WPM");
  if (points.length === 0) return;

  const path = points
    .map((point, index) => {
      const x = xCoordinate(index, points.length);
      const y = yCoordinate(point.average, scale.maximum);
      return `${index === 0 ? "M" : "L"}${x} ${y}`;
    })
    .join(" ");
  wpmChart.append(
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
    const circle = svgElement("circle", {
      cx: String(xCoordinate(index, points.length)),
      cy: String(yCoordinate(point.peak, scale.maximum)),
      r: "3.5",
      fill: "#30d158",
    });
    appendTooltip(
      circle,
      `${point.label}: ${point.average.toFixed(1)} average WPM, ${point.peak.toFixed(1)} peak WPM`,
    );
    wpmChart.append(circle);
  });
};

const renderWordsChart = (points: ChartPoint[]): void => {
  wordsChart.replaceChildren();
  const hasData = points.some((point) => point.words > 0);
  byId<HTMLElement>("empty-words-chart").hidden = hasData;
  const scale = niceScale(Math.max(0, ...points.map((point) => point.words)), 5);
  renderAxes(wordsChart, points, scale, "Words");
  if (points.length === 0) return;

  const slotWidth = (plotRight - plotLeft) / Math.max(points.length, 1);
  const barWidth = Math.max(2, Math.min(30, slotWidth * 0.62));
  points.forEach((point, index) => {
    const x = xCoordinate(index, points.length);
    const y = yCoordinate(point.words, scale.maximum);
    const bar = svgElement("rect", {
      x: String(x - barWidth / 2),
      y: String(y),
      width: String(barWidth),
      height: String(Math.max(0, plotBottom - y)),
      rx: String(Math.min(4, barWidth / 3)),
      fill: "#3cefff",
      "fill-opacity": "0.58",
    });
    appendTooltip(
      bar,
      `${point.label}: ${formatNumber(point.words, 1)} estimated words`,
    );
    wordsChart.append(bar);
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
      byId<HTMLElement>("speed-chart-title").textContent = "Today's WPM";
      byId<HTMLElement>("words-chart-title").textContent = "Today's words";
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
      byId<HTMLElement>("speed-chart-title").textContent =
        selectedDays === 366 ? "Yearly WPM" : `${selectedDays}-day WPM`;
      byId<HTMLElement>("words-chart-title").textContent =
        selectedDays === 366 ? "Yearly words" : `${selectedDays}-day words`;
      byId<HTMLElement>("period-caption").textContent =
        selectedDays === 366 ? "Last 12 months" : `Last ${selectedDays} days`;
    }
    renderSpeedChart(points);
    renderWordsChart(points);
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
