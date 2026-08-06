# ADR 0004: Passive macOS input monitor

- Status: Accepted for macOS V1
- Date: 2026-08-05

## Context

QRY must observe typing activity while another application has focus. This is a
sensitive boundary: an API capable of observing keyboard events could become a
keylogger if key identity, text, or application metadata escaped the adapter.

The spike compared a Swift/Objective-C bridge with direct Rust bindings to Core
Graphics and Core Foundation. The required permission and event-tap APIs are
available through the current Rust crates.

## Decision

- use `core-graphics` for `CGEventTap` and `CFRunLoop` integration;
- use `objc2-core-graphics` for `CGPreflightListenEventAccess` and
  `CGRequestListenEventAccess`;
- do not add a Swift bridge for the macOS V1;
- create a `Session`, `HeadInsertEventTap`, `ListenOnly` tap for `KeyDown`;
- run the tap and run loop on a named dedicated thread;
- keep key-code and flag filtering private to the adapter;
- emit only `TypingActivity { occurred_at: Instant }`;
- deliver through a bounded channel with `try_send`, so the callback never
  waits for the consumer;
- measure callback count, drops, re-enables, and duration using atomics only;
- detect permission revocation and stop with a categorized state;
- re-enable the tap after `TapDisabledByTimeout` or
  `TapDisabledByUserInput`;
- support macOS 10.15 or later.

QRY may open the relevant System Settings page, but it cannot grant permission
on the user's behalf.

## Why `Session` and `ListenOnly`

An HID tap requires broader privileges and is unnecessary. `ListenOnly` makes
the passive behavior explicit: QRY neither changes nor suppresses input and the
callback always returns the original event.

## Secure Input

Protected fields may prevent some events from being observed. QRY accepts the
resulting gap, requests no additional privilege, and does not attempt a bypass.
This is an intentional privacy and security trade-off.

## Consequences

- the callback remains short and contains no disk, network, log, or WebView work;
- core and frontend cannot receive key codes through the public API;
- a slow consumer produces explicit drops instead of blocking keyboard input;
- local unsigned rebuilds may require renewed TCC consent;
- revocation, restoration, and Secure Input require real-macOS tests;
- a future Linux adapter can replace the platform layer without changing the
  portable engine.

## References

- [Apple: CGEvent tap creation](https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate%28tap%3Aplace%3Aoptions%3Aeventsofinterest%3Acallback%3Auserinfo%3A%29?language=objc)
- [Apple: CGEventTapOptions](https://developer.apple.com/documentation/coregraphics/cgeventtapoptions)
- [Apple: CGPreflightListenEventAccess](https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess%28%29?language=objc)
