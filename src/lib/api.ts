import { invoke } from "@tauri-apps/api/core";

export type Mode = "standard" | "message" | "email" | "notes" | "code" | "raw";
export type State = "idle" | "recording" | "working";

export interface Replacement { from: string; to: string }

export interface Config {
  hotkey: string;
  mode: Mode;
  remove_fillers: boolean;
  remove_openers: boolean;
  collapse_stutters: boolean;
  resolve_corrections: boolean;
  replacements: Replacement[];
  vocabulary: string[];
  input_device: string;
  use_typing: boolean;
  keep_history: boolean;
  history_limit: number;
  launch_at_login: boolean;
}

export interface Entry {
  id: number; raw: string; text: string; mode: string; millis: number; at: number;
}

export interface NetworkEvent { at: number; url: string; outcome: string; bytes: number }

export interface Snapshot {
  state: State;
  config: Config;
  model: string;
  model_installed: boolean;
  model_dir: string;
  model_download_mb: number;
  history: Entry[];
  history_enabled: boolean;
  network: NetworkEvent[];
  devices: string[];
  error: string | null;
  hotkeys: { value: string; label: string; note: string | null }[];
  modes: { value: Mode; label: string }[];
  onboarded: boolean;
  conflict: { taken_by: string; fix: string; pane: string } | null;
  /** False when macOS will not let Kivo see the keyboard at all. */
  accessibility: boolean;
}

export type ModelProgress =
  | "Started" | "Verifying" | "Installing" | "Done"
  | { Downloading: { percent: number } }
  | { Failed: { message: string } };

export const api = {
  snapshot: (query = "") => invoke<Snapshot>("snapshot", { query }),
  saveConfig: (config: Config) => invoke<void>("save_config", { config }),
  previewClean: (text: string, config: Config) => invoke<string>("preview_clean", { text, config }),
  clearHistory: () => invoke<void>("clear_history"),
  dismissError: () => invoke<void>("dismiss_error"),
  reloadModel: () => invoke<string>("reload_model"),
  installModel: () => invoke<void>("install_model"),
  removeModel: () => invoke<void>("remove_model"),
  finishOnboarding: () => invoke<void>("finish_onboarding"),
  openAccessibility: () => invoke<void>("open_accessibility_settings"),
  openPane: (pane: string) => invoke<void>("open_settings_pane", { pane }),
  /** Raises the native Accessibility prompt and registers Kivo in the list. */
  requestAccessibility: () => invoke<boolean>("request_accessibility"),
  /** Briefly opens the microphone, which is what makes macOS ask permission. */
  requestMicrophone: () => invoke<boolean>("request_microphone"),
  quitApp: () => invoke<void>("quit_app"),
  demo: () => invoke<{ said: string; typed: string }>("demo"),
};

/** Relative time, for history rows. */
export function ago(unixSeconds: number): string {
  const s = Math.max(0, Math.floor(Date.now() / 1000) - unixSeconds);
  if (s < 60) return "just now";
  if (s < 3600) return `${Math.floor(s / 60)} min ago`;
  if (s < 86400) return `${Math.floor(s / 3600)} h ago`;
  return `${Math.floor(s / 86400)} d ago`;
}
