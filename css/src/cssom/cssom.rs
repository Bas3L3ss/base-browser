use html::DomElement;

use crate::cssom::{
    css_indexes::{CssIndexes, RuleRef},
    rule::CssRule,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Origin {
    UserAgent,
    Author,
}

impl Default for Origin {
    fn default() -> Self {
        Origin::Author
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CssOm {
    pub rules: Vec<CssRule>,
    pub origin: Origin,
    pub indexes: CssIndexes,
}

impl CssOm {
    pub fn append_stylesheet(&mut self, rules: Vec<CssRule>) {
        self.rules.extend(rules);
    }

    pub fn build_indexes(&mut self) {
        self.indexes = CssIndexes::build(self)
    }

    pub(crate) fn lookup_candidates(&self, element: &DomElement) -> Vec<RuleRef> {
        let mut candidates = Vec::new();

        // Extract tag name
        let tag = element.tag_name();

        // Extract id if present
        let id = element.id();

        // Extract class list (split by whitespace)
        let classes = element.classes();

        let attributes = element.attributes.keys();

        // Lookup by id
        if let Some(id_val) = id {
            if let Some(rule_ref) = self.indexes.selectors.by_id.get(id_val) {
                candidates.extend(rule_ref);
            }
        }

        // Lookup by class
        for class in classes {
            if let Some(rule_ref) = self.indexes.selectors.by_class.get(class) {
                candidates.extend(rule_ref);
            }
        }

        // Lookup by attribute
        for attr in attributes {
            if let Some(rule_ref) = self.indexes.selectors.by_attribute.get(attr) {
                candidates.extend(rule_ref);
            }
        }

        // Lookup by tag name
        if let Some(rule_ref) = self.indexes.selectors.by_tag.get(&tag) {
            candidates.extend(rule_ref);
        }

        // Add universal selectors
        candidates.extend(&self.indexes.selectors.universal);

        candidates
    }
}
