use std::{matches, unimplemented};

use anyhow::{Ok, Result};
use utils::profile_scope;

use crate::{
    cssom::{
        component_value::ComponentValue::{self},
        css_value::CssValue,
        declaration::Declaration,
        function::Function,
        property::Property,
        rule::{AtRule, AtRuleBlock, AtRuleKind, CssRule, StyleRule},
        selector::ComplexSelector,
        unit::Unit,
    },
    parser::selector_parser::SelectorParser,
    tokenizer::css_token::CssToken,
};

pub struct Parser<'a> {
    pub(crate) tokens: &'a [CssToken],
    pub(crate) pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [CssToken]) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<CssRule>> {
        profile_scope!("To CSSOM");
        let mut rules = Vec::new();

        profile_scope!("Parse to CSS Rules");

        while !self.is_eof() {
            if let Some(rule) = self.parse_css_rule() {
                rules.push(rule);
            } else {
                self.advance(1);
            }
        }

        Ok(rules)
    }
    fn parse_css_rule(&mut self) -> Option<CssRule> {
        self.skip_whitespace();

        match self.current() {
            CssToken::AtKeyword(_) => self.parse_at_rule().map(CssRule::At),
            _ => self.parse_style_rule().map(CssRule::Style),
        }
    }

    fn parse_at_rule(&mut self) -> Option<AtRule> {
        if let CssToken::AtKeyword(name) = self.current() {
            let name = name.clone();
            self.advance(1);

            let prelude: Vec<CssToken> = self.parse_prelude();

            let block: Option<AtRuleBlock> = if self.expect(CssToken::LeftBrace).is_some() {
                Some(self.parse_block(&name))
            } else {
                None
            };

            self.expect(CssToken::RightBrace);

            Some(AtRule {
                name,
                prelude,
                block,
            })
        } else {
            return None;
        }
    }

    fn parse_block(&mut self, name: &str) -> AtRuleBlock {
        match Self::classify(name) {
            AtRuleKind::RuleList => {
                let mut rules = Vec::new();
                while let Some(rule) = self.parse_css_rule() {
                    rules.push(rule);
                }

                AtRuleBlock::RuleList(rules)
            }

            AtRuleKind::DeclarationList => AtRuleBlock::DeclarationList(self.parse_declarations()),

            AtRuleKind::SpecialGrammar => {
                unimplemented!()
            }
        }
    }

    fn classify(name: &str) -> AtRuleKind {
        match name {
            // { style rules / nested at-rules }
            "media" | "supports" | "layer" | "scope" => AtRuleKind::RuleList,

            // { declarations }
            "font-face" | "page" | "property" => AtRuleKind::DeclarationList,

            // unknown/future CSS
            _ => AtRuleKind::SpecialGrammar,
        }
    }

    fn parse_prelude(&mut self) -> Vec<CssToken> {
        let mut prelude = Vec::new();

        self.skip_whitespace();
        while !matches!(
            self.current(),
            CssToken::LeftBrace | CssToken::Semicolon | CssToken::EOF
        ) {
            prelude.push(self.current().clone());
            self.advance(1);
            self.skip_whitespace();
        }

        prelude
    }

    fn parse_style_rule(&mut self) -> Option<StyleRule> {
        let selectors = self.parse_selector_list();

        if self.expect(CssToken::LeftBrace).is_none() {
            return None;
        }

        let declarations = self.parse_declarations();

        self.expect(CssToken::RightBrace);

        Some(StyleRule {
            selectors,
            declarations,
        })
    }

    fn parse_selector_list(&mut self) -> Vec<ComplexSelector> {
        let mut selectors = Vec::new();

        loop {
            self.skip_whitespace();

            let mut selector_parser = SelectorParser::new(self);

            if let Some(selector) = selector_parser.parse_complex_selector() {
                // Only push if at least first or rest are not empty
                if !selector.first.is_empty() || !selector.rest.is_empty() {
                    selectors.push(selector);
                }
            }

            self.skip_whitespace();

            if self.expect(CssToken::Comma).is_some() {
                continue;
            } else if matches!(self.current(), CssToken::LeftBrace | CssToken::EOF) {
                break;
            } else {
                // syntax error later
                break;
            }
        }

        selectors
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        while !self.is_eof() {
            match self.current() {
                CssToken::RightBrace | CssToken::EOF => break,
                CssToken::Ident(property_name) => {
                    let property = Property::from_str(property_name.as_str());
                    self.advance(1);
                    if matches!(self.current().into(), Some(CssToken::Colon)) {
                        self.advance(1);
                    } else {
                        while !self.is_eof()
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
                    let value = self.parse_css_value();
                    declarations.push(Declaration { property, value });
                }
                _ => {
                    self.advance(1);
                }
            }
            self.expect(CssToken::Semicolon);
        }

        declarations
    }

    fn parse_component_value(&mut self) -> Option<ComponentValue> {
        match self.current() {
            CssToken::Ident(s) => {
                // Could be the start of a function, e.g., rgb(, url(
                if let Some(CssToken::LeftParen) = self.peek(1) {
                    // Attempt to parse a function
                    let name = s.clone();
                    self.advance(2); // skip ident and '('
                    let mut args = Vec::new();
                    let mut paren_depth = 1;
                    while !self.is_eof() && paren_depth > 0 {
                        match self.current() {
                            CssToken::LeftParen => {
                                paren_depth += 1;
                                args.push(ComponentValue::Delim('('));
                                self.advance(1);
                            }
                            CssToken::RightParen => {
                                paren_depth -= 1;
                                if paren_depth == 0 {
                                    self.advance(1); // skip ')'
                                    break;
                                } else {
                                    args.push(ComponentValue::Delim(')'));
                                    self.advance(1);
                                }
                            }
                            CssToken::Comma => {
                                args.push(ComponentValue::Delim(','));
                                self.advance(1);
                            }
                            CssToken::Whitespace => {
                                self.advance(1);
                            }
                            _ => {
                                if let Some(component) = self.parse_component_value() {
                                    args.push(component);
                                } else {
                                    self.advance(1);
                                }
                            }
                        }
                    }
                    Some(ComponentValue::Function(Function::new(name, args)))
                } else {
                    let value = ComponentValue::Ident(s.clone());
                    self.advance(1);
                    Some(value)
                }
            }
            CssToken::String(s) => {
                let value = ComponentValue::String(s.clone());
                self.advance(1);
                Some(value)
            }
            CssToken::Hash(s) => {
                let value = ComponentValue::Hash(s.clone());
                self.advance(1);
                Some(value)
            }
            CssToken::Number(n) => {
                let value = ComponentValue::Number(*n);
                self.advance(1);
                Some(value)
            }
            CssToken::Dimension { value, unit } => {
                let value = ComponentValue::Dimension {
                    value: *value,
                    unit: Unit::from_str(unit),
                };
                self.advance(1);
                Some(value)
            }
            CssToken::Percentage(p) => {
                let value = ComponentValue::Percentage(*p);
                self.advance(1);
                Some(value)
            }
            CssToken::Delim(ch) => {
                let value = ComponentValue::Delim(*ch);
                self.advance(1);
                Some(value)
            }
            CssToken::Plus => {
                self.advance(1);
                Some(ComponentValue::Delim('+'))
            }
            CssToken::GreaterThan => {
                self.advance(1);
                Some(ComponentValue::Delim('>'))
            }
            CssToken::Tilde => {
                self.advance(1);
                Some(ComponentValue::Delim('~'))
            }
            // If not known, try to put them in Delim
            _ => None,
        }
    }

    fn parse_css_value(&mut self) -> CssValue {
        let mut components = Vec::new();
        let mut is_important = false;
        while !self.is_eof() {
            match self.current() {
                CssToken::Semicolon | CssToken::RightBrace | CssToken::EOF => break,
                CssToken::Ident(s) => {
                    if s == "important"
                        && !components.is_empty()
                        && matches!(components.last(), Some(ComponentValue::Delim('!')))
                    {
                        is_important = true;
                        components.pop();
                        self.advance(1);
                        continue;
                    }
                    if let Some(component) = self.parse_component_value() {
                        components.push(component);
                    }
                }
                CssToken::String(_)
                | CssToken::Hash(_)
                | CssToken::Number(_)
                | CssToken::Dimension { .. }
                | CssToken::Percentage(_)
                | CssToken::Delim(_)
                | CssToken::Plus
                | CssToken::GreaterThan
                | CssToken::Tilde => {
                    if let Some(component) = self.parse_component_value() {
                        components.push(component);
                    }
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

    pub(crate) fn advance(&mut self, n: usize) {
        self.pos = usize::min(self.pos.saturating_add(n), self.tokens.len());
    }

    pub(crate) fn current(&self) -> &CssToken {
        &self.tokens[self.pos]
    }

    pub(crate) fn skip_whitespace(&mut self) -> bool {
        let mut saw_ws = false;
        while !self.is_eof() && matches!(self.current(), CssToken::Whitespace) {
            saw_ws = true;
            self.advance(1);
        }
        saw_ws
    }

    pub(crate) fn peek(&self, n: usize) -> Option<&CssToken> {
        self.tokens.get(self.pos + n)
    }

    pub(crate) fn expect(&mut self, expected: CssToken) -> Option<()> {
        if std::mem::discriminant(self.current()) == std::mem::discriminant(&expected) {
            self.advance(1);
            Some(())
        } else {
            None
        }
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.tokens.get(self.pos), Some(CssToken::EOF))
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
            AttrSelectorOperator, Combinator, ComplexSelector, CompoundSelector, SimpleSelector,
        },
        tokenizer::tokenizer::Tokenizer,
    };

    fn parse_first_selector(css: &str) -> ComplexSelector {
        let (sender, receiver) = mpsc::channel();
        let css = css.to_string();
        thread::spawn(move || {
            let tokens = Tokenizer::from_string(&css);
            let mut parser = Parser::new(&tokens);
            let selectors = parser.parse_selector_list();
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
            let selectors = parser.parse_selector_list();
            let result = selectors.into_iter().next();
            sender.send(result).ok();
        });
        match receiver.recv_timeout(Duration::from_secs(3)) {
            std::result::Result::Ok(selector) => {
                println!("{:?}", selector);
                selector
            }

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
        println!("{selector:#?}");
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
        let selectors = parser.parse_selector_list();

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

    #[test]
    fn test_is_pseudo_class_with_child_combinator() {
        // CSS selector input: ":is(div > p)"
        // Should parse as one selector: a pseudo-class :is() with argument [div > p]
        let selector = parse_first_selector(":is(div > p)");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::PseudoClass(
                "is".to_string(),
                Some(vec![ComplexSelector {
                    first: {
                        let mut comp = CompoundSelector::new();
                        comp.push(SimpleSelector::Type("div".to_string()));
                        comp
                    },
                    rest: vec![(Combinator::Child, {
                        let mut comp = CompoundSelector::new();
                        comp.push(SimpleSelector::Type("p".to_string()));
                        comp
                    })]
                }])
            )]
        );
    }

    #[test]
    fn test_is_pseudo_class_with_comma_and_combinator() {
        // CSS selector input: ":is(div, span > a)"
        // Should parse :is with arg [div], [span > a]
        let selector = parse_first_selector(":is(div, span > a)");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::PseudoClass(
                "is".to_string(),
                Some(vec![
                    ComplexSelector {
                        first: {
                            let mut comp = CompoundSelector::new();
                            comp.push(SimpleSelector::Type("div".to_string()));
                            comp
                        },
                        rest: vec![]
                    },
                    ComplexSelector {
                        first: {
                            let mut comp = CompoundSelector::new();
                            comp.push(SimpleSelector::Type("span".to_string()));
                            comp
                        },
                        rest: vec![(Combinator::Child, {
                            let mut comp = CompoundSelector::new();
                            comp.push(SimpleSelector::Type("a".to_string()));
                            comp
                        })]
                    }
                ])
            )]
        );
    }

    #[test]
    fn test_compound_combinators_chain() {
        // CSS selector input: "div > p + span ~ a"
        // Should parse as:
        // ComplexSelector {
        //   first: [Type("div")],
        //   rest: [
        //     (Child, [Type("p")]),
        //     (AdjacentSibling, [Type("span")]),
        //     (GeneralSibling, [Type("a")])
        //   ]
        // }
        let selector = parse_first_selector("div > p + span ~ a");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".to_string())]
        );
        assert_eq!(selector.rest.len(), 3);
        assert_eq!(selector.rest[0].0, Combinator::Child);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("p".to_string())]
        );
        assert_eq!(selector.rest[1].0, Combinator::AdjacentSibling);
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![SimpleSelector::Type("span".to_string())]
        );
        assert_eq!(selector.rest[2].0, Combinator::GeneralSibling);
        assert_eq!(
            selector.rest[2].1.simple_selectors,
            vec![SimpleSelector::Type("a".to_string())]
        );
    }

    #[test]
    fn test_nested_functional_pseudos() {
        let selector = parse_first_selector("div:is(:not(.a), .b)");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::PseudoClass(
                    "is".to_string(),
                    Some(vec![
                        {
                            let mut s = ComplexSelector::default();
                            s.first.simple_selectors.push(SimpleSelector::PseudoClass(
                                "not".to_string(),
                                Some(vec![{
                                    let mut s2 = ComplexSelector::default();
                                    s2.first
                                        .simple_selectors
                                        .push(SimpleSelector::Class("a".to_string()));
                                    s2
                                }]),
                            ));
                            s
                        },
                        {
                            let mut s = ComplexSelector::default();
                            s.first
                                .simple_selectors
                                .push(SimpleSelector::Class("b".to_string()));
                            s
                        }
                    ])
                )
            ]
        );

        let selector = parse_first_selector("div:has(> img)");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::PseudoClass(
                    "has".to_string(),
                    Some(vec![{
                        let mut cs = ComplexSelector::default();
                        cs.rest.push((Combinator::Child, {
                            let mut comp = CompoundSelector::new();
                            comp.push(SimpleSelector::Type("img".to_string()));
                            comp
                        }));
                        cs
                    }])
                )
            ]
        );

        let selector = parse_first_selector("div:not(:is(.a, .b))");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::PseudoClass(
                    "not".to_string(),
                    Some(vec![{
                        let mut cs = ComplexSelector::default();
                        cs.first.simple_selectors.push(SimpleSelector::PseudoClass(
                            "is".to_string(),
                            Some(vec![
                                {
                                    let mut inner = ComplexSelector::default();
                                    inner
                                        .first
                                        .simple_selectors
                                        .push(SimpleSelector::Class("a".to_string()));
                                    inner
                                },
                                {
                                    let mut inner = ComplexSelector::default();
                                    inner
                                        .first
                                        .simple_selectors
                                        .push(SimpleSelector::Class("b".to_string()));
                                    inner
                                },
                            ]),
                        ));
                        cs
                    }])
                )
            ]
        );
    }

    #[test]
    fn test_empty_functional_arguments() {
        let selector = parse_first_selector("div:is()");
        println!("{selector:#?}");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::PseudoClass("is".to_string(), Some(vec![])),
            ]
        );

        let selector = parse_first_selector("div:not()");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::PseudoClass("not".to_string(), Some(vec![])),
            ]
        );
    }

    #[test]
    fn test_deeply_nested_parens() {
        let selector = parse_first_selector("div:is(:not(:has(img)))");
        assert_eq!(
            selector.first.simple_selectors[0],
            SimpleSelector::Type("div".to_string())
        );
        match &selector.first.simple_selectors[1] {
            SimpleSelector::PseudoClass(name, Some(args)) if name == "is" => {
                assert_eq!(args.len(), 1);
                match &args[0].first.simple_selectors[0] {
                    SimpleSelector::PseudoClass(inner_name, Some(inner_args))
                        if inner_name == "not" =>
                    {
                        assert_eq!(inner_args.len(), 1);
                        match &inner_args[0].first.simple_selectors[0] {
                            SimpleSelector::PseudoClass(n, Some(has_args)) if n == "has" => {
                                assert_eq!(has_args.len(), 1);
                                assert_eq!(
                                    has_args[0].first.simple_selectors[0],
                                    SimpleSelector::Type("img".to_string())
                                );
                            }
                            _ => panic!("Expected nested :has"),
                        }
                    }
                    _ => panic!("Expected nested :not"),
                }
            }
            _ => panic!("Expected outer :is"),
        }
    }

    #[test]
    fn test_attribute_selectors_with_spaces() {
        let selector = parse_first_selector(r#"div[data-name = "hello"]"#);
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".into()),
                SimpleSelector::Attribute {
                    name: "data-name".into(),
                    value: Some("hello".into()),
                    operator: Some(AttrSelectorOperator::Equals)
                }
            ]
        );
        let selector = parse_first_selector(r#"div[ title ^= "abc" ]"#);
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".into()),
                SimpleSelector::Attribute {
                    name: "title".into(),
                    value: Some("abc".into()),
                    operator: Some(AttrSelectorOperator::PrefixMatch)
                }
            ]
        );
        let selector = parse_first_selector(r#"div[lang |= en]"#);
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".into()),
                SimpleSelector::Attribute {
                    name: "lang".into(),
                    value: Some("en".into()),
                    operator: Some(AttrSelectorOperator::DashMatch)
                }
            ]
        );
    }

    #[test]
    fn test_multiple_pseudo_classes() {
        let selector = parse_first_selector("div:hover:focus:active");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".to_string()),
                SimpleSelector::PseudoClass("hover".into(), None),
                SimpleSelector::PseudoClass("focus".into(), None),
                SimpleSelector::PseudoClass("active".into(), None),
            ]
        );
    }

    #[test]
    fn test_pseudo_element_after_pseudo_class() {
        let selector = parse_first_selector("div:hover::before");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Type("div".into()),
                SimpleSelector::PseudoClass("hover".into(), None),
                SimpleSelector::PseudoElement("before".into()),
            ]
        );
    }

    #[test]
    fn test_universal_mixed() {
        let selector = parse_first_selector("*.foo");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Universal,
                SimpleSelector::Class("foo".into()),
            ]
        );
        let selector = parse_first_selector("*#id");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Universal, SimpleSelector::Id("id".into()),]
        );
        let selector = parse_first_selector("*[hidden]");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Universal,
                SimpleSelector::Attribute {
                    name: "hidden".into(),
                    value: None,
                    operator: None
                }
            ]
        );
    }

    #[test]
    fn test_compound_selector_without_type() {
        let selector = parse_first_selector(".foo#bar[attr]:hover");
        assert_eq!(
            selector.first.simple_selectors,
            vec![
                SimpleSelector::Class("foo".into()),
                SimpleSelector::Id("bar".into()),
                SimpleSelector::Attribute {
                    name: "attr".into(),
                    value: None,
                    operator: None
                },
                SimpleSelector::PseudoClass("hover".into(), None),
            ]
        );
    }

    #[test]
    fn test_long_combinator_chain() {
        let selector = parse_first_selector("div > ul li + a ~ span");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".into())]
        );
        assert_eq!(selector.rest.len(), 4);
        assert_eq!(selector.rest[0].0, Combinator::Child);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("ul".into())]
        );
        assert_eq!(selector.rest[1].0, Combinator::Descendant);
        assert_eq!(
            selector.rest[1].1.simple_selectors,
            vec![SimpleSelector::Type("li".into())]
        );
        assert_eq!(selector.rest[2].0, Combinator::AdjacentSibling);
        assert_eq!(
            selector.rest[2].1.simple_selectors,
            vec![SimpleSelector::Type("a".into())]
        );
        assert_eq!(selector.rest[3].0, Combinator::GeneralSibling);
        assert_eq!(
            selector.rest[3].1.simple_selectors,
            vec![SimpleSelector::Type("span".into())]
        );
    }

    #[test]
    fn test_multiple_selector_list() {
        let selectors = {
            let (sender, receiver) = mpsc::channel();
            let css = "div, .a, #id, span:hover".to_string();
            std::thread::spawn(move || {
                let tokens = Tokenizer::from_string(&css);
                let mut parser = Parser::new(&tokens);
                let result = parser.parse_selector_list();
                sender.send(result).ok();
            });
            receiver
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap()
        };
        assert_eq!(selectors.len(), 4);

        assert_eq!(
            selectors[0].first.simple_selectors,
            vec![SimpleSelector::Type("div".into())]
        );
        assert_eq!(
            selectors[1].first.simple_selectors,
            vec![SimpleSelector::Class("a".into())]
        );
        assert_eq!(
            selectors[2].first.simple_selectors,
            vec![SimpleSelector::Id("id".into())]
        );
        assert_eq!(
            selectors[3].first.simple_selectors,
            vec![
                SimpleSelector::Type("span".into()),
                SimpleSelector::PseudoClass("hover".into(), None)
            ]
        );
    }

    #[test]
    fn test_whitespace_variations() {
        let selector = parse_first_selector("div    >    span");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".into())]
        );
        assert_eq!(selector.rest.len(), 1);
        assert_eq!(selector.rest[0].0, Combinator::Child);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("span".into())]
        );

        let selector = parse_first_selector("div");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".into())]
        );

        let selector = parse_first_selector("span");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("span".into())]
        );

        let selector = parse_first_selector("div     +      p");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".into())]
        );
        assert_eq!(selector.rest.len(), 1);
        assert_eq!(selector.rest[0].0, Combinator::AdjacentSibling);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("p".into())]
        );

        let selector = parse_first_selector("div~span");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::Type("div".into())]
        );
        assert_eq!(selector.rest.len(), 1);
        assert_eq!(selector.rest[0].0, Combinator::GeneralSibling);
        assert_eq!(
            selector.rest[0].1.simple_selectors,
            vec![SimpleSelector::Type("span".into())]
        );
    }

    #[test]
    fn test_functional_pseudo_with_selector_list() {
        let selector = parse_first_selector(":is(div, span.foo, #id)");
        assert_eq!(selector.first.simple_selectors.len(), 1);
        match &selector.first.simple_selectors[0] {
            SimpleSelector::PseudoClass(name, Some(args)) if name == "is" => {
                assert_eq!(args.len(), 3);
                assert_eq!(
                    args[0].first.simple_selectors,
                    vec![SimpleSelector::Type("div".into())]
                );
                assert_eq!(
                    args[1].first.simple_selectors,
                    vec![
                        SimpleSelector::Type("span".into()),
                        SimpleSelector::Class("foo".into())
                    ]
                );
                assert_eq!(
                    args[2].first.simple_selectors,
                    vec![SimpleSelector::Id("id".into())]
                );
            }
            _ => panic!("Expected :is with selector list"),
        }
    }

    #[test]
    fn test_functional_pseudo_inside_functional() {
        let selector = parse_first_selector(":not(:is(.a, .b))");
        assert_eq!(selector.first.simple_selectors.len(), 1);
        match &selector.first.simple_selectors[0] {
            SimpleSelector::PseudoClass(name, Some(args)) if name == "not" => {
                assert_eq!(args.len(), 1);
                match &args[0].first.simple_selectors[0] {
                    SimpleSelector::PseudoClass(name, Some(is_args)) if name == "is" => {
                        assert_eq!(is_args.len(), 2);
                        assert_eq!(
                            is_args[0].first.simple_selectors,
                            vec![SimpleSelector::Class("a".into())]
                        );
                        assert_eq!(
                            is_args[1].first.simple_selectors,
                            vec![SimpleSelector::Class("b".into())]
                        );
                    }
                    _ => panic!("Expected :is inside :not"),
                }
            }
            _ => panic!("Expected :not at root"),
        };

        let selector = parse_first_selector(":has(:not(.foo))");
        assert_eq!(selector.first.simple_selectors.len(), 1);
        match &selector.first.simple_selectors[0] {
            SimpleSelector::PseudoClass(name, Some(args)) if name == "has" => {
                assert_eq!(args.len(), 1);
                match &args[0].first.simple_selectors[0] {
                    SimpleSelector::PseudoClass(name, Some(not_args)) if name == "not" => {
                        assert_eq!(not_args.len(), 1);

                        assert_eq!(
                            not_args[0].first.simple_selectors,
                            vec![SimpleSelector::Class("foo".into())]
                        );
                    }
                    _ => panic!("Expected :not inside :has"),
                }
            }
            _ => panic!("Expected :has at root"),
        };
    }
    #[test]
    fn test_empty_and_nonempty_where() {
        // Test :where() with no arguments
        let selector = parse_first_selector(":where()");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::PseudoClass("where".into(), Some(vec![]))]
        );

        // Test :where(.a, .b)
        let selector = parse_first_selector(":where(.a, .b)");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::PseudoClass(
                "where".into(),
                Some(vec![
                    {
                        let mut cs = ComplexSelector::default();
                        cs.first
                            .simple_selectors
                            .push(SimpleSelector::Class("a".into()));
                        cs
                    },
                    {
                        let mut cs = ComplexSelector::default();
                        cs.first
                            .simple_selectors
                            .push(SimpleSelector::Class("b".into()));
                        cs
                    }
                ])
            )]
        );
    }

    #[test]
    fn test_not_no_args_and_multiple_args() {
        // Test :not() with no arguments
        let selector = parse_first_selector(":not()");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::PseudoClass("not".into(), Some(vec![]))]
        );

        // Test :not(.a, #id, div)
        let selector = parse_first_selector(":not(.a, #id, div)");
        assert_eq!(
            selector.first.simple_selectors,
            vec![SimpleSelector::PseudoClass(
                "not".into(),
                Some(vec![
                    {
                        let mut cs = ComplexSelector::default();
                        cs.first
                            .simple_selectors
                            .push(SimpleSelector::Class("a".into()));
                        cs
                    },
                    {
                        let mut cs = ComplexSelector::default();
                        cs.first
                            .simple_selectors
                            .push(SimpleSelector::Id("id".into()));
                        cs
                    },
                    {
                        let mut cs = ComplexSelector::default();
                        cs.first
                            .simple_selectors
                            .push(SimpleSelector::Type("div".into()));
                        cs
                    }
                ])
            )]
        );
    }

    #[test]
    fn test_parse_background_radial_gradient_function() {
        let css =
            "div { background: radial-gradient(circle at 50% 50%, red, blue 100%) !important; }";
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1, "Should parse one CSS rule");
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };

        assert_eq!(
            style_rule.selectors.len(),
            1,
            "Rule should have one selector"
        );
        assert_eq!(
            style_rule.declarations.len(),
            1,
            "Rule should have one declaration"
        );
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::Background);

        // Check if the value is marked as important
        assert!(
            decl.value.is_important,
            "The declaration should be marked as important"
        );

        let found = decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "radial-gradient"
            } else {
                false
            }
        });
        assert!(
            found,
            "Did not find a radial-gradient() function in parsed components"
        );
    }

    #[test]
    fn test_parse_background_url_function() {
        let css = "div { background-image: url(\"/foo/bar.png\"); }";
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1, "Should parse one CSS rule");
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(
            style_rule.selectors.len(),
            1,
            "Rule should have one selector"
        );
        assert_eq!(
            style_rule.declarations.len(),
            1,
            "Rule should have one declaration"
        );
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::BackgroundImage);

        let found = decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "url"
            } else {
                false
            }
        });
        assert!(
            found,
            "Did not find a url() function in parsed components for background-image"
        );
    }

    #[test]
    fn test_parse_filter_drop_shadow_function() {
        let css = "div { filter: drop-shadow(1px 2px 4px #000); }";
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };

        assert_eq!(style_rule.selectors.len(), 1, "Should have one selector");
        let selector = &style_rule.selectors[0];
        assert!(
            selector
                .first
                .simple_selectors
                .iter()
                .any(|s| matches!(s, SimpleSelector::Type(t) if t == "div")),
            "First selector should be 'div'"
        );
        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::Filter);

        let found = decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "drop-shadow"
            } else {
                false
            }
        });
        assert!(
            found,
            "Did not find a drop-shadow() function in parsed components for filter"
        );
    }

    #[test]
    fn test_parse_transform_translate_rotate_scale_functions() {
        let css = r#"
            div {
                transform: translate(10px, 20px) rotate(30deg) scale(2);
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };

        assert_eq!(style_rule.selectors.len(), 1, "Should have one selector");
        let selector = &style_rule.selectors[0];
        assert!(
            selector
                .first
                .simple_selectors
                .iter()
                .any(|s| matches!(s, SimpleSelector::Type(t) if t == "div")),
            "First selector should be 'div'"
        );

        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::Transform);

        let mut found_translate = false;
        let mut found_rotate = false;
        let mut found_scale = false;
        for cv in &decl.value.components {
            if let ComponentValue::Function(func) = cv {
                match func.name.as_str() {
                    "translate" => found_translate = true,
                    "rotate" => found_rotate = true,
                    "scale" => found_scale = true,
                    _ => {}
                }
            }
        }
        assert!(
            found_translate,
            "Did not find translate() function in transform"
        );
        assert!(found_rotate, "Did not find rotate() function in transform");
        assert!(found_scale, "Did not find scale() function in transform");
    }

    #[test]
    fn test_parse_color_rgb_and_var_functions() {
        let css = r#"
            div {
                color: rgb(255, 0, 0);
                background: var(--my-color);
            }
        "#;
        println!("{css}");
        let tokens = Tokenizer::from_string(css);
        let tokens_clone = tokens.clone();
        let mut parser = Parser::new(&tokens_clone);
        let rules = parser.parse().expect("Should parse successfully");

        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(
            style_rule.selectors.len(),
            1,
            "Should have a selector for the rule"
        );
        let selector = &style_rule.selectors[0];
        assert!(
            selector
                .first
                .simple_selectors
                .iter()
                .any(|s| matches!(s, SimpleSelector::Type(t) if t == "div")),
            "First selector should be 'div'"
        );

        assert_eq!(style_rule.declarations.len(), 2);
        let color_decl = style_rule
            .declarations
            .iter()
            .find(|d| d.property == Property::Color)
            .expect("Missing color declaration");
        let rgb_found = color_decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "rgb"
            } else {
                false
            }
        });
        assert!(rgb_found, "Did not find rgb() in color value");

        let bg_decl = style_rule
            .declarations
            .iter()
            .find(|d| d.property == Property::Background)
            .expect("Missing background declaration");
        let var_found = bg_decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "var"
            } else {
                false
            }
        });
        assert!(var_found, "Did not find var() in background value");
    }

    #[test]
    fn test_parse_clip_path_polygon_function() {
        let css = r#"
            .foo {
                clip-path: polygon(20% 0%, 100% 0%, 100% 100%, 0% 100%);
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");

        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(style_rule.selectors.len(), 1);
        let selector = &style_rule.selectors[0];
        assert!(
            selector
                .first
                .simple_selectors
                .iter()
                .any(|s| matches!(s, SimpleSelector::Class(t) if t == "foo")),
            "First selector should be .foo"
        );

        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::ClipPath);

        let found = decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "polygon"
            } else {
                false
            }
        });
        assert!(found, "Did not find polygon() in clip-path value");
    }

    #[test]
    fn test_parse_border_radius_multiple_lengths() {
        let css = r#"
            .corner {
                border-radius: 5px 10px 15px 20px;
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(style_rule.selectors.len(), 1);
        let selector = &style_rule.selectors[0];
        assert!(
            selector
                .first
                .simple_selectors
                .iter()
                .any(|s| matches!(s, SimpleSelector::Class(t) if t == "corner"))
        );
        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::BorderRadius);

        let lengths = decl
            .value
            .components
            .iter()
            .filter(|c| matches!(c, ComponentValue::Dimension { .. }))
            .count();
        assert_eq!(lengths, 4, "Should have 4 length component values");
    }

    #[test]
    fn test_parse_font_shorthand_family() {
        let css = r#"
            .serif {
                font: italic small-caps bold 16px/2 Georgia, "Times New Roman", serif;
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(style_rule.selectors.len(), 1);
        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::Font);

        let mut found_georgia = false;
        let mut found_times = false;
        let mut found_serif = false;
        for c in &decl.value.components {
            match c {
                ComponentValue::Ident(s) if s == "serif" => found_serif = true,
                ComponentValue::String(s) if s == "Times New Roman" => found_times = true,
                ComponentValue::Ident(s) if s == "Georgia" => found_georgia = true,
                _ => {}
            }
        }
        assert!(found_georgia, "Should find Georgia font");
        assert!(found_times, "Should find Times New Roman font");
        assert!(found_serif, "Should find serif font");
    }

    #[test]
    fn test_parse_transition_shorthand() {
        let css = r#"
            .trans {
                transition: opacity 0.5s linear 2s;
            }
        "#;
        use crate::cssom::{property::Property, unit::Unit};

        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::Transition);

        let has_opacity = decl
            .value
            .components
            .iter()
            .any(|c| matches!(c, ComponentValue::Ident(s) if s == "opacity"));
        let has_duration = decl.value.components.iter().any(|c| {
            matches!(c, ComponentValue::Dimension { value, unit }
                if (*value - 0.5).abs() < std::f32::EPSILON && *unit == Unit::S)
        });
        let has_linear = decl
            .value
            .components
            .iter()
            .any(|c| matches!(c, ComponentValue::Ident(s) if s == "linear"));
        let has_delay = decl.value.components.iter().any(|c| {
            matches!(c, ComponentValue::Dimension { value, unit }
                if (*value - 2.0).abs() < std::f32::EPSILON && *unit == Unit::S)
        });

        assert!(has_opacity, "Should have identifier opacity");
        assert!(has_duration, "Should have .5s duration");
        assert!(has_linear, "Should have linear timing");
        assert!(has_delay, "Should have 2s delay");
    }

    #[test]
    fn test_parse_multiple_declarations_and_selectors() {
        let css = r#"
            h1, .xx-large {
                font-size: 40px;
                line-height: 1.1;
                color: #222;
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(style_rule.selectors.len(), 2);
        assert_eq!(style_rule.declarations.len(), 3);

        let font_size = style_rule
            .declarations
            .iter()
            .find(|d| d.property == Property::FontSize);
        let line_height = style_rule
            .declarations
            .iter()
            .find(|d| d.property == Property::LineHeight);
        let color = style_rule
            .declarations
            .iter()
            .find(|d| d.property == Property::Color);

        assert!(font_size.is_some(), "Should find font-size");
        assert!(line_height.is_some(), "Should find line-height");
        assert!(color.is_some(), "Should find color");
        let color_decl = color.unwrap();
        let hash_found = color_decl
            .value
            .components
            .iter()
            .any(|c| matches!(c, ComponentValue::Hash(s) if s == "222"));
        assert!(hash_found, "Should find #222 in color");
    }

    #[test]
    fn test_parse_gradient_as_background_image() {
        let css = r#"
            div.gradient {
                background-image: linear-gradient(90deg, #fff, #000 60%);
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::BackgroundImage);

        let has_gradient = decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "linear-gradient"
            } else {
                false
            }
        });
        assert!(has_gradient, "Should have linear-gradient function");
    }

    #[test]
    fn test_parse_background_image_with_url() {
        let css = r#"
            .img-bg {
                background-image: url("https://example.com/image.png");
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::BackgroundImage);

        let has_url = decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "url"
                    && func.args.iter().any(|arg| {
                        matches!(arg, ComponentValue::String(s) if s == "https://example.com/image.png")
                    })
            } else {
                false
            }
        });
        assert!(has_url, "Should have url('https://example.com/image.png')");
    }

    #[test]
    fn test_parse_list_style_image_with_url() {
        let css = r#"
            ul.custom-list {
                list-style-image: url('/static/foo.svg');
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        let style_rule = match rule {
            CssRule::Style(style_rule) => style_rule,
            _ => panic!("Expected StyleRule"),
        };
        assert_eq!(style_rule.declarations.len(), 1);
        let decl = &style_rule.declarations[0];
        assert_eq!(decl.property, Property::ListStyleImage);

        // Should contain a url() function
        let has_url = decl.value.components.iter().any(|c| {
            if let ComponentValue::Function(func) = c {
                func.name == "url"
                    && func.args.iter().any(
                        |arg| matches!(arg, ComponentValue::String(s) if s == "/static/foo.svg"),
                    )
            } else {
                false
            }
        });
        assert!(has_url, "Should have url('/static/foo.svg')");
    }
    #[test]
    fn test_parse_simple_at_rule_with_block() {
        let css = r#"
            @media screen and (max-width: 600px) {
                body {
                    background: red;
                    color: blue;
                }
                p, a {
                    margin: 0;
                }
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1, "Should parse one at-rule");
        match &rules[0] {
            CssRule::At(at_rule) => {
                assert_eq!(at_rule.name, "media");
                assert!(
                    at_rule
                        .prelude
                        .iter()
                        .any(|c| matches!(c, CssToken::Ident(s) if s == "screen"))
                );
                match &at_rule.block {
                    Some(crate::cssom::rule::AtRuleBlock::RuleList(rule_list)) => {
                        assert_eq!(rule_list.len(), 2);
                        // body rule
                        match &rule_list[0] {
                            CssRule::Style(style_rule) => {
                                assert_eq!(style_rule.selectors.len(), 1);
                                assert!(
                                    style_rule
                                        .declarations
                                        .iter()
                                        .any(|decl| decl.property == Property::Background),
                                    "Body rule should have 'background' declaration"
                                );
                                assert!(
                                    style_rule
                                        .declarations
                                        .iter()
                                        .any(|decl| decl.property == Property::Color),
                                    "Body rule should have 'color' declaration"
                                );
                            }
                            _ => panic!("Expected StyleRule inside media block"),
                        }
                        // p, a rule
                        match &rule_list[1] {
                            CssRule::Style(style_rule) => {
                                assert_eq!(
                                    style_rule.selectors.len(),
                                    2,
                                    "Second rule should have two selectors (p, a)"
                                );
                                assert!(
                                    style_rule
                                        .declarations
                                        .iter()
                                        .any(|decl| decl.property == Property::Margin),
                                    "Second rule should have 'margin' declaration"
                                );
                            }
                            _ => panic!("Expected StyleRule inside media block"),
                        }
                    }
                    _ => panic!("Expected RuleList block in @media"),
                }
            }
            _ => panic!("Expected AtRule variant"),
        }
    }

    #[test]
    fn test_parse_at_rule_without_block() {
        let css = r#"
            @import url('foo.css');
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        match &rules[0] {
            CssRule::At(at_rule) => {
                assert_eq!(at_rule.name, "import");
                // Should contain a url() function represented as a CssToken
                assert!(
                    at_rule.prelude.iter().any(|c| match c {
                        CssToken::Ident(s) if s == "url" => true,
                        CssToken::LeftParen => true,
                        CssToken::String(s) if s == "foo.css" => true,
                        _ => false,
                    }),
                    "Prelude should contain a url('foo.css') as CssToken"
                );
                assert!(
                    at_rule.block.is_none(),
                    "Import rule should not have a block"
                );
            }
            _ => panic!("Expected AtRule"),
        }
    }

    #[test]
    fn test_parse_at_rule_with_declaration_block() {
        let css = r#"
            @font-face {
                font-family: 'Open Sans';
                src: url(OpenSans-Regular.ttf);
                font-weight: 400;
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);

        match &rules[0] {
            CssRule::At(at_rule) => {
                assert_eq!(&at_rule.name, "font-face");
                match &at_rule.block {
                    Some(crate::cssom::rule::AtRuleBlock::DeclarationList(decls)) => {
                        assert!(
                            decls
                                .iter()
                                .any(|d| matches!(d.property, Property::FontFamily))
                        );
                        assert!(decls.iter().any(|d| matches!(d.property, Property::Src)));
                        assert!(
                            decls
                                .iter()
                                .any(|d| matches!(d.property, Property::FontWeight))
                        );
                    }
                    _ => panic!("Block should be DeclarationList"),
                }
            }
            _ => panic!("Expected AtRule"),
        }
    }

    #[test]
    fn test_parse_at_rule_with_raw_block() {
        let css = r#"
            @unknown {
                some strangle raw block;
                with non-declaration-stuff;
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);
        match &rules[0] {
            CssRule::At(at_rule) => {
                assert_eq!(&at_rule.name, "unknown");
                match &at_rule.block {
                    Some(crate::cssom::rule::AtRuleBlock::Raw(raw_tokens)) => {
                        // Should contain some delimiter or ident tokens.
                        assert!(
                            raw_tokens.iter().any(|t| matches!(
                                t,
                                crate::tokenizer::css_token::CssToken::Ident(_)
                            ))
                        );
                    }
                    _ => panic!("Block should be raw"),
                }
            }
            _ => panic!("Expected AtRule"),
        }
    }

    #[test]
    fn test_multiple_at_rules() {
        let css = r#"
            @import url(style1.css);
            @import url(style2.css);
            @media (min-width: 800px) { .foo { color: blue; } }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 3);
        assert!(matches!(&rules[0], CssRule::At(..)));
        assert!(matches!(&rules[1], CssRule::At(..)));
        match &rules[2] {
            CssRule::At(at_rule) => {
                assert_eq!(&at_rule.name, "media");
                assert!(matches!(
                    &at_rule.block,
                    Some(crate::cssom::rule::AtRuleBlock::RuleList(_))
                ));
            }
            _ => panic!("Expected AtRule for the third rule"),
        }
    }

    #[test]
    fn test_nested_at_rule() {
        let css = r#"
            @media (min-width: 600px) {
                @supports (display: grid) {
                    .foo { display: grid; }
                }
                @supports (display: flex) {
                    .bar { display: flex; }
                }
                @supports (color: var(--main-color)) {
                    .baz { color: var(--main-color); }
                }
            }
        "#;
        let tokens = Tokenizer::from_string(css);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse successfully");
        assert_eq!(rules.len(), 1);

        match &rules[0] {
            CssRule::At(media_at_rule) => {
                assert_eq!(&media_at_rule.name, "media");
                match &media_at_rule.block {
                    Some(AtRuleBlock::RuleList(inner_rules)) => {
                        assert_eq!(inner_rules.len(), 3);

                        // 1st @supports
                        match &inner_rules[0] {
                            CssRule::At(supports_at_rule) => {
                                assert_eq!(&supports_at_rule.name, "supports");
                                match &supports_at_rule.block {
                                    Some(AtRuleBlock::RuleList(supports_inner_rules)) => {
                                        assert_eq!(supports_inner_rules.len(), 1);
                                        match &supports_inner_rules[0] {
                                            CssRule::Style(style_rule) => {
                                                // Ensure the selector includes 'foo'
                                                assert!(style_rule.selectors.iter().any(|sel| {
                                                    format!("{:?}", sel).contains("foo")
                                                }));
                                                // Ensure a declaration contains property 'display'
                                                assert!(
                                                    style_rule
                                                        .declarations
                                                        .iter()
                                                        .any(|decl| decl.property == crate::cssom::property::Property::Display)
                                                );
                                                // Ensure a declaration contains value 'grid'
                                                assert!(style_rule.declarations.iter().any(
                                                    |decl| decl.value.components.iter().any(|component| match component {
                                                        crate::cssom::component_value::ComponentValue::Ident(s) => s == "grid",
                                                        _ => false,
                                                    })
                                                ));
                                            }
                                            _ => panic!("Expected StyleRule inside supports (1)"),
                                        }
                                    }
                                    _ => panic!("Expected RuleList in supports block (1)"),
                                }
                            }
                            _ => panic!("Expected At(Supports) inside media (1)"),
                        }

                        // 2nd @supports
                        match &inner_rules[1] {
                            CssRule::At(supports_at_rule) => {
                                assert_eq!(&supports_at_rule.name, "supports");
                                match &supports_at_rule.block {
                                    Some(AtRuleBlock::RuleList(supports_inner_rules)) => {
                                        assert_eq!(supports_inner_rules.len(), 1);
                                        match &supports_inner_rules[0] {
                                            CssRule::Style(style_rule) => {
                                                assert!(style_rule.selectors.iter().any(|sel| {
                                                    format!("{:?}", sel).contains("bar")
                                                }));
                                                assert!(
                                                    style_rule
                                                        .declarations
                                                        .iter()
                                                        .any(|decl| decl.property == crate::cssom::property::Property::Display)
                                                );
                                                assert!(style_rule.declarations.iter().any(
                                                    |decl| decl.value.components.iter().any(|component| match component {
                                                        crate::cssom::component_value::ComponentValue::Ident(s) => s == "flex",
                                                        _ => false,
                                                    })
                                                ));
                                            }
                                            _ => panic!("Expected StyleRule inside supports (2)"),
                                        }
                                    }
                                    _ => panic!("Expected RuleList in supports block (2)"),
                                }
                            }
                            _ => panic!("Expected At(Supports) inside media (2)"),
                        }

                        // 3rd @supports
                        match &inner_rules[2] {
                            CssRule::At(supports_at_rule) => {
                                assert_eq!(&supports_at_rule.name, "supports");
                                match &supports_at_rule.block {
                                    Some(AtRuleBlock::RuleList(supports_inner_rules)) => {
                                        assert_eq!(supports_inner_rules.len(), 1);
                                        match &supports_inner_rules[0] {
                                            CssRule::Style(style_rule) => {
                                                assert!(style_rule.selectors.iter().any(|sel| {
                                                    format!("{:?}", sel).contains("baz")
                                                }));
                                                assert!(style_rule.declarations.iter().any(
                                                    |decl| decl.property
                                                        == crate::cssom::property::Property::Color
                                                ));
                                                assert!(style_rule.declarations.iter().any(
                                                    |decl| decl.value.components.iter().any(|component| match component {
                                                        crate::cssom::component_value::ComponentValue::Function(function) => {
                                                            function.name == "var"
                                                                && function.args.iter().any(|arg| match arg {
                                                                    crate::cssom::component_value::ComponentValue::Ident(s) => s == "--main-color",
                                                                    _ => false,
                                                                })
                                                        }
                                                        _ => false,
                                                    })
                                                ));
                                            }
                                            _ => panic!("Expected StyleRule inside supports (3)"),
                                        }
                                    }
                                    _ => panic!("Expected RuleList in supports block (3)"),
                                }
                            }
                            _ => panic!("Expected At(Supports) inside media (3)"),
                        }
                    }
                    _ => panic!("Expected RuleList in media block"),
                }
            }
            _ => panic!("Expected At(Media) as the outermost rule"),
        }
    }
}
