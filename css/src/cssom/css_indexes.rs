use utils::profile_scope;

use crate::{
    FontFaceRule,
    cssom::{
        cssom::CssOm,
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
    pub fonts: HashMap<String, Vec<FontFaceRule>>,
    pub media: Vec<MediaRuleIndex>,
}

impl CssIndexes {
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
        match at_rule.name.to_ascii_lowercase().as_str() {
            "font-face" => {
                // For @font-face, parse the full FontFaceRule (using from_rule). Index by family.
                let font_face = FontFaceRule::from_rule(at_rule);
                if !font_face.family.is_empty() {
                    self.fonts
                        .entry(font_face.family.clone())
                        .or_default()
                        .push(font_face);
                }
            }
            "keyframes" => {
                // TODO: IMPLEMENT FROM RULE AND HANDLE THIS LIKE FONT-FACE

                // For @keyframes, get the name from prelude's first element if it is an Ident
                if at_rule.prelude.len() == 1 {
                    if let Some(CssToken::Ident(ident)) = at_rule.prelude.first() {
                        self.keyframes.insert(ident.clone(), rule_index);
                    }
                }
            }
            "media" => {
                // TODO: IMPLEMENT FROM RULE AND HANDLE THIS LIKE FONT-FACE

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
            _ => {}
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

#[cfg(test)]
mod tests {
    use std::{assert_eq, vec};

    use super::*;
    use crate::tokenizer::css_token::CssToken;
    use crate::{
        Origin,
        cssom::{
            component_value::ComponentValue,
            css_value::CssValue,
            property::Property,
            rule::{AtRule, AtRuleBlock, CssRule, StyleRule},
            selector::{AttrSelectorOperator, ComplexSelector, CompoundSelector, SimpleSelector},
        },
    };

    fn style_rule_with_simple_selector(selector: SimpleSelector) -> CssRule {
        CssRule::Style(StyleRule {
            selectors: vec![ComplexSelector {
                first: {
                    let mut c = CompoundSelector::new();
                    c.push(selector);
                    c
                },
                rest: vec![],
                ..Default::default()
            }],

            declarations: vec![],
        })
    }

    #[test]
    fn test_index_simple_type_selector() {
        let cssom = crate::cssom::cssom::CssOm {
            origin: Origin::Author,
            rules: vec![style_rule_with_simple_selector(SimpleSelector::Type(
                "div".into(),
            ))],
            indexes: Default::default(),
        };
        let indexes = CssIndexes::build(&cssom);
        assert_eq!(indexes.selectors.by_tag.get("div").unwrap().len(), 1);
    }

    #[test]
    fn test_index_by_id_and_class() {
        let cssom = crate::cssom::cssom::CssOm {
            origin: Origin::Author,

            rules: vec![
                style_rule_with_simple_selector(SimpleSelector::Id("main".into())),
                style_rule_with_simple_selector(SimpleSelector::Class("foo".into())),
            ],
            indexes: Default::default(),
        };
        let indexes = CssIndexes::build(&cssom);
        assert_eq!(indexes.selectors.by_id.get("main").unwrap().len(), 1);
        assert_eq!(indexes.selectors.by_class.get("foo").unwrap().len(), 1);
    }

    #[test]
    fn test_index_by_attribute() {
        let cssom = crate::cssom::cssom::CssOm {
            origin: Origin::Author,

            rules: vec![style_rule_with_simple_selector(SimpleSelector::Attribute {
                name: "data-x".into(),
                value: Some("test".into()),
                operator: Some(AttrSelectorOperator::Equals),
            })],
            indexes: Default::default(),
        };
        let indexes = CssIndexes::build(&cssom);
        assert_eq!(
            indexes.selectors.by_attribute.get("data-x").unwrap().len(),
            1
        );
    }

    #[test]
    fn test_index_universal_selector() {
        let cssom = crate::cssom::cssom::CssOm {
            origin: Origin::Author,

            rules: vec![style_rule_with_simple_selector(SimpleSelector::Universal)],
            indexes: Default::default(),
        };
        let indexes = CssIndexes::build(&cssom);
        assert_eq!(indexes.selectors.universal.len(), 1);
    }

    #[test]
    fn test_index_specificity_priority() {
        // Should index in by_id, not by_class/attribute/tag/universal
        let complex_sel = ComplexSelector {
            first: {
                let mut c = CompoundSelector::new();
                c.push(SimpleSelector::Id("super".into()));
                c.push(SimpleSelector::Class("c1".into()));
                c
            },
            rest: vec![],
            ..Default::default()
        };
        let cssom = crate::cssom::cssom::CssOm {
            origin: Origin::Author,

            rules: vec![CssRule::Style(StyleRule {
                selectors: vec![complex_sel],
                declarations: vec![],
            })],
            indexes: Default::default(),
        };
        let indexes = CssIndexes::build(&cssom);
        assert!(indexes.selectors.by_id.get("super").is_some());
        // Should only be present in by_id, not in by_class
        assert!(indexes.selectors.by_class.get("c1").is_none());
    }

    // CSS equivalent:
    //
    // @font-face {
    //   font-family: "Roboto";
    // }

    #[test]
    fn test_index_at_font_face() {
        let cssom = crate::cssom::cssom::CssOm {
            origin: Origin::Author,

            rules: vec![CssRule::At(AtRule {
                name: "font-face".into(),
                prelude: vec![],
                block: Some(AtRuleBlock::DeclarationList(vec![
                    crate::cssom::declaration::Declaration {
                        property: Property::FontFamily,
                        value: CssValue {
                            components: vec![ComponentValue::String("Roboto".into())],
                            is_important: false,
                        },
                    },
                ])),
            })],
            indexes: Default::default(),
        };

        let indexes = CssIndexes::build(&cssom);

        let fonts = indexes.fonts.get("Roboto");
        assert!(fonts.is_some());
        assert_eq!(fonts.unwrap().len(), 1);
    }

    #[test]
    fn test_index_at_keyframes() {
        let cssom = crate::cssom::cssom::CssOm {
            origin: Origin::Author,

            rules: vec![CssRule::At(AtRule {
                name: "keyframes".into(),
                prelude: vec![CssToken::Ident("spin".into())],
                block: None,
            })],
            indexes: Default::default(),
        };
        let indexes = CssIndexes::build(&cssom);
        let keyf = indexes.keyframes.get("spin");
        assert!(keyf.is_some());
    }

    #[test]
    fn test_index_at_media_nested_rules() {
        let inner_rule = style_rule_with_simple_selector(SimpleSelector::Type("span".into()));
        let cssom = crate::cssom::cssom::CssOm {
            origin: Origin::Author,

            rules: vec![CssRule::At(AtRule {
                name: "media".into(),
                prelude: vec![CssToken::Ident("screen".into())],
                block: Some(AtRuleBlock::RuleList(vec![inner_rule.clone()])),
            })],
            indexes: Default::default(),
        };
        let indexes = CssIndexes::build(&cssom);
        assert_eq!(indexes.media.len(), 1);
        let media_idx = &indexes.media[0];
        assert_eq!(
            media_idx
                .indexes
                .selectors
                .by_tag
                .get("span")
                .unwrap()
                .len(),
            1
        );
    }
}
