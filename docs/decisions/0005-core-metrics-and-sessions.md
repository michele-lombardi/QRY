# ADR 0005: Metric, session, and record semantics

- Status: Accepted for V1
- Date: 2026-08-05

## Context

The portable engine converts only `TypingActivity { occurred_at }` into live
WPM, visual state, sessions, and records. Results must be deterministic,
portable, and independent of how often the UI requests a snapshot.

## Decision

### Time

- use monotonic `Instant` values for the live path;
- inject a `Clock` into `TypingEngine`;
- use `SystemClock` in production and `ManualClock` in tests;
- reject activities or ticks older than the last observed activity;
- assign civil dates only in the desktop/persistence composition layer.

### Live WPM

- use a rolling lookback of at most 10 seconds;
- estimate one word per five accepted typing activities;
- during warm-up, divide observed intervals by elapsed time between the first
  and latest activity;
- require at least 250 ms before publishing an estimate;
- use a one-second minimum denominator during the first second to prevent an
  unrealistic projection from a few events;
- cap live output defensively at 300 WPM;
- expire events exactly on the lower window boundary;
- apply an EMA factor of `0.25`, initialized from the first reliable estimate;
- update smoothing on activity, not on UI polling ticks.

### Repetition protection

- reject operating-system auto-repeat;
- allow two consecutive presses of the same key for natural double letters;
- reject the third and later identical press until a different countable key or
  a pause of at least one second;
- retain all key identity only inside the private platform adapter.

### Visual bands

- `Still`: below 30 WPM;
- `Steady`: 30 to below 60 WPM;
- `Fast`: 60 to below 90 WPM;
- `Intense`: 90 WPM or higher.

Thresholds are configurable, finite, non-negative, and strictly increasing.

### Sessions and visibility

- the first activity opens a session;
- Pip stays hidden until the third accepted activity;
- quiet input enters the Breathe presentation state for the configured 1–15
  second disappearance delay;
- 30 seconds without activity closes the session;
- the logical end time is the last activity, not the timeout tick;
- active time sums consecutive gaps no longer than two seconds;
- averages and peak records use samples with at least three seconds of history;
- estimated words retain the fractional `activities / 5` value;
- an activity arriving exactly at session timeout closes the previous session
  and opens a new one in the same update.

### Records

Peak celebrations use qualified samples after at least three seconds. Sustained
records require complete fixed windows of 30 or 60 seconds. Each record type
emits at most one celebration per session. When no historical record exists,
the first valid value establishes the baseline without celebrating every
subsequent warm-up increase.

## Consequences

- tests can advance minutes without sleeping;
- metric meaning does not depend on frontend refresh frequency;
- `Instant` and raw live events are never serialized;
- warm-up remains responsive without contaminating aggregate records;
- local-date assignment and persistence stay outside the live engine;
- words are estimates and must be labeled accordingly in the UI.
