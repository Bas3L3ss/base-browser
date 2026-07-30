use std::collections::HashMap;

use utils::profile_scope;

use crate::cssom::{
    cssom::CssOm,
    rule::{AtRule, CssRule, StyleRule},
    selector::SimpleSelector,
};

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRef {
    pub rule_index: usize,
    pub selector_index: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SelectorIndex {
    pub by_tag: HashMap<String, Vec<RuleRef>>,
    pub by_class: HashMap<String, Vec<RuleRef>>,
    pub by_attribute: HashMap<String, Vec<RuleRef>>,
    pub by_id: HashMap<String, Vec<RuleRef>>,
    pub universal: Vec<RuleRef>,
}

impl SelectorIndex {
    /// Builds and returns a `SelectorIndex` for efficient style rule lookup by fast paths.
    /// "best keys" in priority order: Id > Classes > Attributes > Type > Universal.
    pub fn build(cssom: &CssOm) -> Self {
        profile_scope!("Build Selector Index");
        let mut index = Self::default();

        for (rule_idx, rule) in cssom.rules.iter().enumerate() {
            match rule {
                CssRule::Style(style_rule) => {
                    index.handle_style_rule(style_rule, rule_idx);
                }
                CssRule::At(at_rule) => {
                    index.handle_at_rule(at_rule, rule_idx);
                }
            }
        }

        index
    }

    fn handle_style_rule(&mut self, style_rule: &StyleRule, rule_idx: usize) {
        for (selector_idx, selector) in style_rule.selectors.iter().enumerate() {
            let rightmost = selector.rightmost();
            let rule_ref = RuleRef {
                rule_index: rule_idx,
                selector_index: selector_idx,
            };

            // Collect all "best key"s depending on selector type
            let mut id: Option<&str> = None;
            let mut tag: Option<&str> = None;
            let mut classes: Vec<&str> = Vec::new();
            let mut attrs: Vec<&str> = Vec::new();
            let mut is_universal = false;

            for sel in &rightmost.simple_selectors {
                match sel {
                    SimpleSelector::Id(val) => id = Some(val),
                    SimpleSelector::Class(class) => classes.push(class),
                    SimpleSelector::Attribute { name, .. } => attrs.push(name),
                    SimpleSelector::Type(t) => tag = Some(t),
                    SimpleSelector::Universal => is_universal = true,
                    _ => {}
                }
            }

            // Insert based on the CSS specificity and structure rules
            if let Some(id_val) = id {
                // Only one id allowed per selector, and it's the highest specificity
                self.by_id
                    .entry(id_val.to_string())
                    .or_default()
                    .push(rule_ref);
            } else if !classes.is_empty() {
                for class_val in classes.iter() {
                    self.by_class
                        .entry((*class_val).to_string())
                        .or_default()
                        .push(rule_ref.clone());
                }
            } else if !attrs.is_empty() {
                for attr_val in attrs.iter() {
                    self.by_attribute
                        .entry((*attr_val).to_string())
                        .or_default()
                        .push(rule_ref.clone());
                }
            } else if let Some(tag_val) = tag {
                self.by_tag
                    .entry(tag_val.to_string())
                    .or_default()
                    .push(rule_ref.clone());
            } else if is_universal {
                self.universal.push(rule_ref);
            }
        }
    }

    fn handle_at_rule(&mut self, _at_rule: &AtRule, _rule_idx: usize) {}
}
