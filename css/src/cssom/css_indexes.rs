use utils::profile_scope;

use crate::{
    cssom::{
        cssom::CssOm,
        property::Property,
        rule::{AtRule, AtRuleBlock, CssRule, StyleRule},
        selector::SimpleSelector,
    },
    tokenizer::css_token::CssToken,
};

use std::collections::HashMap;

/// Represents an indexed @media rule and its recursively indexed subtree.
///
/// `rule_index` is the index within the rule list used to build this CssIndexes.
/// This is **not** a global CSSOM index; it's local to the rule list for this `CssIndexes`.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaRuleIndex {
    /// Index within the rule list used to build this CssIndexes.
    pub rule_index: u32,

    /// Indexes built from the rules inside this media block.
    pub indexes: CssIndexes,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuleRef {
    /// Index within the rule list for this CssIndexes.
    pub rule_index: u32,
    pub selector_index: u16,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SelectorIndex {
    pub by_tag: HashMap<String, Vec<RuleRef>>,
    pub by_class: HashMap<String, Vec<RuleRef>>,
    pub by_attribute: HashMap<String, Vec<RuleRef>>,
    pub by_id: HashMap<String, Vec<RuleRef>>,
    pub universal: Vec<RuleRef>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CssIndexes {
    pub selectors: SelectorIndex,
    pub keyframes: HashMap<String, u32>,
    pub fonts: HashMap<String, Vec<u32>>,
    pub media: Vec<MediaRuleIndex>,
}

impl CssIndexes {
    pub fn append_indexes(&mut self, mut indexes: CssIndexes) {
        macro_rules! extend_map_vec {
            ($dst:expr, $src:expr) => {
                for (k, v) in $src.drain() {
                    $dst.entry(k).or_default().extend(v);
                }
            };
        }

        extend_map_vec!(self.selectors.by_tag, indexes.selectors.by_tag);
        extend_map_vec!(self.selectors.by_class, indexes.selectors.by_class);
        extend_map_vec!(self.selectors.by_attribute, indexes.selectors.by_attribute);
        extend_map_vec!(self.selectors.by_id, indexes.selectors.by_id);

        self.selectors
            .universal
            .extend(indexes.selectors.universal.drain(..));

        self.keyframes.extend(indexes.keyframes.into_iter());
        self.fonts.extend(indexes.fonts.into_iter());
        self.media.extend(indexes.media);
    }

    /// Builds and returns `CssIndexes` for efficient style rule lookup by fast paths.
    /// Traverses the CSSOM's rule list.
    pub fn build(cssom: &CssOm) -> Self {
        profile_scope!("Build Selector Index");
        let mut index = Self::default();
        index.build_rules(&cssom.rules);
        index
    }

    /// Traverse a slice of CssRule and index them into this `CssIndexes`.
    fn build_rules(&mut self, rules: &[CssRule]) {
        for (rule_idx, rule) in rules.iter().enumerate() {
            match rule {
                CssRule::Style(style_rule) => {
                    self.handle_style_rule(style_rule, rule_idx as u32);
                }
                CssRule::At(at_rule) => {
                    self.handle_at_rule(at_rule, rule_idx as u32);
                }
            }
        }
    }

    /// Handle @-rules relevant for selector indexing: media, font-face, keyframes; ignore others.
    /// For media, we record their parent index and still process contained style rules.
    /// For font-face and keyframes, we index their parent position, but do not index style selectors.
    fn handle_at_rule(&mut self, at_rule: &AtRule, rule_index: u32) {
        if at_rule.name.eq_ignore_ascii_case("font-face") {
            // For @font-face, parse declarations for font-family property
            if let Some(AtRuleBlock::DeclarationList(decls)) = &at_rule.block {
                for decl in decls {
                    if decl.property == Property::FontFamily && decl.value.components.len() == 1 {
                        let font_family = decl.value.components[0].to_string();
                        self.fonts.entry(font_family).or_default().push(rule_index);
                        break;
                    }
                }
            }
        } else if at_rule.name.eq_ignore_ascii_case("keyframes") {
            // For @keyframes, get the name from prelude's first element if it is an Ident
            if at_rule.prelude.len() == 1 {
                if let Some(CssToken::Ident(ident)) = at_rule.prelude.first() {
                    self.keyframes.insert(ident.clone(), rule_index);
                }
            }
        } else if at_rule.name.eq_ignore_ascii_case("media") {
            // Attempt to find the contained CssRule list (stylesheet block) of @media
            if let Some(AtRuleBlock::RuleList(rules)) = &at_rule.block {
                // Build CssIndexes based on the CssRule list within this @media, without allocating a CssOm
                let mut child_index = CssIndexes::default();
                child_index.build_rules(rules);

                self.media.push(MediaRuleIndex {
                    rule_index,
                    indexes: child_index,
                });
            }
        }
    }

    /// "best keys" in priority order: Id > Classes > Attributes > Type > Universal.
    fn handle_style_rule(&mut self, style_rule: &StyleRule, rule_idx: u32) {
        for (selector_idx, selector) in style_rule.selectors.iter().enumerate() {
            let rightmost = selector.rightmost();
            let rule_ref = RuleRef {
                rule_index: rule_idx,
                selector_index: selector_idx as u16,
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
                self.selectors
                    .by_id
                    .entry(id_val.to_string())
                    .or_default()
                    .push(rule_ref);
            } else if !classes.is_empty() {
                for class_val in classes.iter() {
                    self.selectors
                        .by_class
                        .entry((*class_val).to_string())
                        .or_default()
                        .push(rule_ref);
                }
            } else if !attrs.is_empty() {
                for attr_val in attrs.iter() {
                    self.selectors
                        .by_attribute
                        .entry((*attr_val).to_string())
                        .or_default()
                        .push(rule_ref);
                }
            } else if let Some(tag_val) = tag {
                self.selectors
                    .by_tag
                    .entry(tag_val.to_string())
                    .or_default()
                    .push(rule_ref);
            } else if is_universal {
                self.selectors.universal.push(rule_ref);
            }
        }
    }
}
