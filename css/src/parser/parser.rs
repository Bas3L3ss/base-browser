use std::matches;

use anyhow::{Ok, Result};
use utils::profile_scope;

use crate::{
    cssom::{
        component_value::ComponentValue::{self},
        css_value::CssValue,
        declaration::Declaration,
        property::Property,
        rule::Rule,
        selector::{
            AttrSelectorOperator, Combinator, ComplexSelector, CompoundSelector, SimpleSelector,
        },
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

    /// Collect selectors until a '{' (left brace) is found.
    ///
    /// # Supported
    /// - Simple selectors: element, .class, #id, *, [attr], [attr=val], etc.
    /// - Attribute selector operators: [attr], [attr=val], [attr~=val], [attr|=val], [attr^=val], [attr$=val], [attr*=val].
    /// - Pseudo-classes: Supports :not(), :is(), :where(), :has(), as well as pseudo-classes without arguments (e.g. :hover, :first-child, etc).
    /// - Pseudo-elements: Supports double colon syntax for pseudo-elements (e.g. ::before, ::after), and also single colon for legacy pseudo-elements (e.g. :before).
    ///
    ///
    /// # Not Supported
    /// - Combinators (>, +, ~, descendant whitespace) are not parsed as relationships; each group ends at comma or left brace.
    /// - Complex selector relationships, nesting, or specificity ordering beyond naïve token grouping.
    /// - Arbitrary pseudo-class or pseudo-element arguments unless matched by hardcoded logic for supported pseudo-classes.
    /// - :nth-child, :nth-of-type, and similar with expressions or math as function argument.
    /// - Vendor-prefixed pseudo-elements and custom pseudo-elements.
    /// - Full CSS4 selector features.

    fn collect_selectors_until_left_brace(
        tokens: &[CssToken],
        mut i: usize,
    ) -> (Vec<ComplexSelector>, usize) {
        let mut selectors: Vec<ComplexSelector> = Vec::new();
        let mut cur_selector_tokens: Vec<CssToken> = Vec::new();
        let mut parens: Vec<CssToken> = Vec::new();

        while i < tokens.len() {
            match &tokens[i] {
                CssToken::LeftParen => {
                    cur_selector_tokens.push(tokens[i].clone());
                    parens.push(CssToken::LeftParen);
                    i += 1;
                }
                CssToken::RightParen => {
                    cur_selector_tokens.push(tokens[i].clone());
                    parens.pop();
                    i += 1;
                }
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
                    if parens.len() > 0 {
                        cur_selector_tokens.push(tokens[i].clone());
                        i += 1;
                        continue;
                    }
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

    fn collect_selectors_until_end_of_tokens(tokens: &[CssToken]) -> Vec<ComplexSelector> {
        let mut selectors: Vec<ComplexSelector> = Vec::new();
        let mut cur_selector_tokens: Vec<CssToken> = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i] {
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
                CssToken::EOF => {
                    // End parsing on EOF
                    break;
                }
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
        selectors
    }

    ///
    /// # Supported
    /// - Tag selectors, class selectors, id selectors, universal (*), and attribute selectors as described above.
    /// - Attribute selector operator coverage as above.
    /// - Grouping multiple simple selectors together as a compound selector (e.g. `div.foo#bar[attr]`).
    /// - :not(), :has(), :is(), :where() pseudo-classes, including single and grouped selectors for :is()/:where().
    /// - Pseudo-elements (::before) and pseudo-classes (:hover, :checked, etc) recognized by presence/syntax.
    ///
    /// # Not Supported
    /// - Chained or nested combinators (>, +, ~, descendant).
    /// - Full parsing/argument extraction for arbitrary pseudo-classes with complex expressions.
    /// - Vendor-prefixed, custom, or unknown pseudo-elements or pseudo-classes unless trivially matched.
    /// - Functional or mathematical notation in pseudo-class arguments outside of basic support for those listed above.
    /// - Universal grammar of CSS selectors (parsing is approximate).
    fn selector_from_tokens(selector_tokens: &[CssToken]) -> Option<ComplexSelector> {
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
        let rest: Vec<(Combinator, CompoundSelector)> = Vec::new();
        let mut first = CompoundSelector::new();
        let mut idx = 0;
        let rest_idx: Option<usize> = None;

        while idx < tokens.len() {
            match &tokens[idx] {
                CssToken::Dot => {
                    if let Some(CssToken::Ident(name)) = tokens.get(idx + 1) {
                        first.push(SimpleSelector::Class(name.clone()));
                        idx += 2;
                    } else {
                        // Skip malformed
                        idx += 1;
                    }
                }
                CssToken::Hash(name) => {
                    first.push(SimpleSelector::Id(name.clone()));
                    idx += 1;
                }
                CssToken::Ident(tag) => {
                    first.push(SimpleSelector::Type(tag.clone()));
                    idx += 1;
                }
                CssToken::Star => {
                    first.push(SimpleSelector::Universal);
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
                        // Only consume tokens up to the close bracket as the value
                        let mut val_start = j;
                        let mut val_end = j;
                        while val_end < tokens.len() {
                            match &tokens[val_end] {
                                CssToken::CloseBracket => break,
                                CssToken::Whitespace if val_start == val_end => {
                                    val_start += 1;
                                    val_end += 1;
                                }
                                _ => val_end += 1,
                            }
                        }
                        // Find first non-whitespace token for value
                        value = tokens[val_start..val_end].iter().find_map(|tok| match tok {
                            CssToken::Ident(v) => Some(v.clone()),
                            CssToken::String(v) => Some(v.clone()),
                            CssToken::Hash(v) => Some(v.clone()),
                            CssToken::Number(n) => Some(n.to_string()),
                            CssToken::Dimension { value: v, unit } => {
                                Some(format!("{}{}", v, unit))
                            }
                            CssToken::Percentage(p) => Some(format!("{}%", p)),
                            CssToken::Delim(c) => Some(c.to_string()),
                            _ => None,
                        });
                        j = val_end;
                    }

                    if let Some(attr_name) = name {
                        first.push(SimpleSelector::Attribute {
                            name: attr_name,
                            value,
                            operator,
                        });
                    }
                    idx = j;
                }
                CssToken::Colon => {
                    let mut double_colon = false;
                    let mut j = idx + 1;

                    // Check for double colon (::pseudo-element)
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
                        // Now expect LeftParen for pseudo-classes like :not(), :is(), :where(), :has()
                        if matches!(tokens.get(k), Some(CssToken::LeftParen)) {
                            k += 1;
                            let mut nest_paren = 1;
                            let start = k;

                            // Find matching RightParen to end argument
                            while k < tokens.len() && nest_paren > 0 {
                                match &tokens[k] {
                                    CssToken::LeftParen => {
                                        nest_paren += 1;
                                        k += 1;
                                    }
                                    CssToken::RightParen => {
                                        nest_paren -= 1;
                                        if nest_paren == 0 {
                                            k += 1;
                                            break;
                                        } else {
                                            k += 1;
                                        }
                                    }
                                    _ => k += 1,
                                }
                            }
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

                            let args_list = if arg_tokens.is_empty() {
                                None
                            } else {
                                Some(Self::collect_selectors_until_end_of_tokens(arg_tokens))
                            };

                            first.push(SimpleSelector::PseudoClass(
                                pseudo_name.to_string(),
                                args_list,
                            ));
                        } else {
                            if double_colon {
                                first.push(SimpleSelector::PseudoElement(pseudo_name.to_string()));
                            } else {
                                first.push(SimpleSelector::PseudoClass(
                                    pseudo_name.to_string(),
                                    None,
                                ));
                            }
                        }
                        idx = k;
                        continue;
                    }

                    // Always advance idx to avoid endless loops if pseudo-class is malformed.
                    idx += 1;
                }
                // Unsupported or ignored token for selectors
                CssToken::Whitespace => {
                    idx += 1;
                }
                _ => {
                    idx += 1;
                }
            }
        }

        let complex = ComplexSelector { first, rest };
        Some(complex)
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
    use std::{assert_eq, vec};

    use super::*;
    use crate::cssom::selector::{
        AttrSelectorOperator, ComplexSelector, CompoundSelector, SimpleSelector,
    };

    fn parse_first_selector(css: &str) -> ComplexSelector {
        let tokens = Tokenizer::from_string(css);
        let selectors = Parser::collect_selectors_until_left_brace(&tokens, 0);
        selectors
            .0
            .into_iter()
            .next()
            .expect("Should parse at least one selector")
    }

    #[test]
    fn test_type_selector() {
        let selector = parse_first_selector("div");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".to_string())]
        );
    }

    #[test]
    fn test_class_selector() {
        let selector = parse_first_selector(".foo");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Class("foo".to_string())]
        );
    }

    #[test]
    fn test_id_selector() {
        let selector = parse_first_selector("#main");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Id("main".to_string())]
        );
    }

    #[test]
    fn test_universal_selector() {
        let selector = parse_first_selector("*");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Universal]
        );
    }

    #[test]
    fn test_compound_selector() {
        let selector = parse_first_selector("div.foo#main");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::Class("foo".to_string()),
                SimpleSelector::Id("main".to_string())
            ]
        );
    }

    #[test]
    fn test_attribute_selector_bare() {
        let selector = parse_first_selector("[attr]");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Attribute {
                name: "attr".to_string(),
                operator: None,
                value: None,
            }]
        );
    }

    #[test]
    fn test_attribute_selector_equals() {
        let selector = parse_first_selector("[attr=abc]");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Attribute {
                name: "attr".to_string(),
                operator: Some(AttrSelectorOperator::Equals),
                value: Some("abc".to_string()),
            }]
        );
    }

    #[test]
    fn test_attribute_selector_includes() {
        let selector = parse_first_selector("[attr~=foo]");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Attribute {
                name: "attr".to_string(),
                operator: Some(AttrSelectorOperator::Includes),
                value: Some("foo".to_string()),
            }]
        );
    }

    #[test]
    fn test_attribute_selector_dashmatch() {
        let selector = parse_first_selector("[attr|=val]");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Attribute {
                name: "attr".to_string(),
                operator: Some(AttrSelectorOperator::DashMatch),
                value: Some("val".to_string()),
            }]
        );
    }

    #[test]
    fn test_universal_with_class() {
        let selector = parse_first_selector("*.class");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Universal,
                SimpleSelector::Class("class".to_string())
            ]
        );
    }

    #[test]
    fn test_universal_with_class_and_id() {
        let selector = parse_first_selector("*.hi#div");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Universal,
                SimpleSelector::Class("hi".to_string()),
                SimpleSelector::Id("div".to_string())
            ]
        );
    }

    #[test]
    fn test_complicated_compound_selector() {
        let selector = parse_first_selector("div#main.foo.bar[attr^=5][data-num^=4]:hover::after");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::Id("main".to_string()),
                SimpleSelector::Class("foo".to_string()),
                SimpleSelector::Class("bar".to_string()),
                SimpleSelector::Attribute {
                    name: "attr".to_string(),
                    operator: Some(AttrSelectorOperator::PrefixMatch),
                    value: Some("5".to_string()),
                },
                SimpleSelector::Attribute {
                    name: "data-num".to_string(),
                    operator: Some(AttrSelectorOperator::PrefixMatch),
                    value: Some("4".to_string()),
                },
                SimpleSelector::PseudoClass("hover".to_string(), None),
                SimpleSelector::PseudoElement("after".to_string())
            ]
        );
    }

    #[test]
    fn test_pseudo_class() {
        let selector = parse_first_selector("a:hover");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("a".to_string()),
                SimpleSelector::PseudoClass("hover".to_string(), None)
            ]
        );
    }

    #[test]
    fn test_pseudo_element() {
        let selector = parse_first_selector("div::before");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::PseudoElement("before".to_string())
            ]
        );
    }

    #[test]
    fn test_pseudo_class_with_args() {
        let selector = parse_first_selector(":not(.foo)");
        assert_eq!(
            selector.first.simple_selectors[0],
            SimpleSelector::PseudoClass(
                "not".to_string(),
                Some(vec![{
                    let mut comp = CompoundSelector::new();
                    comp.push(SimpleSelector::Class("foo".to_string()));
                    ComplexSelector {
                        first: comp,
                        rest: Vec::new(),
                    }
                }])
            )
        );
    }

    #[test]
    fn test_pseudo_class_with_compound_arg() {
        let selector = parse_first_selector(":not(.foo.bar#baz)");
        assert_eq!(
            selector.first.simple_selectors[0],
            SimpleSelector::PseudoClass(
                "not".to_string(),
                Some(vec![{
                    let mut comp = CompoundSelector::new();
                    comp.push(SimpleSelector::Class("foo".to_string()));
                    comp.push(SimpleSelector::Class("bar".to_string()));
                    comp.push(SimpleSelector::Id("baz".to_string()));
                    ComplexSelector {
                        first: comp,
                        rest: Vec::new(),
                    }
                }])
            )
        );
    }

    #[test]
    fn test_is_and_where_with_multiple() {
        let selector = parse_first_selector(":is(.a, #b)");
        if let SimpleSelector::PseudoClass(_, Some(groups)) = &selector.first.simple_selectors[0] {
            assert_eq!(groups.len(), 2);
            assert!(
                groups[0]
                    .first
                    .simple_selectors
                    .contains(&SimpleSelector::Class("a".to_string()))
            );
            assert!(
                groups[1]
                    .first
                    .simple_selectors
                    .contains(&SimpleSelector::Id("b".to_string()))
            );
        } else {
            panic!("Expected :is pseudo-class with arguments");
        }

        let selector2 = parse_first_selector(":where(.x, .y)");
        if let SimpleSelector::PseudoClass(_, Some(groups)) = &selector2.first.simple_selectors[0] {
            assert_eq!(groups.len(), 2);
            assert!(
                groups[0]
                    .first
                    .simple_selectors
                    .contains(&SimpleSelector::Class("x".to_string()))
            );
            assert!(
                groups[1]
                    .first
                    .simple_selectors
                    .contains(&SimpleSelector::Class("y".to_string()))
            );
        } else {
            panic!("Expected :where pseudo-class with arguments");
        }
    }
}
