//! Stage 1 — the trigger.
//!
//! The listener runs on its own thread with an OS-level hook. It must never
//! block: a slow handler here does not delay Kivo, it delays the user's entire
//! keyboard. So the handler does exactly one thing — push an event onto a
//! channel — and every decision is made on the far side of it.

use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Down,
    Up,
}

/// Which physical key holds the microphone open.
///
/// A bare modifier is the ergonomic choice: it is reachable without a chord and
/// it cannot collide with an application shortcut, because on its own it does
/// nothing. Right-hand modifiers are preferred since they are rarely bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HotKey {
    #[default]
    RightOption,
    RightCommand,
    RightControl,
    RightShift,
    F13,
}

impl HotKey {
    pub fn label(&self) -> &'static str {
        match self {
            HotKey::RightOption => "Right Option",
            HotKey::RightCommand => "Right Command",
            HotKey::RightControl => "Right Control",
            HotKey::RightShift => "Right Shift",
            HotKey::F13 => "F13",
        }
    }

    fn matches(&self, key: rdev::Key) -> bool {
        use rdev::Key as K;
        matches!(
            (self, key),
            (HotKey::RightOption, K::AltGr)
                | (HotKey::RightCommand, K::MetaRight)
                | (HotKey::RightControl, K::ControlRight)
                | (HotKey::RightShift, K::ShiftRight)
                | (HotKey::F13, K::Function)
        )
    }

    pub fn all() -> &'static [HotKey] {
        &[
            HotKey::RightOption,
            HotKey::RightCommand,
            HotKey::RightControl,
            HotKey::RightShift,
            HotKey::F13,
        ]
    }
}

/// Start listening. Blocks its own thread forever, so spawn it.
///
/// Returns only if the OS refuses the hook — on macOS that means Accessibility
/// permission has not been granted, which the caller surfaces as a real
/// instruction rather than a silent dead hotkey.
pub fn listen(key: HotKey, tx: Sender<Trigger>) -> Result<(), String> {
    // Modifier keys repeat while held. Only the edges are interesting.
    let mut held = false;

    rdev::listen(move |event| match event.event_type {
        rdev::EventType::KeyPress(k) if key.matches(k) && !held => {
            held = true;
            let _ = tx.try_send(Trigger::Down);
        }
        rdev::EventType::KeyRelease(k) if key.matches(k) && held => {
            held = false;
            let _ = tx.try_send(Trigger::Up);
        }
        _ => {}
    })
    .map_err(|e| format!("{e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hotkey_maps_to_exactly_one_physical_key() {
        use rdev::Key as K;
        let keys = [
            K::AltGr,
            K::MetaRight,
            K::ControlRight,
            K::ShiftRight,
            K::Function,
        ];
        for hk in HotKey::all() {
            let hits = keys.iter().filter(|k| hk.matches(**k)).count();
            assert_eq!(hits, 1, "{} matched {hits} keys", hk.label());
        }
    }

    #[test]
    fn an_unrelated_key_never_triggers() {
        assert!(!HotKey::RightOption.matches(rdev::Key::KeyA));
    }
}
