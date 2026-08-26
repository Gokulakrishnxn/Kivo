//! Mode formatting. A mode changes only how the cleaned text is *presented* —
//! it never changes recognition, and never removes content that the tier-1
//! rules left in place.

use serde::{Deserialize, Serialize};

use super::normalise::is_terminal;
use super::token::{Kind, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Standard,
    Message,
    Email,
    Notes,
    Code,
    Raw,
}

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Standard => "Standard",
            Mode::Message => "Message",
            Mode::Email => "Email",
            Mode::Notes => "Notes",
            Mode::Code => "Code",
            Mode::Raw => "Raw",
        }
    }

    /// Raw is the escape hatch: recognition output, untouched.
    pub fn is_raw(&self) -> bool {
        matches!(self, Mode::Raw)
    }
}

/// Spoken punctuation, for Code mode where saying the symbol is the point.
/// (spoken form, symbol, hugs its neighbours). `user_id` hugs; `a = b` does not.
const SPOKEN_SYMBOLS: &[(&[&str], &str, bool)] = &[
    (&["open", "paren"], "(", true),
    (&["close", "paren"], ")", true),
    (&["open", "bracket"], "[", true),
    (&["close", "bracket"], "]", true),
    (&["open", "brace"], "{", true),
    (&["close", "brace"], "}", true),
    (&["underscore"], "_", true),
    (&["semicolon"], ";", true),
    (&["colon"], ":", true),
    (&["dot"], ".", true),
    (&["slash"], "/", true),
    (&["equals"], "=", false),
    (&["arrow"], "->", false),
    (&["fat", "arrow"], "=>", false),
    (&["star"], "*", false),
    (&["ampersand"], "&", false),
    (&["pipe"], "|", false),
];

/// Spoken layout commands, honoured in every mode except Raw.
const SPOKEN_BREAKS: &[(&[&str], &str, bool)] = &[
    (&["new", "paragraph"], "\n\n", true),
    (&["new", "line"], "\n", true),
    (&["next", "line"], "\n", true),
];

fn substitute(tokens: Vec<Token>, table: &[(&[&str], &str, bool)]) -> Vec<Token> {
    let words: Vec<usize> = (0..tokens.len()).filter(|&i| tokens[i].is_word()).collect();
    let mut swap: Vec<(usize, usize, String, bool)> = Vec::new();

    let mut w = 0;
    'outer: while w < words.len() {
        for (phrase, sym, hugs) in table {
            let hit = phrase.iter().enumerate().all(|(k, want)| {
                words
                    .get(w + k)
                    .map(|&i| tokens[i].lower() == *want)
                    .unwrap_or(false)
            });
            if hit {
                swap.push((
                    words[w],
                    words[w + phrase.len() - 1],
                    (*sym).to_string(),
                    *hugs,
                ));
                w += phrase.len();
                continue 'outer;
            }
        }
        w += 1;
    }
    if swap.is_empty() {
        return tokens;
    }

    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if let Some((_, end, sym, hugs)) = swap.iter().find(|(s, _, _, _)| *s == i) {
            if *hugs {
                while out
                    .last()
                    .map(|t: &Token| t.kind == Kind::Space)
                    .unwrap_or(false)
                {
                    out.pop();
                }
            }
            out.push(Token {
                text: sym.clone(),
                kind: Kind::Punct,
            });
            i = end + 1;
            if *hugs {
                while tokens
                    .get(i)
                    .map(|t| t.kind == Kind::Space)
                    .unwrap_or(false)
                {
                    i += 1;
                }
            }
        } else {
            out.push(tokens[i].clone());
            i += 1;
        }
    }
    out
}

fn capitalise_sentences(tokens: Vec<Token>) -> Vec<Token> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut at_start = true;

    for mut tok in tokens {
        if tok.is_word() {
            if at_start {
                let mut cs = tok.text.chars();
                if let Some(f) = cs.next() {
                    tok.text = f.to_uppercase().collect::<String>() + cs.as_str();
                }
                at_start = false;
            }
        } else if is_terminal(&tok) || tok.text.contains('\n') {
            at_start = true;
        }
        out.push(tok);
    }
    out
}

fn ensure_terminal(mut tokens: Vec<Token>) -> Vec<Token> {
    while tokens
        .last()
        .map(|t| t.kind == Kind::Space)
        .unwrap_or(false)
    {
        tokens.pop();
    }
    let ends_well = tokens.last().map(is_terminal).unwrap_or(true);
    if !ends_well && !tokens.is_empty() {
        tokens.push(Token {
            text: ".".into(),
            kind: Kind::Punct,
        });
    }
    tokens
}

fn strip_single_terminal(mut tokens: Vec<Token>) -> Vec<Token> {
    // Only strip when the whole utterance is one sentence — a trailing stop on
    // a single chat line reads as curt, but removing it from a paragraph is wrong.
    let terminals = tokens.iter().filter(|t| is_terminal(t)).count();
    if terminals == 1 && tokens.last().map(is_terminal).unwrap_or(false) {
        tokens.pop();
    }
    tokens
}

pub fn apply(mode: Mode, tokens: Vec<Token>) -> Vec<Token> {
    if mode.is_raw() {
        return tokens;
    }

    let tokens = substitute(tokens, SPOKEN_BREAKS);

    match mode {
        Mode::Raw => tokens,
        Mode::Code => substitute(tokens, SPOKEN_SYMBOLS),
        Mode::Message => strip_single_terminal(capitalise_sentences(tokens)),
        Mode::Notes => capitalise_sentences(tokens),
        Mode::Standard | Mode::Email => ensure_terminal(capitalise_sentences(tokens)),
    }
}

#[cfg(test)]
mod tests {
    use super::super::token::{render, tokenize};
    use super::*;

    fn m(mode: Mode, s: &str) -> String {
        render(&apply(mode, tokenize(s)))
    }

    #[test]
    fn standard_capitalises_and_terminates() {
        assert_eq!(m(Mode::Standard, "hello there"), "Hello there.");
    }

    #[test]
    fn standard_capitalises_every_sentence() {
        assert_eq!(m(Mode::Standard, "one. two. three"), "One. Two. Three.");
    }

    #[test]
    fn message_drops_a_lone_trailing_stop() {
        assert_eq!(m(Mode::Message, "on my way"), "On my way");
    }

    #[test]
    fn message_keeps_stops_inside_a_paragraph() {
        assert_eq!(m(Mode::Message, "one. two."), "One. Two.");
    }

    #[test]
    fn code_maps_spoken_symbols_and_does_not_capitalise() {
        assert_eq!(m(Mode::Code, "user underscore id"), "user_id");
        assert_eq!(m(Mode::Code, "open paren x close paren"), "(x)");
    }

    #[test]
    fn raw_changes_nothing_at_all() {
        assert_eq!(m(Mode::Raw, "um so new line hello"), "um so new line hello");
    }

    #[test]
    fn spoken_breaks_become_real_newlines() {
        assert_eq!(m(Mode::Notes, "first new line second"), "First\nSecond");
    }
}
