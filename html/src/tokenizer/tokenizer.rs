use std::matches;

use crate::tokenizer::{attributes::Attributes, html_tags::HtmlTag, token::Token};

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
        // Advance the correct number of chars
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
                        tokens.push(Token::End(HtmlTag::from_str(&tag)));
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
                        tokens.push(Token::Doctype(dt_val.trim().to_string())); // TODO: DOCTYPE MORE COMPREHENSIVE SUPPORT
                    } else {
                        // fallback: not a correct doctype, just skip (could emit error)
                    }
                }
                // Start or Self-closing tag
                else {
                    let tag = self.get_word();
                    self.skip_whitespace();
                    let attributes = self.parse_attributes();

                    // Self-closing syntax <tag ... />
                    if self.source[self.position..].starts_with("/>") {
                        let is_void = Self::is_void_element(tag.as_str());
                        self.advance(); // skip '/'
                        self.advance(); // skip '>'
                        if is_void {
                            tokens.push(Token::SelfClose(HtmlTag::from_str(&tag), attributes));
                        } else {
                            // TODO: For non-void elements, browser should treat <div/> as <div>, not self-closing.
                            tokens.push(Token::Start(HtmlTag::from_str(&tag), attributes));
                        }
                    }
                    // Normal opening tag <tag ...>
                    else if self.peek() == Some('>') {
                        self.advance();

                        if Self::is_void_element(tag.as_str()) {
                            tokens.push(Token::SelfClose(HtmlTag::from_str(&tag), attributes));
                        } else {
                            tokens.push(Token::Start(HtmlTag::from_str(&tag), attributes));
                        }
                    } else {
                        // If we get here, it's a malformed tag. Advance past whatever broke it.
                        // This is a simple recovery (advance one char).
                        self.advance();
                    }
                }
            } else {
                // Text node: collect content until next tag
                let content = self.parse_content();
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

#[cfg(test)]
mod tests {
    use std::{assert_eq, vec};

    use super::*;
    use crate::tokenizer::html_tags::HtmlTag;
    use crate::tokenizer::token::Token;

    fn tokens_without_eof(mut toks: Vec<Token>) -> Vec<Token> {
        // Remove trailing EOF for easier assertion
        if !toks.is_empty() && matches!(toks.last(), Some(Token::EOF)) {
            toks.pop();
        }
        toks
    }

    #[test]
    fn test_tokenize_simple_tag() {
        let mut tokenizer = Tokenizer::new("<div></div>");
        let tokens = tokenizer.tokenize();
        assert_eq!(
            tokens_without_eof(tokens),
            vec![
                Token::Start(HtmlTag::Div, Attributes::new()),
                Token::End(HtmlTag::Div)
            ]
        );
    }

    #[test]
    fn test_tokenize_with_text_content() {
        let mut tokenizer = Tokenizer::new("<b>Hello World!</b>");
        let tokens = tokens_without_eof(tokenizer.tokenize());
        assert_eq!(
            tokens,
            vec![
                Token::Start(HtmlTag::Bold, Attributes::new()),
                Token::Content("Hello World!".to_string()),
                Token::End(HtmlTag::Bold)
            ]
        );
    }

    #[test]
    fn test_tokenize_self_closing_void_element() {
        let mut tokenizer = Tokenizer::new("<img src=\"x\">");
        let tokens = tokens_without_eof(tokenizer.tokenize());
        assert_eq!(
            tokens,
            vec![Token::SelfClose(HtmlTag::Image, {
                let mut attrs = Attributes::new();
                attrs.add_attribute("src", "x");
                attrs
            })]
        );
    }

    #[test]
    fn test_tokenize_explicit_self_closing_nonvoid_treated_as_start() {
        // Non-void elements like <div/> should produce Start, not SelfClose.
        let mut tokenizer = Tokenizer::new("<div/>");
        let tokens = tokens_without_eof(tokenizer.tokenize());
        assert_eq!(tokens, vec![Token::Start(HtmlTag::Div, Attributes::new())]);
    }

    #[test]
    fn test_tokenize_comment_and_doctype() {
        let mut tokenizer = Tokenizer::new("<!DOCTYPE html><!-- comment --><br>");
        let tokens = tokens_without_eof(tokenizer.tokenize());
        assert_eq!(
            tokens,
            vec![
                Token::Doctype("html".to_string()),
                Token::Comment(" comment ".to_string()),
                Token::SelfClose(HtmlTag::Br, Attributes::new()),
            ]
        );
    }

    #[test]
    fn test_tokenize_attributes() {
        let mut tokenizer = Tokenizer::new("<input type='text' checked placeholder=abc>");
        let tokens = tokens_without_eof(tokenizer.tokenize());
        let mut attrs = Attributes::new();
        attrs.add_attribute("type", "text");
        attrs.add_attribute("checked", "checked");
        attrs.add_attribute("placeholder", "abc");
        assert_eq!(tokens, vec![Token::SelfClose(HtmlTag::Input, attrs)]);
    }

    #[test]
    fn test_tokenize_text_and_whitespace() {
        let mut tokenizer = Tokenizer::new("   <span> x </span>   ");
        let tokens = tokens_without_eof(tokenizer.tokenize());
        assert_eq!(
            tokens,
            vec![
                Token::Start(HtmlTag::Span, Attributes::new()),
                Token::Content(" x ".to_string()),
                Token::End(HtmlTag::Span)
            ]
        );
    }

    #[test]
    fn test_tokenize_malformed_tag_recovery() {
        // This tag has invalid syntax: <div Hello> -- should recover and at least produce a Start div.
        let mut tokenizer = Tokenizer::new("<div Hello>");
        let tokens = tokens_without_eof(tokenizer.tokenize());
        assert_eq!(
            tokens,
            vec![Token::Start(HtmlTag::Div, {
                let mut attrs = Attributes::new();
                attrs.add_attribute("Hello", "Hello");
                attrs
            })]
        );
    }

    #[test]
    fn test_tokenize_multiple_elements() {
        let mut tokenizer = Tokenizer::new("<div>1</div><div>2</div>");
        let tokens = tokens_without_eof(tokenizer.tokenize());
        assert_eq!(
            tokens,
            vec![
                Token::Start(HtmlTag::Div, Attributes::new()),
                Token::Content("1".to_string()),
                Token::End(HtmlTag::Div),
                Token::Start(HtmlTag::Div, Attributes::new()),
                Token::Content("2".to_string()),
                Token::End(HtmlTag::Div),
            ]
        );
    }

    #[test]
    fn test_tokenize_empty_input_gives_eof() {
        let mut tokenizer = Tokenizer::new("");
        let tokens = tokenizer.tokenize();
        // Only EOF token
        assert_eq!(tokens, vec![Token::EOF]);
    }
}
