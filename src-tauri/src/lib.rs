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
pub mod history;
pub mod hotkey;
pub mod inject;
pub mod models;
pub mod pipeline;
pub mod tray;

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
    model_download_mb: u64,
    history: Vec<Entry>,
    history_enabled: bool,
    network: Vec<crate::models::NetworkEvent>,
    devices: Vec<String>,
    error: Option<String>,
    hotkeys: Vec<HotKeyOption>,
    modes: Vec<ModeOption>,
    onboarded: bool,
    /// Set when macOS has already claimed the chosen key.
    conflict: Option<hotkey::conflict::Conflict>,
    /// False when macOS will not let Kivo see the keyboard at all.
    accessibility: bool,
}

#[derive(Serialize)]
struct HotKeyOption {
    value: HotKey,
    label: &'static str,
    /// A collision worth warning about, shown beside the choice.
    note: Option<&'static str>,
}

#[derive(Serialize)]
struct ModeOption {
    value: Mode,
    label: &'static str,
}

#[tauri::command]
fn snapshot(app: tauri::State<'_, Arc<Kivo>>, query: Option<String>) -> Snapshot {
    let dir = asr::parakeet::Parakeet::default_dir();
    let q = query.unwrap_or_default();
    Snapshot {
        state: *app.state.lock(),
        config: app.config.lock().clone(),
        model: app.describe_model(),
        model_installed: asr::parakeet::Parakeet::is_installed(&dir),
        model_dir: dir.to_string_lossy().into_owned(),
        model_download_mb: crate::models::DOWNLOAD_MB,
        history: app.recent_history(50, &q),
        history_enabled: app.history.lock().is_enabled(),
        network: app.network.lock().clone(),
        devices: audio::input_devices(),
        onboarded: Config::path().exists(),
        conflict: hotkey::conflict::check(app.config.lock().hotkey),
        accessibility: hotkey::trust::is_trusted(),
        error: app.last_error.lock().clone(),
        hotkeys: HotKey::all()
            .iter()
            .map(|h| HotKeyOption {
                value: *h,
                label: h.label(),
                note: h.note(),
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
    app.apply_config(config);
    Ok(())
}

/// Download, verify and install the speech model. Runs off the UI thread and
/// reports progress; nothing is installed unless the SHA-256 matches.
#[tauri::command]
fn install_model(app: tauri::AppHandle, kivo: tauri::State<'_, Arc<Kivo>>) {
    let kivo = kivo.inner().clone();
    std::thread::spawn(move || {
        let handle = app.clone();
        // `install` reports its own progress, including Downloading{percent}
        // and a terminal Done or Failed — the caller only needs to record what
        // happened and, on success, load the result.
        let ev = match crate::models::install(move |p| {
            let _ = handle.emit("kivo://model", p);
        }) {
            Ok(ev) => ev,
            Err(e) => {
                // install() only returns Err for a bug in this glue, not for
                // an ordinary download failure — those come back as Ok with a
                // failed outcome, already reported through Progress::Failed.
                tracing::error!("model install returned an unexpected error: {e:#}");
                return;
            }
        };

        let succeeded = !ev.outcome.starts_with("failed:");
        kivo.network.lock().push(ev);

        if succeeded {
            match kivo.reload_model() {
                Ok(desc) => tracing::info!("model ready: {desc}"),
                Err(e) => {
                    let _ = app.emit(
                        "kivo://model",
                        crate::models::Progress::Failed {
                            message: format!(
                                "The model installed, but Kivo could not load it: {e:#}"
                            ),
                        },
                    );
                }
            }
        }
    });
}

#[tauri::command]
fn remove_model(app: tauri::State<'_, Arc<Kivo>>) -> Result<(), String> {
    crate::models::remove().map_err(|e| format!("{e:#}"))?;
    let _ = app.reload_model();
    Ok(())
}

/// Mark onboarding complete by writing settings for the first time.
#[tauri::command]
fn finish_onboarding(app: tauri::State<'_, Arc<Kivo>>) -> Result<(), String> {
    app.config.lock().save().map_err(|e| e.to_string())
}

/// Open the macOS pane where Accessibility permission is granted. Deep-linking
/// beats telling someone to go and find it.
/// Open an arbitrary System Settings pane by URL. Only ever called with a
/// pane string Kivo itself produced.
#[tauri::command]
fn open_settings_pane(pane: String) {
    #[cfg(target_os = "macos")]
    {
        if pane.starts_with("x-apple.systempreferences:") {
            let _ = std::process::Command::new("open").arg(&pane).spawn();
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = pane;
}

/// Ask macOS for Accessibility permission from the interface.
#[tauri::command]
fn request_accessibility() -> bool {
    hotkey::trust::request_trust()
}

/// Briefly open the microphone and immediately close it again.
///
/// This is the API call that makes macOS show its microphone permission
/// dialog, so onboarding can trigger that prompt on the step that explains it
/// instead of waiting for it to appear, unexplained, during someone's first
/// real dictation. Nothing captured here is kept: the samples are discarded
/// and the ring buffer is zeroed by `Capture::stop`, exactly as in normal use.
///
/// This cannot report back a definitive granted/denied answer — that needs a
/// native AVFoundation authorisation check this app does not otherwise need —
/// so the interface treats the attempt itself as the signal, and the existing
/// `kivo://permission` banner remains the safety net if access turns out to
/// have been refused.
#[tauri::command]
fn request_microphone() -> bool {
    match audio::Capture::start("") {
        Ok(c) => {
            std::thread::sleep(std::time::Duration::from_millis(120));
            let _ = c.stop();
            true
        }
        Err(e) => {
            tracing::warn!("microphone probe failed: {e:#}");
            false
        }
    }
}

/// Quit Kivo entirely. The tray menu already offers this; Settings does too,
/// so someone who cannot find (or use) the menu bar is not stuck.
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// Show the settings window as a normal application: Dock icon, Cmd-Tab,
/// Force Quit, all present. Kivo is only a quiet menu-bar accessory while
/// there is no window for anyone to be looking at; the moment one is showing,
/// hiding it from every place a person would look for a stuck or misbehaving
/// app would be actively unhelpful.
fn present_main_window(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn open_accessibility_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }
}

/// Let settings preview the cleanup on sample text without speaking. This is
/// how a user decides whether to trust a rule before turning it loose.
#[tauri::command]
fn preview_clean(text: String, config: Config) -> String {
    clean::clean(&text, &config.cleaning())
}

#[tauri::command]
fn clear_history(app: tauri::State<'_, Arc<Kivo>>) -> Result<(), String> {
    app.history.lock().clear().map_err(|e| format!("{e:#}"))
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
    // A menu bar app has no console, so stdout goes nowhere the user can see.
    // Without a log on disk, "it doesn't work" is undiagnosable — which is
    // exactly the hole this fell into. Logs stay on the machine and go nowhere.
    //
    // Levels are chosen so the default file never contains a transcript: text
    // is logged only at debug, which requires RUST_LOG=debug deliberately.
    let log_dir = dirs::home_dir()
        .unwrap_or_default()
        .join("Library/Logs/Kivo");
    let _ = std::fs::create_dir_all(&log_dir);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("kivo.log"))
    {
        Ok(file) => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_ansi(false)
                .with_writer(std::sync::Mutex::new(file))
                .init();
        }
        Err(_) => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(false)
                .init();
        }
    }
    tracing::info!("--- Kivo {} starting ---", env!("CARGO_PKG_VERSION"));

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
            install_model,
            remove_model,
            finish_onboarding,
            open_accessibility_settings,
            open_settings_pane,
            request_accessibility,
            request_microphone,
            quit_app,
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

            // The menu bar glyph must be the monochrome template image, not the
            // colour app icon. macOS composites a template (black + alpha) into
            // the light/dark/highlighted menu bar states itself; handing it a
            // colour, opaque PNG instead produces a solid filled block, which
            // is exactly the "white square" this used to render as.
            TrayIconBuilder::with_id(tray::TRAY_ID)
                .icon(tray::idle_image()?)
                .icon_as_template(true)
                .tooltip("Kivo — hold your key to dictate")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => present_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            tracing::info!("menu bar item created");

            // The hotkey hook gets its own thread and does nothing but forward
            // edges. Everything else happens on the worker below.
            let (tx, rx) = crossbeam_channel::bounded::<Trigger>(8);
            let key = kivo.config.lock().hotkey;
            let listen_handle = handle.clone();
            std::thread::spawn(move || {
                let trusted = hotkey::trust::is_trusted();
                tracing::info!(
                    "hotkey listener starting: key={} accessibility_trusted={}",
                    key.label(),
                    trusted
                );
                if !trusted {
                    // Raises the system dialog and registers Kivo in the
                    // Accessibility list, so there is a switch to flip.
                    hotkey::trust::request_trust();
                    let _ = listen_handle.emit(
                        "kivo://permission",
                        "Kivo does not have Accessibility permission, so it cannot see \
                         your hotkey. Grant it in System Settings › Privacy & Security › \
                         Accessibility, then quit and reopen Kivo.",
                    );
                }
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
                present_main_window(&handle);
            }

            let worker = kivo.clone();
            let worker_handle = handle.clone();
            let animator = tray::Animator::new();
            std::thread::spawn(move || {
                for trigger in rx {
                    tracing::info!("hotkey trigger: {trigger:?}");

                    // The audio/recognition work below must never be skipped
                    // because a UI call panicked — that would leave the
                    // microphone open with no code path left to close it, and
                    // it would kill this thread, silently ending dictation
                    // until the app is restarted. `catch_unwind` keeps one bad
                    // trigger from taking either of those down with it.
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        match trigger {
                            Trigger::Down => {
                                worker.begin();
                                show_overlay(&worker_handle);
                                animator.start(&worker_handle);
                                broadcast(&worker_handle, &worker);
                            }
                            Trigger::Cancel => {
                                worker.cancel();
                                animator.stop();
                                hide_overlay(&worker_handle);
                                broadcast(&worker_handle, &worker);
                            }
                            Trigger::Up => {
                                broadcast(&worker_handle, &worker);
                                let entry = worker.finish();
                                animator.stop();
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
                    }));

                    if let Err(panic) = outcome {
                        tracing::error!("hotkey trigger handling panicked: {panic:?}");
                        // Whatever failed, the microphone must not stay open.
                        worker.cancel();
                        animator.stop();
                        hide_overlay(&worker_handle);
                        broadcast(&worker_handle, &worker);
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
                    // Kivo is a menu-bar resident while idle: no Dock icon, not
                    // in Cmd-Tab, not in Force Quit. That is only correct while
                    // there is no window for the user to be looking at.
                    #[cfg(target_os = "macos")]
                    let _ = window
                        .app_handle()
                        .set_activation_policy(tauri::ActivationPolicy::Accessory);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Kivo failed to start");
}

/// The overlay never takes focus and never accepts input. It exists so a dead
/// microphone is visible in the first half-second rather than after a paragraph.
///
/// Called from the hotkey worker thread, so every window and cursor call here
/// is dispatched to the main thread rather than run in place: AppKit's view
/// and layer trees are not thread-safe, and a mutation from off the main
/// thread does not reliably fail loudly — it can instead leave a window's
/// compositor state corrupted. That is what produced Settings windows that
/// opened solid black with nothing painted in them.
fn show_overlay(app: &tauri::AppHandle) {
    let for_main = app.clone();
    let _ = app.run_on_main_thread(move || {
        let app = for_main;
        let Some(w) = app.get_webview_window("overlay") else {
            return;
        };

        // Follow the pointer, which is where the user is looking. Fall back to
        // the bottom centre of the primary display if the position is
        // unavailable.
        const W: f64 = 260.0;
        const H: f64 = 96.0;

        let placed = app.cursor_position().ok().and_then(|p| {
            let monitor = w.monitor_from_point(p.x, p.y).ok().flatten()?;
            let scale = monitor.scale_factor();
            let area = monitor.size().to_logical::<f64>(scale);
            let origin = monitor.position().to_logical::<f64>(scale);
            let cursor = p.to_logical::<f64>(scale);

            // Sit just below the pointer, clamped inside the display it is on.
            let x = (cursor.x - W / 2.0).clamp(origin.x + 12.0, origin.x + area.width - W - 12.0);
            let y = (cursor.y + 28.0).clamp(origin.y + 12.0, origin.y + area.height - H - 12.0);
            Some(tauri::LogicalPosition::new(x, y))
        });

        let position = placed.or_else(|| {
            let monitor = w.primary_monitor().ok().flatten()?;
            let scale = monitor.scale_factor();
            let area = monitor.size().to_logical::<f64>(scale);
            let origin = monitor.position().to_logical::<f64>(scale);
            Some(tauri::LogicalPosition::new(
                origin.x + (area.width - W) / 2.0,
                origin.y + area.height - 160.0,
            ))
        });

        if let Some(p) = position {
            let _ = w.set_position(p);
        }
        let _ = w.show();
    });
}

fn hide_overlay(app: &tauri::AppHandle) {
    let for_main = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(w) = for_main.get_webview_window("overlay") {
            let _ = w.hide();
        }
    });
}
