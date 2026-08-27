//! Detecting keys that macOS has already claimed.
//!
//! A hotkey that the system intercepts first is indistinguishable, from the
//! user's side, from Kivo being broken: you press the key, something else
//! happens, and nothing in Kivo reacts. The Fn key is the common case — recent
//! macOS defaults it to the emoji picker — so Kivo checks rather than leaving
//! someone to work it out.

use super::HotKey;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Conflict {
    /// What macOS currently does with the key.
    pub taken_by: String,
    /// Plain instruction for resolving it.
    pub fix: String,
    /// The System Settings pane that resolves it, for a deep link.
    pub pane: String,
}

/// What `AppleFnUsageType` means. Unset behaves as "Show Emoji & Symbols" on
/// recent macOS, which is exactly the case that bites people.
fn fn_usage() -> Option<i64> {
    let out = std::process::Command::new("defaults")
        .args(["read", "com.apple.HIToolbox", "AppleFnUsageType"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None; // unset
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

const KEYBOARD_PANE: &str = "x-apple.systempreferences:com.apple.Keyboard-Settings.extension";
const INPUT_PANE: &str = "x-apple.systempreferences:com.apple.Keyboard-Settings.extension";

/// Check whether the chosen binding is already claimed by the system.
pub fn check(key: HotKey) -> Option<Conflict> {
    match key {
        HotKey::Fn => {
            let taken_by = match fn_usage() {
                Some(0) => return None, // "Do Nothing" — Kivo gets the key
                Some(1) => "changing your input source",
                Some(3) => "starting macOS dictation",
                // Unset defaults to the emoji picker on recent macOS.
                Some(2) | None => "opening the emoji picker",
                Some(_) => "something else",
            };
            Some(Conflict {
                taken_by: format!(
                    "macOS is using the Fn key for {taken_by}, so Kivo never sees it."
                ),
                fix: "Open System Settings › Keyboard, then set “Press 🌐 key to” to \
                      “Do Nothing”. Or pick a different key below — Right Option has \
                      no conflicts."
                    .into(),
                pane: KEYBOARD_PANE.into(),
            })
        }
        HotKey::ControlSpace => Some(Conflict {
            taken_by: "macOS often uses Control + Space to switch input sources.".into(),
            fix: "If dictation does not start, clear that shortcut in System Settings › \
                  Keyboard › Keyboard Shortcuts › Input Sources, or pick a different key."
                .into(),
            pane: INPUT_PANE.into(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_modifiers_are_never_reported_as_conflicting() {
        for k in [
            HotKey::RightOption,
            HotKey::RightCommand,
            HotKey::RightControl,
            HotKey::RightShift,
        ] {
            assert!(check(k).is_none(), "{} should be clear", k.label());
        }
    }

    #[test]
    fn control_space_always_warns() {
        // This one does not depend on a readable preference, so it is stable.
        assert!(check(HotKey::ControlSpace).is_some());
    }

    #[test]
    fn a_conflict_always_names_a_pane_to_open() {
        if let Some(c) = check(HotKey::ControlSpace) {
            assert!(c.pane.starts_with("x-apple.systempreferences:"));
            assert!(!c.fix.is_empty());
        }
    }
}
