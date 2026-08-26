//! Tier 1 — deterministic rules. Sub-millisecond, unit-tested, auditable.
//!
//! Everything here is reversible by a switch in settings, and Raw mode skips
//! the lot. A user who does not trust a rule should be able to turn that rule
//! off rather than abandoning the product.

use super::normalise::is_terminal;
use super::token::{Kind, Token};

/// Consume one space that a just-dropped word would otherwise leave behind —
/// preferring the one after it, falling back to the one before. Without this a
/// deletion leaves "i  think" and every rule downstream has to cope with it.
fn absorb_space(out: &mut Vec<Token>, tokens: &[Token], i: &mut usize) {
    if tokens
        .get(*i)
        .map(|t| t.kind == Kind::Space)
        .unwrap_or(false)
    {
        *i += 1;
    } else if out.last().map(|t| t.kind == Kind::Space).unwrap_or(false) {
        out.pop();
    }
}

/// Sounds, not words. Safe to delete unconditionally.
const FILLERS: &[&str] = &[
    "um", "uhm", "uh", "erm", "er", "ah", "eh", "hmm", "hm", "mm", "mmm", "mhm",
];

/// Words that are real words elsewhere but noise when they open a sentence.
/// Only removed in that position, which is why they are a separate list.
const OPENERS: &[&str] = &[
    "so",
    "well",
    "okay",
    "ok",
    "right",
    "basically",
    "actually",
    "literally",
    "anyway",
];

/// Phrases that signal the speaker is replacing what they just said.
const CORRECTIONS: &[&[&str]] = &[
    &["no", "wait"],
    &["scratch", "that"],
    &["or", "rather"],
    &["i", "mean"],
    &["sorry"],
    &["rather"],
];

fn is_filler(t: &Token) -> bool {
    t.is_word() && FILLERS.contains(&t.lower().as_str())
}

/// Remove filler sounds, and optionally sentence-opening discourse markers.
pub fn remove_fillers(tokens: Vec<Token>, drop_openers: bool) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
    // True when the next word begins a sentence.
    let mut at_start = true;

    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];

        if is_filler(t) {
            i += 1;
            // Swallow a comma that only existed to fence the filler.
            if let Some(next) = tokens.get(i) {
                if next.kind == Kind::Punct && next.text == "," {
                    i += 1;
                }
            }
            absorb_space(&mut out, &tokens, &mut i);
            continue;
        }

        if drop_openers && at_start && t.is_word() && OPENERS.contains(&t.lower().as_str()) {
            i += 1;
            if let Some(next) = tokens.get(i) {
                if next.kind == Kind::Punct && next.text == "," {
                    i += 1;
                }
            }
            absorb_space(&mut out, &tokens, &mut i);
            continue;
        }

        if t.is_word() {
            at_start = false;
        } else if is_terminal(t) {
            at_start = true;
        }

        out.push(t.clone());
        i += 1;
    }
    out
}

/// Collapse an immediately repeated word: "the the" -> "the".
///
/// Runs after filler removal on purpose — "the uh the" only becomes a
/// detectable stutter once the "uh" is gone.
pub fn collapse_stutters(tokens: Vec<Token>) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::with_capacity(tokens.len());

    for tok in tokens {
        if tok.is_word() {
            // Find the previous word, skipping only whitespace. Punctuation in
            // between means it is deliberate ("no, no") and must be preserved.
            let prev_word = out
                .iter()
                .rev()
                .find(|t| t.kind != Kind::Space)
                .filter(|t| t.is_word());

            if let Some(p) = prev_word {
                if p.lower() == tok.lower() {
                    while out.last().map(|t| t.kind == Kind::Space).unwrap_or(false) {
                        out.pop();
                    }
                    out.pop(); // drop the earlier copy, keep the later one
                }
            }
        }
        out.push(tok);
    }
    out
}

/// Resolve a spoken self-correction: "Friday - no wait, Thursday" -> "Thursday".
///
/// Deliberately narrow. It deletes the single word before the marker, plus any
/// dash or comma between them, and the marker itself. Multi-word corrections
/// ("the third of June, sorry, the fourth of June") are beyond a rule that has
/// to be predictable, and are left to the optional tier-2 pass.
pub fn resolve_corrections(tokens: Vec<Token>) -> Vec<Token> {
    let words: Vec<usize> = (0..tokens.len()).filter(|&i| tokens[i].is_word()).collect();
    let mut drop = vec![false; tokens.len()];

    let mut w = 0;
    while w < words.len() {
        let matched = CORRECTIONS.iter().find(|phrase| {
            phrase.iter().enumerate().all(|(k, want)| {
                words
                    .get(w + k)
                    .map(|&i| tokens[i].lower() == *want)
                    .unwrap_or(false)
            })
        });

        let Some(phrase) = matched else {
            w += 1;
            continue;
        };

        // A correction needs something before it to correct, and something
        // after it to replace with.
        if w == 0 || w + phrase.len() >= words.len() {
            w += 1;
            continue;
        }

        let first_marker_tok = words[w];
        let last_marker_tok = words[w + phrase.len() - 1];
        let corrected_word = words[w - 1];

        drop[corrected_word..=last_marker_tok].fill(true);
        // A comma or dash immediately after the marker belonged to it.
        let mut j = last_marker_tok + 1;
        while j < tokens.len() && tokens[j].kind == Kind::Space {
            j += 1;
        }
        if j < tokens.len()
            && tokens[j].kind == Kind::Punct
            && (tokens[j].text == "," || tokens[j].text == "-")
        {
            drop[(last_marker_tok + 1)..=j].fill(true);
        }
        let _ = first_marker_tok;
        w += phrase.len();
    }

    tokens
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !drop[*i])
        .map(|(_, t)| t)
        .collect()
}

/// A spoken-to-written substitution from the user's own vocabulary.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Replacement {
    pub from: String,
    pub to: String,
}

/// Apply user replacements on whole-word boundaries.
///
/// Matching is case-insensitive; the replacement's own casing is used, except
/// that a capitalised match keeps its capital so sentence starts survive.
pub fn apply_replacements(tokens: Vec<Token>, reps: &[Replacement]) -> Vec<Token> {
    if reps.is_empty() {
        return tokens;
    }
    let prepared: Vec<(Vec<String>, String)> = reps
        .iter()
        .map(|r| {
            (
                r.from
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect::<Vec<String>>(),
                r.to.clone(),
            )
        })
        .filter(|(f, _)| !f.is_empty())
        .collect();

    let words: Vec<usize> = (0..tokens.len()).filter(|&i| tokens[i].is_word()).collect();
    let mut replace_at: Vec<(usize, usize, String)> = Vec::new(); // (tok start, tok end, text)

    let mut w = 0;
    'outer: while w < words.len() {
        for (from, to) in &prepared {
            let hit = from.iter().enumerate().all(|(k, want)| {
                words
                    .get(w + k)
                    .map(|&i| tokens[i].lower() == *want)
                    .unwrap_or(false)
            });
            if hit {
                let start = words[w];
                let end = words[w + from.len() - 1];
                let was_capital = tokens[start]
                    .text
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                let mut text = to.clone();
                if was_capital {
                    let mut cs = text.chars();
                    if let Some(f) = cs.next() {
                        text = f.to_uppercase().collect::<String>() + cs.as_str();
                    }
                }
                replace_at.push((start, end, text));
                w += from.len();
                continue 'outer;
            }
        }
        w += 1;
    }

    if replace_at.is_empty() {
        return tokens;
    }

    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if let Some((_, end, text)) = replace_at.iter().find(|(s, _, _)| *s == i) {
            out.push(Token {
                text: text.clone(),
                kind: Kind::Word,
            });
            i = end + 1;
        } else {
            out.push(tokens[i].clone());
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::token::{render, tokenize};
    use super::*;

    fn fill(s: &str, openers: bool) -> String {
        render(&remove_fillers(tokenize(s), openers))
    }

    #[test]
    fn removes_filler_sounds() {
        assert_eq!(fill("i um think so", false), "i think so");
    }

    #[test]
    fn leaves_real_words_that_look_like_fillers() {
        // "Ah" is a filler; "are" is not. Guard against prefix matching.
        assert_eq!(fill("are you there", false), "are you there");
    }

    #[test]
    fn removes_openers_only_at_a_sentence_start() {
        assert_eq!(
            fill("so i went. it was so good", true),
            "i went. it was so good"
        );
    }

    #[test]
    fn keeps_openers_when_the_rule_is_off() {
        assert_eq!(fill("so i went", false), "so i went");
    }

    #[test]
    fn collapses_an_immediate_repeat() {
        let out = render(&collapse_stutters(tokenize("the the cycle")));
        assert_eq!(out, "the cycle");
    }

    #[test]
    fn keeps_a_repeat_that_punctuation_makes_deliberate() {
        let out = render(&collapse_stutters(tokenize("no, no")));
        assert_eq!(out, "no, no");
    }

    #[test]
    fn resolves_a_single_word_correction() {
        let out = render(&resolve_corrections(tokenize(
            "push it to friday - no wait, thursday",
        )));
        assert_eq!(out, "push it to  thursday");
    }

    #[test]
    fn ignores_a_correction_marker_with_nothing_to_correct() {
        let out = render(&resolve_corrections(tokenize("sorry about that")));
        assert_eq!(out, "sorry about that");
    }

    #[test]
    fn replaces_on_word_boundaries_only() {
        let reps = vec![Replacement {
            from: "kubernetes".into(),
            to: "Kubernetes".into(),
        }];
        let out = render(&apply_replacements(
            tokenize("kubernetes and kubernetesctl"),
            &reps,
        ));
        assert_eq!(out, "Kubernetes and kubernetesctl");
    }

    #[test]
    fn replaces_a_multi_word_phrase() {
        let reps = vec![Replacement {
            from: "post gres".into(),
            to: "PostgreSQL".into(),
        }];
        let out = render(&apply_replacements(
            tokenize("we use post gres here"),
            &reps,
        ));
        assert_eq!(out, "we use PostgreSQL here");
    }
}
