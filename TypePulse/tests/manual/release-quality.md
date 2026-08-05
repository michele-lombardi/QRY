# Manual check — release quality and macOS lifecycle

Use only dummy text. Record aggregate CPU/RSS and pass/fail results, never input events,
keys, application names or window titles.

## Resource profile (QA-02)

1. Open the release bundle and find its PID with Activity Monitor or `pgrep`.
2. Measure 60 idle samples:

    ```bash
    ../scripts/sample-resources.sh PID 60 1
    ```

3. Grant Input Monitoring, start QRY and type dummy prose for two minutes.
4. Repeat the sample during ordinary typing.
5. Record average/max CPU percentage and RSS MiB only.

- [ ] Idle baseline recorded.
- [ ] Typing baseline recorded.
- [ ] No sustained idle CPU loop is visible.
- [ ] No unbounded memory growth is visible.

Final thresholds remain `TODO` until the overlay animation exists and the first real
baseline is available.

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

- [ ] `TODO`: execute after `OVR-03`; the overlay does not exist yet.

## Release artifact and Gatekeeper

- [ ] Both SHA-256 files verify their matching archives.
- [ ] `codesign --verify --deep --strict QRY.app` passes.
- [ ] A clean account/Mac follows `docs/gatekeeper.md` successfully.
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
