# Product vision

## The idea

QRY makes typing rhythm visible without learning what a person writes. It is a
small macOS companion that turns anonymous typing activity into immediate,
playful feedback and useful long-term statistics.

The product should feel alive but quiet: present when the user is typing,
resting when they pause, and absent when it has nothing useful to show.

## Product principles

### Rhythm, not content

QRY counts privacy-safe typing activities and time intervals. It does not need
characters, words, passwords, application names, window titles, or websites.
Features that require those inputs are outside the product boundary.

### Local by default

Statistics and preferences stay on the user's computer. There is no account,
cloud dependency, telemetry service, or advertising identifier. Export is an
explicit user action.

### Calm feedback

Live WPM is a signal, not a score. QRY avoids judgmental language, quotas, and
notifications designed to create pressure. Personal records are celebrated
briefly and then the interface returns to rest.

### Native respect

The app follows macOS conventions for permissions, appearance, reduced motion,
tray/menu-bar behavior, and login items. It never attempts to bypass system consent.

### Portable core

Metrics, sessions, and storage contracts are independent of macOS. Platform
adapters may evolve, but the privacy model and metric meaning should remain
consistent.

## The core experience

- QRY starts as a tray or menu-bar accessory after a clear first-run platform
  capability flow.
- Pip appears after three accepted typing activities and reacts to pace.
- Live WPM, estimated words, active time, streaks, and personal records build a
  private picture of typing rhythm.
- When activity stops, Pip breathes for the configured delay and disappears.
- The Today panel gives a quick glance; Statistics provides longer local trends;
  Settings keeps permissions and behavior under user control.

## Product boundaries

QRY is not a keylogger, writing assistant, employee-monitoring tool, accuracy
trainer, or cloud analytics product. It intentionally cannot show per-key
heatmaps, reconstruct text, rank applications, or evaluate the value of a
person's work.

See the [privacy model](privacy.md), [architecture](architecture.md), and
[public roadmap](../ROADMAP.md) for the technical and delivery consequences of
this vision.
