# macOS overlay memory growth

## Impact and affected build

QRY `v0.2.0-2` could accumulate gigabytes of native memory while its background
overlay controller refreshed display placement. The issue was diagnosed on
6 November 2026 on Apple Silicon running macOS 26.6.2. Earlier releases were
not individually tested for this failure.

The observed main process had reached **5,487,055,744 bytes** of physical
footprint, approximately 5.49 GB / 5.11 GiB, after several days of uptime.
The separate capsule process used approximately 32 MB during the same inspection.
The growing allocations belonged to native display metadata in the main process.

The fix is included in `v0.2.1`. Publication and real-device checks are tracked
separately from the successful local regression checks below.

## Cause

The overlay checks display topology every two seconds and can check the focused
display as often as every 250 milliseconds while typing. The installed runtime
converted monitor information on the long-lived Rust overlay worker, invoking
Cocoa display APIs outside AppKit's main event loop.

Those APIs return temporary autoreleased objects, including screen device-description
dictionaries. Rust ownership of the returned monitor DTOs did not drain the
Objective-C autorelease pool. Without a periodically drained pool on that worker,
the temporaries accumulated for the lifetime of the thread, including while the
overlay was hidden between typing sessions.

Native heap inspection found:

| Allocation                                  |                              Count / size |
| ------------------------------------------- | ----------------------------------------: |
| Swift device-description dictionary storage | 6,884,610 instances / 3,084,305,280 bytes |
| Swift bridging buffers                      |                         771,076,928 bytes |
| Native `NSConcreteValue` objects            |                         660,943,312 bytes |
| Deferred dictionary wrappers                |                         330,461,280 bytes |
| Autorelease-pool contents                   |                         185,774,080 bytes |

macOS had compressed much of this memory, so ordinary RSS readings substantially
underreported the problem. Physical footprint exposed the accumulated allocations.

## Correction

- Require Tauri 2.12.5 or newer. The resolved runtime-wry 2.12.3 dispatches
  app-level monitor queries to the main thread; see the
  [upstream monitor-query correction](https://github.com/tauri-apps/tauri/pull/15630).
- Wrap each overlay placement refresh in a scoped macOS autorelease pool, so
  worker-local Cocoa temporaries are released when the refresh returns.
- Use the same scope for the native monitor regression probe.
- Report macOS physical footprint, including compressed allocations, alongside
  CPU and RSS in `scripts/sample-resources.sh`.

The scope is a normal function call on other platforms. No database migration,
statistics reset, input logging, or additional application permission is introduced
by this correction.

## Regression checks

The worker-lifetime unit test creates autoreleased native objects on a Rust
thread and checks via weak references that each refresh releases them before
the next iteration. It runs 1,000 iterations without input access.

The native monitor probe creates its own hidden window, starts no input monitor,
and opens no statistics database. Run it on a logged-in macOS desktop:

```bash
cd QRY
cargo run --locked -p typepulse-app --example monitor_memory --release
```

After 1,000 warmup iterations, it performs four batches of 5,000 display refreshes
and fails if sampled peak physical footprint exceeds the warmup baseline by more
than 16 MiB. It exercises app-level monitor enumeration, primary and point-based
monitor selection, and window geometry queries from a Rust worker.

Local result on 6 November 2026, using Tauri 2.12.5 / runtime-wry 2.12.3:

```text
warmup=1000 footprint_bytes=20169712
refreshes=5000 footprint_bytes=19989488
refreshes=10000 footprint_bytes=19989488
refreshes=15000 footprint_bytes=19989488
refreshes=20000 footprint_bytes=20022256
PASS refreshes=20000 peak_growth_bytes=0
```

This is the query count of more than 11 hours of two-second idle refreshes. It is
an accelerated native-query check, not a completed 11-hour full-app soak. Live
typing, sleep/wake, repeated monitor connection changes, and Intel hardware still
require the [release-quality checklist](../../QRY/tests/manual/release-quality.md).
The rebuilt app's live-input check was deferred while macOS consent awaited renewal.

For full-app measurements, find QRY's PID and run from the repository root:

```bash
./scripts/sample-resources.sh PID 60 1
```

Compare `average_footprint_mib`, `max_footprint_mib`, and `footprint_delta_mib`
across warmup, idle, ordinary typing, and a later multi-hour sample. Collect only
aggregate resource data; do not record keys, typed content, or window titles.

## Updating an affected installation

1. Quit the previous QRY process normally so its accumulated memory is released.
2. Install the matching architecture's corrected bundle and verify its SHA-256 file.
3. Launch QRY and complete its permission flow if macOS treats the rebuilt,
   ad-hoc-signed executable as a new identity.
4. If QRY remains listed as allowed but reports Input Monitoring unavailable,
   switch its entry off and on in **System Settings → Privacy & Security →
   Input Monitoring**. Re-add the exact installed app if the stale entry persists.
5. Renew optional Accessibility access if needed, restore the desired start-at-login
   choice in onboarding, and let QRY restart normally.

Replacing the executable does not drain allocations in a previously running
process. Reopen the corrected build after quitting the old instance.

Apple documents the need for periodically drained pools on long-lived worker
threads in its [thread management guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/CreatingThreads/CreatingThreads.html).
