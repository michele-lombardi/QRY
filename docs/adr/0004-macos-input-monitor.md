# ADR 0004 — Monitor globale macOS

- Data: 5 agosto 2026
- Stato: accettata per la V1 macOS

## Contesto

TypePulse deve riconoscere attività di scrittura anche quando un'altra app è in
primo piano. Il percorso è sensibile: un'API capace di osservare la tastiera può
diventare accidentalmente un keylogger se identità dei tasti, testo o metadati
escono dall'adapter.

Lo spike ha confrontato due opzioni:

1. bridge Swift/Objective-C richiamato da Rust;
2. binding Rust diretti alle API Core Graphics e Core Foundation.

Le API necessarie sono disponibili nei crate correnti senza introdurre un
secondo toolchain applicativo.

## Decisione

- usare `core-graphics` per `CGEventTap` e integrazione con `CFRunLoop`;
- usare `objc2-core-graphics` per `CGPreflightListenEventAccess` e
  `CGRequestListenEventAccess`;
- non introdurre un bridge Swift nella Fase B;
- creare un tap `Session`, `HeadInsertEventTap`, `ListenOnly`, interessato a
  `KeyDown`;
- eseguire tap e run loop su un thread nominato dedicato;
- filtrare key code e flag in un modulo privato dell'adapter;
- produrre all'esterno soltanto `TypingActivity { occurred_at: Instant }`;
- usare un canale bounded con `try_send`: il callback non attende mai il
  consumer;
- misurare conteggio, drop, riattivazioni e durata callback con atomiche;
- verificare periodicamente la revoca del permesso e terminare in stato
  `permission-revoked`;
- riabilitare il tap se macOS invia `TapDisabledByTimeout` o
  `TapDisabledByUserInput`;
- usare macOS 10.15 come versione minima, coerente con le API di consenso
  adottate.

Il deep link delle impostazioni è
`x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent`.
L'app può aprirlo, ma non può concedere il permesso per conto dell'utente.

## Perché `Session` e `ListenOnly`

Un tap HID richiede privilegi più elevati e non è necessario per il prodotto.
`ListenOnly` rende esplicito che TypePulse non modifica, sopprime o sostituisce
eventi. Il callback restituisce sempre l'evento invariato.

## Comportamento con Secure Input

Quando macOS o un'app protegge un campo con Secure Input, alcuni eventi possono
non essere osservabili. TypePulse accetta il buco nel conteggio, non tenta di
aggirarlo, non chiede privilegi aggiuntivi e non simula attività. È una
limitazione intenzionale a tutela dell'utente.

## Conseguenze

Positive:

- un solo linguaggio di sistema nella V1;
- callback corto, senza I/O, log, WebView o allocazioni non necessarie;
- impossibilità per core e frontend di ricevere il key code tramite l'API
  pubblica;
- adapter sostituibile da una futura implementazione Linux.

Trade-off:

- il conteggio può perdere attività durante Secure Input;
- un consumer lento produce drop espliciti invece di bloccare il sistema;
- ogni nuova build non firmata può richiedere una nuova conferma TCC;
- revoca/ripristino e Secure Input richiedono test manuali su un Mac con
  consenso dell'utente.

## Riferimenti

- [Apple — CGEvent tap creation](https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate%28tap%3Aplace%3Aoptions%3Aeventsofinterest%3Acallback%3Auserinfo%3A%29?language=objc)
- [Apple — CGEventTapOptions](https://developer.apple.com/documentation/coregraphics/cgeventtapoptions)
- [Apple — CGPreflightListenEventAccess](https://developer.apple.com/documentation/coregraphics/cgpreflightlisteneventaccess%28%29?language=objc)
