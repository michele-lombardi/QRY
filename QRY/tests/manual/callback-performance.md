# Manual check — callback performance

## Repeatable Rust hot-path reference

```bash
cargo test -p typepulse-platform-desktop --release \
  typing_callback_hot_path_reference -- --ignored --nocapture
```

Baseline on 5 August 2026, Apple M1 Pro arm64, macOS 26.5.2:

```text
250,000 samples; 31 ns/activity
```

The benchmark covers filter, monotonic timestamp, non-blocking channel send and atomic
counters. It excludes Core Graphics dispatch overhead.

## Live check

After granting Input Monitoring, start diagnostics and type ordinary dummy text for at
least two minutes in another app. Record only:

- average and maximum callback microseconds;
- dropped count;
- re-enable count;
- OS and architecture.

Target for the spike: average well below 100 µs, maximum below 1 ms during ordinary
typing, and zero drops. Status: **TODO manuale** pending TCC access.
