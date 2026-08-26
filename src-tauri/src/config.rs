//! Settings. One TOML file, human-readable, no magic.
//!
//! Every preference here is maintained forever, so the bar for adding one is
//! that its absence breaks a real workflow.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::clean::{CleanConfig, Mode, Replacement};
use crate::hotkey::HotKey;
use crate::inject::Method;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub hotkey: HotKey,
    pub mode: Mode,
    pub remove_fillers: bool,
    pub remove_openers: bool,
    pub collapse_stutters: bool,
    pub resolve_corrections: bool,
    pub replacements: Vec<Replacement>,
    /// Words to bias recognition toward — names, acronyms, jargon. Distinct
    /// from `replacements`, which rewrite text after the fact; these make the
    /// model likelier to hear the word at all.
    pub vocabulary: Vec<String>,
    /// Empty means "follow the system default input".
    pub input_device: String,
    /// Paste is faster and near-universal; typing is the fallback for apps
    /// that refuse a synthetic paste.
    pub use_typing: bool,
    /// Off means nothing is ever written to disk.
    pub keep_history: bool,
    pub history_limit: usize,
    pub launch_at_login: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: HotKey::default(),
            mode: Mode::default(),
            remove_fillers: true,
            remove_openers: true,
            collapse_stutters: true,
            resolve_corrections: true,
            replacements: Vec::new(),
            vocabulary: Vec::new(),
            input_device: String::new(),
            use_typing: false,
            keep_history: true,
            history_limit: 100,
            launch_at_login: false,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Kivo")
            .join("settings.toml")
    }

    /// Load, falling back to defaults. A corrupt file must not stop Kivo from
    /// starting — the user would have no way to fix it from inside the app.
    pub fn load() -> Self {
        let p = Self::path();
        match std::fs::read_to_string(&p)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
        {
            Some(cfg) => cfg,
            None => {
                if p.exists() {
                    tracing::warn!(
                        "settings at {} were unreadable; using defaults",
                        p.display()
                    );
                }
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let p = Self::path();
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir).context("could not create the settings directory")?;
        }
        let body = toml::to_string_pretty(self).context("could not serialise settings")?;
        std::fs::write(&p, body).with_context(|| format!("could not write {}", p.display()))?;
        Ok(())
    }

    pub fn cleaning(&self) -> CleanConfig {
        CleanConfig {
            mode: self.mode,
            remove_fillers: self.remove_fillers,
            remove_openers: self.remove_openers,
            collapse_stutters: self.collapse_stutters,
            resolve_corrections: self.resolve_corrections,
            replacements: self.replacements.clone(),
        }
    }

    pub fn injection(&self) -> Method {
        if self.use_typing {
            Method::Typing
        } else {
            Method::Paste
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_through_toml() {
        let a = Config::default();
        let s = toml::to_string_pretty(&a).unwrap();
        let b: Config = toml::from_str(&s).unwrap();
        assert_eq!(a.hotkey, b.hotkey);
        assert_eq!(a.mode, b.mode);
        assert_eq!(a.history_limit, b.history_limit);
    }

    #[test]
    fn a_partial_file_fills_in_the_rest() {
        let cfg: Config = toml::from_str("mode = \"code\"").unwrap();
        assert_eq!(cfg.mode, Mode::Code);
        assert!(cfg.remove_fillers); // still the default
    }

    #[test]
    fn defaults_are_privacy_preserving() {
        let c = Config::default();
        assert!(
            !c.launch_at_login,
            "must not add itself to login items unasked"
        );
    }
}
