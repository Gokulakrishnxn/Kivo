//! A tiny word/punctuation tokeniser.
//!
//! The cleanup rules all operate on word boundaries — "the the" must collapse
//! but "theme" must not, and a replacement for "kubernetes" must not fire
//! inside "kubernetesctl". Doing that with string search invites exactly those
//! bugs, so every rule works on tokens and the text is rendered once at the end.
//! It also keeps us off the regex crate: this is a resident process handling
//! private audio, and every dependency is attack surface.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Word,
    Punct,
    Space,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub kind: Kind,
}

impl Token {
    pub fn is_word(&self) -> bool {
        self.kind == Kind::Word
    }

    /// Lowercased, with any trailing possessive or contraction left intact.
    pub fn lower(&self) -> String {
        self.text.to_lowercase()
    }
}

/// Split text into words, punctuation and runs of whitespace.
///
/// Apostrophes and internal hyphens stay inside the word, so "don't" and
/// "well-known" survive as single tokens rather than fragmenting.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            out.push(Token {
                text: chars[start..i].iter().collect(),
                kind: Kind::Space,
            });
        } else if c.is_alphanumeric() {
            let start = i;
            while i < chars.len() {
                let ch = chars[i];
                let joins_word = ch.is_alphanumeric()
                    || ((ch == '\'' || ch == '\u{2019}' || ch == '-')
                        && i + 1 < chars.len()
                        && chars[i + 1].is_alphanumeric());
                if !joins_word {
                    break;
                }
                i += 1;
            }
            out.push(Token {
                text: chars[start..i].iter().collect(),
                kind: Kind::Word,
            });
        } else {
            out.push(Token {
                text: c.to_string(),
                kind: Kind::Punct,
            });
            i += 1;
        }
    }
    out
}

/// Render tokens back to a string verbatim. Spacing is fixed separately by
/// the tier-0 normaliser, so rules are free to delete tokens without having
/// to think about the whitespace they leave behind.
pub fn render(tokens: &[Token]) -> String {
    tokens.iter().map(|t| t.text.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_contractions_and_hyphens_whole() {
        let t = tokenize("don't well-known");
        let words: Vec<_> = t
            .iter()
            .filter(|t| t.is_word())
            .map(|t| t.text.clone())
            .collect();
        assert_eq!(words, vec!["don't", "well-known"]);
    }

    #[test]
    fn a_trailing_hyphen_is_punctuation_not_part_of_the_word() {
        let t = tokenize("wait - no");
        let words: Vec<_> = t
            .iter()
            .filter(|t| t.is_word())
            .map(|t| t.text.clone())
            .collect();
        assert_eq!(words, vec!["wait", "no"]);
    }

    #[test]
    fn round_trips_exactly() {
        let s = "Hello,  world — it's  fine.";
        assert_eq!(render(&tokenize(s)), s);
    }
}
