import { listen } from "@tauri-apps/api/event";

type OverlayState = {
  visible: boolean;
  displayedWpm: number;
  animationBand: "still" | "steady" | "fast" | "intense";
  content: "wpm" | "animation" | "both";
  size: "small" | "medium" | "large";
  celebrationSequence: number;
};

const card = document.querySelector<HTMLElement>("#overlay-card");
const wpm = document.querySelector<HTMLElement>("#overlay-wpm");

if (!card || !wpm) throw new Error("Missing TypePulse overlay elements");

let celebrationSequence = 0;
let celebrationTimer: number | undefined;

const celebrate = (): void => {
  window.clearTimeout(celebrationTimer);
  card.classList.remove("is-celebrating");
  void card.offsetWidth;
  card.classList.add("is-celebrating");
  celebrationTimer = window.setTimeout(
    () => card.classList.remove("is-celebrating"),
    900,
  );
};

const render = (state: OverlayState): void => {
  wpm.textContent = Math.round(state.displayedWpm).toLocaleString();
  card.dataset.band = state.animationBand;
  card.dataset.content = state.content;
  card.dataset.size = state.size;
  card.classList.toggle("is-visible", state.visible);
  card.setAttribute(
    "aria-label",
    `${Math.round(state.displayedWpm)} words per minute, ${state.animationBand} rhythm`,
  );
  if (
    state.celebrationSequence > 0 &&
    state.celebrationSequence !== celebrationSequence
  ) {
    celebrate();
  }
  celebrationSequence = state.celebrationSequence;
};

void listen<OverlayState>("typepulse://overlay-state", ({ payload }) => {
  render(payload);
});
