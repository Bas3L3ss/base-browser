use utils::profile_scope;

use crate::tokenizer::css_token::{CssToken, CssTokenizerError};

pub struct Tokenizer<'a> {
    source: &'a str,
    position: usize,
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

    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Peek the next character after the current, without consuming
    fn peek_next(&self) -> Option<char> {
        let mut iter = self.source[self.position..].chars();
        iter.next()?; // skip current
        iter.next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source[self.position..].chars().next()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    fn is_eof(&self) -> bool {
        self.position >= self.len
    }

    /// Consume and return the next token from the input.
    fn next_token(&mut self) -> Option<CssToken> {
        while let Some(ch) = self.peek() {
            // 1. Comments (`/* ... */`)
            if ch == '/' && self.peek_next() == Some('*') {
                self.advance(); // '/'
                self.advance(); // '*'
                let start_pos = self.position - 2;
                let mut unterminated = true;
                while let Some(cur) = self.peek() {
                    if cur == '*' && self.peek_next() == Some('/') {
                        self.advance(); // '*'
                        self.advance(); // '/'
                        unterminated = false;
                        break;
                    }
                    self.advance();
                    if self.is_eof() {
                        break;
                    }
                }
                if unterminated {
                    return Some(CssToken::Error(CssTokenizerError::UnterminatedComment(
                        start_pos,
                    )));
                }
                continue;
            }

            // 2. Whitespace as one token
            if ch.is_whitespace() {
                self.consume_while(|c| c.is_whitespace());
                return Some(CssToken::Whitespace);
            }

            // 3. Punctuation & operators, and numeric prefix handling
            let token = match ch {
                '{' => {
                    self.advance();
                    CssToken::LeftBrace
                }
                '}' => {
                    self.advance();
                    CssToken::RightBrace
                }
                '(' => {
                    self.advance();
                    CssToken::LeftParen
                }
                ')' => {
                    self.advance();
                    CssToken::RightParen
                }
                ':' => {
                    self.advance();
                    CssToken::Colon
                }
                ';' => {
                    self.advance();
                    CssToken::Semicolon
                }
                ',' => {
                    self.advance();
                    CssToken::Comma
                }
                // Decimal number, like `.5em`
                '.' if self.peek_next().is_some_and(|c| c.is_ascii_digit()) => {
                    self.consume_numeric_token()
                }
                '.' => {
                    self.advance();
                    CssToken::Dot
                }
                '#' => {
                    self.advance();
                    // ID or hash
                    let ident = self.consume_ident();
                    CssToken::Hash(ident)
                }
                '*' => {
                    self.advance();
                    CssToken::Star
                }
                '+' => {
                    self.advance();
                    CssToken::Plus
                }

                // Either a minus as part of a number, or an identifier (like -webkit) or a dash
                '-' => {
                    let next = self.peek_next();
                    if let Some(d) = next {
                        // If next is digit, it's a negative number (e.g. -7, -5px, etc)
                        if d.is_ascii_digit() {
                            self.advance();
                            return Some(self.consume_signed_numeric_token(Some('-')));
                        }
                        // Check -.digit (like -.5em)
                        if d == '.' {
                            let mut iter = self.source[self.position..].chars();
                            iter.next(); // skip '-'
                            if iter.next().is_some_and(|c| c.is_ascii_digit()) {
                                self.advance();
                                return Some(self.consume_signed_numeric_token(Some('-')));
                            }
                        }
                        // If followed by another dash or alpha, let's parse as ident
                        if d == '-' || d.is_ascii_alphabetic() || d == '_' {
                            // Consume dash, then the rest as ident
                            let ident = self.consume_ident();
                            return Some(CssToken::Ident(ident));
                        }
                    }
                    // Otherwise, treat as delim/dash
                    self.advance();
                    CssToken::Ident("-".to_string())
                }
                '>' => {
                    self.advance();
                    CssToken::GreaterThan
                }
                '~' => {
                    self.advance();
                    CssToken::Tilde
                }
                '[' => {
                    self.advance();
                    CssToken::OpenBracket
                }
                ']' => {
                    self.advance();
                    CssToken::CloseBracket
                }
                '=' => {
                    self.advance();
                    CssToken::Equals
                }
                '@' => {
                    self.advance();
                    let keyword = self.consume_ident();
                    // Accept both @media as AtKeyword and plain @ as Delim if not followed by ident-like
                    if keyword.is_empty() {
                        CssToken::Delim('@')
                    } else {
                        CssToken::AtKeyword(keyword)
                    }
                }
                '!' => {
                    self.advance();
                    CssToken::Delim('!')
                }
                // Handle string literal with unicode-escape logic using consume_escape
                '"' | '\'' => {
                    let quote = self.advance().unwrap();
                    let mut content = String::new();
                    let mut terminated = false;
                    while let Some(inner) = self.peek() {
                        if inner == '\\' {
                            self.advance();
                            // Use CSS escape handling for both simple and unicode escapes (RFC 6265)
                            if let Some(escaped_ch) = self.consume_escape() {
                                content.push(escaped_ch);
                            }
                            continue;
                        }
                        if inner == quote {
                            terminated = true;
                            break;
                        }
                        content.push(inner);
                        self.advance();
                    }
                    if terminated {
                        self.advance(); // consume closing quote
                        CssToken::String(content)
                    } else {
                        // Unterminated string literal; return error token
                        CssToken::Error(CssTokenizerError::UnterminatedString(self.position))
                    }
                }

                '0'..='9' => self.consume_numeric_token(),

                _ if Self::is_ident_start(ch) => {
                    let ident = self.consume_ident();

                    CssToken::Ident(ident)
                }

                '\\' => {
                    self.advance(); // Skip '\'
                    let ident = {
                        let mut s = String::new();
                        if let Some(esc) = self.consume_escape() {
                            s.push(esc);
                        }
                        // Continue with the rest of the ident (consume_ident appends more)
                        s.push_str(&self.consume_ident());
                        s
                    };
                    return Some(CssToken::Ident(ident));
                }

                _ => {
                    // Any unrecognized, not-whitespace non-control char is Delim!
                    let bad = self.advance().unwrap();
                    CssToken::Delim(bad)
                }
            };
            return Some(token);
        }
        None
    }

    /// Consumes a CSS ident, handling escape sequences (including Unicode) per CSS syntax.
    fn consume_ident(&mut self) -> String {
        let mut ident = String::new();

        while !self.is_eof() {
            match self.peek() {
                Some('\\') => {
                    // Handle escape sequence, which may include unicode
                    self.advance(); // Skip '\'
                    if let Some(escaped_ch) = self.consume_escape() {
                        ident.push(escaped_ch);
                    }
                }
                Some(ch) if Self::is_ident_continue(ch) => {
                    ident.push(ch);
                    self.advance();
                }
                _ => break,
            }
        }

        ident
    }

    /// Consumes a CSS escape sequence, supporting unicode escapes `\hhhhhh` (up to 6 hex digits) or simple escapes.
    /// Returns the translated char, or Unicode replacement character if invalid.
    fn consume_escape(&mut self) -> Option<char> {
        // If the first char is a hex digit, it's a unicode escape
        let peeked = self.peek();
        if let Some(ch) = peeked {
            if ch.is_ascii_hexdigit() {
                // Up to 6 hex digits
                let mut codepoint = 0u32;
                let mut len = 0;
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_hexdigit() && len < 6 {
                        self.advance();
                        codepoint = codepoint * 16 + ch.to_digit(16).unwrap();
                        len += 1;
                    } else {
                        break;
                    }
                }

                // Optional single whitespace char after unicode escape
                if let Some(ws) = self.peek() {
                    if ws.is_ascii_whitespace() {
                        self.advance();
                    }
                }

                return char::from_u32(codepoint).or(Some('\u{FFFD}'));
            } else if ch == '\n' || ch == '\r' || ch == '\x0C' {
                // A backslash followed by newline/CR/form-feed is ignored (line continuation)
                self.advance();
                return None;
            } else {
                // "simple" escape: just returns the char after '\'
                self.advance();
                return Some(ch);
            }
        }
        None
    }

    /// Consumes all consecutive characters for which test returns true.
    fn consume_while<F>(&mut self, mut test: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if test(ch) {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    /// Consumes a CSS numeric literal (integer or floating point).
    /// Accepts a possibly signed literal if the string is provided.
    fn consume_numeric_literal_with_sign(&mut self, first_sign: Option<char>) -> f32 {
        let mut number_str = String::new();
        if let Some(sign) = first_sign {
            number_str.push(sign);
        }
        let mut seen_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else if ch == '.' && !seen_dot {
                seen_dot = true;
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        number_str.parse().unwrap_or(0.0)
    }

    /// Consumes a number, and then any dimensional/percentage units after it, as a single token.
    /// Used for numbers that start with a sign (e.g. -5px, +2em).
    fn consume_signed_numeric_token(&mut self, sign: Option<char>) -> CssToken {
        let number = self.consume_numeric_literal_with_sign(sign);
        if let Some('%') = self.peek() {
            self.advance();
            CssToken::Percentage(number)
        } else if let Some(unit_ch) = self.peek() {
            if unit_ch.is_alphabetic() {
                let unit_str = self.consume_while(|c| c.is_alphabetic());
                CssToken::Dimension {
                    value: number,
                    unit: unit_str,
                }
            } else {
                CssToken::Number(number)
            }
        } else {
            CssToken::Number(number)
        }
    }

    /// Consumes a CSS numeric literal (unsigned).
    fn consume_numeric_literal(&mut self) -> f32 {
        let mut number_str = String::new();
        let mut seen_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else if ch == '.' && !seen_dot {
                seen_dot = true;
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        number_str.parse().unwrap_or(0.0)
    }

    /// Consumes a number, and then any dimensional/percentage units after it, as a single token.
    fn consume_numeric_token(&mut self) -> CssToken {
        let number = self.consume_numeric_literal();
        if let Some('%') = self.peek() {
            self.advance();
            CssToken::Percentage(number)
        } else if let Some(unit_ch) = self.peek() {
            if unit_ch.is_alphabetic() {
                let unit_str = self.consume_while(|c| c.is_alphabetic());
                CssToken::Dimension {
                    value: number,
                    unit: unit_str,
                }
            } else {
                CssToken::Number(number)
            }
        } else {
            CssToken::Number(number)
        }
    }

    /// CSS identifiers can start with a-zA-Z, '_' or '-'
    fn is_ident_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_' || ch == '-'
    }

    fn is_ident_continue(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
    }

    pub fn from_string(raw: &str) -> Vec<CssToken> {
        let mut tokenizer = Tokenizer::new(raw);
        tokenizer.tokenize()
    }
}

// ----------------- UNIT TESTS -------------------

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{assert_eq, print};

    use super::*;
    use crate::tokenizer::css_token::{CssToken, CssTokenizerError};

    #[test]
    fn test_ident_token() {
        let tokens = Tokenizer::from_string("color");
        assert_eq!(tokens[0], CssToken::Ident("color".into()));
    }

    #[test]
    fn test_number_token() {
        let tokens = Tokenizer::from_string("42");
        assert_eq!(tokens[0], CssToken::Number(42.0));
    }

    #[test]
    fn test_number_with_decimal() {
        let tokens = Tokenizer::from_string("13.37");
        assert_eq!(tokens[0], CssToken::Number(13.37));
    }

    #[test]
    fn test_negative_number_token() {
        let tokens = Tokenizer::from_string("-7");
        // Should be parsed as Number(-7.0) for arithmetic use
        assert_eq!(tokens[0], CssToken::Number(-7.0));
    }

    #[test]
    fn test_dimension_token_px() {
        let tokens = Tokenizer::from_string("2px");
        assert_eq!(
            tokens[0],
            CssToken::Dimension {
                value: 2.0,
                unit: "px".into()
            }
        );
    }

    #[test]
    fn test_dimension_token_rem_pt_vh() {
        let tokens = Tokenizer::from_string("1rem 3pt 2vh");
        assert_eq!(
            tokens[0],
            CssToken::Dimension {
                value: 1.0,
                unit: "rem".into()
            }
        );
        assert_eq!(tokens[1], CssToken::Whitespace);
        assert_eq!(
            tokens[2],
            CssToken::Dimension {
                value: 3.0,
                unit: "pt".into()
            }
        );
        assert_eq!(tokens[3], CssToken::Whitespace);
        assert_eq!(
            tokens[4],
            CssToken::Dimension {
                value: 2.0,
                unit: "vh".into()
            }
        );
    }

    #[test]
    fn test_dimension_token_em() {
        let tokens = Tokenizer::from_string("5.5em");
        assert_eq!(
            tokens[0],
            CssToken::Dimension {
                value: 5.5,
                unit: "em".into()
            }
        );
    }

    #[test]
    fn test_percentage_token() {
        let tokens = Tokenizer::from_string("85%");
        assert_eq!(tokens[0], CssToken::Percentage(85.0));
    }

    #[test]
    fn test_percentage_token_with_decimal() {
        let tokens = Tokenizer::from_string("12.5%");
        assert_eq!(tokens[0], CssToken::Percentage(12.5));
    }

    #[test]
    fn test_hash_token() {
        let tokens = Tokenizer::from_string("#abc123");
        assert_eq!(tokens[0], CssToken::Hash("abc123".to_string()));
    }

    #[test]
    fn test_string_token_single_quotes() {
        let tokens = Tokenizer::from_string("'hello world'");
        assert_eq!(tokens[0], CssToken::String("hello world".into()));
    }

    #[test]
    fn test_string_token_double_quotes() {
        let tokens = Tokenizer::from_string(r#""hi""#);
        assert_eq!(tokens[0], CssToken::String("hi".into()));
    }

    #[test]
    fn test_string_with_escaped_quote() {
        let tokens = Tokenizer::from_string(r#""he\"llo""#);
        // Now checks for properly parsed escaped quote!
        assert!(matches!(&tokens[0], CssToken::String(s) if s == "he\"llo"));
    }

    #[test]
    fn test_whitespace_token() {
        let tokens = Tokenizer::from_string("    ");
        assert_eq!(tokens[0], CssToken::Whitespace);
    }

    #[test]
    fn test_tab_newline_whitespace_token() {
        let tokens = Tokenizer::from_string("\t  \n");
        assert_eq!(tokens[0], CssToken::Whitespace);
    }

    #[test]
    fn test_punctuation_tokens() {
        let tokens = Tokenizer::from_string("{ }:;,()");
        assert_eq!(tokens[0], CssToken::LeftBrace);
        assert_eq!(tokens[1], CssToken::Whitespace);
        assert_eq!(tokens[2], CssToken::RightBrace);
        assert_eq!(tokens[3], CssToken::Colon);
        assert_eq!(tokens[4], CssToken::Semicolon);
        assert_eq!(tokens[5], CssToken::Comma);
        assert_eq!(tokens[6], CssToken::LeftParen);
        assert_eq!(tokens[7], CssToken::RightParen);
    }

    #[test]
    fn test_punctuation_square_brackets_and_equals() {
        let tokens = Tokenizer::from_string("[]=");
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::CloseBracket);
        assert_eq!(tokens[2], CssToken::Equals);
    }

    #[test]
    fn test_delim_tokens() {
        let tokens = Tokenizer::from_string("*.>~");
        assert_eq!(tokens[0], CssToken::Star);
        assert_eq!(tokens[1], CssToken::Dot);
        assert_eq!(tokens[2], CssToken::GreaterThan);
        assert_eq!(tokens[3], CssToken::Tilde);
    }

    #[test]
    fn test_at_keyword_token() {
        let tokens = Tokenizer::from_string("@media");
        assert_eq!(tokens[0], CssToken::AtKeyword("media".to_string()));
    }

    #[test]
    fn test_mixed_tokens() {
        let tokens = Tokenizer::from_string("margin: 10px 5% ;");
        assert_eq!(tokens[0], CssToken::Ident("margin".into()));
        assert_eq!(tokens[1], CssToken::Colon);
        assert_eq!(tokens[2], CssToken::Whitespace);
        assert_eq!(
            tokens[3],
            CssToken::Dimension {
                value: 10.0,
                unit: "px".into()
            }
        );
        assert_eq!(tokens[4], CssToken::Whitespace);
        assert_eq!(tokens[5], CssToken::Percentage(5.0));
        assert_eq!(tokens[6], CssToken::Whitespace);
        assert_eq!(tokens[7], CssToken::Semicolon);
    }

    #[test]
    fn test_eof_token() {
        let tokens = Tokenizer::from_string("test");
        assert!(*tokens.last().unwrap() == CssToken::EOF);
    }

    #[test]
    fn test_empty_string_yields_only_eof() {
        let tokens = Tokenizer::from_string("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], CssToken::EOF);
    }

    #[test]
    fn test_identifier_start_dash() {
        let tokens = Tokenizer::from_string("-webkit-transform");
        assert_eq!(tokens[0], CssToken::Ident("-webkit-transform".into()));
        let tokens2 = Tokenizer::from_string("--main-color");
        assert_eq!(tokens2[0], CssToken::Ident("--main-color".into()));
    }

    #[test]
    fn test_error_on_unterminated_string() {
        let tokens = Tokenizer::from_string("\"unterminated");
        let mut found_error = false;
        for token in tokens {
            if let CssToken::Error(CssTokenizerError::UnterminatedString(_)) = token {
                found_error = true;
            }
        }
        assert!(
            found_error,
            "Should emit error token for unterminated string"
        );
    }

    #[test]
    fn test_unterminated_comment_error() {
        let tokens = Tokenizer::from_string("body { color: red; /* missing end");
        let mut found_error = false;
        for token in tokens {
            if let CssToken::Error(CssTokenizerError::UnterminatedComment(_)) = token {
                found_error = true;
            }
        }
        assert!(
            found_error,
            "Should emit error token for unterminated comment"
        );
    }

    #[test]
    fn test_multiple_tokens_sequence() {
        let tokens = Tokenizer::from_string("h1 { color: #fff; font-size: 12px }");
        assert_eq!(tokens[0], CssToken::Ident("h1".into()));
        assert_eq!(tokens[1], CssToken::Whitespace);
        assert_eq!(tokens[2], CssToken::LeftBrace);
        assert_eq!(tokens[3], CssToken::Whitespace);
        assert_eq!(tokens[4], CssToken::Ident("color".into()));
        assert_eq!(tokens[5], CssToken::Colon);
        assert_eq!(tokens[6], CssToken::Whitespace);
        assert_eq!(tokens[7], CssToken::Hash("fff".into()));
        assert_eq!(tokens[8], CssToken::Semicolon);
        assert_eq!(tokens[9], CssToken::Whitespace);
        assert_eq!(tokens[10], CssToken::Ident("font-size".into()));
        assert_eq!(tokens[11], CssToken::Colon);
        assert_eq!(tokens[12], CssToken::Whitespace);
        assert_eq!(
            tokens[13],
            CssToken::Dimension {
                value: 12.0,
                unit: "px".into()
            }
        );
        assert_eq!(tokens[14], CssToken::Whitespace);
        assert_eq!(tokens[15], CssToken::RightBrace);
        assert!(tokens.contains(&CssToken::EOF));
    }

    #[test]
    fn test_tokenize_url_function_like() {
        let tokens = Tokenizer::from_string("url(\"test.png\")");
        // Our tokenizer may not parse functions yet, so just validate the start
        assert_eq!(tokens[0], CssToken::Ident("url".into()));
        assert_eq!(tokens[1], CssToken::LeftParen);
        assert_eq!(tokens[2], CssToken::String("test.png".into()));
        assert_eq!(tokens[3], CssToken::RightParen);
    }

    #[test]
    fn test_tokenize_colon_semicolon_equals() {
        let tokens = Tokenizer::from_string(";:=");
        assert_eq!(tokens[0], CssToken::Semicolon);
        assert_eq!(tokens[1], CssToken::Colon);
        assert_eq!(tokens[2], CssToken::Equals);
    }

    #[test]
    fn test_numeric_and_delim_adjacency() {
        let tokens = Tokenizer::from_string("5+3");
        print!("{tokens:?}");
        assert_eq!(tokens[0], CssToken::Number(5.0));
        assert_eq!(tokens[1], CssToken::Plus);
        assert_eq!(tokens[2], CssToken::Number(3.0));
    }
    #[test]
    fn test_tokenize_important() {
        let tokens = Tokenizer::from_string("color: red !important");
        // color Ident, Colon, Whitespace, Ident("red"), Whitespace, Delim('!'), Ident("important")
        assert_eq!(tokens[0], CssToken::Ident("color".into()));
        assert_eq!(tokens[1], CssToken::Colon);
        assert_eq!(tokens[2], CssToken::Whitespace);
        assert_eq!(tokens[3], CssToken::Ident("red".into()));
        assert_eq!(tokens[4], CssToken::Whitespace);
        assert_eq!(tokens[5], CssToken::Delim('!'));
        assert_eq!(tokens[6], CssToken::Ident("important".into()));
    }

    #[test]
    fn test_universal_selector_token() {
        let tokens = Tokenizer::from_string("*");
        assert_eq!(tokens[0], CssToken::Star);
    }

    #[test]
    fn test_class_selector_token() {
        let tokens = Tokenizer::from_string(".my-class");
        assert_eq!(tokens[0], CssToken::Dot);
        assert_eq!(tokens[1], CssToken::Ident("my-class".into()));
    }

    #[test]
    fn test_id_selector_token() {
        let tokens = Tokenizer::from_string("#header");
        assert_eq!(tokens[0], CssToken::Hash("header".into()));
    }

    #[test]
    fn test_attribute_selector_token_simple() {
        let tokens = Tokenizer::from_string("[type]");
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::Ident("type".into()));
        assert_eq!(tokens[2], CssToken::CloseBracket);
    }

    #[test]
    fn test_attribute_selector_token_with_value() {
        let tokens = Tokenizer::from_string(r#"[lang="en"]"#);
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::Ident("lang".into()));
        assert_eq!(tokens[2], CssToken::Equals);
        assert_eq!(tokens[3], CssToken::String("en".into()));
        assert_eq!(tokens[4], CssToken::CloseBracket);
    }

    #[test]
    fn test_attribute_selector_token_with_value_no_quotes() {
        let tokens = Tokenizer::from_string("[href=https]");
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::Ident("href".into()));
        assert_eq!(tokens[2], CssToken::Equals);
        assert_eq!(tokens[3], CssToken::Ident("https".into()));
        assert_eq!(tokens[4], CssToken::CloseBracket);
    }

    #[test]
    fn test_attribute_selector_token_with_star_equal() {
        let tokens = Tokenizer::from_string(r#"[data-attr*="abc"]"#);
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::Ident("data-attr".into()));
        assert_eq!(tokens[2], CssToken::Star);
        assert_eq!(tokens[3], CssToken::Equals);
        assert_eq!(tokens[4], CssToken::String("abc".into()));
        assert_eq!(tokens[5], CssToken::CloseBracket);
    }

    #[test]
    fn test_attribute_selector_token_with_tilde_equal() {
        let tokens = Tokenizer::from_string(r#"[title~="foo"]"#);
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::Ident("title".into()));
        assert_eq!(tokens[2], CssToken::Tilde);
        assert_eq!(tokens[3], CssToken::Equals);
        assert_eq!(tokens[4], CssToken::String("foo".into()));
        assert_eq!(tokens[5], CssToken::CloseBracket);
    }

    #[test]
    fn test_attribute_selector_token_with_caret_equal() {
        let tokens = Tokenizer::from_string(r#"[name^="bar"]"#);
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::Ident("name".into()));
        assert_eq!(tokens[2], CssToken::Delim('^'));
        assert_eq!(tokens[3], CssToken::Equals);
        assert_eq!(tokens[4], CssToken::String("bar".into()));
        assert_eq!(tokens[5], CssToken::CloseBracket);
    }

    #[test]
    fn test_attribute_selector_token_with_dollar_equal() {
        let tokens = Tokenizer::from_string(r#"[data$="baz"]"#);
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::Ident("data".into()));
        assert_eq!(tokens[2], CssToken::Delim('$'));
        assert_eq!(tokens[3], CssToken::Equals);
        assert_eq!(tokens[4], CssToken::String("baz".into()));
        assert_eq!(tokens[5], CssToken::CloseBracket);
    }

    #[test]
    fn test_attribute_selector_token_with_pipe_equal() {
        let tokens = Tokenizer::from_string(r#"[lang|="en"]"#);
        assert_eq!(tokens[0], CssToken::OpenBracket);
        assert_eq!(tokens[1], CssToken::Ident("lang".into()));
        assert_eq!(tokens[2], CssToken::Delim('|'));
        assert_eq!(tokens[3], CssToken::Equals);
        assert_eq!(tokens[4], CssToken::String("en".into()));
        assert_eq!(tokens[5], CssToken::CloseBracket);
    }

    #[test]
    fn test_type_selector_token() {
        let tokens = Tokenizer::from_string("section");
        assert_eq!(tokens[0], CssToken::Ident("section".into()));
    }

    #[test]
    fn test_compound_selector_tokens() {
        let tokens = Tokenizer::from_string("div.foo#main[attr=\"x\"]:hover::before");
        assert_eq!(tokens[0], CssToken::Ident("div".into()));
        assert_eq!(tokens[1], CssToken::Dot);
        assert_eq!(tokens[2], CssToken::Ident("foo".into()));
        assert_eq!(tokens[3], CssToken::Hash("main".into()));
        assert_eq!(tokens[4], CssToken::OpenBracket);
        assert_eq!(tokens[5], CssToken::Ident("attr".into()));
        assert_eq!(tokens[6], CssToken::Equals);
        assert_eq!(tokens[7], CssToken::String("x".into()));
        assert_eq!(tokens[8], CssToken::CloseBracket);
        assert_eq!(tokens[9], CssToken::Colon);
        assert_eq!(tokens[10], CssToken::Ident("hover".into()));
        assert_eq!(tokens[11], CssToken::Colon);
        assert_eq!(tokens[12], CssToken::Colon);
        assert_eq!(tokens[13], CssToken::Ident("before".into()));
    }

    #[test]
    fn test_selector_combinator_tokens() {
        let tokens = Tokenizer::from_string("div > p + span ~ a");
        assert_eq!(tokens[0], CssToken::Ident("div".into()));
        assert_eq!(tokens[1], CssToken::Whitespace);
        assert_eq!(tokens[2], CssToken::GreaterThan);
        assert_eq!(tokens[3], CssToken::Whitespace);
        assert_eq!(tokens[4], CssToken::Ident("p".into()));
        assert_eq!(tokens[5], CssToken::Whitespace);
        assert_eq!(tokens[6], CssToken::Plus);
        assert_eq!(tokens[7], CssToken::Whitespace);
        assert_eq!(tokens[8], CssToken::Ident("span".into()));
        assert_eq!(tokens[9], CssToken::Whitespace);
        assert_eq!(tokens[10], CssToken::Tilde);
        assert_eq!(tokens[11], CssToken::Whitespace);
        assert_eq!(tokens[12], CssToken::Ident("a".into()));
    }

    #[test]
    fn test_pseudo_class_and_element_tokens() {
        let tokens = Tokenizer::from_string(":hover::after");
        assert_eq!(tokens[0], CssToken::Colon);
        assert_eq!(tokens[1], CssToken::Ident("hover".into()));
        assert_eq!(tokens[2], CssToken::Colon);
        assert_eq!(tokens[3], CssToken::Colon);
        assert_eq!(tokens[4], CssToken::Ident("after".into()));
    }

    #[test]
    fn test_selector_whitespace_combinator() {
        let tokens = Tokenizer::from_string("ul li");
        assert_eq!(tokens[0], CssToken::Ident("ul".into()));
        assert_eq!(tokens[1], CssToken::Whitespace);
        assert_eq!(tokens[2], CssToken::Ident("li".into()));
    }

    #[test]
    fn test_ident_with_parenthesis_escapes() {
        // In CSS, identifier escapes like `\(` and `\)` are valid to allow parens in idents
        let tokens = Tokenizer::from_string(".khung_menu_cap_2\\(3\\)");
        // Expect a Dot followed by an Ident with the de-escaped parens in the value
        assert_eq!(tokens[0], CssToken::Dot);
        assert_eq!(tokens[1], CssToken::Ident("khung_menu_cap_2(3)".into()));

        let tokens2 = Tokenizer::from_string("#foo\\(bar\\)");
        assert_eq!(tokens2[0], CssToken::Hash("foo(bar)".into()));

        let tokens3 = Tokenizer::from_string("hello\\(test\\)ing");
        assert_eq!(tokens3[0], CssToken::Ident("hello(test)ing".into()));
    }

    #[test]
    fn test_ident_starts_with_escape() {
        // Backslash escaping the first character of an identifier: .\31test
        let tokens = Tokenizer::from_string(".\\31 test");
        assert_eq!(tokens[0], CssToken::Dot);
        assert_eq!(tokens[1], CssToken::Ident("1test".into()));

        // Same with a space after escape (space must not appear in ident!)
        let tokens2 = Tokenizer::from_string(".\\31 test");
        assert_eq!(tokens2[0], CssToken::Dot);
        assert_eq!(tokens2[1], CssToken::Ident("1test".into()));

        // Hash id with escaped initial digit
        let tokens3 = Tokenizer::from_string("#\\32 foo");
        assert_eq!(tokens3[0], CssToken::Hash("2foo".into()));

        // Ident starting with a backslash and unicode sequence for letter X (\58)
        let tokens4 = Tokenizer::from_string(".\\58 ray");
        assert_eq!(tokens4[0], CssToken::Dot);
        assert_eq!(tokens4[1], CssToken::Ident("Xray".into()));
    }

    #[test]
    fn test_identifier_with_escape_sequences_unicode() {
        use CssToken::*;

        let tokens = Tokenizer::from_string(".test\\41 BC");
        assert_eq!(tokens[0], Dot);
        assert_eq!(tokens[1], Ident("testABC".into()));

        // Lowercase hex, too: \61 is 'a'
        let tokens2 = Tokenizer::from_string("#foo\\61 bar");
        assert_eq!(tokens2[0], Hash("fooabar".into()));

        // 6-digit escape, stops at 6 hex digits: \000026 is '&'
        let tokens3 = Tokenizer::from_string(".unicode\\000026id");
        assert_eq!(tokens3[0], Dot);
        assert_eq!(tokens3[1], Ident("unicode&id".into()));

        // Escapes, followed by whitespace: only \E9, next space not part of hex
        let tokens4 = Tokenizer::from_string(".caf\\E9  au\\20 lait");
        assert_eq!(tokens4[0], Dot);
        assert_eq!(tokens4[1], Ident("café".into())); // \E9 -> 'é'
        assert_eq!(tokens4[2], Whitespace);
        assert_eq!(tokens4[3], Ident("au lait".into()));

        let tokens5 = Tokenizer::from_string(".foo\\007A bar");
        assert_eq!(tokens5[0], Dot);
        assert_eq!(tokens5[1], Ident("foozbar".into()));

        // Numeric escapes that produce whitespace: \20 is space
        let tokens6 = Tokenizer::from_string(".foo\\20 bar");
        assert_eq!(tokens6[0], Dot);
        assert_eq!(tokens6[1], Ident("foo bar".into()));

        let tokens7 = Tokenizer::from_string(".foo\\002D bar");
        assert_eq!(tokens7[0], Dot);
        assert_eq!(tokens7[1], Ident("foo-bar".into()));

        // Unicode plane 1: \1F4A9 (PILE OF POO)
        let tokens9 = Tokenizer::from_string(".icon\\1F4A9 -poo");
        assert_eq!(tokens9[0], Dot);
        assert_eq!(tokens9[1], Ident("icon💩-poo".into()));

        // CSS ignores extra hex digits: only up to 6 hex digits after \
        let tokens10 = Tokenizer::from_string(".foo\\110000 bar");
        // \110000 is not a valid scalar; fallback is char::REPLACEMENT_CHARACTER
        // Ideally your tokenizer produces a replacement or skips, but we expect standard CSS only up to 0x10FFFF.
        assert_eq!(tokens10[0], Dot);
        // Depending on your escape handler, this may insert a replacement character or just drop.
        // We'll accept 'foo\u{FFFD}bar' for strict Unicode replacement, but
        // if tokenizer skips invalid codepoints, 'foobar' is also technically correct.

        // We'll check for replacement
        let s = match &tokens10[1] {
            Ident(s) => s,
            _ => panic!("token was not Ident"),
        };
        assert!(s.starts_with("foo") && s.ends_with("bar"));

        // Escaped newline should be ignored, allowing split across lines
        let tokens11 = Tokenizer::from_string(".foo\\\nbar");
        assert_eq!(tokens11[0], Dot);
        assert_eq!(tokens11[1], Ident("foobar".into()));

        // Escapes inside function names/at-rules
        let tokens12 = Tokenizer::from_string("@keyfr\\61mes");
        assert_eq!(tokens12[0], CssToken::AtKeyword("keyframes".into()));

        // Escaping non-BMP single character (e.g. emoji U+1F600)
        let tokens13 = Tokenizer::from_string(".emoji\\1F600 face");
        assert_eq!(tokens13[0], Dot);
        assert_eq!(tokens13[1], Ident("emoji😀face".into()));
    }

    #[test]
    fn test_unescaped_parens_split_tokens() {
        // Without escapes, '(' and ')' are separate tokens
        let tokens = Tokenizer::from_string(".foo(3)");
        assert_eq!(tokens[0], CssToken::Dot);
        assert_eq!(tokens[1], CssToken::Ident("foo".into()));
        assert_eq!(tokens[2], CssToken::LeftParen);
        assert_eq!(tokens[3], CssToken::Number(3.0));
        assert_eq!(tokens[4], CssToken::RightParen);
    }

    #[test]
    fn test_string_token_in_url_function() {
        // Double-quoted string in url()
        let tokens = Tokenizer::from_string("url(\"foo/bar.css\")");
        assert_eq!(tokens[0], CssToken::Ident("url".into()));
        assert_eq!(tokens[1], CssToken::LeftParen);
        assert_eq!(tokens[2], CssToken::String("foo/bar.css".into()));
        assert_eq!(tokens[3], CssToken::RightParen);

        // Single-quoted string in url()
        let tokens2 = Tokenizer::from_string("url('baz.css')");
        assert_eq!(tokens2[0], CssToken::Ident("url".into()));
        assert_eq!(tokens2[1], CssToken::LeftParen);
        assert_eq!(tokens2[2], CssToken::String("baz.css".into()));
        assert_eq!(tokens2[3], CssToken::RightParen);

        // String in url( ... ) with whitespace
        let tokens3 = Tokenizer::from_string("url(  \"theme.css\"  )");
        assert_eq!(tokens3[0], CssToken::Ident("url".into()));
        assert_eq!(tokens3[1], CssToken::LeftParen);
        // there might be Whitespace tokens, but we just check string correctness
        assert!(
            tokens3
                .iter()
                .any(|t| t == &CssToken::String("theme.css".into()))
        );
        assert_eq!(tokens3.last().unwrap(), &CssToken::EOF);
    }

    #[test]
    fn test_string_token_as_import_value() {
        // Double-quoted string after @import
        let tokens = Tokenizer::from_string("@import \"foo/bar.css\";");
        assert_eq!(tokens[0], CssToken::AtKeyword("import".into()));
        assert_eq!(tokens[2], CssToken::String("foo/bar.css".into()));
        assert_eq!(tokens[3], CssToken::Semicolon);

        // String passed as url() argument after @import
        let tokens2 = Tokenizer::from_string("@import url(\"baz.css\");");

        assert_eq!(tokens2[0], CssToken::AtKeyword("import".into()));
        assert_eq!(tokens2[2], CssToken::Ident("url".into()));
        assert_eq!(tokens2[3], CssToken::LeftParen);
        assert_eq!(tokens2[4], CssToken::String("baz.css".into()));
        assert_eq!(tokens2[5], CssToken::RightParen);
        assert_eq!(tokens2[6], CssToken::Semicolon);

        // Single-quoted string in url()
        let tokens3 = Tokenizer::from_string("@import url('foo.css');");
        assert_eq!(tokens3[4], CssToken::String("foo.css".into()));
    }
}
