//! A macOS keyboard event tap, written directly against CoreGraphics.
//!
//! This replaces `rdev`, which was silently delivering nothing inside the
//! packaged app while working fine from a terminal. Two reasons it does better:
//!
//! * **Session-level tap, not HID.** `rdev` hardcodes `kCGHIDEventTap`, the
//!   lowest and most privileged location. A session tap sees the same physical
//!   keystrokes with far fewer ways to be refused.
//! * **It re-enables itself.** macOS disables a tap that is slow to respond, or
//!   on certain user input, and never tells the application. `rdev` has no
//!   handling for this, so a tap that dies stays dead and the app just stops
//!   responding to its hotkey — with nothing in any log.
//!
//! Keys are matched on raw virtual keycodes rather than a translated enum,
//! and every modifier binding accepts either physical key (left or right) —
//! see `HotKey::trigger_codes` for why.

use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult, EventField,
};
use crossbeam_channel::Sender;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGEventTapEnable(tap: *mut std::ffi::c_void, enable: bool);
}

use super::{HotKey, Trigger};

/// macOS virtual keycodes. `kVK_*` in Carbon's Events.h.
pub mod keycode {
    pub const SPACE: i64 = 49;
    pub const ESCAPE: i64 = 53;
    pub const COMMAND_LEFT: i64 = 55;
    pub const COMMAND_RIGHT: i64 = 54;
    pub const SHIFT_LEFT: i64 = 56;
    pub const OPTION_LEFT: i64 = 58;
    pub const SHIFT_RIGHT: i64 = 60;
    pub const OPTION_RIGHT: i64 = 61;
    pub const CONTROL_RIGHT: i64 = 62;
    pub const FUNCTION: i64 = 63;
    pub const CONTROL_LEFT: i64 = 59;
}

impl HotKey {
    /// The keycode(s) that start and stop recording.
    ///
    /// Every modifier binding accepts EITHER physical key (left or right).
    /// Distinguishing them by side only ever produced confusion — a keyboard
    /// might not have a distinct right-hand key, or a user genuinely cannot
    /// tell which one macOS means — and there is no benefit to picking a side,
    /// since a bare modifier does nothing on its own regardless of which one
    /// it is.
    pub(crate) fn trigger_codes(&self) -> &'static [i64] {
        match self {
            HotKey::Fn => &[keycode::FUNCTION],
            HotKey::RightOption => &[keycode::OPTION_LEFT, keycode::OPTION_RIGHT],
            HotKey::RightCommand => &[keycode::COMMAND_LEFT, keycode::COMMAND_RIGHT],
            HotKey::RightControl => &[keycode::CONTROL_LEFT, keycode::CONTROL_RIGHT],
            HotKey::RightShift => &[keycode::SHIFT_LEFT, keycode::SHIFT_RIGHT],
            HotKey::OptionSpace | HotKey::ControlSpace | HotKey::ShiftSpace => &[keycode::SPACE],
        }
    }

    /// The modifier flag that must be held, for chord bindings.
    pub(crate) fn modifier_flag(&self) -> Option<CGEventFlags> {
        match self {
            HotKey::OptionSpace => Some(CGEventFlags::CGEventFlagAlternate),
            HotKey::ControlSpace => Some(CGEventFlags::CGEventFlagControl),
            HotKey::ShiftSpace => Some(CGEventFlags::CGEventFlagShift),
            _ => None,
        }
    }

    /// For a bare modifier, the flag whose presence means "held down".
    /// `flagsChanged` reports no up/down of its own — the flags say which.
    fn own_flag(&self) -> Option<CGEventFlags> {
        match self {
            HotKey::Fn => Some(CGEventFlags::CGEventFlagSecondaryFn),
            HotKey::RightOption => Some(CGEventFlags::CGEventFlagAlternate),
            HotKey::RightCommand => Some(CGEventFlags::CGEventFlagCommand),
            HotKey::RightControl => Some(CGEventFlags::CGEventFlagControl),
            HotKey::RightShift => Some(CGEventFlags::CGEventFlagShift),
            _ => None,
        }
    }
}

/// Listen forever. Returns only if the tap could not be created, which on
/// macOS means Accessibility permission was refused.
pub fn listen(key: HotKey, tx: Sender<Trigger>) -> Result<(), String> {
    let trigger = key.trigger_codes();
    let modifier = key.modifier_flag();
    let own = key.own_flag();

    // CGEventTap wants an `Fn`, so the gesture state needs interior mutability.
    // Exactly one thread — the tap's run loop — ever touches this, so the lock
    // is always uncontended and the callback never actually blocks.
    let state: Mutex<(super::Gesture, bool)> = Mutex::new((super::Gesture::default(), false));

    // The callback has to be able to switch the tap back on, but the tap does
    // not exist until after the callback is built. The port address is shared
    // through here and filled in once creation succeeds.
    let port_addr = Arc::new(AtomicUsize::new(0));
    let port_for_cb = port_addr.clone();

    let tap = CGEventTap::new(
        // Session, not HID: the same physical keys, far fewer refusals.
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        // Listen only — Kivo observes, it never swallows your keystrokes.
        CGEventTapOptions::ListenOnly,
        vec![
            CGEventType::KeyDown,
            CGEventType::KeyUp,
            CGEventType::FlagsChanged,
            CGEventType::TapDisabledByTimeout,
            CGEventType::TapDisabledByUserInput,
        ],
        move |_proxy, event_type, event: &CGEvent| {
            let code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            let flags = event.get_flags();
            let mut st = state.lock();
            let (gesture, down) = &mut *st;

            let out = match event_type {
                // macOS disabled us. Nothing arrives again until re-enabled,
                // and it never says so — this is why the hotkey would die
                // silently and stay dead.
                // macOS switches a tap off if it is slow to respond, or on
                // certain user input, and never tells the application. Left
                // alone the hotkey dies silently and stays dead — which is
                // precisely how this bug presented. Switch it back on.
                CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                    let addr = port_for_cb.load(Ordering::SeqCst);
                    if addr != 0 {
                        unsafe { CGEventTapEnable(addr as *mut std::ffi::c_void, true) };
                        tracing::warn!("event tap was disabled by the system; re-enabled it");
                    } else {
                        tracing::error!("event tap disabled before its port was recorded");
                    }
                    None
                }
                CGEventType::FlagsChanged => {
                    // A modifier reports no up/down; its flag being set means held.
                    if trigger.contains(&code) {
                        if let Some(f) = own {
                            let held = flags.contains(f);
                            if held && !*down {
                                *down = true;
                                gesture.press(std::time::Instant::now())
                            } else if !held && *down {
                                *down = false;
                                gesture.release(std::time::Instant::now())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else if let Some(m) = modifier {
                        // The modifier half of a chord was let go mid-recording.
                        if !flags.contains(m) {
                            gesture.break_chord()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                CGEventType::KeyDown => {
                    if code == keycode::ESCAPE {
                        gesture.escape()
                    } else if trigger.contains(&code) {
                        let armed = modifier.map(|m| flags.contains(m)).unwrap_or(true);
                        if armed && !*down {
                            *down = true;
                            gesture.press(std::time::Instant::now())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                CGEventType::KeyUp => {
                    if trigger.contains(&code) && *down {
                        *down = false;
                        gesture.release(std::time::Instant::now())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(t) = out {
                let _ = tx.try_send(t);
            }
            // Always pass the keystroke through: Kivo observes, never swallows.
            CallbackResult::Keep
        },
    )
    .map_err(|_| "could not create the keyboard event tap".to_string())?;

    let source = tap
        .mach_port()
        .create_runloop_source(0)
        .map_err(|_| "could not attach the event tap to a run loop".to_string())?;

    // Record the port before enabling, so a disable arriving immediately can
    // still be recovered from.
    {
        use core_foundation::base::TCFType;
        port_addr.store(
            tap.mach_port().as_concrete_TypeRef() as usize,
            Ordering::SeqCst,
        );
    }

    let run_loop = CFRunLoop::get_current();
    unsafe { run_loop.add_source(&source, kCFRunLoopCommonModes) };
    tap.enable();
    tracing::info!("event tap installed at session level and enabled");

    CFRunLoop::run_current();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_accepts_either_physical_key() {
        // The whole point of matching on raw keycodes: a keyboard might not
        // have a distinct right-hand key, and a user cannot always tell which
        // one macOS means. Both must trigger identically.
        assert_ne!(keycode::OPTION_LEFT, keycode::OPTION_RIGHT);
        let codes = HotKey::RightOption.trigger_codes();
        assert!(codes.contains(&keycode::OPTION_LEFT));
        assert!(codes.contains(&keycode::OPTION_RIGHT));
    }

    #[test]
    fn every_bare_modifier_accepts_both_sides() {
        for (k, left, right) in [
            (
                HotKey::RightCommand,
                keycode::COMMAND_LEFT,
                keycode::COMMAND_RIGHT,
            ),
            (
                HotKey::RightControl,
                keycode::CONTROL_LEFT,
                keycode::CONTROL_RIGHT,
            ),
            (HotKey::RightShift, keycode::SHIFT_LEFT, keycode::SHIFT_RIGHT),
        ] {
            let codes = k.trigger_codes();
            assert!(codes.contains(&left), "{} missing its left key", k.label());
            assert!(
                codes.contains(&right),
                "{} missing its right key",
                k.label()
            );
        }
    }

    #[test]
    fn fn_is_keycode_63_not_f13() {
        assert_eq!(HotKey::Fn.trigger_codes(), &[63]);
    }

    #[test]
    fn every_space_chord_carries_a_modifier_flag() {
        for k in [
            HotKey::OptionSpace,
            HotKey::ControlSpace,
            HotKey::ShiftSpace,
        ] {
            assert_eq!(k.trigger_codes(), &[keycode::SPACE]);
            assert!(k.modifier_flag().is_some());
        }
    }

    #[test]
    fn bare_modifiers_have_their_own_flag_and_no_chord() {
        for k in [
            HotKey::RightOption,
            HotKey::RightCommand,
            HotKey::RightShift,
            HotKey::Fn,
        ] {
            assert!(k.own_flag().is_some(), "{} needs a flag", k.label());
            assert!(k.modifier_flag().is_none());
        }
    }
}
