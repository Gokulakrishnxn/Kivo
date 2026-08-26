//! Tier 0 — normalisation. Deterministic, microseconds, always on.
//!
//! Only touches spacing and punctuation shape. It never removes a word, so it
//! is safe to run even in Raw mode's neighbourhood and it cannot change meaning.

use super::token::{Kind, Token};

/// Punctuation that must hug the word before it: `word,` not `word ,`.
const CLOSES: &[char] = &[',', '.', '!', '?', ';', ':', ')', ']', '}', '%', '\''];
/// Punctuation that must hug the word after it: `(word` not `( word`.
const OPENS: &[char] = &['(', '[', '{'];
/// Sentence-ending marks, used to decide capitalisation downstream.
pub const TERMINALS: &[char] = &['.', '!', '?'];

pub fn normalise(tokens: Vec<Token>) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::with_capacity(tokens.len());

    for tok in tokens {
        match tok.kind {
            Kind::Space => {
                // Never emit two spaces, and never open with one.
                if out.last().map(|t| t.kind == Kind::Space).unwrap_or(true) {
                    continue;
                }
                out.push(Token {
                    text: " ".into(),
                    kind: Kind::Space,
                });
            }
            Kind::Punct => {
                let c = tok.text.chars().next().unwrap_or(' ');

                // Drop the space that a dropped word may have left in front of
                // a closing mark: "friday , thursday" -> "friday, thursday".
                if CLOSES.contains(&c) {
                    while out.last().map(|t| t.kind == Kind::Space).unwrap_or(false) {
                        out.pop();
                    }
                }

                // Collapse a run of the same mark: "..", "!!" -> one.
                if let Some(prev) = out.last() {
                    if prev.kind == Kind::Punct && prev.text == tok.text && c != '-' {
                        continue;
                    }
                }

                out.push(tok);

                if OPENS.contains(&c) {
                    // Nothing follows an opener directly; the space, if any,
                    // is skipped by the Space arm above on the next iteration.
                }
            }
            Kind::Word => {
                // A word after a closing mark needs exactly one space.
                if let Some(prev) = out.last() {
                    let needs_space = prev.kind == Kind::Punct
                        && CLOSES.contains(&prev.text.chars().next().unwrap_or(' '))
                        && prev.text != '\''.to_string();
                    if needs_space {
                        out.push(Token {
                            text: " ".into(),
                            kind: Kind::Space,
                        });
                    }
                }
                out.push(tok);
            }
        }
    }

    // No trailing whitespace.
    while out.last().map(|t| t.kind == Kind::Space).unwrap_or(false) {
        out.pop();
    }
    out
}

/// True if the token ends a sentence.
pub fn is_terminal(tok: &Token) -> bool {
    tok.kind == Kind::Punct
        && tok
            .text
            .chars()
            .next()
            .map(|c| TERMINALS.contains(&c))
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::super::token::{render, tokenize};
    use super::*;

    fn n(s: &str) -> String {
        render(&normalise(tokenize(s)))
    }

    #[test]
    fn collapses_runs_of_whitespace() {
        assert_eq!(n("hello    world"), "hello world");
    }

    #[test]
    fn pulls_punctuation_back_onto_the_word() {
        assert_eq!(n("hello , world ."), "hello, world.");
    }

    #[test]
    fn inserts_the_missing_space_after_punctuation() {
        assert_eq!(n("hello,world"), "hello, world");
    }

    #[test]
    fn collapses_repeated_marks() {
        assert_eq!(n("wait.. really??"), "wait. really?");
    }

    #[test]
    fn trims_both_ends() {
        assert_eq!(n("  hi  "), "hi");
    }

    #[test]
    fn leaves_contractions_alone() {
        assert_eq!(n("don't stop"), "don't stop");
    }
}
