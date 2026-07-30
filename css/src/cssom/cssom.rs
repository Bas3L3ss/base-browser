use html::dom::dom_element::DomElement;

use crate::cssom::{
    css_indexes::{CssIndexes, RuleRef},
    rule::CssRule,
};

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

    pub(crate) fn lookup_candidates(&self, element: &DomElement) -> Vec<RuleRef> {
        let mut candidates = Vec::new();

        // Extract tag name
        let tag = element.tag_name();

        // Extract id if present
        let id = element.id();

        // Extract class list (split by whitespace)
        let classes = element.classes();

        // Extract all attribute names except id and class
        let attributes = element.attribute_keys();

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
