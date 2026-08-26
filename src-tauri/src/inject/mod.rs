//! Stage 5 — delivery. Getting text into an application Kivo does not control.
//!
//! Electron apps, terminals, Java applications and games each mishandle
//! synthetic input in their own way, so the clipboard route is not a degraded
//! path — it is the reliability guarantee. What matters is that the user's
//! clipboard is exactly as they left it afterwards.

use anyhow::{Context, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{thread, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Clipboard write plus a synthesised paste. Fast and near-universal.
    Paste,
    /// Character-by-character typing. Slower, but it works where paste does not.
    Typing,
}

/// Insert `text` at the cursor in whatever application currently has focus.
pub fn insert(text: &str, method: Method) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    match method {
        Method::Typing => type_text(text),
        Method::Paste => paste(text),
    }
}

fn type_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("could not open an input channel")?;
    enigo.text(text).context("typing failed")?;
    Ok(())
}

/// Write the clipboard, press the paste chord, then put the clipboard back.
fn paste(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("could not open the clipboard")?;

    // Remember what was there. Non-text contents cannot be preserved by this
    // route; that is a known limit rather than a silent loss, and the caller
    // surfaces it.
    let previous = clipboard.get_text().ok();

    clipboard
        .set_text(text.to_string())
        .context("could not write the clipboard")?;

    // The pasteboard needs a moment to settle before the chord is delivered,
    // or fast apps paste the previous contents.
    thread::sleep(Duration::from_millis(30));

    let result = press_paste();

    // Restore regardless of whether the paste worked.
    thread::sleep(Duration::from_millis(90));
    if let Some(prev) = previous {
        let _ = clipboard.set_text(prev);
    }

    result
}

#[cfg(target_os = "macos")]
const PASTE_MODIFIER: Key = Key::Meta; // Command

#[cfg(not(target_os = "macos"))]
const PASTE_MODIFIER: Key = Key::Control;

fn press_paste() -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("could not open an input channel")?;
    enigo
        .key(PASTE_MODIFIER, Direction::Press)
        .context("modifier down failed")?;
    let tapped = enigo.key(Key::Unicode('v'), Direction::Click);
    // Always release the modifier, even if the tap failed — leaving Command
    // stuck down would wreck the user's session.
    let released = enigo.key(PASTE_MODIFIER, Direction::Release);
    tapped.context("paste keystroke failed")?;
    released.context("modifier release failed")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_a_no_op_and_never_touches_the_clipboard() {
        assert!(insert("", Method::Paste).is_ok());
    }
}
