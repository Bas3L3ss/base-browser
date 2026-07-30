use std::unimplemented;

use html::dom::node::Node;

use crate::cssom::{css_indexes::CssIndexes, rule::CssRule};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CssOm {
    pub rules: Vec<CssRule>,
    pub indexes: CssIndexes,
}

impl CssOm {
    pub(crate) fn append_stylesheet(&mut self, rules: Vec<CssRule>) {
        self.rules.extend(rules);
    }

    pub(crate) fn build_indexes(&mut self) {
        self.indexes = CssIndexes::build(self)
    }

    pub(crate) fn lookup_candidates(&self, node: &Node) -> Vec<u32> {
        unimplemented!()
    }
}
