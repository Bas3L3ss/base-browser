use std::{matches, println, vec};

use crate::{
    cssom::selector::{
        AttrSelectorOperator, Combinator, ComplexSelector, CompoundSelector, SimpleSelector,
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
            Some(ComplexSelector { first, rest })
        } else {
            None
        }
    }

    fn parse_compound_selector(&mut self) -> CompoundSelector {
        let mut compound = CompoundSelector::new();
        while self.parser.pos < self.parser.tokens.len() {
            let before = self.parser.pos;
            if let Some(simple) = self.parse_simple_selector() {
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
                while self.parser.pos < self.parser.tokens.len()
                    && !matches!(self.parser.current(), CssToken::CloseBracket)
                {
                    self.parser.advance(1);
                }
                if self.parser.pos < self.parser.tokens.len()
                    && matches!(self.parser.current(), CssToken::CloseBracket)
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

        let mut args: Vec<ComplexSelector> = Vec::new();

        loop {
            if let Some(selector) = self.parse_complex_selector() {
                println!("hi");
                args.push(selector);
            } else {
                break;
            }

            self.parser.skip_whitespace();

            if self.parser.expect(CssToken::Comma).is_none() {
                break;
            }

            self.parser.skip_whitespace();
        }

        self.parser.expect(CssToken::RightParen)?;

        Some(SimpleSelector::PseudoClass(name, Some(args)))
    }

    // fn parse_pseudo_selector(&mut self) -> Option<SimpleSelector> {
    //     // At ':' or '::'
    //     let mut double_colon = false;
    //     self.parser.advance(1);
    //     if self.parser.pos < self.parser.tokens.len()
    //         && matches!(self.parser.current(), CssToken::Colon)
    //     {
    //         double_colon = true;
    //         self.parser.advance(1);
    //     }
    //     if self.parser.pos < self.parser.tokens.len() {
    //         if let CssToken::Ident(pseudo_name) = self.parser.current() {
    //             let pseudo_name = pseudo_name.clone();
    //             self.parser.advance(1);
    //             // Handle functional pseudo-class
    //             if self.parser.pos < self.parser.tokens.len()
    //                 && matches!(self.parser.current(), CssToken::LeftParen)
    //             {
    //                 self.parser.advance(1);
    //                 while self.parser.pos < self.parser.tokens.len()
    //                     && matches!(self.parser.current(), CssToken::Whitespace)
    //                 {
    //                     self.parser.advance(1);
    //                 }
    //                 let args_start = self.parser.pos;
    //                 let mut paren_depth = 1;
    //                 while self.parser.pos < self.parser.tokens.len() && paren_depth > 0 {
    //                     match self.parser.current() {
    //                         CssToken::LeftParen => {
    //                             paren_depth += 1;
    //                             self.parser.advance(1);
    //                         }
    //                         CssToken::RightParen => {
    //                             paren_depth -= 1;
    //                             self.parser.advance(1);
    //                         }
    //                         _ => self.parser.advance(1),
    //                     }
    //                 }
    //                 let arg_end = if paren_depth == 0 {
    //                     self.parser.pos.saturating_sub(1)
    //                 } else {
    //                     self.parser.tokens.len()
    //                 };
    //                 let arg_tokens = if args_start < arg_end && arg_end <= self.parser.tokens.len()
    //                 {
    //                     &self.parser.tokens[args_start..arg_end]
    //                 } else {
    //                     &[]
    //                 };
    //                 let args_list = (!arg_tokens.is_empty()).then(|| self.parse_args(arg_tokens));
    //                 return Some(SimpleSelector::PseudoClass(pseudo_name, args_list));
    //             } else if double_colon {
    //                 return Some(SimpleSelector::PseudoElement(pseudo_name));
    //             } else {
    //                 return Some(SimpleSelector::PseudoClass(pseudo_name, None));
    //             }
    //         }
    //     }
    //     None
    // }

    // fn parse_args(tokens: &[CssToken]) -> Vec<ComplexSelector> {
    //     let mut selectors = Vec::new();
    //     let mut cur_selector_tokens = Vec::new();
    //     let mut i = 0;
    //     while i < tokens.len() {
    //         match &tokens[i] {
    //             CssToken::Comma => {
    //                 Self::push_selector_if_not_empty(&mut cur_selector_tokens, &mut selectors);
    //                 i += 1;
    //                 while matches!(tokens.get(i), Some(CssToken::Whitespace)) {
    //                     i += 1;
    //                 }
    //             }
    //             CssToken::EOF => break,
    //             _ => {
    //                 cur_selector_tokens.push(tokens[i].clone());
    //                 i += 1;
    //             }
    //         }
    //     }
    //     Self::push_selector_if_not_empty(&mut cur_selector_tokens, &mut selectors);
    //     selectors
    // }

    fn parse_attribute_selector(
        &mut self,
    ) -> (Option<String>, Option<String>, Option<AttrSelectorOperator>) {
        let mut name = None;
        let mut value = None;
        let mut operator = None;

        // Collect attribute name
        while self.parser.pos < self.parser.tokens.len() {
            match self.parser.current() {
                CssToken::Ident(attr_name) => {
                    name = Some(attr_name.clone());
                    self.parser.advance(1);
                }
                CssToken::Whitespace => self.parser.advance(1),
                _ => break,
            }
        }

        if self.parser.pos < self.parser.tokens.len() {
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
            while self.parser.pos < self.parser.tokens.len()
                && matches!(self.parser.current(), CssToken::Whitespace)
            {
                self.parser.advance(1);
            }
            let val_start = self.parser.pos;
            while self.parser.pos < self.parser.tokens.len() {
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
