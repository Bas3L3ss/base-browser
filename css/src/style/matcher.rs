use std::unreachable;

use html::{
    dom::{dom::Dom, dom_element::DomElement, node::Node, node_type::NodeType},
    tokenizer::html_tags::HtmlTag,
};

use crate::cssom::{
    css_indexes::RuleRef,
    cssom::CssOm,
    rule::CssRule,
    selector::{self, AttrSelectorOperator, Combinator, CompoundSelector, SimpleSelector},
};

pub struct Matcher;

impl Matcher {
    pub(crate) fn match_rules(
        node: &Node,
        dom: &Dom,
        candidates: &[RuleRef],
        cssom: &CssOm,
    ) -> Option<Vec<u32>> {
        let mut matched = Vec::new();

        for candidate in candidates {
            let rule = cssom.rules.get(candidate.rule_index as usize);
            if rule.is_none() {
                continue;
            }

            // This must be a style rule, get as StyleRule variant and access its selectors
            let complex_selector = match rule.unwrap() {
                CssRule::Style(style_rule) => {
                    let sel = style_rule.selectors.get(candidate.rule_index as usize);
                    if sel.is_none() {
                        continue;
                    }
                    sel.unwrap()
                }
                _ => continue,
            };

            let mut cur_node = node;
            match complex_selector.rest.is_empty() {
                true => {
                    if Self::match_compound(cur_node, &complex_selector.first) {
                        matched.push(candidate.rule_index);
                    }
                }
                false => {}
            }
        }

        if !matched.is_empty() {
            Some(matched)
        } else {
            None
        }
    }

    fn match_compound(node: &Node, compound_selector: &CompoundSelector) -> bool {
        match &node.ntype {
            NodeType::Element(element) => {
                for selector in &compound_selector.simple_selectors {
                    if !Self::match_simple(element, selector) {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }

    fn match_simple(element: &DomElement, selector: &SimpleSelector) -> bool {
        match selector {
            SimpleSelector::Class(classname) => element.classes().contains(&classname.as_str()),
            SimpleSelector::Id(id) => element.id().map_or(false, |my_id: &str| my_id == id),
            SimpleSelector::Type(tag) => element.tag == HtmlTag::from_str(tag),

            SimpleSelector::Attribute {
                name,
                value,
                operator,
            } => {
                //
                if let Some(attr_val) = element.attributes.contains(name) {
                    Self::match_attribute_operators(value, operator, attr_val)
                } else {
                    false
                }
            }
            SimpleSelector::Universal => true,
            _ => false, // TODO
        }
    }

    fn match_attribute_operators(
        value: &Option<String>,
        operator: &Option<AttrSelectorOperator>,
        attr_val: String,
    ) -> bool {
        match operator {
            None => true,
            Some(AttrSelectorOperator::Equals) => {
                // 2. Exact Match: [attr="value"]
                if let Some(expected) = value {
                    &attr_val == expected
                } else {
                    false
                }
            }
            Some(AttrSelectorOperator::Includes) => {
                // 3. Whitespace-separated Contains: [attr~="value"]
                if let Some(expected) = value {
                    attr_val
                        .split_ascii_whitespace()
                        .any(|part| part == expected)
                } else {
                    false
                }
            }
            Some(AttrSelectorOperator::DashMatch) => {
                // 4. Hyphen Prefix: [attr|="value"]
                if let Some(expected) = value {
                    &attr_val == expected || attr_val.starts_with(&format!("{}-", expected))
                } else {
                    false
                }
            }
            Some(AttrSelectorOperator::PrefixMatch) => {
                // 5. Prefix: [attr^="value"]
                if let Some(expected) = value {
                    attr_val.starts_with(expected)
                } else {
                    false
                }
            }
            Some(AttrSelectorOperator::SuffixMatch) => {
                // 6. Suffix: [attr$="value"]
                if let Some(expected) = value {
                    attr_val.ends_with(expected)
                } else {
                    false
                }
            }
            Some(AttrSelectorOperator::SubstringMatch) => {
                // 7. Substring: [attr*="value"]
                if let Some(expected) = value {
                    attr_val.contains(expected)
                } else {
                    false
                }
            }
        }
    }
}

// TODO: #idea 4
