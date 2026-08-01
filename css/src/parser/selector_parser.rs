use std::{matches, panic};

use crate::{
    cssom::{
        selector::{
            AttrSelectorOperator, Combinator, ComplexSelector, CompoundSelector, PseudoArg,
            SimpleSelector,
        },
        specificity::Specificity,
    },
    parser::parser::Parser,
    tokenizer::css_token::CssToken,
};
pub struct SelectorParser<'a, 'b> {
    parser: &'b mut Parser<'a>,
}

impl<'a, 'b> SelectorParser<'a, 'b> {
    pub fn new(parser: &'b mut Parser<'a>) -> Self {
        Self { parser }
    }

    pub fn parse_complex_selector(&mut self) -> Option<ComplexSelector> {
        if self.parser.tokens.is_empty() || self.parser.pos >= self.parser.tokens.len() {
            return None;
        }
        let first = self.parse_compound_selector();
        let mut rest = Vec::new();

        while let Some(selector) = self.parse_combinator_selector() {
            rest.push(selector);
        }
        if !first.is_empty() || !rest.is_empty() {
            let specificity: Specificity = Specificity::compute_complex_selector(&first, &rest);

            Some(ComplexSelector {
                first,
                rest,
                specificity,
            })
        } else {
            None
        }
    }

    fn parse_compound_selector(&mut self) -> CompoundSelector {
        let mut compound = CompoundSelector::new();
        while !self.parser.is_eof() {
            let before = self.parser.pos;
            if let Some(simple) = self.parse_simple_selector() {
                if !simple.is_subject_selector() && compound.is_empty() {
                    compound.push(SimpleSelector::Universal);
                }

                compound.push(simple);
            } else if before == self.parser.pos {
                break;
            }
        }
        compound
    }

    fn parse_simple_selector(&mut self) -> Option<SimpleSelector> {
        if self.parser.pos >= self.parser.tokens.len() {
            return None;
        }
        match self.parser.current() {
            CssToken::Dot => {
                if let Some(CssToken::Ident(name)) = self.parser.peek(1).cloned() {
                    self.parser.advance(2);
                    Some(SimpleSelector::Class(name))
                } else {
                    self.parser.advance(1);
                    None
                }
            }
            CssToken::Hash(name) => {
                let name = name.clone();
                self.parser.advance(1);
                Some(SimpleSelector::Id(name))
            }
            CssToken::Ident(tag) => {
                let tag = tag.clone();
                self.parser.advance(1);
                Some(SimpleSelector::Type(tag))
            }
            CssToken::Star => {
                self.parser.advance(1);
                Some(SimpleSelector::Universal)
            }
            CssToken::OpenBracket => {
                self.parser.advance(1); // Skip '['
                let (name, value, operator) = self.parse_attribute_selector();
                // Advance to after closing ']'
                while !self.parser.is_eof()
                    && !matches!(self.parser.current(), CssToken::CloseBracket)
                {
                    self.parser.advance(1);
                }
                if !self.parser.is_eof() && matches!(self.parser.current(), CssToken::CloseBracket)
                {
                    self.parser.advance(1); // Move past ']'
                }
                name.map(|attr_name| SimpleSelector::Attribute {
                    name: attr_name,
                    value,
                    operator,
                })
            }
            CssToken::Colon => {
                let start_pos = self.parser.pos;

                let selector = self.parse_pseudo_selector();

                if selector.is_some() {
                    selector
                } else {
                    self.parser.pos = start_pos;
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_combinator_selector(&mut self) -> Option<(Combinator, CompoundSelector)> {
        let start = self.parser.pos;

        let combinator = self.parse_combinator()?;

        let selector = self.parse_compound_selector();

        if selector.is_empty() {
            self.parser.pos = start;
            return None;
        }

        Some((combinator, selector))
    }

    fn parse_combinator(&mut self) -> Option<Combinator> {
        let saw_ws = self.parser.skip_whitespace();

        match self.parser.current() {
            CssToken::GreaterThan => {
                self.parser.advance(1);
                self.parser.skip_whitespace();
                Some(Combinator::Child)
            }
            CssToken::Plus => {
                self.parser.advance(1);
                self.parser.skip_whitespace();
                Some(Combinator::AdjacentSibling)
            }
            CssToken::Tilde => {
                self.parser.advance(1);
                self.parser.skip_whitespace();
                Some(Combinator::GeneralSibling)
            }
            _ if saw_ws => Some(Combinator::Descendant),
            _ => None,
        }
    }

    fn parse_pseudo_selector(&mut self) -> Option<SimpleSelector> {
        self.parser.expect(CssToken::Colon)?;

        let double_colon = self.parser.expect(CssToken::Colon).is_some();

        let CssToken::Ident(name) = self.parser.current() else {
            return None;
        };
        let name = name.clone();
        self.parser.advance(1);

        if self.parser.expect(CssToken::LeftParen).is_none() {
            return Some(if double_colon {
                SimpleSelector::PseudoElement(name)
            } else {
                SimpleSelector::PseudoClass(name, None)
            });
        }

        self.parser.skip_whitespace();

        let args: Option<PseudoArg> = if matches!(name.as_str(), "not" | "is" | "where" | "has") {
            self.parse_selector_list_pseudo_arg()
        } else {
            self.parse_values_pseudo_arg()
        };

        self.parser.expect(CssToken::RightParen)?;

        Some(SimpleSelector::PseudoClass(name, args))
    }

    fn parse_selector_list_pseudo_arg(&mut self) -> Option<PseudoArg> {
        let mut selectors = Vec::new();
        loop {
            if let Some(selector) = self.parse_complex_selector() {
                selectors.push(selector);
            } else {
                break;
            }
            self.parser.skip_whitespace();
            if self.parser.expect(CssToken::Comma).is_none() {
                break;
            }
            self.parser.skip_whitespace();
        }
        if selectors.is_empty() {
            None
        } else {
            Some(PseudoArg::SelectorList(selectors))
        }
    }

    fn parse_values_pseudo_arg(&mut self) -> Option<PseudoArg> {
        let mut values = Vec::new();
        loop {
            if let Some(value) = self.parser.parse_component_value() {
                values.push(value);
            } else if self.parser.expect(CssToken::Comma).is_some() {
                // Found a comma, continue to try parsing next value
                self.parser.skip_whitespace();
                continue;
            } else {
                break;
            }
        }
        if values.is_empty() {
            None
        } else {
            Some(PseudoArg::Values(values))
        }
    }
    fn parse_attribute_selector(
        &mut self,
    ) -> (Option<String>, Option<String>, Option<AttrSelectorOperator>) {
        let mut name = None;
        let mut value = None;
        let mut operator = None;

        // Collect attribute name
        while !self.parser.is_eof() {
            match self.parser.current() {
                CssToken::Ident(attr_name) => {
                    name = Some(attr_name.clone());
                    self.parser.advance(1);
                }
                CssToken::Whitespace => self.parser.advance(1),
                _ => break,
            }
        }

        if !self.parser.is_eof() {
            operator = match self.parser.current() {
                CssToken::Equals => {
                    self.parser.advance(1);
                    Some(AttrSelectorOperator::Equals)
                }
                CssToken::Tilde if matches!(self.parser.peek(1), Some(CssToken::Equals)) => {
                    self.parser.advance(2);
                    Some(AttrSelectorOperator::Includes)
                }
                CssToken::Delim('|') if matches!(self.parser.peek(1), Some(CssToken::Equals)) => {
                    self.parser.advance(2);
                    Some(AttrSelectorOperator::DashMatch)
                }
                CssToken::Delim('^') if matches!(self.parser.peek(1), Some(CssToken::Equals)) => {
                    self.parser.advance(2);
                    Some(AttrSelectorOperator::PrefixMatch)
                }
                CssToken::Delim('$') if matches!(self.parser.peek(1), Some(CssToken::Equals)) => {
                    self.parser.advance(2);
                    Some(AttrSelectorOperator::SuffixMatch)
                }
                CssToken::Star if matches!(self.parser.peek(1), Some(CssToken::Equals)) => {
                    self.parser.advance(2);
                    Some(AttrSelectorOperator::SubstringMatch)
                }
                _ => None,
            };
        }

        // Collect value if operator is found
        if operator.is_some() {
            while !self.parser.is_eof() && matches!(self.parser.current(), CssToken::Whitespace) {
                self.parser.advance(1);
            }
            let val_start = self.parser.pos;
            while !self.parser.is_eof() {
                match self.parser.current() {
                    CssToken::CloseBracket => break,
                    CssToken::Whitespace if val_start == self.parser.pos => self.parser.advance(1),
                    _ => self.parser.advance(1),
                }
            }
            value = self.parser.tokens[val_start..self.parser.pos]
                .iter()
                .find_map(|tok| match tok {
                    CssToken::Ident(v) => Some(v.clone()),
                    CssToken::String(v) => Some(v.clone()),
                    CssToken::Hash(v) => Some(v.clone()),
                    CssToken::Number(n) => Some(n.to_string()),
                    CssToken::Dimension { value: v, unit } => Some(format!("{}{}", v, unit)),
                    CssToken::Percentage(p) => Some(format!("{}%", p)),
                    CssToken::Delim(c) => Some(c.to_string()),
                    _ => None,
                });
        }
        (name, value, operator)
    }
}
