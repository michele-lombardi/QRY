//! Pure filtering of macOS keyboard metadata.
//!
//! Key identity is consumed only inside this private module and is never part
//! of the crate's public API.

use std::sync::atomic::{AtomicU16, AtomicU64, AtomicU8, Ordering};

use core_graphics::event::{CGEventFlags, CGKeyCode, KeyCode};

const NO_KEY: u16 = u16::MAX;
const MAX_IDENTICAL_RUN: u8 = 2;
const REPETITION_RESET_AFTER_MS: u64 = 1_000;

pub(super) struct RepetitionGuard {
    last_key: AtomicU16,
    last_event_ms: AtomicU64,
    identical_run: AtomicU8,
}

impl RepetitionGuard {
    pub(super) const fn new() -> Self {
        Self {
            last_key: AtomicU16::new(NO_KEY),
            last_event_ms: AtomicU64::new(0),
            identical_run: AtomicU8::new(0),
        }
    }

    pub(super) fn accepts(&self, key_code: CGKeyCode, elapsed_ms: u64) -> bool {
        let previous_key = self.last_key.load(Ordering::Relaxed);
        let previous_ms = self.last_event_ms.swap(elapsed_ms, Ordering::Relaxed);
        if previous_key != key_code
            || elapsed_ms.saturating_sub(previous_ms) >= REPETITION_RESET_AFTER_MS
        {
            self.last_key.store(key_code, Ordering::Relaxed);
            self.identical_run.store(1, Ordering::Relaxed);
            return true;
        }

        let run = self.identical_run.load(Ordering::Relaxed).saturating_add(1);
        self.identical_run.store(run, Ordering::Relaxed);
        run <= MAX_IDENTICAL_RUN
    }
}

pub(super) fn counts_as_typing(
    key_code: CGKeyCode,
    flags: CGEventFlags,
    is_auto_repeat: bool,
) -> bool {
    let blocked_modifiers = CGEventFlags::CGEventFlagCommand
        | CGEventFlags::CGEventFlagControl
        | CGEventFlags::CGEventFlagAlternate
        | CGEventFlags::CGEventFlagSecondaryFn;

    !is_auto_repeat && !flags.intersects(blocked_modifiers) && is_textual_key_position(key_code)
}

const fn is_textual_key_position(key_code: CGKeyCode) -> bool {
    matches!(
        key_code,
        0x00..=0x23
            | 0x25..=0x2F
            | KeyCode::SPACE
            | KeyCode::ANSI_GRAVE
            | KeyCode::ANSI_KEYPAD_DECIMAL
            | KeyCode::ANSI_KEYPAD_MULTIPLY
            | KeyCode::ANSI_KEYPAD_PLUS
            | KeyCode::ANSI_KEYPAD_DIVIDE
            | KeyCode::ANSI_KEYPAD_MINUS
            | KeyCode::ANSI_KEYPAD_EQUAL
            | KeyCode::ANSI_KEYPAD_0
            | KeyCode::ANSI_KEYPAD_1
            | KeyCode::ANSI_KEYPAD_2
            | KeyCode::ANSI_KEYPAD_3
            | KeyCode::ANSI_KEYPAD_4
            | KeyCode::ANSI_KEYPAD_5
            | KeyCode::ANSI_KEYPAD_6
            | KeyCode::ANSI_KEYPAD_7
            | KeyCode::ANSI_KEYPAD_8
            | KeyCode::ANSI_KEYPAD_9
            | KeyCode::JIS_YEN
            | KeyCode::JIS_UNDERSCORE
            | KeyCode::JIS_KEYPAD_COMMA
    )
}

#[cfg(test)]
mod tests {
    use core_graphics::event::{CGEventFlags, KeyCode};

    use super::{counts_as_typing, RepetitionGuard};

    #[test]
    fn accepts_letters_numbers_space_and_punctuation() {
        for key_code in [
            KeyCode::ANSI_A,
            KeyCode::ANSI_0,
            KeyCode::SPACE,
            KeyCode::ANSI_COMMA,
            KeyCode::ANSI_SLASH,
            KeyCode::ANSI_KEYPAD_7,
            KeyCode::ISO_SECTION,
        ] {
            assert!(counts_as_typing(key_code, CGEventFlags::empty(), false));
        }
    }

    #[test]
    fn accepts_shift_because_it_changes_written_characters() {
        assert!(counts_as_typing(
            KeyCode::ANSI_A,
            CGEventFlags::CGEventFlagShift,
            false
        ));
    }

    #[test]
    fn rejects_shortcut_modifiers() {
        for flags in [
            CGEventFlags::CGEventFlagCommand,
            CGEventFlags::CGEventFlagControl,
            CGEventFlags::CGEventFlagAlternate,
            CGEventFlags::CGEventFlagSecondaryFn,
        ] {
            assert!(!counts_as_typing(KeyCode::ANSI_A, flags, false));
        }
    }

    #[test]
    fn rejects_navigation_modifiers_and_control_keys() {
        for key_code in [
            KeyCode::RETURN,
            KeyCode::TAB,
            KeyCode::DELETE,
            KeyCode::ESCAPE,
            KeyCode::LEFT_ARROW,
            KeyCode::RIGHT_ARROW,
            KeyCode::UP_ARROW,
            KeyCode::DOWN_ARROW,
            KeyCode::F1,
            KeyCode::F12,
        ] {
            assert!(!counts_as_typing(key_code, CGEventFlags::empty(), false));
        }
    }

    #[test]
    fn rejects_operating_system_key_auto_repeat() {
        assert!(!counts_as_typing(
            KeyCode::ANSI_A,
            CGEventFlags::empty(),
            true
        ));
    }

    #[test]
    fn repetition_guard_allows_double_letters_but_stops_a_same_key_run() {
        let guard = RepetitionGuard::new();
        assert!(guard.accepts(KeyCode::ANSI_A, 0));
        assert!(guard.accepts(KeyCode::ANSI_A, 100));
        assert!(!guard.accepts(KeyCode::ANSI_A, 200));
        assert!(!guard.accepts(KeyCode::ANSI_A, 300));
        assert!(guard.accepts(KeyCode::ANSI_S, 400));
        assert!(guard.accepts(KeyCode::ANSI_A, 500));
        assert!(guard.accepts(KeyCode::ANSI_A, 1_500));
    }
}
