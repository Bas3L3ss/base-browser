use std::matches;

use crate::{
    cssom::selector::{
        AttrSelectorOperator, Combinator, ComplexSelector, CompoundSelector, SimpleSelector,
    },
    tokenizer::css_token::CssToken,
};

pub struct SelectorParser<'a> {
    tokens: &'a [CssToken],
    pos: usize,
}

impl<'a> SelectorParser<'a> {
    pub fn new(tokens: &'a [CssToken]) -> Self {
        SelectorParser { tokens, pos: 0 }
    }

    pub fn advance(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.pos = usize::min(self.pos.saturating_add(n), self.tokens.len());
    }

    pub fn peek(&self, offset: usize) -> Option<&CssToken> {
        self.tokens.get(self.pos.saturating_add(offset))
    }

    pub fn current(&self) -> &CssToken {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            static WS: CssToken = CssToken::Whitespace;
            &WS
        }
    }

    #[inline]
    fn push_selector_if_not_empty(
        cur_selector_tokens: &mut Vec<CssToken>,
        selectors: &mut Vec<ComplexSelector>,
    ) {
        if !cur_selector_tokens.is_empty() {
            // Trim leading and trailing whitespace
            while matches!(cur_selector_tokens.first(), Some(CssToken::Whitespace)) {
                cur_selector_tokens.remove(0);
            }
            while matches!(cur_selector_tokens.last(), Some(CssToken::Whitespace)) {
                cur_selector_tokens.pop();
            }

            if !cur_selector_tokens.is_empty() {
                let mut selector_parser = SelectorParser::new(&cur_selector_tokens);
                if let Some(sel) = selector_parser.parse_complex_selector() {
                    selectors.push(sel);
                }
            }
            cur_selector_tokens.clear();
        }
    }

    pub fn parse_complex_selector(&mut self) -> Option<ComplexSelector> {
        if self.tokens.is_empty() || self.pos >= self.tokens.len() {
            return None;
        }
        let first = self.parse_compound_selector();
        let mut rest = Vec::new();

        while self.pos < self.tokens.len() {
            if let Some(selector) = self.parse_combinator_selector() {
                rest.push(selector);
            } else {
                self.advance(1);
            }
        }
        Some(ComplexSelector { first, rest })
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.tokens.len() && matches!(self.current(), CssToken::Whitespace) {
            self.advance(1);
        }
    }
    pub fn parse_combinator_selector(&mut self) -> Option<(Combinator, CompoundSelector)> {
        let start_pos = self.pos;

        if self.pos < self.tokens.len() {
            let combinator = match self.current() {
                CssToken::Whitespace => {
                    self.advance(1);
                    self.skip_whitespace();
                    Some(Combinator::Descendant)
                }
                CssToken::Plus => {
                    self.advance(1);
                    self.skip_whitespace();
                    Some(Combinator::AdjacentSibling)
                }
                CssToken::GreaterThan => {
                    self.advance(1);
                    self.skip_whitespace();
                    Some(Combinator::Child)
                }
                CssToken::Tilde => {
                    self.advance(1);
                    self.skip_whitespace();
                    Some(Combinator::GeneralSibling)
                }
                _ => None,
            };

            if let Some(comb) = combinator {
                let selector = self.parse_compound_selector();
                if !selector.is_empty() || comb != Combinator::Descendant {
                    // For descendant, only return if selector isn't empty
                    return Some((comb, selector));
                } else {
                    self.pos = start_pos;
                }
            }
        }
        None
    }

    fn parse_compound_selector(&mut self) -> CompoundSelector {
        let mut compound = CompoundSelector::new();
        while self.pos < self.tokens.len() {
            let before = self.pos;
            if let Some(simple) = self.parse_simple_selector() {
                compound.push(simple);
            } else if before == self.pos {
                break;
            }
        }
        compound
    }

    fn parse_simple_selector(&mut self) -> Option<SimpleSelector> {
        if self.pos >= self.tokens.len() {
            return None;
        }
        match self.current() {
            CssToken::Dot => {
                if let Some(CssToken::Ident(name)) = self.peek(1).cloned() {
                    self.advance(2);
                    Some(SimpleSelector::Class(name))
                } else {
                    self.advance(1);
                    None
                }
            }
            CssToken::Hash(name) => {
                let name = name.clone();
                self.advance(1);
                Some(SimpleSelector::Id(name))
            }
            CssToken::Ident(tag) => {
                let tag = tag.clone();
                self.advance(1);
                Some(SimpleSelector::Type(tag))
            }
            CssToken::Star => {
                self.advance(1);
                Some(SimpleSelector::Universal)
            }
            CssToken::OpenBracket => {
                self.advance(1); // Skip '['
                let (name, value, operator) = self.parse_attribute_selector();
                // Advance to after closing ']'
                while self.pos < self.tokens.len()
                    && !matches!(self.current(), CssToken::CloseBracket)
                {
                    self.advance(1);
                }
                if self.pos < self.tokens.len() && matches!(self.current(), CssToken::CloseBracket)
                {
                    self.advance(1); // Move past ']'
                }
                name.map(|attr_name| SimpleSelector::Attribute {
                    name: attr_name,
                    value,
                    operator,
                })
            }
            CssToken::Colon => {
                let start_pos = self.pos;
                let selector = self.parse_pseudo_selector();
                if selector.is_some() {
                    selector
                } else {
                    self.pos = start_pos;
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_pseudo_selector(&mut self) -> Option<SimpleSelector> {
        // At ':' or '::'
        let mut double_colon = false;
        self.advance(1);
        if self.pos < self.tokens.len() && matches!(self.current(), CssToken::Colon) {
            double_colon = true;
            self.advance(1);
        }
        if self.pos < self.tokens.len() {
            if let CssToken::Ident(pseudo_name) = self.current() {
                let pseudo_name = pseudo_name.clone();
                self.advance(1);
                // Handle functional pseudo-class
                if self.pos < self.tokens.len() && matches!(self.current(), CssToken::LeftParen) {
                    self.advance(1);
                    while self.pos < self.tokens.len()
                        && matches!(self.current(), CssToken::Whitespace)
                    {
                        self.advance(1);
                    }
                    let args_start = self.pos;
                    let mut paren_depth = 1;
                    while self.pos < self.tokens.len() && paren_depth > 0 {
                        match self.current() {
                            CssToken::LeftParen => {
                                paren_depth += 1;
                                self.advance(1);
                            }
                            CssToken::RightParen => {
                                paren_depth -= 1;
                                self.advance(1);
                            }
                            _ => self.advance(1),
                        }
                    }
                    let arg_end = if paren_depth == 0 {
                        self.pos.saturating_sub(1)
                    } else {
                        self.tokens.len()
                    };
                    let arg_tokens = if args_start < arg_end && arg_end <= self.tokens.len() {
                        &self.tokens[args_start..arg_end]
                    } else {
                        &[]
                    };
                    let args_list = (!arg_tokens.is_empty()).then(|| Self::parse_args(arg_tokens));
                    return Some(SimpleSelector::PseudoClass(pseudo_name, args_list));
                } else if double_colon {
                    return Some(SimpleSelector::PseudoElement(pseudo_name));
                } else {
                    return Some(SimpleSelector::PseudoClass(pseudo_name, None));
                }
            }
        }
        None
    }

    fn parse_args(tokens: &[CssToken]) -> Vec<ComplexSelector> {
        let mut selectors = Vec::new();
        let mut cur_selector_tokens = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i] {
                CssToken::Comma => {
                    Self::push_selector_if_not_empty(&mut cur_selector_tokens, &mut selectors);
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
        Self::push_selector_if_not_empty(&mut cur_selector_tokens, &mut selectors);
        selectors
    }

    fn parse_attribute_selector(
        &mut self,
    ) -> (Option<String>, Option<String>, Option<AttrSelectorOperator>) {
        let mut name = None;
        let mut value = None;
        let mut operator = None;

        // Collect attribute name
        while self.pos < self.tokens.len() {
            match self.current() {
                CssToken::Ident(attr_name) => {
                    name = Some(attr_name.clone());
                    self.advance(1);
                }
                CssToken::Whitespace => self.advance(1),
                _ => break,
            }
        }

        if self.pos < self.tokens.len() {
            operator = match self.current() {
                CssToken::Equals => {
                    self.advance(1);
                    Some(AttrSelectorOperator::Equals)
                }
                CssToken::Tilde if matches!(self.peek(1), Some(CssToken::Equals)) => {
                    self.advance(2);
                    Some(AttrSelectorOperator::Includes)
                }
                CssToken::Delim('|') if matches!(self.peek(1), Some(CssToken::Equals)) => {
                    self.advance(2);
                    Some(AttrSelectorOperator::DashMatch)
                }
                CssToken::Delim('^') if matches!(self.peek(1), Some(CssToken::Equals)) => {
                    self.advance(2);
                    Some(AttrSelectorOperator::PrefixMatch)
                }
                CssToken::Delim('$') if matches!(self.peek(1), Some(CssToken::Equals)) => {
                    self.advance(2);
                    Some(AttrSelectorOperator::SuffixMatch)
                }
                CssToken::Star if matches!(self.peek(1), Some(CssToken::Equals)) => {
                    self.advance(2);
                    Some(AttrSelectorOperator::SubstringMatch)
                }
                _ => None,
            };
        }

        // Collect value if operator is found
        if operator.is_some() {
            while self.pos < self.tokens.len() && matches!(self.current(), CssToken::Whitespace) {
                self.advance(1);
            }
            let val_start = self.pos;
            while self.pos < self.tokens.len() {
                match self.current() {
                    CssToken::CloseBracket => break,
                    CssToken::Whitespace if val_start == self.pos => self.advance(1),
                    _ => self.advance(1),
                }
            }
            value = self.tokens[val_start..self.pos]
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
