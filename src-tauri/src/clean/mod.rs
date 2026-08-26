//! The cleanup engine — the difference between what you said and what you meant.
//!
//! Recognition is a solved-enough problem that Kivo's value lives almost
//! entirely here. Three tiers, so the default path never pays for the expensive
//! one:
//!
//!   Tier 0  normalise   spacing and punctuation shape   microseconds  always
//!   Tier 1  rules       fillers, stutters, corrections  sub-ms        default on
//!   Tier 2  local LLM   register, multi-word repair     200-800 ms    opt-in
//!
//! Tier 2 is not implemented yet; the boundary is here so adding it does not
//! disturb anything above.

pub mod modes;
pub mod normalise;
pub mod rules;
pub mod token;

pub use modes::Mode;
pub use rules::Replacement;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanConfig {
    pub mode: Mode,
    /// Delete "um", "uh", "er" and friends.
    pub remove_fillers: bool,
    /// Delete "so", "well", "basically" when they open a sentence.
    pub remove_openers: bool,
    /// "the the" -> "the".
    pub collapse_stutters: bool,
    /// "Friday - no wait, Thursday" -> "Thursday".
    pub resolve_corrections: bool,
    /// The user's own spoken-to-written vocabulary.
    pub replacements: Vec<Replacement>,
}

impl Default for CleanConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Standard,
            remove_fillers: true,
            remove_openers: true,
            collapse_stutters: true,
            resolve_corrections: true,
            replacements: Vec::new(),
        }
    }
}

/// Turn recognition output into text a person would have typed.
pub fn clean(input: &str, cfg: &CleanConfig) -> String {
    if input.trim().is_empty() {
        return String::new();
    }
    // Raw is the debugging baseline and the user's escape hatch: it must be
    // byte-for-byte what recognition produced, so it returns before every tier.
    if cfg.mode.is_raw() {
        return input.to_string();
    }

    let mut t = token::tokenize(input);

    if cfg.resolve_corrections {
        t = rules::resolve_corrections(t);
    }
    if cfg.remove_fillers {
        t = rules::remove_fillers(t, cfg.remove_openers);
    }
    if cfg.collapse_stutters {
        t = rules::collapse_stutters(t);
    }
    t = rules::apply_replacements(t, &cfg.replacements);

    // Normalise before mode formatting so capitalisation sees real sentence
    // boundaries rather than the gaps left by deleted words.
    t = normalise::normalise(t);
    t = modes::apply(cfg.mode, t);
    t = normalise::normalise(t);

    token::render(&t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The worked example from PRD section 7.4 and the landing page. If this
    /// ever fails, the site is making a promise the product no longer keeps.
    #[test]
    fn the_prd_example_produces_the_advertised_output() {
        // This is the *raw ASR output* quoted in PRD 7.4 — Parakeet restores the
        // punctuation and case, and Kivo removes the disfluency on top.
        let said = "Um, so I think we should uh push the release to Friday. No wait, \
                    Thursday, because the uh the QA cycle needs two full days.";
        let got = clean(said, &CleanConfig::default());
        assert_eq!(
            got,
            "I think we should push the release to Thursday, because the QA cycle needs two full days."
        );
    }

    #[test]
    fn raw_mode_is_byte_for_byte_untouched() {
        let said = "Um, so   I think..";
        let cfg = CleanConfig {
            mode: Mode::Raw,
            ..Default::default()
        };
        assert_eq!(clean(said, &cfg), said);
    }

    #[test]
    fn every_rule_can_be_switched_off_independently() {
        let said = "um so the the thing";
        let cfg = CleanConfig {
            remove_fillers: false,
            remove_openers: false,
            collapse_stutters: false,
            resolve_corrections: false,
            ..Default::default()
        };
        // Only tier 0 and mode formatting remain.
        assert_eq!(clean(said, &cfg), "Um so the the thing.");
    }

    #[test]
    fn user_vocabulary_is_applied() {
        let cfg = CleanConfig {
            replacements: vec![Replacement {
                from: "cuber netties".into(),
                to: "Kubernetes".into(),
            }],
            ..Default::default()
        };
        assert_eq!(
            clean("we deploy on cuber netties", &cfg),
            "We deploy on Kubernetes."
        );
    }

    #[test]
    fn empty_and_whitespace_input_yield_nothing() {
        assert_eq!(clean("", &CleanConfig::default()), "");
        assert_eq!(clean("   \n ", &CleanConfig::default()), "");
    }

    #[test]
    fn a_message_reads_like_a_message() {
        let cfg = CleanConfig {
            mode: Mode::Message,
            ..Default::default()
        };
        assert_eq!(clean("um on my way", &cfg), "On my way");
    }

    #[test]
    fn code_mode_leaves_identifiers_alone() {
        let cfg = CleanConfig {
            mode: Mode::Code,
            ..Default::default()
        };
        assert_eq!(
            clean("let user underscore id equals nine", &cfg),
            "let user_id = nine"
        );
    }
}
