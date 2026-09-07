//! The menu bar presence.
//!
//! macOS menu bar icons are *template* images: black plus alpha, from which the
//! system derives the light and dark appearance and the highlighted state. Any
//! colour here would be discarded, which is why these are drawn monochrome
//! rather than reusing the app icon.
//!
//! While recording, the icon animates a small waveform. That is the only
//! feedback a user gets when dictating into a full-screen app where the overlay
//! may not be visible, so it matters more than it looks.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::image::Image;
use tauri::AppHandle;

pub const TRAY_ID: &str = "kivo";

const IDLE: &[u8] = include_bytes!("../icons/tray/idle.png");
const FRAMES: [&[u8]; 6] = [
    include_bytes!("../icons/tray/rec0.png"),
    include_bytes!("../icons/tray/rec1.png"),
    include_bytes!("../icons/tray/rec2.png"),
    include_bytes!("../icons/tray/rec3.png"),
    include_bytes!("../icons/tray/rec4.png"),
    include_bytes!("../icons/tray/rec5.png"),
];

/// Roughly nine frames a second — fast enough to read as live, slow enough
/// that it never looks frantic and never busies the CPU.
const FRAME_MS: u64 = 110;

#[derive(Default)]
pub struct Animator {
    running: Arc<AtomicBool>,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Begin animating. Safe to call repeatedly; only one loop ever runs.
    ///
    /// The ticking itself lives on a background thread — nothing here can be
    /// allowed to compete with the audio/recognition pipeline for CPU — but
    /// every `NSStatusItem` icon update is handed to `run_on_main_thread`.
    /// AppKit's view and layer trees are not thread-safe, and mutating them
    /// from a background thread does not reliably fail loudly: it can instead
    /// leave a window's compositor state corrupted, which is what a stray
    /// off-main-thread call here or in the overlay looked like from the
    /// outside — a settings window that painted solid black.
    pub fn start(&self, app: &AppHandle) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let running = self.running.clone();
        let app = app.clone();

        std::thread::spawn(move || {
            let mut i = 0usize;
            while running.load(Ordering::SeqCst) {
                let frame = FRAMES[i % FRAMES.len()];
                let for_main = app.clone();
                let _ = app.run_on_main_thread(move || {
                    if let Some(tray) = for_main.tray_by_id(TRAY_ID) {
                        if let Ok(img) = Image::from_bytes(frame) {
                            let _ = tray.set_icon(Some(img));
                        }
                    }
                });
                i = i.wrapping_add(1);
                std::thread::sleep(Duration::from_millis(FRAME_MS));
            }
            // Always land back on the resting icon, even if the loop exits
            // because recording failed rather than finished.
            set_idle(&app);
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Restore the resting glyph. Safe to call from any thread — the actual
/// `NSStatusItem` mutation is always dispatched to the main thread.
pub fn set_idle(app: &AppHandle) {
    let for_main = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(tray) = for_main.tray_by_id(TRAY_ID) {
            if let Ok(img) = Image::from_bytes(IDLE) {
                let _ = tray.set_icon(Some(img));
            }
        }
    });
}

pub fn idle_image() -> tauri::Result<Image<'static>> {
    Image::from_bytes(IDLE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_frame_is_a_real_png() {
        for (i, f) in FRAMES.iter().enumerate() {
            assert!(
                f.starts_with(&[0x89, b'P', b'N', b'G']),
                "frame {i} is not a PNG"
            );
        }
        assert!(IDLE.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn the_animation_is_cheap_enough_to_leave_running() {
        // Nine repaints a second is the budget; anything faster is decoration
        // paid for out of the user's battery.
        assert!(
            FRAME_MS >= 80,
            "frame interval {FRAME_MS}ms is too aggressive"
        );
    }

    #[test]
    fn stopping_an_unstarted_animator_is_harmless() {
        let a = Animator::new();
        a.stop();
        assert!(!a.running.load(Ordering::SeqCst));
    }
}
