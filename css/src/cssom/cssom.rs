use crate::cssom::{rule::CssRule, selector_index::SelectorIndex};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CssOm {
    pub rules: Vec<CssRule>,
    pub selector_index: SelectorIndex,
}

impl CssOm {
    pub fn append_stylesheet(&mut self, rules: Vec<CssRule>) {
        self.rules.extend(rules);
    }
}
