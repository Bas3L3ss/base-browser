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
            AttrSelectorOperator,
            Combinator::{self},
            ComplexSelector, CompoundSelector, SimpleSelector,
        },
        unit::Unit,
    },
    tokenizer::css_token::CssToken,
};

pub struct Parser<'a> {
    tokens: &'a [CssToken],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [CssToken]) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Rule>> {
        profile_scope!("To CSSOM");
        let mut rules = Vec::new();

        profile_scope!("Parse to CSS Rules");

        self.find_selector_start();

        while self.pos < self.tokens.len() {
            self.find_selector_start();
            let selectors = self.collect_selectors_until_left_brace();

            if matches!(self.current().into(), Some(CssToken::LeftBrace)) {
                self.advance(1);
            } else {
                self.advance(1);
                continue;
            }

            let declarations = self.parse_declarations();
            if matches!(self.current().into(), Some(CssToken::RightBrace)) {
                self.advance(1);
            }

            rules.push(Rule {
                selectors,
                declarations,
            });
        }
        Ok(rules)
    }

    fn find_selector_start(&mut self) {
        while self.pos < self.tokens.len() {
            match self.current() {
                CssToken::Ident(_)
                | CssToken::Star
                | CssToken::Dot
                | CssToken::Hash(_)
                | CssToken::OpenBracket => break,
                _ => self.advance(1),
            }
        }
    }

    /// Advances the parser position by `n` steps.
    /// Ensures that the position does not exceed the length of the tokens.
    pub fn advance(&mut self, n: usize) {
        self.pos = usize::min(self.pos.saturating_add(n), self.tokens.len());
    }

    /// Peeks at the token at an offset from the current parser position.
    /// Returns `Some(&CssToken)` if in bounds, or `None` otherwise.
    pub fn peek(&self, offset: usize) -> Option<&CssToken> {
        self.tokens.get(self.pos.saturating_add(offset))
    }

    pub fn current(&self) -> &CssToken {
        &self.tokens[self.pos]
    }
    #[inline]
    fn push_selector_from_tokens(selectors: &mut Vec<ComplexSelector>, buf: &mut Vec<CssToken>) {
        if !buf.is_empty() {
            if let Some(sel) = Self::selector_from_tokens(buf) {
                selectors.push(sel);
            }
            buf.clear();
        }
    }

    fn collect_selectors_until_left_brace(&mut self) -> Vec<ComplexSelector> {
        let mut selectors = Vec::new();
        let mut cur_selector_tokens = Vec::new();
        let mut parens = 0;

        while self.pos < self.tokens.len() {
            match self.current() {
                CssToken::LeftParen => {
                    cur_selector_tokens.push(self.tokens[self.pos].clone());
                    parens += 1;
                    self.advance(1);
                }
                CssToken::RightParen => {
                    cur_selector_tokens.push(self.tokens[self.pos].clone());
                    if parens > 0 {
                        parens -= 1;
                    }
                    self.advance(1);
                }
                CssToken::LeftBrace => {
                    Self::push_selector_from_tokens(&mut selectors, &mut cur_selector_tokens);
                    break;
                }
                CssToken::Comma if parens == 0 => {
                    Self::push_selector_from_tokens(&mut selectors, &mut cur_selector_tokens);
                    self.advance(1);
                    while matches!(self.current().into(), Some(CssToken::Whitespace)) {
                        self.advance(1);
                    }
                }
                CssToken::EOF => break,
                _ => {
                    cur_selector_tokens.push(self.tokens[self.pos].clone());
                    self.advance(1);
                }
            }
        }

        Self::push_selector_from_tokens(&mut selectors, &mut cur_selector_tokens);

        selectors
    }

    fn collect_selectors_until_end_of_tokens(tokens: &[CssToken]) -> Vec<ComplexSelector> {
        let mut selectors = Vec::new();
        let mut cur_selector_tokens = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i] {
                CssToken::Comma => {
                    Self::push_selector_from_tokens(&mut selectors, &mut cur_selector_tokens);
                    i += 1;
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
        Self::push_selector_from_tokens(&mut selectors, &mut cur_selector_tokens);
        selectors
    }

    fn selector_from_tokens(selector_tokens: &[CssToken]) -> Option<ComplexSelector> {
        if selector_tokens.is_empty() {
            return None;
        }

        // Trim whitespace
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
        let mut rest: Vec<(Combinator, CompoundSelector)> = Vec::new();
        let mut first = CompoundSelector::new();
        let mut idx = 0;

        while idx < tokens.len() {
            match &tokens[idx] {
                CssToken::Dot => {
                    if let Some(CssToken::Ident(name)) = tokens.get(idx + 1) {
                        first.push(SimpleSelector::Class(name.clone()));
                        idx += 2;
                    } else {
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
                CssToken::OpenBracket => {
                    let (name, value, operator, j) =
                        Self::parse_attribute_selector(tokens, idx + 1);
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
                    let (handled, new_idx) = Self::parse_pseudo(tokens, idx, &mut first);
                    idx = new_idx;
                    if handled {
                        continue;
                    }
                    idx += 1;
                }
                CssToken::Whitespace => {
                    idx += 1;
                    while idx < tokens.len() && matches!(tokens[idx], CssToken::Whitespace) {
                        idx += 1;
                    }
                    let (selectors, new_idx) = Self::parse_compound_selectors(tokens, idx);
                    if !selectors.is_empty() {
                        rest.push((Combinator::Descendant, selectors));
                        idx = new_idx;
                    }
                }
                CssToken::Plus => {
                    idx += 1;
                    while idx < tokens.len() && matches!(tokens[idx], CssToken::Whitespace) {
                        idx += 1;
                    }
                    let (selectors, new_idx) = Self::parse_compound_selectors(tokens, idx);
                    rest.push((Combinator::AdjacentSibling, selectors));
                    idx = new_idx;
                }
                CssToken::GreaterThan => {
                    idx += 1;
                    while idx < tokens.len() && matches!(tokens[idx], CssToken::Whitespace) {
                        idx += 1;
                    }
                    let (selectors, new_idx) = Self::parse_compound_selectors(tokens, idx);
                    rest.push((Combinator::Child, selectors));
                    idx = new_idx;
                }
                CssToken::Tilde => {
                    idx += 1;
                    while idx < tokens.len() && matches!(tokens[idx], CssToken::Whitespace) {
                        idx += 1;
                    }
                    let (selectors, new_idx) = Self::parse_compound_selectors(tokens, idx);
                    rest.push((Combinator::GeneralSibling, selectors));
                    idx = new_idx;
                }
                _ => {
                    idx += 1;
                }
            }
        }
        Some(ComplexSelector { first, rest })
    }

    fn parse_compound_selectors(
        selector_tokens: &[CssToken],
        mut idx: usize,
    ) -> (CompoundSelector, usize) {
        let mut selectors = CompoundSelector::new();

        while idx < selector_tokens.len() {
            match &selector_tokens[idx] {
                CssToken::Dot => {
                    if let Some(CssToken::Ident(name)) = selector_tokens.get(idx + 1) {
                        selectors.push(SimpleSelector::Class(name.clone()));
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                CssToken::Hash(name) => {
                    selectors.push(SimpleSelector::Id(name.clone()));
                    idx += 1;
                }
                CssToken::Ident(tag) => {
                    selectors.push(SimpleSelector::Type(tag.clone()));
                    idx += 1;
                }
                CssToken::Star => {
                    selectors.push(SimpleSelector::Universal);
                    idx += 1;
                }
                CssToken::OpenBracket => {
                    let (name, value, operator, j) =
                        Self::parse_attribute_selector(selector_tokens, idx + 1);
                    if let Some(attr_name) = name {
                        selectors.push(SimpleSelector::Attribute {
                            name: attr_name,
                            value,
                            operator,
                        });
                    }
                    idx = j;
                }
                CssToken::Colon => {
                    let (handled, new_idx) =
                        Self::parse_pseudo(selector_tokens, idx, &mut selectors);
                    idx = new_idx;
                    if handled {
                        continue;
                    }
                    idx += 1;
                }
                _ => break,
            }
        }

        (selectors, idx)
    }

    fn parse_attribute_selector(
        tokens: &[CssToken],
        start_idx: usize,
    ) -> (
        Option<String>,
        Option<String>,
        Option<AttrSelectorOperator>,
        usize,
    ) {
        let mut name = None;
        let mut value = None;
        let mut operator = None;
        let mut j = start_idx;

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

        // Operators
        if j < tokens.len() {
            operator = match &tokens[j] {
                CssToken::Equals => {
                    j += 1;
                    Some(AttrSelectorOperator::Equals)
                }
                CssToken::Tilde if matches!(tokens.get(j + 1), Some(CssToken::Equals)) => {
                    j += 2;
                    Some(AttrSelectorOperator::Includes)
                }
                CssToken::Delim('|') if matches!(tokens.get(j + 1), Some(CssToken::Equals)) => {
                    j += 2;
                    Some(AttrSelectorOperator::DashMatch)
                }
                CssToken::Delim('^') if matches!(tokens.get(j + 1), Some(CssToken::Equals)) => {
                    j += 2;
                    Some(AttrSelectorOperator::PrefixMatch)
                }
                CssToken::Delim('$') if matches!(tokens.get(j + 1), Some(CssToken::Equals)) => {
                    j += 2;
                    Some(AttrSelectorOperator::SuffixMatch)
                }
                CssToken::Star if matches!(tokens.get(j + 1), Some(CssToken::Equals)) => {
                    j += 2;
                    Some(AttrSelectorOperator::SubstringMatch)
                }
                _ => None,
            };
        }

        // Collect value if operator is found
        if operator.is_some() {
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
            value = tokens[val_start..val_end].iter().find_map(|tok| match tok {
                CssToken::Ident(v) => Some(v.clone()),
                CssToken::String(v) => Some(v.clone()),
                CssToken::Hash(v) => Some(v.clone()),
                CssToken::Number(n) => Some(n.to_string()),
                CssToken::Dimension { value: v, unit } => Some(format!("{}{}", v, unit)),
                CssToken::Percentage(p) => Some(format!("{}%", p)),
                CssToken::Delim(c) => Some(c.to_string()),
                _ => None,
            });
            j = val_end;
        }
        (name, value, operator, j)
    }

    fn parse_pseudo(
        tokens: &[CssToken],
        idx: usize,
        selectors: &mut CompoundSelector,
    ) -> (bool, usize) {
        let mut double_colon = false;
        let mut j = idx + 1;

        if let Some(CssToken::Colon) = tokens.get(j) {
            double_colon = true;
            j += 1;
        }

        if let Some(CssToken::Ident(pseudo_name)) = tokens.get(j) {
            let mut k = j + 1;
            while k < tokens.len() && matches!(tokens[k], CssToken::Whitespace) {
                k += 1;
            }

            if matches!(tokens.get(k), Some(CssToken::LeftParen)) {
                k += 1;
                let mut nest_paren = 1;
                let start = k;

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

                selectors.push(SimpleSelector::PseudoClass(
                    pseudo_name.to_string(),
                    args_list,
                ));
                return (true, k);
            } else {
                if double_colon {
                    selectors.push(SimpleSelector::PseudoElement(pseudo_name.to_string()));
                } else {
                    selectors.push(SimpleSelector::PseudoClass(pseudo_name.to_string(), None));
                }
                return (true, k);
            }
        }
        (false, j)
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        while self.pos < self.tokens.len() {
            match self.current() {
                CssToken::RightBrace | CssToken::EOF => break,
                CssToken::Ident(property_name) => {
                    let property = Property::from_str(property_name.as_str());
                    self.advance(1);
                    if matches!(self.current().into(), Some(CssToken::Colon)) {
                        self.advance(1);
                    } else {
                        while self.pos < self.tokens.len()
                            && !matches!(
                                self.current().into(),
                                Some(CssToken::Semicolon) | Some(CssToken::RightBrace)
                            )
                        {
                            self.advance(1);
                        }
                        if matches!(self.current().into(), Some(CssToken::Semicolon)) {
                            self.advance(1);
                        }
                        continue;
                    }
                    let value = self.parse_value();
                    declarations.push(Declaration { property, value });
                }
                _ => {
                    self.advance(1);
                }
            }
            if matches!(self.current().into(), Some(CssToken::Semicolon)) {
                self.advance(1);
            }
        }

        declarations
    }

    fn parse_value(&mut self) -> CssValue {
        use CssToken::*;
        let mut components = Vec::new();
        let mut is_important = false;
        while self.pos < self.tokens.len() {
            match self.current() {
                Semicolon | RightBrace | EOF => break,
                Ident(s) => {
                    if s == "important"
                        && !components.is_empty()
                        && matches!(components.last(), Some(ComponentValue::Delim('!')))
                    {
                        is_important = true;
                        components.pop();
                        self.advance(1);
                        continue;
                    }
                    components.push(ComponentValue::Ident(s.clone()));
                    self.advance(1);
                }
                String(s) => {
                    components.push(ComponentValue::String(s.clone()));
                    self.advance(1);
                }
                Hash(s) => {
                    components.push(ComponentValue::Hash(s.clone()));
                    self.advance(1);
                }
                Number(n) => {
                    components.push(ComponentValue::Number(*n));
                    self.advance(1);
                }
                Dimension { value, unit } => {
                    components.push(ComponentValue::Dimension {
                        value: *value,
                        unit: Unit::from_str(unit),
                    });
                    self.advance(1);
                }
                Percentage(p) => {
                    components.push(ComponentValue::Percentage(*p));
                    self.advance(1);
                }
                Delim(ch) => {
                    components.push(ComponentValue::Delim(*ch));
                    self.advance(1);
                }
                Plus => {
                    components.push(ComponentValue::Delim('+'));
                    self.advance(1);
                }
                GreaterThan => {
                    components.push(ComponentValue::Delim('>'));
                    self.advance(1);
                }
                Tilde => {
                    components.push(ComponentValue::Delim('~'));
                    self.advance(1);
                }
                _ => {
                    self.advance(1);
                }
            }
        }

        CssValue {
            components,
            is_important,
        }
    }
}

// ----------------- UNIT TESTS -------------------

#[cfg(test)]
mod tests {
    use core::panic;

    use std::{assert_eq, println, sync::mpsc, thread, time::Duration, vec};

    use super::*;
    use crate::{
        cssom::selector::{
            AttrSelectorOperator, ComplexSelector, CompoundSelector, SimpleSelector,
        },
        tokenizer::tokenizer::Tokenizer,
    };

    fn parse_first_selector(css: &str) -> ComplexSelector {
        let (sender, receiver) = mpsc::channel();
        let css = css.to_string();
        thread::spawn(move || {
            let tokens = Tokenizer::from_string(&css);
            let mut parser = Parser::new(&tokens);
            let selectors = parser.collect_selectors_until_left_brace();
            let result = selectors
                .into_iter()
                .next()
                .expect("Should parse at least one selector");
            sender.send(result).ok();
        });
        match receiver.recv_timeout(Duration::from_secs(3)) {
            std::result::Result::Ok(selector) => selector,
            Err(_) => {
                panic!("Parsing took too long (more than 3 seconds) - possible endless loop.")
            }
        }
    }

    fn parse_first_selector_opt(css: &str) -> Option<ComplexSelector> {
        let (sender, receiver) = mpsc::channel();
        let css = css.to_string();
        thread::spawn(move || {
            let tokens = Tokenizer::from_string(&css);
            let mut parser = Parser::new(&tokens);
            let selectors = parser.collect_selectors_until_left_brace();
            let result = selectors.into_iter().next();
            sender.send(result).ok();
        });
        match receiver.recv_timeout(Duration::from_secs(3)) {
            std::result::Result::Ok(selector) => selector,
            Err(_) => {
                panic!("Parsing took too long (more than 3 seconds) - possible endless loop.")
            }
        }
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

    #[test]
    fn test_descendant_type_selector() {
        // CSS selector input: "div span"
        // Should succeed
        // Expected selector tree:
        // ComplexSelector {
        //     first: CompoundSelector([Type("div")])
        //     rest: [(Descendant, CompoundSelector([Type("span")]))]
        // }
        let selector = parse_first_selector("div span");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".to_string())]
        );
        assert_eq!(
            selector.rest,
            vec![(Combinator::Descendant, {
                let mut comp = CompoundSelector::new();
                comp.push(SimpleSelector::Type("span".to_string()));
                comp
            })]
        );
    }

    #[test]
    fn test_descendant_chain() {
        // CSS selector input: "div article section p"
        // Should succeed
        // Expected selector tree:
        // ComplexSelector {
        //     first: CompoundSelector([Type("div")])
        //     rest: [
        //         (Descendant, CompoundSelector([Type("article")])),
        //         (Descendant, CompoundSelector([Type("section")])),
        //         (Descendant, CompoundSelector([Type("p")]))
        //     ]
        // }
        let selector = parse_first_selector("div article section p");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".to_string())]
        );
        assert_eq!(selector.rest.len(), 3);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("article".to_string())]
        );
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![SimpleSelector::Type("section".to_string())]
        );
        assert_eq!(
            selector.rest[2].1.simple_selectors,
            vec![SimpleSelector::Type("p".to_string())]
        );
    }

    #[test]
    fn test_descendant_with_compound_selectors() {
        // CSS selector input: "div.foo span.bar"
        // Should succeed
        // Expected selector tree:
        // ComplexSelector {
        //     first: CompoundSelector([Type("div"), Class("foo")])
        //     rest: [(Descendant, CompoundSelector([Type("span"), Class("bar")]))]
        // }
        let selector = parse_first_selector("div.foo span.bar");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::Class("foo".to_string())
            ]
        );
        assert_eq!(selector.rest.len(), 1);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![
                SimpleSelector::Type("span".to_string()),
                SimpleSelector::Class("bar".to_string())
            ]
        );
    }

    #[test]
    fn test_descendants_with_attribute_selectors() {
        // CSS selector input: "input[type=text] label[for=username]"
        // Should succeed
        // Expected selector tree:
        // ComplexSelector {
        //   first: CompoundSelector([Type("input"), Attribute{name: "type", operator: Some(Equals), value: Some("text")}])
        //   rest: [(Descendant, CompoundSelector([Type("label"), Attribute{name: "for", operator: Some(Equals), value: Some("username")}]))]
        // }
        let selector = parse_first_selector("input[type=text] label[for=username]");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("input".to_string()),
                SimpleSelector::Attribute {
                    name: "type".to_string(),
                    operator: Some(AttrSelectorOperator::Equals),
                    value: Some("text".to_string())
                }
            ]
        );
        assert_eq!(selector.rest.len(), 1);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![
                SimpleSelector::Type("label".to_string()),
                SimpleSelector::Attribute {
                    name: "for".to_string(),
                    operator: Some(AttrSelectorOperator::Equals),
                    value: Some("username".to_string())
                }
            ]
        );
    }

    #[test]
    fn test_descendants_with_pseudo_classes_and_elements() {
        // CSS selector input: "ul > li a:hover::before"
        // Should succeed (assuming support for Child combinator)
        // Expected selector tree:
        // ComplexSelector {
        //   first: CompoundSelector([Type("ul")])
        //   rest: [
        //     (Child, CompoundSelector([Type("li")])),
        //     (Descendant, CompoundSelector([Type("a"), PseudoClass("hover", None), PseudoElement("before")]))
        //   ]
        // }
        let selector = parse_first_selector("ul > li a:hover::before");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("ul".to_string())]
        );
        assert_eq!(selector.rest.len(), 2);
        // rest[0] is (Child, li)
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("li".to_string())]
        );
        // rest[1] is (Descendant, a:hover::before)
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![
                SimpleSelector::Type("a".to_string()),
                SimpleSelector::PseudoClass("hover".to_string(), None),
                SimpleSelector::PseudoElement("before".to_string())
            ]
        );
    }

    #[test]
    fn test_descendant_with_universal_selector() {
        // CSS selector input: "div *"
        // Should succeed
        // Expected selector tree:
        // ComplexSelector {
        //   first: CompoundSelector([Type("div")])
        //   rest: [(Descendant, CompoundSelector([Universal]))]
        // }
        let selector = parse_first_selector("div *");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".to_string())]
        );
        assert_eq!(selector.rest.len(), 1);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Universal]
        );
    }

    #[test]
    fn test_descendant_with_mixed_whitespace() {
        // CSS selector input: "nav\t\n   ul  li"
        // Should succeed
        // Expected selector tree:
        // ComplexSelector {
        //   first: CompoundSelector([Type("nav")])
        //   rest: [
        //     (Descendant, CompoundSelector([Type("ul")])),
        //     (Descendant, CompoundSelector([Type("li")]))
        //   ]
        // }
        let selector = parse_first_selector("nav\t\n   ul  li");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("nav".to_string())]
        );
        assert_eq!(selector.rest.len(), 2);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("ul".to_string())]
        );
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![SimpleSelector::Type("li".to_string())]
        );
    }

    #[test]
    fn test_descendant_with_leading_trailing_whitespace() {
        // CSS selector input: "  div   span  "
        // Should succeed
        // Expected selector tree:
        // ComplexSelector {
        //   first: CompoundSelector([Type("div")])
        //   rest: [(Descendant, CompoundSelector([Type("span")]))]
        // }

        let selector = parse_first_selector("  div   span  ");
        println!("{selector:#?}");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".to_string())]
        );
        assert_eq!(selector.rest.len(), 1);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("span".to_string())]
        );
    }

    #[test]
    fn test_multiple_nested_whitespace_combinators() {
        // CSS selector input: "div    section\t\n   span  a"
        // Should parse to:
        // ComplexSelector {
        //   first: CompoundSelector([Type("div")])
        //   rest: [
        //     (Descendant, CompoundSelector([Type("section")])),
        //     (Descendant, CompoundSelector([Type("span")])),
        //     (Descendant, CompoundSelector([Type("a")]))
        //   ]
        // }
        let selector: ComplexSelector = parse_first_selector("div    section\t\n   span  a");

        println!("{selector:?}");

        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".to_string())]
        );
        assert_eq!(selector.rest.len(), 3);
        assert_eq!(selector.rest[0].0, Combinator::Descendant);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("section".to_string())]
        );
        assert_eq!(selector.rest[1].0, Combinator::Descendant);
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![SimpleSelector::Type("span".to_string())]
        );
        assert_eq!(selector.rest[2].0, Combinator::Descendant);
        assert_eq!(
            selector.rest[2].1.simple_selectors,
            vec![SimpleSelector::Type("a".to_string())]
        );
    }

    #[test]
    fn test_nested_whitespace_only_between_selectors() {
        // CSS selector input: "main   \n   nav    ul    li"
        // Should parse to a chain of descendant combinators.
        let selector = parse_first_selector("main   \n   nav    ul    li");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("main".to_string())]
        );
        assert_eq!(selector.rest.len(), 3);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("nav".to_string())]
        );
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![SimpleSelector::Type("ul".to_string())]
        );
        assert_eq!(
            selector.rest[2].1.simple_selectors,
            vec![SimpleSelector::Type("li".to_string())]
        );
    }

    #[test]
    fn test_multiple_compound_within_nested_descendant() {
        // CSS selector input: "div.foo   span#bar.bz  a.link"
        // Compound selectors with descendant combinators and multiple simple selectors per compound.
        let selector = parse_first_selector("div.foo   span#bar.bz  a.link");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::Class("foo".to_string()),
            ]
        );
        assert_eq!(selector.rest.len(), 2);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![
                SimpleSelector::Type("span".to_string()),
                SimpleSelector::Id("bar".to_string()),
                SimpleSelector::Class("bz".to_string()),
            ]
        );
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![
                SimpleSelector::Type("a".to_string()),
                SimpleSelector::Class("link".to_string()),
            ]
        );
    }

    #[test]
    fn test_multiple_combinators() {
        // CSS selector input: ".a > .b + .c ~ .d"
        // Should parse combinators Child, AdjacentSibling, GeneralSibling
        let selector = parse_first_selector(".a > .b + .c ~ .d");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Class("a".to_string())]
        );
        assert_eq!(selector.rest.len(), 3);
        assert_eq!(selector.rest[0].0, Combinator::Child);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Class("b".to_string())]
        );
        assert_eq!(selector.rest[1].0, Combinator::AdjacentSibling);
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![SimpleSelector::Class("c".to_string())]
        );
        assert_eq!(selector.rest[2].0, Combinator::GeneralSibling);
        assert_eq!(
            selector.rest[2].1.simple_selectors,
            vec![SimpleSelector::Class("d".to_string())]
        );
    }

    #[test]
    fn test_grouped_selectors_with_varied_combinators() {
        // CSS selector input: "a, b > c, d + e ~ f"
        let tokens = Tokenizer::from_string("a, b > c, d + e ~ f");
        let mut parser = Parser::new(&tokens);
        let selectors = parser.collect_selectors_until_left_brace();

        println!("{selectors:?}");
        assert_eq!(selectors.len(), 3);

        // First: a
        assert_eq!(
            selectors[0].first.simple_selectors,
            vec![SimpleSelector::Type("a".to_string())]
        );
        assert_eq!(selectors[0].rest.len(), 0);

        // Second: b > c
        assert_eq!(
            selectors[1].first.simple_selectors,
            vec![SimpleSelector::Type("b".to_string())]
        );
        assert_eq!(selectors[1].rest.len(), 1);
        assert_eq!(selectors[1].rest[0].0, Combinator::Child);
        assert_eq!(
            selectors[1].rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("c".to_string())]
        );

        // Third: d + e ~ f
        assert_eq!(
            selectors[2].first.simple_selectors,
            vec![SimpleSelector::Type("d".to_string())]
        );
        assert_eq!(selectors[2].rest.len(), 2);
        assert_eq!(selectors[2].rest[0].0, Combinator::AdjacentSibling);
        assert_eq!(
            selectors[2].rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("e".to_string())]
        );
        assert_eq!(selectors[2].rest[1].0, Combinator::GeneralSibling);
        assert_eq!(
            selectors[2].rest[1].1.simple_selectors,
            vec![SimpleSelector::Type("f".to_string())]
        );
    }

    #[test]
    fn test_malformed_descendant_selector_empty() {
        // CSS selector input: "div   "
        // Should succeed (single type selector)
        // Expected selector tree:
        // ComplexSelector { first: CompoundSelector([Type("div")]), rest: [] }
        let selector = parse_first_selector("div   ");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".to_string())]
        );
        assert!(selector.rest.is_empty());
    }

    #[test]
    fn test_malformed_descendant_selector_space_combinator_only() {
        // CSS selector input: " "
        // Should fail (empty/invalid selector)
        assert!(parse_first_selector_opt(" ").is_none());
    }

    // #[test]
    // fn test_malformed_descendant_leading_combinator() {
    //     // CSS selector input: "  > div"
    //     // Should fail (combinator without left-hand selector)
    //     assert!(parse_first_selector_opt("  > div").is_none());
    // }

    // #[test]
    // fn test_malformed_descendant_double_combinator() {
    //     // CSS selector input: "div  >  >  span"
    //     // Should fail or skip incorrectly chained combinators
    //     assert!(parse_first_selector_opt("div  >  >  span").is_none());
    // }
}
