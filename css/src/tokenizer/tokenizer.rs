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
                    let ident = self.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_');
                    CssToken::Hash(ident)
                }
                '*' => {
                    self.advance();
                    CssToken::Star
                }
                // Either a plus as part of a number, or a combinator
                '+' => {
                    let next = self.peek_next();
                    if next.is_some() && next.unwrap().is_ascii_digit() {
                        self.advance();
                        CssToken::Plus
                    } else {
                        self.advance();
                        CssToken::Plus
                    }
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
                            let ident = self.consume_while(Self::is_ident_continue);
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
                    let keyword = self.consume_while(Self::is_ident_continue);
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
                // Add Delim fallback for any other single-char symbol not matched above (not identifier start etc)
                '"' | '\'' => {
                    // String literal (handle escape sequences!)
                    let quote = self.advance().unwrap();
                    let mut content = String::new();
                    let mut terminated = false;
                    let mut prev_is_escape = false;
                    while let Some(inner) = self.peek() {
                        if prev_is_escape {
                            // Handle common escapes: only simple ones for now
                            match inner {
                                'n' => content.push('\n'),
                                'r' => content.push('\r'),
                                't' => content.push('\t'),
                                '\\' => content.push('\\'),
                                '\'' => content.push('\''),
                                '"' => content.push('"'),
                                _ => content.push(inner),
                            }
                            self.advance();
                            prev_is_escape = false;
                            continue;
                        }
                        if inner == '\\' {
                            prev_is_escape = true;
                            self.advance();
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
                    let ident = self.consume_while(Self::is_ident_continue);
                    CssToken::Ident(ident)
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
}
