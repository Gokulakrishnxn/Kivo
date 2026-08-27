//! Stage 1 — the trigger.
//!
//! The listener runs on its own thread with an OS-level hook. It must never
//! block: a slow handler here does not delay Kivo, it delays the user's entire
//! keyboard. So the handler does exactly one thing — push an event onto a
//! channel — and every decision is made on the far side of it.

pub mod conflict;
pub mod trust;

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

/// Which key holds the microphone open.
///
/// Two shapes are offered, because neither suits everyone:
///
/// * **A bare modifier** (Fn, Right Option…) — one key, nothing to chord, and
///   it cannot collide with an application shortcut because on its own it does
///   nothing. This is the default.
/// * **A chord ending in Space** — for people who want Space as the trigger.
///
/// Space *on its own* is deliberately not offered. Kivo observes the keyboard
/// rather than intercepting it, so it cannot swallow the keystroke: a bare
/// Space trigger would type a space into whatever you are working in every
/// time you started dictating, and every space you typed would start a
/// recording. A chord has neither problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HotKey {
    /// The globe / Fn key. Ergonomically the best of these, but see `note()`.
    Fn,
    #[default]
    RightOption,
    RightCommand,
    RightControl,
    RightShift,
    OptionSpace,
    ControlSpace,
    ShiftSpace,
}

impl HotKey {
    pub fn label(&self) -> &'static str {
        match self {
            HotKey::Fn => "Fn (globe)",
            HotKey::RightOption => "Right Option",
            HotKey::RightCommand => "Right Command",
            HotKey::RightControl => "Right Control",
            HotKey::RightShift => "Right Shift",
            HotKey::OptionSpace => "Option + Space",
            HotKey::ControlSpace => "Control + Space",
            HotKey::ShiftSpace => "Shift + Space",
        }
    }

    /// A caveat worth showing next to the choice, or none.
    pub fn note(&self) -> Option<&'static str> {
        match self {
            HotKey::Fn => Some(
                "macOS may already use Fn for dictation or emoji. Set                  System Settings › Keyboard › “Press 🌐 key to” to “Do Nothing”.",
            ),
            HotKey::ControlSpace => Some(
                "macOS often uses Control + Space to switch input sources.",
            ),
            _ => None,
        }
    }

    /// The modifier that must be held, for chord bindings.
    fn modifier(&self) -> Option<&'static [rdev::Key]> {
        use rdev::Key as K;
        match self {
            HotKey::OptionSpace => Some(&[K::Alt, K::AltGr]),
            HotKey::ControlSpace => Some(&[K::ControlLeft, K::ControlRight]),
            HotKey::ShiftSpace => Some(&[K::ShiftLeft, K::ShiftRight]),
            _ => None,
        }
    }

    /// The key that actually starts and stops recording.
    fn trigger(&self) -> rdev::Key {
        use rdev::Key as K;
        match self {
            // rdev maps Key::Function to macOS keycode 63, which is the real
            // Fn/globe key — not F13.
            HotKey::Fn => K::Function,
            HotKey::RightOption => K::AltGr,
            HotKey::RightCommand => K::MetaRight,
            HotKey::RightControl => K::ControlRight,
            HotKey::RightShift => K::ShiftRight,
            HotKey::OptionSpace | HotKey::ControlSpace | HotKey::ShiftSpace => K::Space,
        }
    }

    pub fn all() -> &'static [HotKey] {
        &[
            HotKey::RightOption,
            HotKey::Fn,
            HotKey::RightCommand,
            HotKey::RightControl,
            HotKey::RightShift,
            HotKey::OptionSpace,
            HotKey::ControlSpace,
            HotKey::ShiftSpace,
        ]
    }
}

/// Start listening. Blocks its own thread forever, so spawn it.
///
/// Two gestures, on whichever binding is configured:
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
    let trigger = key.trigger();
    let modifier = key.modifier();
    // Which of the accepted modifier keys are currently down. A chord binding
    // only fires while at least one of them is held.
    let mut mods_held: Vec<rdev::Key> = Vec::new();

    rdev::listen(move |event| {
        let mut out = None;

        // At debug this prints every key the OS hands us, which is the only way
        // to tell "the hook is dead" apart from "the binding never matches".
        match event.event_type {
            rdev::EventType::KeyPress(k) => tracing::debug!("key down: {k:?}"),
            rdev::EventType::KeyRelease(k) => tracing::debug!("key up:   {k:?}"),
            _ => {}
        }

        match event.event_type {
            rdev::EventType::KeyPress(k) => {
                if let Some(accepted) = modifier {
                    if accepted.contains(&k) && !mods_held.contains(&k) {
                        mods_held.push(k);
                    }
                }
                if k == trigger {
                    let armed = modifier.is_none() || !mods_held.is_empty();
                    if armed {
                        out = state.press(Instant::now());
                    }
                }
            }
            rdev::EventType::KeyRelease(k) => {
                if k == trigger {
                    out = state.release(Instant::now());
                }
                if let Some(accepted) = modifier {
                    if accepted.contains(&k) {
                        mods_held.retain(|h| *h != k);
                        // Letting go of the modifier ends the chord. Without
                        // this, releasing Option before Space would leave the
                        // microphone open with no obvious way to close it.
                        if mods_held.is_empty() {
                            if let Some(t) = state.break_chord() {
                                out = Some(t);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if let rdev::EventType::KeyPress(rdev::Key::Escape) = event.event_type {
            if let Some(t) = state.escape() {
                out = Some(t);
            }
        }

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

    /// The modifier half of a chord was released.
    fn break_chord(&mut self) -> Option<Trigger> {
        if self.held || self.latched {
            self.held = false;
            self.latched = false;
            self.last_tap = None;
            return Some(Trigger::Up);
        }
        None
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
