//! Kivo — turns voice into clean text.
//!
//! The Rust side owns the whole pipeline: hotkey, capture, recognition,
//! cleanup and delivery. The webview owns only settings, onboarding and the
//! recording overlay, which is why Kivo can idle where an Electron equivalent
//! starts.

pub mod asr;
pub mod audio;
pub mod clean;
pub mod config;
pub mod hotkey;
pub mod inject;
pub mod pipeline;

use std::sync::Arc;

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, WindowEvent,
};

use crate::clean::{CleanConfig, Mode};
use crate::config::Config;
use crate::hotkey::{HotKey, Trigger};
use crate::pipeline::{Entry, Kivo, State};

/// Everything the interface needs to render itself in one round trip.
#[derive(Serialize)]
struct Snapshot {
    state: State,
    config: Config,
    model: String,
    model_installed: bool,
    model_dir: String,
    history: Vec<Entry>,
    error: Option<String>,
    hotkeys: Vec<HotKeyOption>,
    modes: Vec<ModeOption>,
}

#[derive(Serialize)]
struct HotKeyOption {
    value: HotKey,
    label: &'static str,
}

#[derive(Serialize)]
struct ModeOption {
    value: Mode,
    label: &'static str,
}

#[tauri::command]
fn snapshot(app: tauri::State<'_, Arc<Kivo>>) -> Snapshot {
    let dir = asr::parakeet::Parakeet::default_dir();
    Snapshot {
        state: *app.state.lock(),
        config: app.config.lock().clone(),
        model: app.describe_model(),
        model_installed: asr::parakeet::Parakeet::is_installed(&dir),
        model_dir: dir.to_string_lossy().into_owned(),
        history: app.history.lock().clone(),
        error: app.last_error.lock().clone(),
        hotkeys: HotKey::all()
            .iter()
            .map(|h| HotKeyOption {
                value: *h,
                label: h.label(),
            })
            .collect(),
        modes: [
            Mode::Standard,
            Mode::Message,
            Mode::Email,
            Mode::Notes,
            Mode::Code,
            Mode::Raw,
        ]
        .iter()
        .map(|m| ModeOption {
            value: *m,
            label: m.label(),
        })
        .collect(),
    }
}

#[tauri::command]
fn save_config(app: tauri::State<'_, Arc<Kivo>>, config: Config) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())?;
    *app.config.lock() = config;
    Ok(())
}

/// Let settings preview the cleanup on sample text without speaking. This is
/// how a user decides whether to trust a rule before turning it loose.
#[tauri::command]
fn preview_clean(text: String, config: Config) -> String {
    clean::clean(&text, &config.cleaning())
}

#[tauri::command]
fn clear_history(app: tauri::State<'_, Arc<Kivo>>) {
    app.history.lock().clear();
}

#[tauri::command]
fn dismiss_error(app: tauri::State<'_, Arc<Kivo>>) {
    *app.last_error.lock() = None;
}

#[tauri::command]
fn reload_model(app: tauri::State<'_, Arc<Kivo>>) -> Result<String, String> {
    app.reload_model().map_err(|e| format!("{e:#}"))
}

/// The worked example from the PRD, so onboarding can demonstrate the product
/// before the microphone is ever opened.
#[tauri::command]
fn demo() -> serde_json::Value {
    let said = "Um, so I think we should uh push the release to Friday. No wait, \
                Thursday, because the uh the QA cycle needs two full days.";
    serde_json::json!({
        "said": said,
        "typed": clean::clean(said, &CleanConfig::default()),
    })
}

fn broadcast(app: &tauri::AppHandle, kivo: &Arc<Kivo>) {
    let _ = app.emit("kivo://state", *kivo.state.lock());
}

pub fn run() {
    // No telemetry, ever — logs stay on the machine and go nowhere.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let kivo = Kivo::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(kivo.clone())
        .invoke_handler(tauri::generate_handler![
            snapshot,
            save_config,
            preview_clean,
            clear_history,
            dismiss_error,
            reload_model,
            demo
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // Kivo lives in the menu bar. Closing the settings window hides it
            // rather than quitting — the hotkey must keep working.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let open = MenuItem::with_id(app, "open", "Settings…", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Kivo", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&open, &sep, &quit])?;

            TrayIconBuilder::with_id("kivo")
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/32x32.png"
                ))?)
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            // The hotkey hook gets its own thread and does nothing but forward
            // edges. Everything else happens on the worker below.
            let (tx, rx) = crossbeam_channel::bounded::<Trigger>(8);
            let key = kivo.config.lock().hotkey;
            let listen_handle = handle.clone();
            std::thread::spawn(move || {
                if let Err(e) = hotkey::listen(key, tx) {
                    tracing::error!("hotkey listener stopped: {e}");
                    let _ = listen_handle.emit(
                        "kivo://permission",
                        "Kivo needs Accessibility permission to see the hotkey. \
                         Grant it in System Settings › Privacy & Security › Accessibility, \
                         then restart Kivo.",
                    );
                }
            });

            // First run: show the window rather than leaving a new user staring
            // at a menu bar wondering whether anything installed.
            if !Config::path().exists() {
                if let Some(w) = handle.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }

            let worker = kivo.clone();
            let worker_handle = handle.clone();
            std::thread::spawn(move || {
                for trigger in rx {
                    match trigger {
                        Trigger::Down => {
                            worker.begin();
                            show_overlay(&worker_handle);
                            broadcast(&worker_handle, &worker);
                        }
                        Trigger::Up => {
                            broadcast(&worker_handle, &worker);
                            let entry = worker.finish();
                            hide_overlay(&worker_handle);
                            broadcast(&worker_handle, &worker);
                            if let Some(e) = entry {
                                let _ = worker_handle.emit("kivo://inserted", e);
                            }
                            if let Some(err) = worker.last_error.lock().clone() {
                                let _ = worker_handle.emit("kivo://error", err);
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide instead of close, so the tray and hotkey survive.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Kivo failed to start");
}

/// The overlay never takes focus and never accepts input. It exists so a dead
/// microphone is visible in the first half-second rather than after a paragraph.
fn show_overlay(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("overlay") {
        if let Ok(Some(monitor)) = w.primary_monitor() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let w_logical = 260.0;
            let x = (size.width as f64 / scale - w_logical) / 2.0;
            let y = size.height as f64 / scale - 160.0;
            let _ = w.set_position(tauri::LogicalPosition::new(x, y));
        }
        let _ = w.show();
    }
}

fn hide_overlay(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.hide();
    }
}
