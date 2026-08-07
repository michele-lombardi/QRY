//! Pure filtering of private Windows Raw Input keyboard metadata.
//!
//! Virtual keys, scan codes, modifier state, and pressed-key identity never
//! leave this module. Its only output answers whether one native event should
//! become a privacy-safe `TypingActivity`.

use std::collections::HashSet;

const RI_KEY_BREAK: u16 = 0x0001;
const RI_KEY_E0: u16 = 0x0002;
const RI_KEY_E1: u16 = 0x0004;

const VK_CONTROL: u16 = 0x11;
const VK_MENU: u16 = 0x12;
const VK_SPACE: u16 = 0x20;
const VK_0: u16 = 0x30;
const VK_9: u16 = 0x39;
const VK_A: u16 = 0x41;
const VK_Z: u16 = 0x5A;
const VK_LWIN: u16 = 0x5B;
const VK_RWIN: u16 = 0x5C;
const VK_NUMPAD0: u16 = 0x60;
const VK_DIVIDE: u16 = 0x6F;
const VK_LCONTROL: u16 = 0xA2;
const VK_RCONTROL: u16 = 0xA3;
const VK_LMENU: u16 = 0xA4;
const VK_RMENU: u16 = 0xA5;
const VK_OEM_1: u16 = 0xBA;
const VK_OEM_8: u16 = 0xDF;
const VK_OEM_102: u16 = 0xE2;

const MAX_IDENTICAL_RUN: u8 = 2;
const REPETITION_RESET_AFTER_MS: u64 = 1_000;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct KeyIdentity(u32);

impl KeyIdentity {
    fn from_native(make_code: u16, flags: u16, virtual_key: u16) -> Self {
        let extension = flags & (RI_KEY_E0 | RI_KEY_E1);
        Self(u32::from(make_code) | (u32::from(extension) << 16) | (u32::from(virtual_key) << 20))
    }
}

#[derive(Default)]
struct RepetitionGuard {
    last_key: Option<KeyIdentity>,
    last_event_ms: u64,
    identical_run: u8,
}

impl RepetitionGuard {
    fn accepts(&mut self, key: KeyIdentity, elapsed_ms: u64) -> bool {
        if self.last_key != Some(key)
            || elapsed_ms.saturating_sub(self.last_event_ms) >= REPETITION_RESET_AFTER_MS
        {
            self.last_key = Some(key);
            self.last_event_ms = elapsed_ms;
            self.identical_run = 1;
            return true;
        }

        self.last_event_ms = elapsed_ms;
        self.identical_run = self.identical_run.saturating_add(1);
        self.identical_run <= MAX_IDENTICAL_RUN
    }
}

#[derive(Default)]
struct ModifierState {
    control: HashSet<KeyIdentity>,
    alt: HashSet<KeyIdentity>,
    right_alt: HashSet<KeyIdentity>,
    windows: HashSet<KeyIdentity>,
}

impl ModifierState {
    fn update(&mut self, key: KeyIdentity, virtual_key: u16, flags: u16, pressed: bool) {
        let target = match virtual_key {
            VK_CONTROL | VK_LCONTROL | VK_RCONTROL => Some(&mut self.control),
            VK_MENU | VK_LMENU | VK_RMENU => Some(&mut self.alt),
            VK_LWIN | VK_RWIN => Some(&mut self.windows),
            _ => None,
        };
        if let Some(target) = target {
            if pressed {
                target.insert(key);
            } else {
                target.remove(&key);
            }
        }

        if is_right_alt(virtual_key, flags) {
            if pressed {
                self.right_alt.insert(key);
            } else {
                self.right_alt.remove(&key);
            }
        }
    }

    fn blocks_typing(&self) -> bool {
        if !self.windows.is_empty() {
            return true;
        }

        let altgr = !self.control.is_empty()
            && !self.right_alt.is_empty()
            && self.alt.len() == self.right_alt.len();
        (!self.control.is_empty() || !self.alt.is_empty()) && !altgr
    }
}

/// Stateful, thread-confined classifier for one Raw Input worker.
#[derive(Default)]
pub(super) struct WindowsEventFilter {
    pressed: HashSet<KeyIdentity>,
    modifiers: ModifierState,
    repetition: RepetitionGuard,
}

impl WindowsEventFilter {
    /// Observes one raw keyboard make/break event.
    pub(super) fn accepts(
        &mut self,
        make_code: u16,
        flags: u16,
        virtual_key: u16,
        elapsed_ms: u64,
    ) -> bool {
        let key = KeyIdentity::from_native(make_code, flags, virtual_key);
        let released = flags & RI_KEY_BREAK != 0;
        if released {
            self.modifiers.update(key, virtual_key, flags, false);
            self.pressed.remove(&key);
            return false;
        }

        let auto_repeat = !self.pressed.insert(key);
        self.modifiers.update(key, virtual_key, flags, true);
        !auto_repeat
            && !self.modifiers.blocks_typing()
            && is_textual_virtual_key(virtual_key)
            && self.repetition.accepts(key, elapsed_ms)
    }
}

const fn is_right_alt(virtual_key: u16, flags: u16) -> bool {
    virtual_key == VK_RMENU || (virtual_key == VK_MENU && flags & RI_KEY_E0 != 0)
}

const fn is_textual_virtual_key(virtual_key: u16) -> bool {
    matches!(
        virtual_key,
        VK_0..=VK_9
            | VK_A..=VK_Z
            | VK_SPACE
            | VK_NUMPAD0..=VK_DIVIDE
            | VK_OEM_1..=VK_OEM_8
            | VK_OEM_102
    )
}

#[cfg(test)]
mod tests {
    use super::{
        WindowsEventFilter, RI_KEY_BREAK, RI_KEY_E0, VK_A, VK_CONTROL, VK_LWIN, VK_MENU, VK_OEM_1,
        VK_RMENU, VK_SPACE,
    };

    const VK_BACK: u16 = 0x08;
    const VK_TAB: u16 = 0x09;
    const VK_RETURN: u16 = 0x0D;
    const VK_SHIFT: u16 = 0x10;
    const VK_ESCAPE: u16 = 0x1B;
    const VK_PRIOR: u16 = 0x21;
    const VK_DOWN: u16 = 0x28;
    const VK_INSERT: u16 = 0x2D;
    const VK_DELETE: u16 = 0x2E;
    const VK_F1: u16 = 0x70;
    const VK_F24: u16 = 0x87;

    #[test]
    fn accepts_text_positions_and_shifted_text() {
        let mut filter = WindowsEventFilter::default();
        assert!(filter.accepts(0x1E, 0, VK_A, 0));
        assert!(!filter.accepts(0x1E, RI_KEY_BREAK, VK_A, 10));
        assert!(!filter.accepts(0x2A, 0, VK_SHIFT, 20));
        assert!(filter.accepts(0x27, 0, VK_OEM_1, 30));
        assert!(filter.accepts(0x39, 0, VK_SPACE, 40));
    }

    #[test]
    fn rejects_control_alt_windows_and_non_text_keys() {
        for blocked_modifier in [VK_CONTROL, VK_MENU, VK_LWIN] {
            let mut filter = WindowsEventFilter::default();
            assert!(!filter.accepts(0x1D, 0, blocked_modifier, 0));
            assert!(!filter.accepts(0x1E, 0, VK_A, 10));
        }

        for key in [
            VK_BACK, VK_TAB, VK_RETURN, VK_ESCAPE, VK_PRIOR, VK_DOWN, VK_INSERT, VK_DELETE, VK_F1,
            VK_F24,
        ] {
            let mut filter = WindowsEventFilter::default();
            assert!(!filter.accepts(key, 0, key, 0));
        }
    }

    #[test]
    fn altgr_allows_text_without_counting_the_modifiers() {
        let mut filter = WindowsEventFilter::default();
        assert!(!filter.accepts(0x1D, 0, VK_CONTROL, 0));
        assert!(!filter.accepts(0x38, RI_KEY_E0, VK_RMENU, 10));
        assert!(filter.accepts(0x12, 0, VK_A, 20));
    }

    #[test]
    fn drops_os_repeat_and_long_identical_runs_but_allows_double_letters() {
        let mut filter = WindowsEventFilter::default();
        assert!(filter.accepts(0x1E, 0, VK_A, 0));
        assert!(!filter.accepts(0x1E, 0, VK_A, 50));
        assert!(!filter.accepts(0x1E, RI_KEY_BREAK, VK_A, 60));
        assert!(filter.accepts(0x1E, 0, VK_A, 100));
        assert!(!filter.accepts(0x1E, RI_KEY_BREAK, VK_A, 110));
        assert!(!filter.accepts(0x1E, 0, VK_A, 200));
        assert!(!filter.accepts(0x1E, RI_KEY_BREAK, VK_A, 210));
        assert!(filter.accepts(0x1E, 0, VK_A, 1_300));
    }
}
