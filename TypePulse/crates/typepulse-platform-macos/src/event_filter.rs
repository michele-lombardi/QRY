//! Pure filtering of macOS keyboard metadata.
//!
//! Key identity is consumed only inside this private module and is never part
//! of the crate's public API.

use core_graphics::event::{CGEventFlags, CGKeyCode, KeyCode};

pub(super) fn counts_as_typing(key_code: CGKeyCode, flags: CGEventFlags) -> bool {
    let blocked_modifiers = CGEventFlags::CGEventFlagCommand
        | CGEventFlags::CGEventFlagControl
        | CGEventFlags::CGEventFlagAlternate
        | CGEventFlags::CGEventFlagSecondaryFn;

    !flags.intersects(blocked_modifiers) && is_textual_key_position(key_code)
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

    use super::counts_as_typing;

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
            assert!(counts_as_typing(key_code, CGEventFlags::empty()));
        }
    }

    #[test]
    fn accepts_shift_because_it_changes_written_characters() {
        assert!(counts_as_typing(
            KeyCode::ANSI_A,
            CGEventFlags::CGEventFlagShift
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
            assert!(!counts_as_typing(KeyCode::ANSI_A, flags));
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
            assert!(!counts_as_typing(key_code, CGEventFlags::empty()));
        }
    }
}
