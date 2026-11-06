# Manual check — release quality and macOS lifecycle

Use only dummy text. Record aggregate CPU/memory and pass/fail results, never input
events, keys, application names or window titles.

## Resource profile (QA-02)

1. Open the release bundle and find its PID with Activity Monitor or `pgrep`.
2. Measure 60 idle samples:

    ```bash
    ../scripts/sample-resources.sh PID 60 1
    ```

3. Grant Input Monitoring, start QRY and type dummy prose for two minutes.
4. Repeat the sample during ordinary typing.
5. Record average/max CPU, RSS and physical footprint MiB. On macOS, RSS alone excludes
   compressed memory and can hide multi-gigabyte native allocation growth.
6. Leave the app running for several hours and repeat the measurements. Check physical
   footprint growth as well as resident memory.

For a permission-free regression check of native display queries, run from `QRY/`:

```bash
cargo run -p typepulse-app --example monitor_memory --release
```

This uses a separate hidden native window and no input monitor or database. After warmup
it performs 20,000 worker-thread display refreshes (over 11 hours of idle refreshes at
the normal two-second interval) and fails if physical footprint grows by more than 16
MiB. It covers both the runtime's main-thread monitor dispatch and QRY's per-refresh
autorelease scope. It requires a logged-in macOS desktop session.

- [ ] Idle baseline recorded.
- [ ] Typing baseline recorded.
- [ ] No sustained idle CPU loop is visible.
- [ ] No unbounded memory growth is visible.

Record the first distributed-build baseline here; it will define comparison thresholds
for later betas.

### Native memory regression — 6 November 2026

Local arm64 check on macOS 26.6.2, with Tauri 2.12.5 / runtime-wry 2.12.3 and the
per-refresh native scope:

```text
warmup=1000 footprint_bytes=20169712
refreshes=5000 footprint_bytes=19989488
refreshes=10000 footprint_bytes=19989488
refreshes=15000 footprint_bytes=19989488
refreshes=20000 footprint_bytes=20022256
PASS refreshes=20000 peak_growth_bytes=0
```

The earlier installed build had reached 5,487,055,744 bytes of physical footprint; heap
inspection found 6,884,610 deferred display-description dictionaries and 185,774,080
bytes of autorelease-pool contents. The native probe checks repeated display queries;
the multi-hour full-app lifecycle check remains separate.

## Suspend, logout and restart (QA-03)

- [ ] An active session is flushed or safely recoverable when QRY quits.
- [ ] Sleep/wake does not create invented activity or extreme WPM.
- [ ] Revoked Input Monitoring remains visibly revoked after wake.
- [ ] With automatic startup enabled, logout/login creates one process and one menu-bar
      icon.
- [ ] With automatic startup disabled, login does not launch QRY.

## Timezone and date (QA-04)

Automated tests cover Gregorian rollover, isolated daily queries and the same instant
resolving to different local dates in two offsets. Do not change the system clock solely
for this test.

- [ ] A natural midnight rollover starts a new day without deleting yesterday.
- [ ] A normal timezone change does not rewrite historical rows.

## Multi-monitor (QA-05)

- [ ] With Accessibility enabled, Pip follows the focused window across two displays.
- [ ] Moving directly between displays with different scale factors keeps Pip visible.
- [ ] Revoking Accessibility keeps the last valid display, then uses the documented
      fallback.

## Release artifact and Gatekeeper

- [ ] Both SHA-256 files verify their matching archives.
- [ ] `codesign --verify --deep --strict QRY.app` passes.
- [ ] A clean account/Mac follows `docs/installation.md` successfully.
- [ ] Input Monitoring is requested separately and never bypassed.
- [ ] Install, launch, upgrade and uninstall through the personal cask pass.

## Result

- Tag / commit: `TODO`
- macOS / architecture: `TODO`
- Idle CPU/RSS: `TODO`
- Typing CPU/RSS: `TODO`
- Lifecycle result: `TODO`
- Gatekeeper result: `TODO`
- Homebrew result: `TODO`
