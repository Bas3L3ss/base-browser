use std::matches;

use crate::tokenizer::{attributes::Attributes, html_tags::HtmlTags, token::Token};

pub struct Tokenizer<'a> {
    source: &'a str,
    position: usize, // Byte offset in source
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    /// Returns true if we've reached the end of input.
    fn eof(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Returns the current character without advancing, or None if at end.
    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Peeks the next character (without advancing), after the current one.
    #[allow(dead_code)]
    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.position..].chars();
        chars.next()?; // skip current
        chars.next()
    }

    /// Advances the position and returns the character, or None if at end.
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Skips whitespace characters (' ', '\t', '\n', '\r')
    fn skip_whitespace(&mut self) {
        while self.peek().map_or(false, |ch| ch.is_whitespace()) {
            self.advance();
        }
    }

    /// Retrieves a contiguous word (alphanumeric/underscore/hyphen).
    fn get_word(&mut self) -> String {
        let start_pos = self.position;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                self.advance();
            } else {
                break;
            }
        }
        self.source[start_pos..self.position].to_string()
    }

    /// Helper to parse attribute key-value pairs for tags.
    fn parse_attributes(&mut self) -> Attributes {
        let mut attributes = Attributes::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                Some(c) if c == '/' || c == '>' => break,
                Some(_) => {}
                None => break,
            };

            // Parse attribute key
            let mut key = String::new();
            while let Some(ch) = self.peek() {
                if ch == '=' || ch.is_whitespace() || ch == '/' || ch == '>' {
                    break;
                }
                key.push(ch);
                self.advance();
            }
            self.skip_whitespace();

            // Parse '=' and value if exists
            let mut value = String::new();
            if self.peek() == Some('=') {
                self.advance();
                self.skip_whitespace();

                if let Some(quote) = self.peek() {
                    if quote == '"' || quote == '\'' {
                        self.advance(); // skip quote
                        while let Some(vc) = self.peek() {
                            if vc == quote {
                                self.advance();
                                break;
                            }
                            value.push(vc);
                            self.advance();
                        }
                    } else {
                        while let Some(vc) = self.peek() {
                            if vc.is_whitespace() || vc == '/' || vc == '>' {
                                break;
                            }
                            value.push(vc);
                            self.advance();
                        }
                    }
                }
            } else {
                // Boolean attribute, value is the key itself
                value = key.clone();
            }

            attributes.add_attribute(key.trim(), value.trim());
        }

        attributes
    }

    /// Consumes input until a closing sequence (e.g. ">", "-->") and returns the content found until then.
    fn consume_until(&mut self, end: &str) -> String {
        let mut result = String::new();
        loop {
            if self.eof() || self.source[self.position..].starts_with(end) {
                break;
            }
            if let Some(ch) = self.advance() {
                result.push(ch);
            }
        }
        // Advance the correct number of chars, NOT bytes!
        for _ in end.chars() {
            self.advance();
        }
        result
    }

    /// Helper to parse and collect content until the next '<' character.
    fn parse_content(&mut self) -> String {
        let mut content = String::new();
        while !self.eof() {
            if let Some('<') = self.peek() {
                break;
            }
            if let Some(ch) = self.advance() {
                content.push(ch);
            } else {
                break;
            }
        }
        content
    }

    /// Determines if a tag is a self-closing void element by HTML5 spec.
    fn is_void_element(tag: &str) -> bool {
        // List from https://html.spec.whatwg.org/multipage/syntax.html#void-elements
        matches!(
            tag,
            "area"
                | "base"
                | "br"
                | "col"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.eof() {
            let ch = match self.peek() {
                Some(ch) => ch,
                None => break,
            };

            if ch == '<' {
                self.advance();
                self.skip_whitespace();

                // End tag (e.g. </div>)
                if self.peek() == Some('/') {
                    self.advance();
                    self.skip_whitespace();
                    let tag = self.get_word();
                    self.skip_whitespace();
                    if self.peek() == Some('>') {
                        self.advance();
                    }
                    if !tag.is_empty() {
                        tokens.push(Token::End(HtmlTags::from_str(&tag)));
                    }
                }
                // Comment (<!-- ... -->)
                else if self.peek() == Some('!')
                    && self.source[self.position..].starts_with("!--")
                {
                    for _ in 0..3 {
                        self.advance();
                    }
                    let comment = self.consume_until("-->");
                    tokens.push(Token::Comment(comment));
                }
                // Doctype (e.g., <!DOCTYPE html>)
                else if self.peek() == Some('!') && {
                    let rem = &self.source[self.position..].to_ascii_lowercase();
                    rem.starts_with("!doctype")
                } {
                    self.advance(); // skip '!'
                    let word = self.get_word();
                    if word.eq_ignore_ascii_case("doctype") {
                        self.skip_whitespace();
                        let dt_val = self.consume_until(">");
                        tokens.push(Token::Doctype(dt_val.trim().to_string()));
                    } else {
                        // fallback: not a correct doctype, just skip (could emit error)
                    }
                }
                // Start or Self-closing tag
                else {
                    let tag = self.get_word();
                    self.skip_whitespace();
                    let attributes = self.parse_attributes();

                    // -- Browser-correct handling of <img/> vs <div/> vs <img> --

                    // Self-closing syntax <tag ... />
                    if self.source[self.position..].starts_with("/>") {
                        let is_void = Self::is_void_element(tag.as_str());
                        self.advance(); // skip '/'
                        self.advance(); // skip '>'
                        if is_void {
                            tokens.push(Token::SelfClose(HtmlTags::from_str(&tag), attributes));
                        } else {
                            // For non-void elements, browsers treat <div/> as <div>, not self-closing.
                            // Could emit a parse warning here if desired.
                            tokens.push(Token::Start(HtmlTags::from_str(&tag), attributes));
                        }
                    }
                    // Normal opening tag <tag ...>
                    else if self.peek() == Some('>') {
                        self.advance();

                        if Self::is_void_element(tag.as_str()) {
                            tokens.push(Token::SelfClose(HtmlTags::from_str(&tag), attributes));
                        } else {
                            tokens.push(Token::Start(HtmlTags::from_str(&tag), attributes));
                        }
                    }
                    // If we get here, it's a malformed tag. Advance past whatever broke it.
                    else {
                        // This is a simple recovery (advance one char).
                        self.advance();
                    }
                }
            } else {
                // Text node: collect content until next tag
                let content = self.parse_content();
                // Only emit content tokens for non-empty, non-whitespace-only strings
                if !content.is_empty() && !content.chars().all(|ch| ch.is_whitespace()) {
                    tokens.push(Token::Content(content));
                }
            }
        }

        // Always emit EOF token at the end
        tokens.push(Token::EOF);
        tokens
    }
}
