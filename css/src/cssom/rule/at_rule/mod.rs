pub mod import;
pub use import::*;

pub mod layout;
pub use layout::*;

pub mod media_query;
pub use media_query::*;

use crate::{CssToken, Declaration, Keyframe, rule::rule::CssRule};

#[derive(Debug, Clone, PartialEq)]
pub enum AtRuleBlock {
    RuleList(Vec<CssRule>),
    DeclarationList(Vec<Declaration>),
    KeyframesRule(Vec<Keyframe>),
    Raw(Vec<CssToken>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtRuleKind {
    RuleList,
    DeclarationList,
    Unknown,
    KeyFrames,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AtRule {
    pub name: String,
    pub prelude: Vec<CssToken>,
    pub block: Option<AtRuleBlock>,
}

impl AtRule {
    /// Attempt to extract the href/url from an @import rule's prelude tokens.
    /// Returns Some(href) if successful, otherwise None.
    pub fn parse_import(&self) -> Import {
        // Simple parser: Look for url(...) or string in the prelude.
        let mut iter = self.prelude.iter();

        let mut url_contents: Option<String> = None;

        while let Some(token) = iter.next() {
            match token {
                CssToken::Ident(func_name) if func_name.eq_ignore_ascii_case("url") => {
                    if let Some(CssToken::LeftParen) = iter.next() {
                        let mut url_string = String::new();
                        if let Some(first_param_token) = iter.next() {
                            match first_param_token {
                                CssToken::String(s) => {
                                    url_string.push_str(s);
                                    while let Some(param_token) = iter.next() {
                                        if let CssToken::RightParen = param_token {
                                            break;
                                        }
                                    }
                                }
                                _ => {
                                    match first_param_token {
                                        CssToken::Ident(s) => url_string.push_str(s),
                                        _ => {}
                                    }
                                    while let Some(param_token) = iter.next() {
                                        match param_token {
                                            CssToken::RightParen => break,
                                            CssToken::Whitespace => { /* skip */ }
                                            CssToken::Ident(s) => url_string.push_str(s),
                                            CssToken::String(s) => url_string.push_str(s),
                                            _ => {
                                                url_string.push_str(&param_token.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !url_string.is_empty() {
                            url_contents = Some(url_string);
                            break;
                        }
                    }
                }
                CssToken::String(s) => {
                    url_contents = Some(s.clone());
                    break;
                }
                _ => {}
            }
        }

        Import {
            url: url_contents,
            media: None,
            layer: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{assert_eq, println, vec};

    use crate::parser::parser::Parser;
    use crate::rule::at_rule::AtRule;
    use crate::rule::rule::CssRule;
    use crate::tokenizer::css_token::CssToken;
    use crate::tokenizer::tokenizer::Tokenizer;

    /// Utility to parse an input string and return the @import AtRule object.
    fn parse_import_at_rule(input: &str) -> AtRule {
        let tokens = Tokenizer::from_string(input);
        let mut parser = Parser::new(&tokens);
        // Parse returns Vec<CssRule>
        let rules = parser.parse().expect("Should parse successfully");
        match &rules[0] {
            CssRule::At(at_rule) => at_rule.clone(),
            other => panic!("Expected AtRule, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_import_string() {
        let input = r#"@import "foo/bar.css";"#;
        let at_rule = parse_import_at_rule(input);
        let import = at_rule.parse_import();
        assert_eq!(import.url, Some("foo/bar.css".to_string()));
    }

    #[test]
    fn test_parse_import_url_function_string() {
        let input = r#"@import url("baz.css");"#;
        let at_rule = parse_import_at_rule(input);
        let import = at_rule.parse_import();
        assert_eq!(import.url, Some("baz.css".to_string()));
    }

    #[test]
    fn test_parse_import_url_function_ident() {
        let input = r#"@import url(foo.css);"#;
        let at_rule = parse_import_at_rule(input);
        println!("{at_rule:#?}");
        let import = at_rule.parse_import();
        assert_eq!(import.url, Some("foo.css".to_string()));
    }

    #[test]
    fn test_parse_import_url_function_single_quotes() {
        let input = "@import url('foo.css');";
        let at_rule = parse_import_at_rule(input);
        let import = at_rule.parse_import();
        assert_eq!(import.url, Some("foo.css".to_string()));
    }

    #[test]
    fn test_parse_import_with_media() {
        let input = r#"@import url("theme.css") screen and (max-width: 480px);"#;
        let at_rule = parse_import_at_rule(input);
        // Even with extra stuff, should extract url
        let import = at_rule.parse_import();
        assert_eq!(import.url, Some("theme.css".to_string()));
    }

    #[test]
    fn test_parse_import_no_url() {
        let input = r#"@import ;"#;
        let at_rule = parse_import_at_rule(input);
        let import = at_rule.parse_import();
        assert_eq!(import.url, None);
    }

    #[test]
    fn test_parse_import_empty_prelude() {
        let at_rule = AtRule {
            name: "import".to_string(),
            prelude: vec![],
            block: None,
        };
        let import = at_rule.parse_import();
        assert_eq!(import.url, None);
    }

    #[test]
    fn test_parse_import_url_function_with_unexpected_tokens() {
        // This test ensures that parse_import ignores tokens it doesn't handle;
        // here we synthesize the tokens like the parser would emit for @import url("weird.css");
        let at_rule = AtRule {
            name: "import".to_string(),
            prelude: vec![
                CssToken::Ident("url".to_string()),
                CssToken::LeftParen,
                CssToken::String("weird.css".to_string()),
                CssToken::Whitespace,
                CssToken::RightParen,
            ],
            block: None,
        };
        let import = at_rule.parse_import();
        assert_eq!(import.url, Some("weird.css".to_string()));
    }
}
