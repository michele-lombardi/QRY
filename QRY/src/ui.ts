export const byId = <T extends Element>(id: string): T => {
  const element = document.querySelector<T>(`#${id}`);
  if (!element) throw new Error(`Missing QRY element: ${id}`);
  return element;
};

export const formatNumber = (value: number, maximumFractionDigits = 0): string =>
  new Intl.NumberFormat(undefined, { maximumFractionDigits }).format(value);

export const formatDuration = (seconds: number): string => {
  const safe = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(safe / 3_600);
  const minutes = Math.floor((safe % 3_600) / 60);
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m ${safe % 60}s`;
};

export const formatClock = (unixMs: number): string => {
  if (unixMs <= 0) return "—";
  return new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(unixMs));
};

export const pulsePath =
  "M2 18 C7 4 12 4 17 18 S27 32 32 18 S42 4 47 18 S57 32 62 18 S72 4 77 18 S87 32 92 18 S102 4 107 18 S117 32 122 18 S132 4 137 18 S147 32 152 18";
