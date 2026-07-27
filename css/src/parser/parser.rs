use std::{matches, print, println};

use anyhow::{Ok, Result};
use utils::profile_scope;

use crate::{
    cssom::{
        component_value::ComponentValue::{self},
        css_value::CssValue,
        declaration::Declaration,
        property::Property,
        rule::Rule,
        selector::{AttrSelectorOperator, Selector},
        unit::Unit,
    },
    tokenizer::{css_token::CssToken, tokenizer::Tokenizer},
};

pub struct Parser;

impl Parser {
    pub fn to_cssom(dom: &str) -> Result<Vec<Rule>> {
        profile_scope!("To CSSOM");
        let tokens = Tokenizer::from_string(dom);
        let mut rules = Vec::new();
        let mut i = 0;

        profile_scope!("Parse to CSS Rules");

        while i < tokens.len() {
            if let Some(selector_start) = Self::find_selector_start(&tokens, i) {
                i = selector_start;
                let (selectors, next_i) = Self::collect_selectors_until_left_brace(&tokens, i);
                i = next_i;

                if matches!(tokens.get(i), Some(CssToken::LeftBrace)) {
                    i += 1;
                } else {
                    i += 1;
                    continue;
                }

                let (declarations, next_i) = Self::parse_declarations(&tokens, i);
                i = next_i;
                if matches!(tokens.get(i), Some(CssToken::RightBrace)) {
                    i += 1;
                }
                rules.push(Rule {
                    selectors,
                    declarations,
                });
            } else {
                break;
            }
        }
        Ok(rules)
    }

    fn find_selector_start(tokens: &[CssToken], mut i: usize) -> Option<usize> {
        while i < tokens.len() {
            match &tokens[i] {
                CssToken::Ident(_)
                | CssToken::Star
                | CssToken::Dot
                | CssToken::Hash(_)
                | CssToken::OpenBracket => {
                    return Some(i);
                }
                CssToken::EOF => return None,
                _ => i += 1,
            }
        }
        None
    }

    // CURRENTLY NOT SUPPORTED:
    // - Combinators (descendant selector ' ', child '>', adjacent sibling '+', general sibling '~')
    // - Complex selector chaining/relations (e.g. grouping selectors using combinators)
    //   Example: div > .foo + #bar[attr=value] ~ span
    // - Error reporting: Only obvious malformed selectors (such as a dot/period without following identifier) are skipped,
    //   but there is otherwise no structured error reporting on malformed or incomplete selectors.
    fn collect_selectors_until_left_brace(
        tokens: &[CssToken],
        mut i: usize,
    ) -> (Vec<Selector>, usize) {
        let mut selectors: Vec<Selector> = Vec::new();
        let mut cur_selector_tokens: Vec<CssToken> = Vec::new();

        while i < tokens.len() {
            match &tokens[i] {
                CssToken::LeftBrace => {
                    // Finish last selector before brace
                    if !cur_selector_tokens.is_empty() {
                        if let Some(sel) = Self::selector_from_tokens(&cur_selector_tokens) {
                            selectors.push(sel);
                        }
                        cur_selector_tokens.clear();
                    }
                    break;
                }
                CssToken::Comma => {
                    // End current selector in group
                    if !cur_selector_tokens.is_empty() {
                        if let Some(sel) = Self::selector_from_tokens(&cur_selector_tokens) {
                            selectors.push(sel);
                        }
                        cur_selector_tokens.clear();
                    }
                    i += 1;
                    // Skip whitespace after comma
                    while matches!(tokens.get(i), Some(CssToken::Whitespace)) {
                        i += 1;
                    }
                }
                CssToken::EOF => break,
                _ => {
                    cur_selector_tokens.push(tokens[i].clone());
                    i += 1;
                }
            }
        }
        // Handle last group if any
        if !cur_selector_tokens.is_empty() {
            if let Some(sel) = Self::selector_from_tokens(&cur_selector_tokens) {
                selectors.push(sel);
            }
        }
        (selectors, i)
    }

    /// Supports: id, class, type/universal, attribute selectors, and simple pseudo selectors :not(), :has(), :is(), :where().
    /// DOES NOT SUPPORT: combinators, pseudo-elements, advanced pseudo-classes, or selector chaining.
    fn selector_from_tokens(selector_tokens: &[CssToken]) -> Option<Selector> {
        if selector_tokens.is_empty() {
            return None;
        }

        // Remove leading/trailing whitespace.
        let mut start = 0;
        let mut end = selector_tokens.len();
        while start < end && matches!(selector_tokens[start], CssToken::Whitespace) {
            start += 1;
        }
        while end > start && matches!(selector_tokens[end - 1], CssToken::Whitespace) {
            end -= 1;
        }
        if start >= end {
            return None;
        }
        let tokens = &selector_tokens[start..end];
        let mut simple_selectors = Vec::new();
        let mut idx = 0;

        while idx < tokens.len() {
            match &tokens[idx] {
                CssToken::Dot => {
                    if let Some(CssToken::Ident(name)) = tokens.get(idx + 1) {
                        simple_selectors.push(Selector::Class(name.clone()));
                        idx += 2;
                    } else {
                        // Skip malformed
                        idx += 1;
                    }
                }
                CssToken::Hash(name) => {
                    simple_selectors.push(Selector::Id(name.clone()));
                    idx += 1;
                }
                CssToken::Ident(tag) => {
                    simple_selectors.push(Selector::Type(tag.clone()));
                    idx += 1;
                }
                CssToken::Star => {
                    simple_selectors.push(Selector::Universal);
                    idx += 1;
                }
                // Attribute selectors: [attr], [attr=value], etc.
                CssToken::OpenBracket => {
                    let mut name = None;
                    let mut value = None;
                    let mut operator = None;
                    let mut j = idx + 1;

                    // Collect attribute name.
                    while j < tokens.len() {
                        match &tokens[j] {
                            CssToken::Ident(attr_name) => {
                                name = Some(attr_name.clone());
                                j += 1;
                            }
                            CssToken::Whitespace => j += 1,
                            _ => break,
                        }
                    }

                    // Check for attribute operators (=, ~=, |=, ^=, $=, *=)
                    if j < tokens.len() {
                        match &tokens[j] {
                            CssToken::Equals => {
                                operator = Some(AttrSelectorOperator::Equals);
                                j += 1;
                            }
                            CssToken::Tilde => {
                                if matches!(tokens.get(j + 1), Some(CssToken::Equals)) {
                                    operator = Some(AttrSelectorOperator::Includes);
                                    j += 2;
                                }
                            }
                            CssToken::Delim('|') => {
                                if matches!(tokens.get(j + 1), Some(CssToken::Equals)) {
                                    operator = Some(AttrSelectorOperator::DashMatch);
                                    j += 2;
                                }
                            }
                            CssToken::Delim('^') => {
                                if matches!(tokens.get(j + 1), Some(CssToken::Equals)) {
                                    operator = Some(AttrSelectorOperator::PrefixMatch);
                                    j += 2;
                                }
                            }
                            CssToken::Delim('$') => {
                                if matches!(tokens.get(j + 1), Some(CssToken::Equals)) {
                                    operator = Some(AttrSelectorOperator::SuffixMatch);
                                    j += 2;
                                }
                            }
                            CssToken::Star => {
                                if matches!(tokens.get(j + 1), Some(CssToken::Equals)) {
                                    operator = Some(AttrSelectorOperator::SubstringMatch);
                                    j += 2;
                                }
                            }

                            _ => {}
                        }
                    }

                    // If there is an operator, collect value.
                    if operator.is_some() {
                        // Skip whitespace to value
                        while j < tokens.len() && matches!(&tokens[j], CssToken::Whitespace) {
                            j += 1;
                        }
                        if let Some(CssToken::Ident(val)) = tokens.get(j) {
                            value = Some(val.clone());
                            j += 1;
                        } else if let Some(CssToken::String(val)) = tokens.get(j) {
                            value = Some(val.clone());
                            j += 1;
                        }
                    }

                    // Advance to close bracket ]
                    while j < tokens.len() {
                        match &tokens[j] {
                            CssToken::CloseBracket => {
                                j += 1;
                                break;
                            }
                            _ => j += 1,
                        }
                    }

                    if let Some(attr_name) = name {
                        simple_selectors.push(Selector::Attribute {
                            name: attr_name,
                            value,
                            operator,
                        });
                    }
                    idx = j;
                }
                // Support for certain pseudo-classes and pseudo-functions.
                CssToken::Colon => {
                    let mut double_colon = false;
                    let mut j = idx + 1;

                    // Check for double colon
                    if let Some(CssToken::Colon) = tokens.get(j) {
                        double_colon = true;
                        j += 1;
                    }

                    if let Some(CssToken::Ident(pseudo_name)) = tokens.get(j) {
                        let mut k = j + 1;
                        // Skip whitespace after pseudo-class name
                        while k < tokens.len() && matches!(&tokens[k], CssToken::Whitespace) {
                            k += 1;
                        }
                        // Now expect OpenParen for all of not/has/is/where

                        if matches!(tokens.get(k), Some(CssToken::LeftParen)) {
                            k += 1;
                            // Collect tokens for argument of pseudo-class
                            let mut nest_paren = 1;
                            let start = k;

                            // Avoid endless loop: Make sure we don't walk past tokens.len()
                            while k < tokens.len() && nest_paren > 0 {
                                match &tokens[k] {
                                    CssToken::LeftParen => {
                                        nest_paren += 1;
                                        k += 1;
                                    }
                                    CssToken::RightParen => {
                                        nest_paren -= 1;
                                        // If nest_paren == 0 here, we should not include this token
                                        if nest_paren == 0 {
                                            k += 1; // move *after* the closing paren
                                            break;
                                        } else {
                                            k += 1;
                                        }
                                    }
                                    _ => {
                                        k += 1;
                                    }
                                }
                            }
                            // NOTE: [start, k-1] is the range inside parens (k may be out-of-bounds if run to end)
                            // Defensive: Do not index past k or tokens.len()
                            let end = if k > tokens.len() {
                                tokens.len()
                            } else {
                                k.saturating_sub(1)
                            };
                            let arg_tokens = if start < end {
                                &tokens[start..end]
                            } else {
                                &[]
                            };

                            match pseudo_name.as_str() {
                                "not" => {
                                    let arg_selector = Self::selector_from_tokens(arg_tokens);
                                    if let Some(sel) = arg_selector {
                                        simple_selectors.push(Selector::Not(Box::new(sel)));
                                    }
                                }
                                "has" => {
                                    let arg_selector = Self::selector_from_tokens(arg_tokens);
                                    if let Some(sel) = arg_selector {
                                        simple_selectors.push(Selector::Has(Box::new(sel)));
                                    }
                                }
                                "is" => {
                                    // Accepts a selector list, split by commas
                                    let mut selector_list = Vec::new();
                                    let mut temp = Vec::new();
                                    for token in arg_tokens {
                                        if let CssToken::Comma = token {
                                            if !temp.is_empty() {
                                                if let Some(sel) = Self::selector_from_tokens(&temp)
                                                {
                                                    selector_list.push(sel);
                                                }
                                                temp.clear();
                                            }
                                        } else {
                                            temp.push(token.clone());
                                        }
                                    }
                                    if !temp.is_empty() {
                                        if let Some(sel) = Self::selector_from_tokens(&temp) {
                                            selector_list.push(sel);
                                        }
                                    }
                                    if !selector_list.is_empty() {
                                        simple_selectors.push(Selector::Is(selector_list));
                                    }
                                }
                                "where" => {
                                    // Accepts a selector list, split by commas
                                    let mut selector_list = Vec::new();
                                    let mut temp = Vec::new();
                                    for token in arg_tokens {
                                        if let CssToken::Comma = token {
                                            if !temp.is_empty() {
                                                if let Some(sel) = Self::selector_from_tokens(&temp)
                                                {
                                                    selector_list.push(sel);
                                                }
                                                temp.clear();
                                            }
                                        } else {
                                            temp.push(token.clone());
                                        }
                                    }
                                    if !temp.is_empty() {
                                        if let Some(sel) = Self::selector_from_tokens(&temp) {
                                            selector_list.push(sel);
                                        }
                                    }
                                    if !selector_list.is_empty() {
                                        simple_selectors.push(Selector::Where(selector_list));
                                    }
                                }
                                _ => {
                                    // For pseudo-classes with a parenthesized argument, but not specifically handled above,
                                    let arg_string = arg_tokens
                                        .iter()
                                        .map(|t| match t {
                                            CssToken::String(s) => s.as_str(),
                                            _ => "",
                                        })
                                        .collect::<String>();
                                    simple_selectors.push(Selector::PseudoClass(
                                        pseudo_name.to_string(),
                                        Some(arg_string.trim().to_string()),
                                    ));
                                }
                            }
                            // skip to after the last processed token (after closing paren if all went well)
                        } else {
                            if double_colon {
                                simple_selectors
                                    .push(Selector::PseudoElement(pseudo_name.to_string()));
                            } else {
                                simple_selectors
                                    .push(Selector::PseudoClass(pseudo_name.to_string(), None));
                                // set idx past this pseudo-class and its argument (if any)
                            }
                        }
                        idx = k;
                        continue;
                    }

                    // Always advance idx to avoid endless loops if pseudo-class is malformed.
                    idx += 1;
                }
                _ => {
                    idx += 1;
                }
            }
        }

        if simple_selectors.is_empty() {
            None
        } else if simple_selectors.len() == 1 {
            Some(simple_selectors.remove(0))
        } else {
            Some(Selector::Compound(simple_selectors))
        }
    }

    fn parse_declarations(tokens: &[CssToken], mut i: usize) -> (Vec<Declaration>, usize) {
        let mut declarations = Vec::new();

        while i < tokens.len() {
            match &tokens[i] {
                CssToken::RightBrace | CssToken::EOF => break,
                CssToken::Ident(property_name) => {
                    let property = Property::from_str(property_name.as_str());
                    i += 1;
                    if matches!(tokens.get(i), Some(CssToken::Colon)) {
                        i += 1;
                    } else {
                        while i < tokens.len()
                            && !matches!(tokens[i], CssToken::Semicolon | CssToken::RightBrace)
                        {
                            i += 1;
                        }
                        if matches!(tokens.get(i), Some(CssToken::Semicolon)) {
                            i += 1;
                        }
                        continue;
                    }
                    let (value, next_i) = Self::parse_value(tokens, i);
                    i = next_i;
                    declarations.push(Declaration { property, value });
                }
                _ => {
                    i += 1;
                }
            }
            if matches!(tokens.get(i), Some(CssToken::Semicolon)) {
                i += 1;
            }
        }

        (declarations, i)
    }

    fn parse_value(tokens: &[CssToken], mut i: usize) -> (CssValue, usize) {
        use CssToken::*;
        let mut components = Vec::new();
        let mut is_important = false;
        while i < tokens.len() {
            match &tokens[i] {
                Semicolon | RightBrace | EOF => break,
                Ident(s) => {
                    if s == "important"
                        && !components.is_empty()
                        && matches!(components.last(), Some(ComponentValue::Delim('!')))
                    {
                        is_important = true;
                        components.pop();
                        i += 1;
                        continue;
                    }
                    components.push(ComponentValue::Ident(s.clone()));
                    i += 1;
                }
                String(s) => {
                    components.push(ComponentValue::String(s.clone()));
                    i += 1;
                }
                Hash(s) => {
                    components.push(ComponentValue::Hash(s.clone()));
                    i += 1;
                }
                Number(n) => {
                    components.push(ComponentValue::Number(*n));
                    i += 1;
                }
                Dimension { value, unit } => {
                    components.push(ComponentValue::Dimension {
                        value: *value,
                        unit: Unit::from_str(unit),
                    });
                    i += 1;
                }
                Percentage(p) => {
                    components.push(ComponentValue::Percentage(*p));
                    i += 1;
                }
                Delim(ch) => {
                    components.push(ComponentValue::Delim(*ch));
                    i += 1;
                }
                Plus => {
                    components.push(ComponentValue::Delim('+'));
                    i += 1;
                }
                GreaterThan => {
                    components.push(ComponentValue::Delim('>'));
                    i += 1;
                }
                Tilde => {
                    components.push(ComponentValue::Delim('~'));
                    i += 1;
                }

                _ => {
                    i += 1;
                }
            }
        }

        (
            CssValue {
                components,
                is_important,
            },
            i,
        )
    }
}

// ----------------- UNIT TESTS -------------------

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{assert_eq, println, vec};

    use super::*;
    use crate::cssom::component_value::ComponentValue;
    use crate::cssom::selector::Selector::{self, *};
    use crate::tokenizer::css_token::CssToken;

    #[test]
    fn selector_from_tokens_simple_cases() {
        // .class
        assert_eq!(
            Parser::selector_from_tokens(&[CssToken::Dot, CssToken::Ident("foo".into())]),
            Some(Class("foo".into()))
        );

        // #id
        assert_eq!(
            Parser::selector_from_tokens(&[CssToken::Hash("foo".into())]),
            Some(Id("foo".into()))
        );

        // div
        assert_eq!(
            Parser::selector_from_tokens(&[CssToken::Ident("div".into())]),
            Some(Type("div".into()))
        );

        // *
        assert_eq!(
            Parser::selector_from_tokens(&[CssToken::Star]),
            Some(Universal)
        );
    }

    #[test]
    fn collect_selectors_until_left_brace_simple_cases() {
        // Group: div, .foo, #id
        let tokens = vec![
            CssToken::Ident("div".into()),
            CssToken::Comma,
            CssToken::Dot,
            CssToken::Ident("foo".into()),
            CssToken::Comma,
            CssToken::Hash("id".into()),
            CssToken::LeftBrace,
        ];
        let (selectors, idx) = Parser::collect_selectors_until_left_brace(&tokens, 0);
        assert_eq!(
            selectors,
            vec![Type("div".into()), Class("foo".into()), Id("id".into()),]
        );
        assert_eq!(idx, tokens.len() - 1);
    }

    #[test]
    fn to_cssom_simple_rule() {
        let css = "span { color: blue; }";
        let rules = Parser::to_cssom(css).unwrap();
        assert_eq!(rules.len(), 1);
        let selectors = &rules[0].selectors;
        assert_eq!(selectors, &[Selector::Type("span".into())]);
        assert_eq!(
            rules[0]
                .declarations
                .iter()
                .any(|d| d.property == Property::Color),
            true
        );
    }

    #[test]
    fn parse_value_and_important_simple() {
        // value + important
        let tokens = vec![
            CssToken::Ident("red".into()),
            CssToken::Delim('!'),
            CssToken::Ident("important".into()),
            CssToken::Semicolon,
        ];
        let (val, idx) = Parser::parse_value(&tokens, 0);
        assert!(val.is_important);
        assert_eq!(idx, 3);

        // simple value
        let tokens = vec![CssToken::Ident("red".into()), CssToken::Semicolon];
        let (val, idx) = Parser::parse_value(&tokens, 0);
        assert!(!val.is_important);
        assert_eq!(val.components, vec![ComponentValue::Ident("red".into())]);
        assert_eq!(idx, 1);
    }

    #[test]
    fn selector_compound_and_multiple_classes() {
        // .foo.bar#id
        let tokens = vec![
            CssToken::Dot,
            CssToken::Ident("foo".into()),
            CssToken::Dot,
            CssToken::Ident("bar".into()),
            CssToken::Hash("id".into()),
        ];
        let selector = Parser::selector_from_tokens(&tokens)
            .expect("Should parse .foo.bar#id into Compound selector");
        match selector {
            Selector::Compound(parts) => {
                assert_eq!(
                    parts,
                    vec![
                        Selector::Class("foo".into()),
                        Selector::Class("bar".into()),
                        Selector::Id("id".into())
                    ]
                );
            }
            _ => panic!("Expected Compound selector"),
        }
    }

    #[test]
    fn selector_type_and_class_compound() {
        // div.hello
        let tokens = vec![
            CssToken::Ident("div".into()),
            CssToken::Dot,
            CssToken::Ident("hello".into()),
        ];
        let selector = Parser::selector_from_tokens(&tokens)
            .expect("Should parse div.hello into Compound selector");
        match selector {
            Selector::Compound(parts) => {
                assert_eq!(
                    parts,
                    vec![
                        Selector::Type("div".into()),
                        Selector::Class("hello".into())
                    ]
                );
            }
            _ => panic!("Expected Compound selector"),
        }
    }

    #[test]
    fn selector_leading_trailing_whitespace() {
        let tokens = vec![
            CssToken::Whitespace,
            CssToken::Dot,
            CssToken::Ident("foo".into()),
            CssToken::Whitespace,
        ];
        let selector =
            Parser::selector_from_tokens(&tokens).expect("Should parse selector with whitespace");
        match selector {
            Selector::Class(s) => assert_eq!(s, "foo"),
            _ => panic!("Expected Class selector"),
        }
    }

    #[test]
    fn selector_only_whitespace() {
        // "     "
        let tokens = vec![CssToken::Whitespace, CssToken::Whitespace];
        let selector = Parser::selector_from_tokens(&tokens);
        assert!(selector.is_none(), "Should return None for all-whitespace");
    }

    #[test]
    fn selector_malformed_dot_without_ident() {
        // "."
        let tokens = vec![CssToken::Dot];
        let selector = Parser::selector_from_tokens(&tokens);
        assert!(
            selector.is_none(),
            "Should return None for dot without ident"
        );
    }
    #[test]
    fn selector_simple_attribute() {
        // [foo]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Ident("foo".into()),
            CssToken::CloseBracket,
        ];
        let selector =
            Parser::selector_from_tokens(&tokens).expect("Should parse simple attribute selector");
        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "foo");
                assert!(value.is_none());
                assert!(operator.is_none());
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_attribute_equals() {
        // [bar=baz]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Ident("bar".into()),
            CssToken::Equals,
            CssToken::Ident("baz".into()),
            CssToken::CloseBracket,
        ];
        let selector =
            Parser::selector_from_tokens(&tokens).expect("Should parse attribute selector with =");
        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "bar");
                assert_eq!(value, Some("baz".into()));
                assert_eq!(
                    operator,
                    Some(crate::cssom::selector::AttrSelectorOperator::Equals)
                );
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_attribute_includes() {
        // [foo~=bar]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Ident("foo".into()),
            CssToken::Tilde,
            CssToken::Equals,
            CssToken::Ident("bar".into()),
            CssToken::CloseBracket,
        ];
        let selector =
            Parser::selector_from_tokens(&tokens).expect("Should parse attribute selector with ~=");
        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "foo");
                assert_eq!(value, Some("bar".into()));
                assert_eq!(
                    operator,
                    Some(crate::cssom::selector::AttrSelectorOperator::Includes)
                );
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_attribute_dashmatch() {
        // [lang|=en]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Ident("lang".into()),
            CssToken::Delim('|'),
            CssToken::Equals,
            CssToken::Ident("en".into()),
            CssToken::CloseBracket,
        ];
        let selector =
            Parser::selector_from_tokens(&tokens).expect("Should parse attribute selector with |=");
        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "lang");
                assert_eq!(value, Some("en".into()));
                assert_eq!(
                    operator,
                    Some(crate::cssom::selector::AttrSelectorOperator::DashMatch)
                );
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_attribute_prefixmatch() {
        // [foo^=bar]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Ident("foo".into()),
            CssToken::Delim('^'),
            CssToken::Equals,
            CssToken::Ident("bar".into()),
            CssToken::CloseBracket,
        ];
        let selector =
            Parser::selector_from_tokens(&tokens).expect("Should parse attribute selector with ^=");
        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "foo");
                assert_eq!(value, Some("bar".into()));
                assert_eq!(
                    operator,
                    Some(crate::cssom::selector::AttrSelectorOperator::PrefixMatch)
                );
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_attribute_suffixmatch() {
        // [foo$=bar]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Ident("foo".into()),
            CssToken::Delim('$'),
            CssToken::Equals,
            CssToken::Ident("bar".into()),
            CssToken::CloseBracket,
        ];
        let selector =
            Parser::selector_from_tokens(&tokens).expect("Should parse attribute selector with $=");
        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "foo");
                assert_eq!(value, Some("bar".into()));
                assert_eq!(
                    operator,
                    Some(crate::cssom::selector::AttrSelectorOperator::SuffixMatch)
                );
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_attribute_substringmatch() {
        // [foo*=bar]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Ident("foo".into()),
            CssToken::Star,
            CssToken::Equals,
            CssToken::Ident("bar".into()),
            CssToken::CloseBracket,
        ];
        let selector =
            Parser::selector_from_tokens(&tokens).expect("Should parse attribute selector with *=");
        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "foo");
                assert_eq!(value, Some("bar".into()));
                assert_eq!(
                    operator,
                    Some(crate::cssom::selector::AttrSelectorOperator::SubstringMatch)
                );
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_attribute_with_string_value() {
        // [foo="bar"]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Ident("foo".into()),
            CssToken::Equals,
            CssToken::String("bar".into()),
            CssToken::CloseBracket,
        ];
        let selector = Parser::selector_from_tokens(&tokens)
            .expect("Should parse attribute selector with string value");
        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "foo");
                assert_eq!(value, Some("bar".into()));
                assert_eq!(
                    operator,
                    Some(crate::cssom::selector::AttrSelectorOperator::Equals)
                );
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_attribute_with_extra_whitespace() {
        // [  foo   =  bar  ]
        let tokens = vec![
            CssToken::OpenBracket,
            CssToken::Whitespace,
            CssToken::Ident("foo".into()),
            CssToken::Whitespace,
            CssToken::Equals,
            CssToken::Whitespace,
            CssToken::Ident("bar".into()),
            CssToken::Whitespace,
            CssToken::CloseBracket,
        ];
        let selector = Parser::selector_from_tokens(&tokens)
            .expect("Should parse attribute selector with whitespace");
        println!("{selector:?}");

        match selector {
            Selector::Attribute {
                name,
                value,
                operator,
            } => {
                assert_eq!(name, "foo");
                assert_eq!(value, Some("bar".into()));
                assert_eq!(operator, Some(AttrSelectorOperator::Equals));
            }
            _ => panic!("Expected Attribute selector"),
        }
    }

    #[test]
    fn selector_malformed_attribute_no_ident() {
        // []
        let tokens = vec![CssToken::OpenBracket, CssToken::CloseBracket];
        let selector = Parser::selector_from_tokens(&tokens);
        assert!(
            selector.is_none(),
            "Should return None for [ ] with no ident"
        );
    }

    #[test]
    fn selector_not_pseudo_class() {
        // :not(.foo)
        let tokens = vec![
            CssToken::Colon,
            CssToken::Ident("not".into()),
            CssToken::LeftParen,
            CssToken::Dot,
            CssToken::Ident("foo".into()),
            CssToken::RightParen,
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        println!("{selector:?}");
        // Should be Some(Not(Class("foo")))
        match selector {
            Some(Selector::Not(inner)) => {
                assert_eq!(*inner, Selector::Class("foo".into()));
            }
            _ => panic!("Expected Selector::Not(Class(\"foo\"))"),
        }
    }

    #[test]
    fn selector_is_pseudo_class() {
        // :is(div, .foo)
        let tokens = vec![
            CssToken::Colon,
            CssToken::Ident("is".into()),
            CssToken::LeftParen,
            CssToken::Ident("div".into()),
            CssToken::Comma,
            CssToken::Dot,
            CssToken::Ident("foo".into()),
            CssToken::RightParen,
        ];

        let selector = Parser::selector_from_tokens(&tokens);

        // Should produce Some(Is([Type("div"), Class("foo")]))
        match selector {
            Some(Selector::Is(ref v)) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], Selector::Type("div".into()));
                assert_eq!(v[1], Selector::Class("foo".into()));
            }
            _ => panic!("Expected Selector::Is([Type(\"div\"), Class(\"foo\")])"),
        }
    }

    #[test]
    fn selector_where_pseudo_class() {
        // :where(#id)
        let tokens = vec![
            CssToken::Colon,
            CssToken::Ident("where".into()),
            CssToken::LeftParen,
            CssToken::Hash("id".into()),
            CssToken::RightParen,
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        println!("{selector:?}");
        // Should produce Some(Where([Id("id")]))
        match selector {
            Some(Selector::Where(ref inner)) => {
                assert_eq!(inner.len(), 1);
                assert_eq!(inner[0], Selector::Id("id".into()));
            }
            _ => panic!("Expected Selector::Where([Id(\"id\")])"),
        }
    }

    #[test]
    fn selector_has_pseudo_class() {
        // :has(.bar)
        let tokens = vec![
            CssToken::Colon,
            CssToken::Ident("has".into()),
            CssToken::LeftParen,
            CssToken::Dot,
            CssToken::Ident("bar".into()),
            CssToken::RightParen,
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        assert!(matches!(selector, Some(Selector::Has(_))));
    }
    #[test]
    fn selector_pseudo_class_simple() {
        // :hover
        let tokens = vec![CssToken::Colon, CssToken::Ident("hover".into())];
        let selector = Parser::selector_from_tokens(&tokens);
        println!("{selector:?}");
        // Should produce Some(PseudoClass("hover", None))
        match selector {
            Some(Selector::PseudoClass(ref name, None)) if name == "hover" => {}
            _ => panic!("Expected Selector::PseudoClass(\"hover\", None)"),
        }
    }

    #[test]
    fn selector_pseudo_element_simple() {
        // ::before
        let tokens = vec![
            CssToken::Colon,
            CssToken::Colon,
            CssToken::Ident("before".into()),
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        println!("{selector:?}");
        // Should produce Some(PseudoElement("before"))
        match selector {
            Some(Selector::PseudoElement(ref name)) if name == "before" => {}
            _ => panic!("Expected Selector::PseudoElement(\"before\")"),
        }
    }

    #[test]
    fn selector_has_with_compound_inside() {
        // :has(.bar#foo[attr=value])
        let tokens = vec![
            CssToken::Colon,
            CssToken::Ident("has".into()),
            CssToken::LeftParen,
            CssToken::Dot,
            CssToken::Ident("bar".into()),
            CssToken::Hash("foo".into()),
            CssToken::OpenBracket,
            CssToken::Ident("attr".into()),
            CssToken::Equals,
            CssToken::Ident("value".into()),
            CssToken::CloseBracket,
            CssToken::RightParen,
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        match selector {
            Some(Selector::Has(inner)) => {
                // inner should be a Compound with three selectors
                match *inner {
                    Selector::Compound(ref v) => {
                        // Should have .bar, #foo, and [attr=value]
                        assert!(
                            v.iter()
                                .any(|s| matches!(s, Selector::Class(cls) if cls == "bar"))
                        );
                        assert!(
                            v.iter()
                                .any(|s| matches!(s, Selector::Id(id) if id == "foo"))
                        );
                        assert!(v.iter().any(|s| {
                            matches!(s, Selector::Attribute { name, value: Some(val), operator: Some(op) }
                                if name == "attr" && val == "value" && *op == AttrSelectorOperator::Equals)
                        }));
                        assert_eq!(v.len(), 3);
                    }
                    _ => panic!("Expected Compound inside :has(...)"),
                }
            }
            _ => panic!("Expected Selector::Has(Compound(..))"),
        }
    }
    #[test]
    fn selector_pseudo_element_single_colon() {
        // :before (should be parsed as pseudo-class in this parser)
        let tokens = vec![CssToken::Colon, CssToken::Ident("before".into())];
        let selector = Parser::selector_from_tokens(&tokens);
        match selector {
            Some(Selector::PseudoClass(name, None)) => {
                assert_eq!(name, "before");
            }
            _ => panic!("Expected PseudoClass selector for :before"),
        }
    }

    #[test]
    fn selector_pseudo_element_double_colon() {
        // ::after
        let tokens = vec![
            CssToken::Colon,
            CssToken::Colon,
            CssToken::Ident("after".into()),
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        match selector {
            Some(Selector::PseudoElement(name)) => {
                assert_eq!(name, "after");
            }
            _ => panic!("Expected PseudoElement selector for ::after"),
        }
    }

    #[test]
    fn selector_compound_with_class_and_pseudo() {
        // .foo:hover
        let tokens = vec![
            CssToken::Dot,
            CssToken::Ident("foo".into()),
            CssToken::Colon,
            CssToken::Ident("hover".into()),
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        println!("{selector:?}");
        match selector {
            Some(Selector::Compound(items)) => {
                assert_eq!(items.len(), 2);
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::Class(name) if name == "foo")),
                    "Should contain the class selector"
                );
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::PseudoClass(name, None) if name == "hover")),
                    "Should contain the pseudo-class selector"
                );
            }
            _ => panic!("Expected Compound selector for .foo:hover"),
        }
    }

    #[test]
    fn selector_type_with_pseudo_element() {
        // div::before
        let tokens = vec![
            CssToken::Ident("div".into()),
            CssToken::Colon,
            CssToken::Colon,
            CssToken::Ident("before".into()),
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        match selector {
            Some(Selector::Compound(items)) => {
                assert_eq!(items.len(), 2);
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::Type(t) if t == "div")),
                    "Should contain the type selector"
                );
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::PseudoElement(pe) if pe == "before")),
                    "Should contain the pseudo element selector"
                );
            }
            _ => panic!("Expected Compound selector for div::before"),
        }
    }
    #[test]
    fn selector_advanced_compound_class_and_pseudo_element() {
        // .btn.primary::after:hover
        let tokens = vec![
            CssToken::Dot,
            CssToken::Ident("btn".into()),
            CssToken::Dot,
            CssToken::Ident("primary".into()),
            CssToken::Colon,
            CssToken::Colon,
            CssToken::Ident("after".into()),
            CssToken::Colon,
            CssToken::Ident("hover".into()),
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        match selector {
            Some(Selector::Compound(items)) => {
                // Should contain 2 classes, 1 pseudo-element, 1 pseudo-class
                assert_eq!(items.len(), 4);

                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::Class(cls) if cls == "btn")),
                    "Should contain the .btn class selector"
                );
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::Class(cls) if cls == "primary")),
                    "Should contain the .primary class selector"
                );
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::PseudoElement(name) if name == "after")),
                    "Should contain the ::after pseudo element"
                );
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::PseudoClass(name, None) if name == "hover")),
                    "Should contain the :hover pseudo-class"
                );
            }
            _ => panic!("Expected Compound selector for .btn.primary::after:hover"),
        }
    }

    #[test]
    fn selector_compound_type_class_and_pseudo_element_and_pseudo_class() {
        // button.large::before:active
        let tokens = vec![
            CssToken::Ident("button".into()),
            CssToken::Dot,
            CssToken::Ident("large".into()),
            CssToken::Colon,
            CssToken::Colon,
            CssToken::Ident("before".into()),
            CssToken::Colon,
            CssToken::Ident("active".into()),
        ];
        let selector = Parser::selector_from_tokens(&tokens);
        match selector {
            Some(Selector::Compound(items)) => {
                // Type, class, pseudo-element, pseudo-class
                assert_eq!(items.len(), 4);

                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::Type(t) if t == "button")),
                    "Should contain the type selector 'button'"
                );
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::Class(cls) if cls == "large")),
                    "Should contain the .large class selector"
                );
                assert!(
                    items
                        .iter()
                        .any(|s| matches!(s, Selector::PseudoElement(name) if name == "before")),
                    "Should contain the ::before pseudo element"
                );
                assert!(
                    items.iter().any(
                        |s| matches!(s, Selector::PseudoClass(name, None) if name == "active")
                    ),
                    "Should contain the :active pseudo-class"
                );
            }
            _ => panic!("Expected Compound selector for button.large::before:active"),
        }
    }
}
