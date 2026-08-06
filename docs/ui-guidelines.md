# UI guidelines

QRY should feel like a calm macOS companion: lightweight, direct, and alive
only when useful. This document records stable interface behavior rather than a
screen-by-screen implementation backlog.

## Voice and tone

- describe facts without judging productivity;
- prefer short, plain language over motivational pressure;
- explain permissions before macOS asks for them;
- state required and optional choices explicitly;
- celebrate records briefly, without notifications or persistent interruption;
- never imply that QRY knows what the user typed.

## Product surfaces

| Surface    | Entry point                           | Purpose                                                           |
| ---------- | ------------------------------------- | ----------------------------------------------------------------- |
| Onboarding | first launch or permission revocation | explain privacy, collect required consent, offer optional choices |
| Today      | left-click menu-bar Pulse             | quick live and daily summary                                      |
| Statistics | menu or Today                         | local history, charts, records, and export                        |
| Settings   | menu                                  | monitoring, startup, Pip, permissions, and privacy controls       |
| Pip        | automatic after typing begins         | click-through rhythm feedback                                     |

After onboarding, QRY is an accessory app and stays out of the Dock and
`Cmd + Tab`. Closing a regular window hides it; **Quit QRY** terminates the
background process and flushes aggregate state.

## Menu-bar behavior

- left click toggles the Today panel;
- right click opens native quick actions;
- the optional WPM title reserves a stable three-character slot to avoid layout
  movement;
- showing or hiding menu-bar WPM does not affect Pip or statistics;
- Start/Pause controls the current monitor and remains separate from Start at
  login.

## Onboarding and permissions

The first-run sequence has three responsibilities:

1. explain that QRY measures rhythm without recording content;
2. request required Input Monitoring and close if it is denied or times out;
3. offer optional Accessibility and an unchecked Start at login choice.

QRY performs one clean restart after required access is granted. Optional
choices must have a complete skip path. Runtime permission revocation returns
to the same gate instead of leaving a non-functional shell visible.

## Pip behavior

- remain hidden until the third accepted typing activity;
- use Walk/Run motion bands that respond to current pace;
- trigger a short Jump/Cheer celebration for a new qualified peak, 30-second
  record, or 60-second record;
- after typing stops, enter Breathe for the configured 1–15 second delay, then
  fade out;
- remain click-through and never steal keyboard focus;
- follow the focused display only when optional Accessibility access is valid;
- fall back safely to the primary display when geometry is unavailable.

When the background card is disabled, remove the card fill, border, blur,
radius, and panel shadow. A small element-level contrast shadow is acceptable
for readability, but it must not recreate a floating rectangle.

## Statistics and charts

Statistics support Today, 7 days, 30 days, and one year. Separate speed from
volume:

- average WPM is a line and peak WPM uses distinct points;
- estimated words use bars in a separate chart;
- numeric values belong on the vertical axis and time on the horizontal axis;
- headline values may merge the active in-memory session, while charts remain
  based on bounded aggregate buckets;
- empty states explain what will appear without inventing sample user data.

All values are estimates based on five accepted activities per word. UI copy
must not describe the number as exact text analysis.

## Accessibility and motion

- follow system Light/Dark appearance unless an explicit user override exists;
- honor `prefers-reduced-motion` and keep information understandable without
  animation;
- use semantic labels and logical keyboard order for every interactive control;
- never encode a record or permission state by color alone;
- maintain readable contrast in transparent and card modes;
- keep focus visible in onboarding, Settings, Today, and Statistics.

## Compatibility identifiers

The visible name is **QRY**. The bundle identifier
`app.typepulse.desktop`, database filename `typepulse.sqlite3`, and internal
crate/event names remain intentionally stable to preserve permissions, login
state, preferences, and local history. Changing them requires an explicit data
and TCC migration plan.
