use std::unimplemented;

use utils::profile_scope;

use crate::tokenizer::css_token::CssToken;

pub struct Tokenizer<'a> {
    source: &'a str,
    position: usize, // Index in the &str
    len: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            len: source.len(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<CssToken> {
        profile_scope!("Tokenize CSS");
        let mut tokens = Vec::new();

        while !self.is_eof() {
            if let Some(css_token) = self.next_token() {
                tokens.push(css_token);
            }
        }
        tokens.push(CssToken::EOF);
        tokens
    }

    /// Peeks at the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Advances the position and returns the next character.
    fn advance(&mut self) -> Option<char> {
        let ch = self.source[self.position..].chars().next()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Checks if the tokenizer has reached the end of the input.
    fn is_eof(&self) -> bool {
        self.position >= self.len
    }

    /// Consumes the next token(s) from the input stream (placeholder, not implemented yet).
    fn next_token(&mut self) -> Option<CssToken> {
        // This function would parse slices of the input string as tokens.
        // Placeholder: consume whitespace until non-whitespace or end.
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
        // Not implemented, so always return None.
        None
    }

    pub fn from_string(raw: &str) -> Vec<CssToken> {
        unimplemented!()
    }
}
