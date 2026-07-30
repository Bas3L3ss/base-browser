use crate::cssom::{css_indexes::CssIndexes, rule::CssRule};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CssOm {
    pub rules: Vec<CssRule>,
    pub indexes: CssIndexes,
}

impl CssOm {
    pub fn append_stylesheet(&mut self, rules: Vec<CssRule>) {
        self.rules.extend(rules);
    }

    pub fn append_indexes(&mut self, indexes: CssIndexes) {
        self.indexes.append_indexes(indexes);
    }
}
