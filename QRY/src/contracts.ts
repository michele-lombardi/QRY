export type PermissionStatus = { status: "granted" | "denied" | "unknown" };

export type MonitorStatus = {
  state: string;
  totalActivities: number;
  lastActivityUnixMs: number;
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
  currentSessionActiveTypingSeconds: number;
  currentSessionCharacters: number;
  currentSessionAverageWpm: number;
  currentSessionPeakWpm: number;
  personalBestWpm: number;
  lastError: string | null;
};

export type StartupPreference = {
  autoStartEnabled: boolean;
  loginItemRegistered: boolean;
};

export type MenuBarPreference = {
  wpmEnabled: boolean;
};

export type OverlayPreference = {
  enabled: boolean;
  position: "top-left" | "top-right" | "bottom-left" | "bottom-right";
  size: "small" | "medium" | "large";
  content: "wpm" | "animation" | "both";
  backgroundEnabled: boolean;
};

export type DailySummary = {
  date: string;
  estimatedCharacterCount: number;
  estimatedWordCount: number;
  averageWpm: number;
  peakWpm: number;
  activeTypingSeconds: number;
  sessionCount: number;
};

export type MetricBucket = {
  intervalStartUnixMs: number;
  intervalDurationSeconds: number;
  estimatedCharacterCount: number;
  averageWpm: number;
  peakWpm: number;
};
