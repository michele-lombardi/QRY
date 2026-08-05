# TypePulse brand implementation

- Source of truth: `brand identity/TypePulse Brand Identity.dc.html`
- Identity version: v0.1
- Product name adopted: TypePulse
- Implementation date: 5 August 2026

## Rules adopted

The product is treated as a visible rhythm and companion, not as a productivity
tracker. The current app uses the brand vocabulary, system typography and five
canonical colors:

| Token | Value | Purpose |
| --- | --- | --- |
| Almost Black | `#111111` | primary dark surface and text on light |
| Warm White | `#F5F5F3` | primary light surface and text on dark |
| Electric Cyan | `#3CEFFF` | active Pulse mark, Pip and live rhythm |
| Soft Green | `#30D158` | personal record only |
| Amber | `#FF9F0A` | warnings and degraded states |

Electric Cyan is not used as a decorative surface or ordinary heading color.
Metrics use tabular figures and the stack starts with SF Pro through the macOS
system fonts.

## Assets

Canonical, editable SVG sources live in `TypePulse/assets/brand/`. Generated
platform PNG, ICNS and ICO files remain in `TypePulse/src-tauri/icons/`.

- normal Pulse mark: four alternating semicircles;
- menu-bar mark below 20 pt: the allowed three-beat variant;
- idle menu-bar state: a low-opacity flatline;
- application icon: dark squircle with a cyan Pulse mark;
- no gradient, distortion or thin/flat stroke is applied to the mark itself.

The menu-bar icon changes between flatline and Pulse. During live typing it also
shows the rounded WPM value as the native status-item title.

Both PNGs are monochrome alpha masks installed as native macOS template images.
Every runtime icon replacement updates image and template status atomically, so
AppKit renders the mark light on a dark menu bar and dark on a light menu bar.
The WPM string remains the native status-item title and follows the same system
appearance without application-defined foreground colors.

When enabled, the native title uses a fixed three-digit slot made from figure
spaces, so changes such as 9 → 10 → 100 do not move the Pulse mark. A checked
**Show WPM in menu bar** item persists the preference; disabling it removes only
the menu-bar number and leaves the PiP unchanged.

## Pip behavior mapping

Pip is implemented as inline SVG: one circle, two eyes and two capsule feet. It
has no mouth, arms, outline, shadow or accessories and never enters the logo.

| Identity behavior | Trigger in the app | Status |
| --- | --- | --- |
| Breathe | zero WPM renderer, 3.4 s scale | Implemented; overlay lifecycle normally keeps idle hidden |
| Walk | `1–69` WPM, with a quicker step in the core fast band | Implemented |
| Run | `70+` WPM, 10° lean and dash lines | Implemented |
| Jump | new personal record, one 1.5 s motion | Implemented |
| Cheer | same record event, Pip becomes green once | Implemented |
| Tired | at least 90 minutes of aggregate active typing in one session | Implemented |
| Sleep | five minutes idle | Future: current overlay hides after 2 s and the session ends after 30 s |
| Dance | stable rhythm for more than two minutes | Future: core does not yet measure rhythm stability |

The identity leaves `41–69 WPM` between the documented Walk and Run examples.
The implementation treats that range as a faster Walk rather than inventing a
new mood. This keeps every state tied to a measurable trigger.

## Motion

- the Pulse ring interpolates from 1.6 seconds at rest to 0.4 seconds at 120 WPM;
- ambient Breathe motion uses 3.4 seconds;
- transitions use the identity easing `cubic-bezier(.4, 0, .2, 1)`;
- continuous character effects animate only transform and opacity;
- reduced motion keeps state and color changes, removes continuous movement and
  retains an opacity-only overlay transition.

## Voice

Visible copy now favors rhythm language: “Find your rhythm”, “Your keyboard has
a heartbeat” and “The rhythm is paused”. Productivity, optimization and
comparative performance language remains prohibited. Pip itself does not speak;
future companion messages belong beside it in the menu-bar panel.

## Future implementation tasks

| ID | Task | Priority | Dependency | Acceptance criterion |
| --- | --- | --- | --- | --- |
| BRD-07 | Add a portable rhythm-stability metric for Dance | P1 | core metrics | deterministic tests identify two stable minutes without inspecting key identity |
| BRD-08 | Decide where Sleep lives and add a five-minute idle state | P1 | product lifecycle, UI-02 | no conflict with 2 s overlay hide or 30 s session end; state is visible in an intentional surface |
| BRD-09 | Use the moving Pulse wave as the live/daily chart | P1 | UI-04 | chart uses the mark geometry without a chart library and remains accessible |
| BRD-10 | Add at most three companion messages per local day | P2/v2 | UI-02, local message state | messages appear only inside the popover, never as system notifications |
| BRD-11 | Complete branded onboarding and insights screens | P0 | UI-03, UI-04, UI-09 | all final screens use tokens, voice and Pip rules consistently |
| BRD-12 | Add an About panel endorsement | P1 | owner confirmation | exact legal display name is confirmed; app uses text only and no co-brand lockup |
| BRD-13 | Verify TypePulse name, domain and trademark availability | P0 before public launch | project owner/external research | result and final naming decision are recorded before stable release |
| BRD-14 | Produce website horizontal/vertical lockups and product GIF | P2 | public website | homepage demonstrates live motion and follows clear-space rules |
| BRD-15 | Perform visual regression and contrast audit | P0 before RC | final Phase F screens | icon, tray, light/dark UI, reduced motion and overlays pass reference review |

The Micro-Y logo referenced by the identity HTML is not included in the supplied
folder. No logo was recreated or extracted. Only the text endorsement is used
in the current app; its final legal wording remains gated by `BRD-12`.
