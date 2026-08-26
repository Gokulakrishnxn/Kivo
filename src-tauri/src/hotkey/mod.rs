//! Stage 1 — the trigger.
//!
//! The listener runs on its own thread with an OS-level hook. It must never
//! block: a slow handler here does not delay Kivo, it delays the user's entire
//! keyboard. So the handler does exactly one thing — push an event onto a
//! channel — and every decision is made on the far side of it.

use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Down,
    Up,
    /// Esc while recording: throw the audio away, insert nothing.
    Cancel,
}

/// A press shorter than this is a tap rather than a hold.
const TAP: Duration = Duration::from_millis(280);
/// Two taps inside this window latch recording on, for hands-free dictation.
const DOUBLE_TAP: Duration = Duration::from_millis(550);

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
/// Two gestures, both on the same key:
///   * **hold** — record while held, insert on release (push to talk)
///   * **tap twice** — latch recording on; a single tap stops it (hands free)
///
/// Esc cancels whatever is being recorded.
///
/// Returns only if the OS refuses the hook — on macOS that means Accessibility
/// permission has not been granted, which the caller surfaces as a real
/// instruction rather than a silent dead hotkey.
pub fn listen(key: HotKey, tx: Sender<Trigger>) -> Result<(), String> {
    let mut state = Gesture::default();

    rdev::listen(move |event| {
        let out = match event.event_type {
            // Modifier keys repeat while held; only edges are interesting.
            rdev::EventType::KeyPress(k) if key.matches(k) => state.press(Instant::now()),
            rdev::EventType::KeyRelease(k) if key.matches(k) => state.release(Instant::now()),
            rdev::EventType::KeyPress(rdev::Key::Escape) => state.escape(),
            _ => None,
        };
        if let Some(t) = out {
            let _ = tx.try_send(t);
        }
    })
    .map_err(|e| format!("{e:?}"))
}

/// The gesture state machine, separated from rdev so it can be tested without
/// an OS hook or a real keyboard.
#[derive(Default)]
struct Gesture {
    held: bool,
    latched: bool,
    pressed_at: Option<Instant>,
    last_tap: Option<Instant>,
}

impl Gesture {
    fn press(&mut self, now: Instant) -> Option<Trigger> {
        if self.held {
            return None; // key repeat
        }
        self.held = true;

        if self.latched {
            // A tap while latched stops the recording.
            self.latched = false;
            self.last_tap = None;
            return Some(Trigger::Up);
        }
        self.pressed_at = Some(now);
        Some(Trigger::Down)
    }

    fn release(&mut self, now: Instant) -> Option<Trigger> {
        if !self.held {
            return None;
        }
        self.held = false;

        if self.latched {
            return None; // stay recording
        }
        let was_tap = self
            .pressed_at
            .map(|p| now.duration_since(p) < TAP)
            .unwrap_or(false);

        if was_tap {
            let double = self
                .last_tap
                .map(|t| now.duration_since(t) < DOUBLE_TAP)
                .unwrap_or(false);
            if double {
                // Second tap: latch on and keep the microphone open.
                self.latched = true;
                self.last_tap = None;
                return None;
            }
            self.last_tap = Some(now);
        }
        Some(Trigger::Up)
    }

    fn escape(&mut self) -> Option<Trigger> {
        if self.held || self.latched {
            self.held = false;
            self.latched = false;
            self.last_tap = None;
            return Some(Trigger::Cancel);
        }
        None
    }
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

    fn at(ms: u64) -> Instant {
        // A fixed origin plus an offset, so tests are deterministic.
        *ORIGIN + Duration::from_millis(ms)
    }
    use std::sync::LazyLock;
    static ORIGIN: LazyLock<Instant> = LazyLock::new(Instant::now);

    #[test]
    fn holding_records_and_releasing_inserts() {
        let mut g = Gesture::default();
        assert_eq!(g.press(at(0)), Some(Trigger::Down));
        assert_eq!(g.release(at(1500)), Some(Trigger::Up));
    }

    #[test]
    fn key_repeat_while_held_emits_nothing_extra() {
        let mut g = Gesture::default();
        assert_eq!(g.press(at(0)), Some(Trigger::Down));
        assert_eq!(g.press(at(30)), None);
        assert_eq!(g.press(at(60)), None);
    }

    #[test]
    fn two_quick_taps_latch_recording_on() {
        let mut g = Gesture::default();
        g.press(at(0));
        assert_eq!(g.release(at(100)), Some(Trigger::Up)); // first tap ends normally
        assert_eq!(g.press(at(200)), Some(Trigger::Down));
        assert_eq!(
            g.release(at(300)),
            None,
            "second tap should latch, not stop"
        );
        assert!(g.latched);
    }

    #[test]
    fn a_tap_while_latched_stops_recording() {
        let mut g = Gesture::default();
        g.press(at(0));
        g.release(at(100));
        g.press(at(200));
        g.release(at(300)); // latched
        assert_eq!(g.press(at(4000)), Some(Trigger::Up));
        assert!(!g.latched);
    }

    #[test]
    fn taps_too_far_apart_do_not_latch() {
        let mut g = Gesture::default();
        g.press(at(0));
        g.release(at(100));
        g.press(at(2000));
        assert_eq!(g.release(at(2100)), Some(Trigger::Up));
        assert!(!g.latched);
    }

    #[test]
    fn a_long_hold_is_never_treated_as_a_tap() {
        let mut g = Gesture::default();
        g.press(at(0));
        g.release(at(900));
        g.press(at(1000));
        assert_eq!(g.release(at(1900)), Some(Trigger::Up));
        assert!(!g.latched);
    }

    #[test]
    fn escape_cancels_only_while_recording() {
        let mut g = Gesture::default();
        assert_eq!(g.escape(), None, "nothing to cancel when idle");
        g.press(at(0));
        assert_eq!(g.escape(), Some(Trigger::Cancel));
        assert!(!g.held && !g.latched);
    }
}
