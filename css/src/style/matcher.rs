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

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::cssom::css_indexes::{CssIndexes, RuleRef};
    use crate::cssom::cssom::CssOm;
    use crate::cssom::rule::{CssRule, StyleRule};
    use crate::cssom::selector::{ComplexSelector, CompoundSelector, SimpleSelector};
    use html::dom::dom_element::DomElement;
    use html::dom::node_type::NodeType;
    use html::tokenizer::attributes::Attributes;

    fn make_node(dom_elem: DomElement) -> Node {
        Node {
            parent: None,
            children: vec![],
            ntype: NodeType::Element(dom_elem),
        }
    }

    fn single_candidate_for(selector: SimpleSelector) -> (CssOm, Vec<RuleRef>) {
        let mut cssom = CssOm::default();

        let mut compound = CompoundSelector::new();
        compound.push(selector);

        let complex = ComplexSelector {
            first: compound,
            rest: vec![],
        };

        let style_rule = StyleRule {
            selectors: vec![complex],
            declarations: vec![],
        };

        let rule_index = cssom.rules.len() as u32;
        cssom.rules.push(CssRule::Style(style_rule.clone()));

        // Match the RuleRef struct fields from @css/src/cssom/css_indexes.rs
        let candidates = vec![RuleRef {
            rule_index,
            selector_index: 0,
        }];
        (cssom, candidates)
    }

    fn dom_with_node(node: Node) -> Dom {
        Dom {
            nodes: vec![node],
            style_sheets: vec![],
        }
    }

    #[test]
    fn test_match_class_selector() {
        // <div class="foo bar"/>
        let mut attrs = Attributes::new();
        attrs.class = Some("foo bar".to_string());
        let elem = DomElement::new(HtmlTag::Div, attrs);

        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Class("foo".to_string()));

        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Class("bar".to_string()));
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Class("baz".to_string()));
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_id_selector() {
        let mut attrs = Attributes::new();
        attrs.id = Some("myid".to_string());
        let elem = DomElement::new(HtmlTag::Span, attrs);

        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Id("myid".to_string()));
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Id("other".to_string()));
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_compound_selector() {
        // <div id="myid" class="foo bar" title="Hello"></div>
        let mut attrs = Attributes::new();
        attrs.id = Some("myid".to_string());
        attrs.class = Some("foo bar".to_string());
        attrs.title = Some("Hello".to_string());
        let elem = DomElement::new(HtmlTag::Div, attrs);
        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        // .foo#myid[title="Hello"]
        let compound = CompoundSelector {
            simple_selectors: vec![
                SimpleSelector::Class("foo".to_string()),
                SimpleSelector::Id("myid".to_string()),
                SimpleSelector::Attribute {
                    name: "title".into(),
                    value: Some("Hello".into()),
                    operator: Some(AttrSelectorOperator::Equals),
                },
            ],
        };

        let selector = selector::ComplexSelector {
            first: compound.clone(),
            rest: vec![],
        };

        let style_rule = CssRule::Style(crate::cssom::rule::StyleRule {
            selectors: vec![selector],
            declarations: vec![],
        });

        let cssom = CssOm {
            rules: vec![style_rule],
            indexes: Default::default(),
        };

        let candidates = vec![RuleRef {
            rule_index: 0,
            selector_index: 0,
        }];

        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        // Negative test: use wrong ID
        let compound_wrong_id = CompoundSelector {
            simple_selectors: vec![
                SimpleSelector::Class("foo".to_string()),
                SimpleSelector::Id("otherid".to_string()),
                SimpleSelector::Attribute {
                    name: "title".into(),
                    value: Some("Hello".into()),
                    operator: Some(AttrSelectorOperator::Equals),
                },
            ],
        };

        let selector_wrong_id = selector::ComplexSelector {
            first: compound_wrong_id,
            rest: vec![],
        };

        let style_rule_wrong_id = CssRule::Style(crate::cssom::rule::StyleRule {
            selectors: vec![selector_wrong_id],
            declarations: vec![],
        });

        let cssom_wrong_id = CssOm {
            rules: vec![style_rule_wrong_id],
            indexes: Default::default(),
        };

        let candidates_wrong_id = vec![RuleRef {
            rule_index: 0,
            selector_index: 0,
        }];

        let matched_wrong =
            Matcher::match_rules(&node, &dom, &candidates_wrong_id, &cssom_wrong_id);
        assert_eq!(matched_wrong, None);
    }

    #[test]
    fn test_match_type_selector() {
        let attrs = Attributes::new();
        let elem = DomElement::new(HtmlTag::Div, attrs);

        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Type("div".to_string()));
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Type("span".to_string()));
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_universal_selector() {
        let attrs = Attributes::new();
        let elem = DomElement::new(HtmlTag::Div, attrs);
        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Universal);
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));
    }

    #[test]
    fn test_match_attribute_equals_selector() {
        let mut attrs = Attributes::new();
        attrs.title = Some("Hello".to_string());

        let elem = DomElement::new(HtmlTag::Div, attrs);

        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("Hello".into()),
            operator: Some(AttrSelectorOperator::Equals),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("Other".into()),
            operator: Some(AttrSelectorOperator::Equals),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_attribute_includes_selector() {
        let mut attrs = Attributes::new();
        attrs.title = Some("foo bar baz".to_string());

        let elem = DomElement::new(HtmlTag::Div, attrs);

        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("bar".into()),
            operator: Some(AttrSelectorOperator::Includes),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("qux".into()),
            operator: Some(AttrSelectorOperator::Includes),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_attribute_dashmatch_selector() {
        let mut attrs = Attributes::new();
        attrs.title = Some("en-US".to_string());

        let elem = DomElement::new(HtmlTag::Div, attrs);
        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("en".into()),
            operator: Some(AttrSelectorOperator::DashMatch),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("en-US".into()),
            operator: Some(AttrSelectorOperator::DashMatch),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("fr".into()),
            operator: Some(AttrSelectorOperator::DashMatch),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_attribute_prefix_suffix_substring() {
        let mut attrs = Attributes::new();
        attrs.title = Some("prefix-middle-suffix".to_string());

        let elem = DomElement::new(HtmlTag::Div, attrs);
        let node = make_node(elem);

        let dom = dom_with_node(node.clone());

        // PrefixMatch
        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("prefix".into()),
            operator: Some(AttrSelectorOperator::PrefixMatch),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        // SuffixMatch
        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("suffix".into()),
            operator: Some(AttrSelectorOperator::SuffixMatch),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        // SubstringMatch
        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("middle".into()),
            operator: Some(AttrSelectorOperator::SubstringMatch),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        // Negative
        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("nope".into()),
            operator: Some(AttrSelectorOperator::SubstringMatch),
        });
        let matched = Matcher::match_rules(&node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }
}
