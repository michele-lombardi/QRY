# QRY local aggregate bridge, version 1

This contract describes the macOS bridge in the QRY `0.2.0-2` development
source. Earlier QRY releases may not export it. External integrations must
validate the schema and handle an unavailable producer.

Producer: [integrations.rs](../../QRY/src-tauri/src/integrations.rs).
Reference decoder: [QRYBridge.swift](../../integrations/native/QRYBridge.swift).
Example: [synthetic JSON fixture](fixtures/bridge-v1.json).

## Transport and lifecycle

The producer atomically replaces this UTF-8 JSON file in the current account:

```text
~/Library/Caches/app.typepulse.desktop/integrations/state-v1.json
```

There is no HTTP endpoint. The directory is created with mode `0700`; the file
uses `0600`. Sharing is off by default in QRY. With only the capsule enabled,
the core snapshot can exist while external sharing is false. The optional
`statistics` object is published only when external sharing is enabled.

Updates occur about every 500 ms while QRY is running and a consumer is enabled.
The 90-day persisted history is cached for ten seconds; the current day is
replaced with a transaction-consistent merge of saved and active-session totals.
Normal exit or disabling both consumers removes the snapshot. A crash may leave
an old file, which consumers must reject after its freshness limit.

The consumer does not write this file, inspect the producer's SQLite database,
reset permissions or enable sharing. The capsule control file
`preferences-v1.json` is not required for the statistics adapter.

## Fields

| Field                                           | Meaning                                                                                                                                         |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `schemaVersion`                                 | Integer `1`; incompatible versions are rejected.                                                                                                |
| `generatedAtUnixMs`                             | Snapshot creation time in Unix milliseconds.                                                                                                    |
| `instanceId`, `sequence`                        | Producer instance token and unsigned sequence; neither identifies an app being typed into.                                                      |
| `status`                                        | `monitoring`, `paused` or `permission_required`; describes QRY's collector.                                                                     |
| `displayedWpm`                                  | Prepared instantaneous presentation value, integer 0–999. Not the daily mean.                                                                   |
| `animationBand`                                 | `still`, `steady`, `fast` or `intense`.                                                                                                         |
| `celebrationSequence`                           | Unsigned aggregate record-celebration counter.                                                                                                  |
| `capsuleEnabled`                                | QRY capsule preference; not an upstream Vorssaint availability switch.                                                                          |
| `externalEnabled`                               | Whether the consumer may present shared aggregates. Respect false even if other fields exist.                                                   |
| `notchPipEnabled`                               | QRY's opt-in to the optional compact animation; unnecessary for a statistics-only PR.                                                           |
| `presentation.visible`, `presentation.behavior` | Prepared animation state; behavior is `breathe`, `walk`, `run` or `tired`.                                                                      |
| `today.date`                                    | Producer's local calendar date, `YYYY-MM-DD`.                                                                                                   |
| `today.estimatedWords`                          | Anonymous character estimate divided by five; not counted text words.                                                                           |
| `today.peakWpm`                                 | Highest qualified speed for that day.                                                                                                           |
| `today.activeTypingSeconds`                     | Active typing time excluding long idle periods.                                                                                                 |
| `statistics`                                    | Optional additive object. Missing means detailed statistics are unavailable, not zero.                                                          |
| `statistics.records`                            | Nonnegative `peakWpm`, `sustained30Wpm` and `sustained60Wpm`, all-time qualified records. Zero represents no qualified record in this producer. |
| `statistics.days`                               | Up to 90 ordered daily aggregates: `date`, `estimatedWords`, `averageWpm`, `peakWpm`.                                                           |

The daily mean uses character-weighted session mean speeds, consistent with
QRY's stored summaries. It is not derived by dividing words by wall-clock time.
A range mean can weight each day's mean by `estimatedWords`, which uses the same
constant factor per character. Records are all-time values and do not change
scope when the graph range changes.

The producer includes empty calendar days. A graph should leave gaps for days
with no estimated words, rather than imply measured typing at zero WPM. A valid
active day can contain zero qualified speed if no qualifying sample was available.

## Reader validation

The reference decoder reads at most 65,537 bytes and rejects inputs above 65,536.
It accepts creation times no more than five seconds ahead or thirty seconds old.
It rejects unknown schema/status/animation enums, invalid WPM, non-finite or
negative aggregates and empty/oversized instance tokens.

For detailed statistics it additionally requires at most 90 days, valid calendar
dates in strict ascending order, no dates after `today.date`, and means no greater
than their corresponding peaks. Records must be finite and nonnegative. Unknown
additional JSON fields are ignored; the optional block preserves decoding of the
earlier schema-1 snapshots that did not include history.

A stale or rejected file must not leave values presented as current. A missing
file alone cannot distinguish QRY stopped, sharing off or a cleared cache.
The QRY reference decoder returns no snapshot for all these failures. An external
service should expose useful read states where there is sufficient evidence;
it must not invent a diagnosis from file absence.

The same-account file is not an authentication boundary against another local
process. The reader treats it as untrusted bounded input and does not execute
paths, URLs or commands from JSON.

## Opening QRY

QRY's bundle identifier is `app.typepulse.desktop`. It accepts only these
navigation links:

```text
qry://open/today
qry://open/statistics
qry://open/settings
```

They are user actions, not instructions from the snapshot. The bundled capsule
prefers `/Applications/QRY.app` and falls back to the registered URL handler.
External adapters should resolve QRY by bundle ID through `NSWorkspace`.

## Fixture and compatibility tests

The fixture contains invented values and a fixed clock. Decode it using
`now = generatedAtUnixMs / 1000` in tests; it is intentionally stale at real time.
Never copy it into the user's cache as a way to demonstrate real collection.

Test valid older snapshots without `statistics`, 90 valid dates, empty history,
absent records, duplicate/out-of-order dates, invalid values, unsupported schema,
oversized input, stale/future times and local-date rollover. Producer tests must
verify that active activity is merged once across session persistence.

An external integration should link the tested QRY revision and record the
minimum compatible producer version before claiming release compatibility. No public
network fetch or runtime QRY dependency download is part of the adapter.
