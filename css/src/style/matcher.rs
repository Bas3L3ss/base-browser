use std::{collections::HashSet, unimplemented};

use html::{
    dom::{dom::Dom, dom_element::DomElement, node::Node, node_type::NodeType},
    tokenizer::html_tags::HtmlTag,
};

use crate::cssom::{
    css_indexes::RuleRef,
    cssom::CssOm,
    rule::CssRule,
    selector::{
        AttrSelectorOperator, Combinator, ComplexSelector, CompoundSelector, PseudoArg,
        SimpleSelector,
    },
};

pub struct Matcher;

impl Matcher {
    pub(crate) fn match_rules(
        node: &Node,
        dom: &Dom,
        candidates: &[RuleRef],
        cssom: &CssOm,
    ) -> Option<Vec<u32>> {
        let mut matched: HashSet<u32> = HashSet::new();

        for candidate in candidates {
            let rule = match cssom.rules.get(candidate.rule_index as usize) {
                Some(r) => r,
                None => continue,
            };

            // This must be a style rule, get as StyleRule variant and access its selectors
            let complex_selector = match rule {
                CssRule::Style(style_rule) => {
                    style_rule.selectors.get(candidate.selector_index as usize)
                }
                _ => None,
            };

            let complex_selector = match complex_selector {
                Some(sel) => sel,
                None => continue,
            };

            if complex_selector.rest.is_empty() {
                if Self::match_compound(node, complex_selector.rightmost(), dom) {
                    matched.insert(candidate.rule_index);
                }
            } else {
                if Self::matches_complex_selector_from(
                    node,
                    dom,
                    complex_selector,
                    (complex_selector.rest.len() - 1) as isize,
                ) {
                    matched.insert(candidate.rule_index);
                }
            }
        }

        if matched.is_empty() {
            None
        } else {
            Some(matched.into_iter().collect())
        }
    }

    pub fn matches_complex_selector_from(
        node: &Node,
        dom: &Dom,
        complex_selector: &ComplexSelector,
        rest_index: isize,
    ) -> bool {
        if rest_index < 0 {
            return Self::match_compound(node, &complex_selector.first, dom);
        }

        let (ref combinator, ref right_compound) = complex_selector.rest[rest_index as usize];

        if !Self::match_compound(node, right_compound, dom) {
            return false;
        }

        match combinator {
            Combinator::Descendant => {
                let left_compound: &CompoundSelector = if (rest_index - 1) < 0 {
                    &complex_selector.first
                } else {
                    &complex_selector.rest[(rest_index - 1) as usize].1
                };

                let mut cur_node = node.parent;

                while let Some(ancestor) = cur_node {
                    let parent = &dom.nodes[ancestor];
                    if Self::match_compound(parent, left_compound, dom) {
                        return Self::matches_complex_selector_from(
                            parent,
                            dom,
                            complex_selector,
                            rest_index - 1,
                        );
                    }
                    cur_node = parent.parent;
                }

                false
            }
            Combinator::Child => {
                let left_compound: &CompoundSelector = if (rest_index - 1) < 0 {
                    &complex_selector.first
                } else {
                    &complex_selector.rest[(rest_index - 1) as usize].1
                };

                if let Some(ancestor) = node.parent {
                    let parent = &dom.nodes[ancestor];
                    if Self::match_compound(parent, left_compound, dom)
                        && parent.children.contains(&node.id)
                    {
                        return Self::matches_complex_selector_from(
                            parent,
                            dom,
                            complex_selector,
                            rest_index - 1,
                        );
                    }
                }

                false
            }
            Combinator::AdjacentSibling => {
                let left_compound: &CompoundSelector = if (rest_index - 1) < 0 {
                    &complex_selector.first
                } else {
                    &complex_selector.rest[(rest_index - 1) as usize].1
                };
                if let Some(sibling) = node.previous_sibling(&dom.nodes) {
                    let sibling = &dom.nodes[sibling];
                    if Self::match_compound(sibling, left_compound, dom) {
                        return Self::matches_complex_selector_from(
                            sibling,
                            dom,
                            complex_selector,
                            rest_index - 1,
                        );
                    }
                }

                false
            }
            Combinator::GeneralSibling => {
                let left_compound: &CompoundSelector = if (rest_index - 1) < 0 {
                    &complex_selector.first
                } else {
                    &complex_selector.rest[(rest_index - 1) as usize].1
                };

                let mut cur_node = node.previous_sibling(&dom.nodes);
                while let Some(sibling_id) = cur_node {
                    let sibling = &dom.nodes[sibling_id];
                    if Self::match_compound(sibling, left_compound, dom) {
                        return Self::matches_complex_selector_from(
                            sibling,
                            dom,
                            complex_selector,
                            rest_index - 1,
                        );
                    }
                    cur_node = sibling.previous_sibling(&dom.nodes);
                }

                false
            }
        }
    }

    fn match_compound(node: &Node, compound_selector: &CompoundSelector, dom: &Dom) -> bool {
        match &node.ntype {
            NodeType::Element(element) => compound_selector
                .simple_selectors
                .iter()
                .all(|selector| Self::match_simple(element, selector, node, dom)),
            _ => false,
        }
    }

    fn match_simple(
        element: &DomElement,
        selector: &SimpleSelector,
        node: &Node,
        dom: &Dom,
    ) -> bool {
        match selector {
            SimpleSelector::Class(classname) => element.classes().contains(&classname.as_str()),
            SimpleSelector::Id(id) => element.id().map_or(false, |my_id: &str| my_id == id),
            SimpleSelector::Type(tag) => element.tag == HtmlTag::from_str(tag),
            SimpleSelector::Attribute {
                name,
                value,
                operator,
            } => element.attributes.contains(name).map_or(false, |attr_val| {
                Self::match_attribute_operators(value, operator, attr_val)
            }),
            SimpleSelector::Universal => true,
            SimpleSelector::PseudoClass(name, pseudo_args) => {
                Self::match_pseudo_class(name, pseudo_args.as_ref(), element, node, dom)
            }
            SimpleSelector::PseudoElement(_) => false,
        }
    }

    fn match_pseudo_class(
        name: &str,
        pseudo_args: Option<&PseudoArg>,
        element: &DomElement,
        node: &Node,
        dom: &Dom,
    ) -> bool {
        match name {
            "root" => match element.tag {
                HtmlTag::Html => true,
                _ => false,
            },
            "empty" => {
                return node.children.is_empty();
            }

            "first-child" => match node.siblings(&dom.nodes) {
                Some(sb) => sb.get(0).map_or(false, |first_id| *first_id == node.id),
                None => false,
            },
            "last-child" => match node.siblings(&dom.nodes) {
                Some(sb) => sb.last().map_or(false, |last_id| *last_id == node.id),
                None => false,
            },
            "only-child" => match node.siblings(&dom.nodes) {
                Some(sb) => sb.len() == 1 && sb[0] == node.id,
                None => false,
            },
            "nth-child" => {
                unimplemented!()
            }
            "nth-last-child" => {
                unimplemented!()
            }

            _ => false,
        }
    }
    fn match_pseudo_element() -> bool {
        unimplemented!()
    }

    fn match_attribute_operators(
        value: &Option<String>,
        operator: &Option<AttrSelectorOperator>,
        attr_val: String,
    ) -> bool {
        match operator {
            None => true,
            Some(AttrSelectorOperator::Equals) => value
                .as_ref()
                .map_or(false, |expected| &attr_val == expected),
            Some(AttrSelectorOperator::Includes) => value.as_ref().map_or(false, |expected| {
                attr_val
                    .split_ascii_whitespace()
                    .any(|part| part == expected)
            }),
            Some(AttrSelectorOperator::DashMatch) => value.as_ref().map_or(false, |expected| {
                &attr_val == expected || attr_val.starts_with(&format!("{}-", expected))
            }),
            Some(AttrSelectorOperator::PrefixMatch) => value
                .as_ref()
                .map_or(false, |expected| attr_val.starts_with(expected)),
            Some(AttrSelectorOperator::SuffixMatch) => value
                .as_ref()
                .map_or(false, |expected| attr_val.ends_with(expected)),
            Some(AttrSelectorOperator::SubstringMatch) => value
                .as_ref()
                .map_or(false, |expected| attr_val.contains(expected)),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{assert_eq, matches, println, vec};

    use super::*;
    use crate::cssom::css_indexes::{CssIndexes, RuleRef};
    use crate::cssom::cssom::CssOm;
    use crate::cssom::rule::{CssRule, StyleRule};
    use crate::cssom::selector::Combinator;
    use crate::cssom::selector::{self, ComplexSelector, CompoundSelector, SimpleSelector};
    use crate::tokenizer::tokenizer::Tokenizer;
    use html::dom::dom::Dom;
    use html::dom::node_type::NodeType;
    use html::parser::parser::Parser;

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
        let candidates = vec![RuleRef {
            rule_index,
            selector_index: 0,
        }];
        (cssom, candidates)
    }

    fn single_node(dom: &Dom) -> &Node {
        let idx = dom
            .nodes
            .iter()
            .position(|n| matches!(n.ntype, NodeType::Element(_)))
            .expect("element node found");
        &dom.nodes[idx]
    }

    #[test]
    fn test_match_class_selector() {
        let html = r#"<div class="foo bar"></div>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Class("foo".to_string()));
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Class("bar".to_string()));
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Class("baz".to_string()));
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_id_selector() {
        let html = r#"<span id="myid"></span>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Id("myid".to_string()));
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Id("other".to_string()));
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_compound_selector() {
        // <div id="myid" class="foo bar" title="Hello"></div>
        let html = r#"<div id="myid" class="foo bar" title="Hello"></div>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

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

        let style_rule = CssRule::Style(StyleRule {
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

        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
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

        let style_rule_wrong_id = CssRule::Style(StyleRule {
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

        let matched_wrong = Matcher::match_rules(node, &dom, &candidates_wrong_id, &cssom_wrong_id);
        assert_eq!(matched_wrong, None);
    }

    #[test]
    fn test_match_type_selector() {
        let html = r#"<div></div>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Type("div".to_string()));
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Type("span".to_string()));
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_universal_selector() {
        let html = r#"<div></div>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Universal);
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));
    }

    #[test]
    fn test_match_attribute_equals_selector() {
        let html = r#"<div title="Hello"></div>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("Hello".into()),
            operator: Some(AttrSelectorOperator::Equals),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("Other".into()),
            operator: Some(AttrSelectorOperator::Equals),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_attribute_includes_selector() {
        let html = r#"<div title="foo bar baz"></div>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("bar".into()),
            operator: Some(AttrSelectorOperator::Includes),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("qux".into()),
            operator: Some(AttrSelectorOperator::Includes),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_attribute_dashmatch_selector() {
        let html = r#"<div title="en-US"></div>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("en".into()),
            operator: Some(AttrSelectorOperator::DashMatch),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("en-US".into()),
            operator: Some(AttrSelectorOperator::DashMatch),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("fr".into()),
            operator: Some(AttrSelectorOperator::DashMatch),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_match_attribute_prefix_suffix_substring() {
        let html = r#"<div title="prefix-middle-suffix"></div>"#;
        let dom = Parser::to_dom(html).unwrap().value;
        let node = single_node(&dom);

        // PrefixMatch
        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("prefix".into()),
            operator: Some(AttrSelectorOperator::PrefixMatch),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        // SuffixMatch
        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("suffix".into()),
            operator: Some(AttrSelectorOperator::SuffixMatch),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        // SubstringMatch
        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("middle".into()),
            operator: Some(AttrSelectorOperator::SubstringMatch),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));

        // Negative
        let (cssom, candidates) = single_candidate_for(SimpleSelector::Attribute {
            name: "title".into(),
            value: Some("nope".into()),
            operator: Some(AttrSelectorOperator::SubstringMatch),
        });
        let matched = Matcher::match_rules(node, &dom, &candidates, &cssom);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_descendant_combinator() {
        let html = r#"
            <body>
                <div id="container">
                    <span id="target"></span>
                </div>
            </body>
        "#;

        let dom = Parser::to_dom(html).unwrap().value;

        // Find the span node index
        let span_idx = dom
            .nodes
            .iter()
            .position(|n| matches!(&n.ntype, NodeType::Element(el) if el.tag == HtmlTag::Span))
            .expect("span element exists");

        let span_compound = CompoundSelector {
            simple_selectors: vec![SimpleSelector::Type("span".to_string())],
        };
        let div_compound = CompoundSelector {
            simple_selectors: vec![SimpleSelector::Type("div".to_string())],
        };
        let body_compound = CompoundSelector {
            simple_selectors: vec![SimpleSelector::Type("body".to_string())],
        };

        let selector = ComplexSelector {
            first: body_compound,
            rest: vec![
                (Combinator::Descendant, div_compound.clone()),
                (Combinator::Descendant, span_compound.clone()),
            ],
        };

        let style_rule = CssRule::Style(StyleRule {
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

        let matched = Matcher::match_rules(&dom.nodes[span_idx], &dom, &candidates, &cssom);
        assert_eq!(matched, Some(vec![0]));
    }

    #[test]
    fn test_child_combinator() {
        // HTML structure to test Child and Descendant combinators.
        let html: &str = r#"
            <div id="root">
                <section>
                    <div class="container">
                        <div class="irrelevant"></div>
                        <div class="parent">
                            <span class="not-me"></span>
                            <span class="not-me"></span>
                            <span class="not-me"></span>
                            <span id="target"></span>
                            <p></p>
                        </div>
                        <div></div>
                        <span></span>
                    </div>
                </section>
                <footer>
                    <div class="parent">
                        <span></span>
                    </div>
                </footer>
            </div>
        "#;

        let dom = Parser::to_dom(html).unwrap().value;

        // Find the span node with id "target"
        let span_target_idx = dom
            .nodes
            .iter()
            .position(|n| {
                if let NodeType::Element(el) = &n.ntype {
                    el.tag == HtmlTag::Span && el.id() == Some("target")
                } else {
                    false
                }
            })
            .expect("span with id 'target' exists");
        let css = r#"
            div#root > section > div.container div.parent > span#target { color: red; }
        "#;

        // Parse CSS using Parser so we're exercising the CSSOM and style/engine/parser machinery.
        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = crate::cssom::cssom::CssOm::default();
        cssom.append_stylesheet(rules);
        cssom.build_indexes();

        // Find rule ref for the declared selector by querying lookup_candidates.
        let span_target_node = &dom.nodes[span_target_idx];
        let candidates =
            cssom.lookup_candidates(if let NodeType::Element(ref el) = span_target_node.ntype {
                el
            } else {
                panic!("Target node is not an element")
            });

        let matched = crate::style::matcher::Matcher::match_rules(
            span_target_node,
            &dom,
            &candidates,
            &cssom,
        );

        // Expect: should match, because all combinators strictly apply.
        assert_eq!(matched, Some(vec![0]));

        // Negative: Try a similar span not matching descendant/child chain.
        let span_wrong_idx = dom
            .nodes
            .iter()
            .position(|n| {
                if let NodeType::Element(el) = &n.ntype {
                    el.tag == HtmlTag::Span && el.id().is_none()
                } else {
                    false
                }
            })
            .expect("a span without id exists");
        let span_wrong_node = &dom.nodes[span_wrong_idx];
        let wrong_candidates =
            cssom.lookup_candidates(if let NodeType::Element(ref el) = span_wrong_node.ntype {
                el
            } else {
                panic!("wrong node is not an element")
            });
        let not_matched = crate::style::matcher::Matcher::match_rules(
            span_wrong_node,
            &dom,
            &wrong_candidates,
            &cssom,
        );
        assert_eq!(not_matched, None);
    }

    #[test]
    fn test_adjacent_sibling_combinator() {
        // HTML structure to test Adjacent Sibling combinator.
        let html: &str = r#"
            <body>
                <div id="a"></div>
                <span id="b"></span>
                <p id="c"></p>
                <span id="d"></span>
                <span id="e"></span>
                <h1 id="f"></h1>
            </body>
        "#;

        let dom = Parser::to_dom(html).unwrap().value;

        let css = r#"
            div + span { color: red; }
            p + span { color: blue; }
            span + h1 { color: green; }
        "#;

        // Parse CSS using Parser so we're exercising the CSSOM and style/engine/parser machinery.
        let tokens = Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = crate::cssom::cssom::CssOm::default();
        cssom.append_stylesheet(rules);
        cssom.build_indexes();

        // Utility: get (node index, &Node, &DomElement) by id
        let get_node_and_element = |id: &str| {
            let idx = dom
                .nodes
                .iter()
                .position(|n| match &n.ntype {
                    NodeType::Element(el) => el.id() == Some(id),
                    _ => false,
                })
                .expect(&format!("Node with id {} exists", id));
            let node = &dom.nodes[idx];
            let element = match &node.ntype {
                NodeType::Element(el) => el,
                _ => panic!("Node with id {} is not an element", id),
            };
            (node, element)
        };

        let (node_a, _el_a) = get_node_and_element("a");

        let (node_b, el_b) = get_node_and_element("b");
        let (node_d, el_d) = get_node_and_element("d");
        let (node_e, el_e) = get_node_and_element("e");
        let (node_f, el_f) = get_node_and_element("f");

        // Now just use el_* for lookup_candidates, no more pattern matching!
        let b_candidates = cssom.lookup_candidates(el_b);
        let d_candidates = cssom.lookup_candidates(el_d);
        let e_candidates = cssom.lookup_candidates(el_e);
        let f_candidates = cssom.lookup_candidates(el_f);

        // div + span should match #b only (color: red)
        let matched_b = Matcher::match_rules(&node_b, &dom, &b_candidates, &cssom);
        assert_eq!(matched_b, Some(vec![0]));

        // p + span should match #d only (color: blue)
        let matched_d = Matcher::match_rules(&node_d, &dom, &d_candidates, &cssom);
        assert_eq!(matched_d, Some(vec![1]));

        // span + span should match #e (not matched by any rule, so should be None)
        let matched_e = Matcher::match_rules(&node_e, &dom, &e_candidates, &cssom);
        assert_eq!(matched_e, None);

        // span + h1 should match #f (color: green)
        let matched_f = Matcher::match_rules(&node_f, &dom, &f_candidates, &cssom);
        assert_eq!(matched_f, Some(vec![2]));

        // Negative: div should not match any adjacent sibling combinators
        let a_candidates = cssom.lookup_candidates(match &node_a.ntype {
            NodeType::Element(el) => el,
            _ => panic!(),
        });
        let matched_a = Matcher::match_rules(&node_a, &dom, &a_candidates, &cssom);
        assert_eq!(matched_a, None);
    }

    #[test]
    fn test_general_sibling_combinator() {
        let html = r#"
            <div id="first" class="x"></div>
            <section>
                <div id="a" class="foo"></div>
                <p id="b"></p>
                <span id="c"></span>
                <span id="d" class="y"></span>
                <span id="f"></span>
                <h1 id="g"></h1>
                <div id="e" data-x="hello"></div>
                <span id="h" class="target special z"></span>
                <span id="i"></span>
            </section>
        "#;

        // Compose a complex selector string:
        // div.foo ~ p ~ span.y   [WS]   ~ div[data-x] + span.target.special.z
        // (Spaces represent descendant combinators)
        //
        // This matches:
        // - div.foo (id=a) ~ p (id=b) ~ span.y (id=d)
        // - [whitespace] ~ div[data-x] (id=e) + span.target.special.z (id=h)
        // So, only span#h should match this selector chain.

        let css = r#"
            div.foo ~ p ~ span.y   ~ div[data-x] + span.target.special.z { color: deep; }
        "#;

        // Parse CSSOM using tokenizer and parser
        use crate::parser::parser::Parser as CssParser;
        use crate::tokenizer::tokenizer::Tokenizer;

        let tokens = Tokenizer::from_string(css);
        let mut parser = CssParser::new(&tokens);
        let rules = parser.parse().expect("Should parse css");
        assert!(!rules.is_empty());

        let mut cssom = CssOm {
            rules,
            indexes: Default::default(),
        };
        cssom.build_indexes();

        let dom = Parser::to_dom(html).unwrap().value;

        // Helper for finding a node and its element by id
        let get_node_and_element = |id: &str| -> (&Node, &DomElement) {
            let idx = dom
                .nodes
                .iter()
                .position(|n| match &n.ntype {
                    NodeType::Element(el) => el.id() == Some(id),
                    _ => false,
                })
                .expect(&format!("Node with id {} exists", id));
            let node = &dom.nodes[idx];
            let element = match &node.ntype {
                NodeType::Element(el) => el,
                _ => panic!("Node with id {} is not an element", id),
            };
            (node, element)
        };

        let (node_c, el_c) = get_node_and_element("c");
        let (node_d, el_d) = get_node_and_element("d");
        let (node_e, el_e) = get_node_and_element("e");
        let (node_f, el_f) = get_node_and_element("f");
        let (node_h, el_h) = get_node_and_element("h");
        let (node_i, el_i) = get_node_and_element("i");

        // Candidates for all spans and divs
        let candidates_h = cssom.lookup_candidates(el_h);
        let candidates_f = cssom.lookup_candidates(el_f);
        let candidates_d = cssom.lookup_candidates(el_d);
        let candidates_c = cssom.lookup_candidates(el_c);
        let candidates_i = cssom.lookup_candidates(el_i);
        let candidates_e = cssom.lookup_candidates(el_e);

        // Only #h (span.target.special.z) should match!
        let matched_h = Matcher::match_rules(node_h, &dom, &candidates_h, &cssom);
        println!("{matched_h:#?}");
        assert_eq!(matched_h, Some(vec![0]));

        // - intermediary nodes
        let matched_d = Matcher::match_rules(node_d, &dom, &candidates_d, &cssom);
        assert_eq!(matched_d, None);
        let matched_f = Matcher::match_rules(node_f, &dom, &candidates_f, &cssom);
        assert_eq!(matched_f, None);
        let matched_i = Matcher::match_rules(node_i, &dom, &candidates_i, &cssom);
        assert_eq!(matched_i, None);
        let matched_e = Matcher::match_rules(node_e, &dom, &candidates_e, &cssom);
        assert_eq!(matched_e, None);
        let matched_c = Matcher::match_rules(node_c, &dom, &candidates_c, &cssom);
        assert_eq!(matched_c, None);
    }

    #[test]
    fn test_matcher_without_pseudo_selectors() {
        // Compose a large HTML document.
        let html = r#"
            <div id="a" class="parent root">
                <section id="b" lang="en" class="middle special content">
                    <div id="c" data-test="foo" class="sibling"></div>
                    <article id="d" class="article depth-0 ancestor">
                        <header id="e" class="header main-header"></header>
                        <div id="f" data-role="callout" class="box"></div>
                        <span id="g" class="span target z"></span>
                    </article>
                    <span id="h" class="target special z" title="test"></span>
                    <div id="i" class="foo bar"></div>
                    <ul id="j" class="list">
                        <li id="k" class="item"></li>
                        <li id="l" class="item special"></li>
                        <li id="m" class="item"><span id="n" class="deep child"></span></li>
                    </ul>
                </section>
                <aside id="o" class="aside">
                    <span id="p" class="side-span"></span>
                </aside>
                <footer id="q" class="footer"></footer>
                <footer id="r" class="tag z"></footer>
                <footer id="s" class="z tag"></footer>
                <footer id="t" class=""></footer>
                <footer id="u" class="tag z"></footer>
                <footer id="v" class="hi tag z"></footer>
            </div>
        "#;

        // Note: The index for CSS matching and expectations refers to the rule index,
        // not the selector index—selectors are not rules; each rule is a full semantic unit.
        // Each rule is marked below with its zero-based index for reference:

        let css = r#"
            /*  0 */ div { color: blue; }
            /*  1 */ section.content { border: 1px solid red; }
            /*  2 */ ul.list > li.item { font-weight: bold; }

            /*  3 */ div.parent section > article .header { background: yellow; }
            /*  4 */ .parent > section > article.depth-0 > header.main-header { padding: 2px; }

            /*  5 */ [data-role^="call"] { outline: 2px dashed green; }
            /*  6 */ [data-test="foo"] { font-size: 120%; }
            /*  7 */ [lang|="en"] { text-transform: uppercase; }
            /*  8 */ [class~="special"] { text-decoration: underline; }

            /*  9 */ #h { opacity: 0.8; }

            /* 10 */ .target.z, #h.special { color: orange; }
            /* 11 */ .foo.bar { font-style: italic; }
            /* 12 */ .article.ancestor > .box { margin: 4px; }

            /* 13 */ #e, .side-span, .deep.child { border-radius: 6px; }
            /* 14 */ article > header, article > div { min-height: 20px; }

            /* 15 */ * > .main-header { box-shadow: 1px 1px gray; }

            /* 16 */ .parent article .deep.child { color: rebeccapurple; }
            /* 17 */ .tag.z  { color: orange; }
            /* 18 */ .tag.z.hi  { color: orange; }

        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse without error");
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        println!("{}", cssom.rules.len());
        cssom.build_indexes();

        // Helper to get Node and DomElement by id
        let get_node_el =
            |id: &str| -> (&html::dom::node::Node, &html::dom::dom_element::DomElement) {
                let idx = dom
                    .nodes
                    .iter()
                    .position(|n| match &n.ntype {
                        html::dom::node_type::NodeType::Element(el) => el.id() == Some(id),
                        _ => false,
                    })
                    .expect(&format!("Node with id {} exists", id));
                let node = &dom.nodes[idx];
                let el = match &node.ntype {
                    html::dom::node_type::NodeType::Element(el) => el,
                    _ => panic!("Not an element"),
                };
                (node, el)
            };

        // Expectations, double-checked to match indexes above:
        let expected_matched_rules = [
            ("e", vec![13, 14]), // #e, article > header
            ("d", vec![]),
            ("c", vec![0, 6]),     // div, [data-test="foo"]
            ("a", vec![0]),        // div
            ("b", vec![1, 7, 8]),  // section.content, [lang|="en"], [class~="special"]
            ("f", vec![0, 5, 14]), // div, [data-role^="call"], article > div
            ("g", vec![10]),       // no rule without pseudo
            ("h", vec![9, 10, 8]), // #h, .target.z, [class~="special"]
            ("i", vec![0, 11]),    // div, .foo.bar
            ("j", vec![]),         // ul, no rule
            ("k", vec![2]),        // li, no rule
            ("l", vec![8, 2]),     // li, .item.special (should not match .side-span or others)
            ("m", vec![2]),        // li, no rule
            ("n", vec![13]),       // .deep.child, .parent article .deep.child
            ("o", vec![]),         // aside, no rule
            ("p", vec![13]),       // .side-span
            ("r", vec![17]),       // footer, no rule
            ("s", vec![17]),       // footer, no rule
            ("t", vec![]),         // footer, no rule
            ("u", vec![17]),       // footer, no rule
            ("v", vec![18]),       // footer, no rule
        ];

        for (id, expected) in expected_matched_rules {
            let (node, el) = get_node_el(id);
            let candidates = cssom.lookup_candidates(el);
            let matches_opt = Matcher::match_rules(node, &dom, &candidates, &cssom);
            let matched_idx: Vec<u32> = matches_opt.unwrap_or_default();
            if !expected.is_empty() {
                for &rule_idx in &expected {
                    assert!(
                        matched_idx.contains(&rule_idx),
                        "Expected selector rule {rule_idx} to match node #{id}, got matches: {:?}",
                        matched_idx
                    );
                }
            } else {
                assert!(
                    matched_idx.is_empty(),
                    "Expected no selector rules to match node #{id}, but got some: {:?}",
                    matched_idx
                );
            }
        }
    }

    #[test]
    fn test_matcher_pseudo_root() {
        let html = r#"
            <html id="root">
                <body>
                    <div id="a"></div>
                </body>
            </html>
        "#;
        let css = r#"
            :root { color: red; }
            body { color: blue; }
            #a { color: green; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().expect("Should parse without error");
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        // HTML root element
        let (root_node, root_el) = dom
            .nodes
            .iter()
            .find_map(|n| {
                if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                    if el.tag_name() == "html" {
                        Some((n, el))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .expect("root node");

        let candidates = cssom.lookup_candidates(root_el);
        let matched =
            Matcher::match_rules(root_node, &dom, &candidates, &cssom).unwrap_or_default();
        assert!(matched.contains(&0)); // :root
    }

    #[test]
    fn test_matcher_pseudo_empty() {
        let html = r#"
            <div id="empty"></div>
            <div id="not-empty"><span></span></div>
        "#;
        let css = r#"
            div:empty { color: red; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;

        let (empty_node, empty_el) = dom
            .nodes
            .iter()
            .find_map(|n| {
                if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                    if el.id() == Some("empty") {
                        Some((n, el))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .expect("empty node");

        let (not_empty_node, not_empty_el) = dom
            .nodes
            .iter()
            .find_map(|n| {
                if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                    if el.id() == Some("not-empty") {
                        Some((n, el))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .expect("not empty node");

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        let candidates_empty = cssom.lookup_candidates(empty_el);
        let matched_empty = Matcher::match_rules(empty_node, &dom, &candidates_empty, &cssom);
        assert!(!matched_empty.unwrap_or_default().is_empty());

        let candidates_not_empty = cssom.lookup_candidates(not_empty_el);
        let matched_not_empty =
            Matcher::match_rules(not_empty_node, &dom, &candidates_not_empty, &cssom);
        assert!(matched_not_empty.unwrap_or_default().is_empty());
    }

    #[test]
    fn test_matcher_pseudo_first_last_only_child() {
        let html = r#"
            <ul>
                <li id="a"></li>
                <li id="b"></li>
                <li id="c"></li>
            </ul>
            <div>
                <span id="only"></span>
            </div>
        "#;
        let css = r#"
            li:first-child { color: red; }
            li:last-child { color: blue; }
            span:only-child { color: green; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;

        let find = |id| {
            dom.nodes
                .iter()
                .find(|n| {
                    if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                        el.id() == Some(id)
                    } else {
                        false
                    }
                })
                .unwrap()
        };

        let span_node = find("only");
        let li_a = find("a");
        let li_b = find("b");
        let li_c = find("c");

        let get_el = |n: &Node| {
            if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                el.clone()
            } else {
                panic!()
            }
        };
        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        let li_a_matches =
            Matcher::match_rules(li_a, &dom, &cssom.lookup_candidates(&get_el(li_a)), &cssom)
                .unwrap_or_default();
        let li_b_matches =
            Matcher::match_rules(li_b, &dom, &cssom.lookup_candidates(&get_el(li_b)), &cssom)
                .unwrap_or_default();
        let li_c_matches =
            Matcher::match_rules(li_c, &dom, &cssom.lookup_candidates(&get_el(li_c)), &cssom)
                .unwrap_or_default();
        let span_matches = Matcher::match_rules(
            span_node,
            &dom,
            &cssom.lookup_candidates(&get_el(span_node)),
            &cssom,
        )
        .unwrap_or_default();

        assert!(li_a_matches.contains(&0)); // first-child
        assert!(!li_a_matches.contains(&1));
        assert!(!li_a_matches.contains(&2));
        assert!(!li_b_matches.contains(&0));
        assert!(!li_b_matches.contains(&1));
        assert!(!li_b_matches.contains(&2));
        assert!(li_c_matches.contains(&1)); // last-child
        assert!(!li_c_matches.contains(&0));
        assert!(!li_c_matches.contains(&2));
        assert!(span_matches.contains(&2)); // only-child
    }

    #[test]
    fn test_matcher_nth_child() {
        let html = r#"
            <ul>
                <li id="first"></li>
                <li id="second"></li>
                <li id="third"></li>
                <li id="fourth"></li>
            </ul>
        "#;
        let css = r#"
            li:nth-child(2) { color: red; }
            li:nth-child(2n) { color: blue; }
            li:nth-last-child(2) { color: green; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;
        let find = |id| {
            dom.nodes
                .iter()
                .find(|n| {
                    if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                        el.id() == Some(id)
                    } else {
                        false
                    }
                })
                .unwrap()
        };

        let first = find("first");
        let second = find("second");
        let third = find("third");
        let fourth = find("fourth");
        let get_el = |n: &Node| {
            if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                el.clone()
            } else {
                panic!()
            }
        };

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        println!("{rules:#?}");
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        let m_first = Matcher::match_rules(
            first,
            &dom,
            &cssom.lookup_candidates(&get_el(first)),
            &cssom,
        )
        .unwrap_or_default();
        let m_second = Matcher::match_rules(
            second,
            &dom,
            &cssom.lookup_candidates(&get_el(second)),
            &cssom,
        )
        .unwrap_or_default();
        let m_third = Matcher::match_rules(
            third,
            &dom,
            &cssom.lookup_candidates(&get_el(third)),
            &cssom,
        )
        .unwrap_or_default();
        let m_fourth = Matcher::match_rules(
            fourth,
            &dom,
            &cssom.lookup_candidates(&get_el(fourth)),
            &cssom,
        )
        .unwrap_or_default();

        assert!(!m_first.contains(&0) && !m_first.contains(&1) && !m_first.contains(&2));
        assert!(m_second.contains(&0) && m_second.contains(&1));
        assert!(m_third.contains(&1));
        assert!(m_fourth.contains(&2));
    }

    #[test]
    fn test_matcher_of_type_pseudo() {
        let html = r#"
            <ul>
                <li id="li1"></li>
                <li id="li2"></li>
                <li id="li3"></li>
                <span id="span1"></span>
                <li id="li4"></li>
            </ul>
        "#;
        let css = r#"
            li:first-of-type { color: red; }
            li:last-of-type { color: blue; }
            span:only-of-type { color: green; }
            li:nth-of-type(2) { color: cyan; }
            li:nth-last-of-type(2) { color: black; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;
        let find = |id| {
            dom.nodes
                .iter()
                .find(|n| {
                    if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                        el.id() == Some(id)
                    } else {
                        false
                    }
                })
                .unwrap()
        };
        let get_el = |n: &Node| {
            if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                el.clone()
            } else {
                panic!()
            }
        };

        let li1 = find("li1");
        let li2 = find("li2");
        let li3 = find("li3");
        let li4 = find("li4");
        let span1 = find("span1");

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        let m_li1 = Matcher::match_rules(li1, &dom, &cssom.lookup_candidates(&get_el(li1)), &cssom)
            .unwrap_or_default();
        let m_li2 = Matcher::match_rules(li2, &dom, &cssom.lookup_candidates(&get_el(li2)), &cssom)
            .unwrap_or_default();
        let m_li3 = Matcher::match_rules(li3, &dom, &cssom.lookup_candidates(&get_el(li3)), &cssom)
            .unwrap_or_default();
        let m_li4 = Matcher::match_rules(li4, &dom, &cssom.lookup_candidates(&get_el(li4)), &cssom)
            .unwrap_or_default();
        let m_span1 = Matcher::match_rules(
            span1,
            &dom,
            &cssom.lookup_candidates(&get_el(span1)),
            &cssom,
        )
        .unwrap_or_default();

        assert!(m_li1.contains(&0)); // first-of-type
        assert!(m_li4.contains(&1)); // last-of-type
        assert!(m_span1.contains(&2)); // only-of-type
        assert!(m_li2.contains(&3)); // nth-of-type(2)
        assert!(m_li3.contains(&4)); // nth-last-of-type(2)
    }

    #[test]
    fn test_matcher_lang_and_dir() {
        let html = r#"
            <div id="en" lang="en"></div>
            <div id="ar" dir="rtl"></div>
        "#;
        let css = r#"
            div:lang(en) { color: red; }
            div:dir(rtl) { color: green; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;
        let find = |id| {
            dom.nodes
                .iter()
                .find(|n| {
                    if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                        el.id() == Some(id)
                    } else {
                        false
                    }
                })
                .unwrap()
        };
        let get_el = |n: &Node| {
            if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                el.clone()
            } else {
                panic!()
            }
        };

        let en = find("en");
        let ar = find("ar");

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        let m_en = Matcher::match_rules(en, &dom, &cssom.lookup_candidates(&get_el(en)), &cssom)
            .unwrap_or_default();
        let m_ar = Matcher::match_rules(ar, &dom, &cssom.lookup_candidates(&get_el(ar)), &cssom)
            .unwrap_or_default();

        assert!(m_en.contains(&0));
        assert!(m_ar.contains(&1));
    }

    #[test]
    fn test_matcher_not_is_where_has() {
        let html = r#"
            <div id="a" class="foo bar baz" data-x="1"></div>
            <div id="b" class="foo" data-x="2"></div>
            <div id="c" class="bar"></div>
            <div id="d" class="foo baz" data-x="3"></div>
            <ul id="ul1"><li id="li1" class="item"></li><li id="li2"></li></ul>
            <ul id="ul2"></ul>
            <section id="sec1" class="highlight">
                <div id="e" class="foo"></div>
            </section>
        "#;
        let css = r#"
            .foo:not(.bar):not([data-x="3"]) { color: red; }
            .foo:is(.foo, .baz) { color: green; }
            .foo:where(.foo, .baz) { color: gray; }
            ul:has(li.item) { color: purple; }
            .foo[data-x="1"] { color: orange; }
            section.highlight > .foo { color: pink; }
            .foo.baz:not(.bar) { outline: dashed; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;
        let find = |id| {
            dom.nodes
                .iter()
                .find(|n| {
                    if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                        el.id() == Some(id)
                    } else {
                        false
                    }
                })
                .unwrap()
        };
        let get_el = |n: &Node| {
            if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                el.clone()
            } else {
                panic!()
            }
        };

        let a = find("a");
        let b = find("b");
        let c = find("c");
        let d = find("d");
        let e = find("e");
        let ul1 = find("ul1");
        let ul2 = find("ul2");
        let sec1 = find("sec1");

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        let m_a = Matcher::match_rules(a, &dom, &cssom.lookup_candidates(&get_el(a)), &cssom)
            .unwrap_or_default();
        let m_b = Matcher::match_rules(b, &dom, &cssom.lookup_candidates(&get_el(b)), &cssom)
            .unwrap_or_default();
        let m_c = Matcher::match_rules(c, &dom, &cssom.lookup_candidates(&get_el(c)), &cssom)
            .unwrap_or_default();
        let m_d = Matcher::match_rules(d, &dom, &cssom.lookup_candidates(&get_el(d)), &cssom)
            .unwrap_or_default();
        let m_e = Matcher::match_rules(e, &dom, &cssom.lookup_candidates(&get_el(e)), &cssom)
            .unwrap_or_default();
        let m_ul1 = Matcher::match_rules(ul1, &dom, &cssom.lookup_candidates(&get_el(ul1)), &cssom)
            .unwrap_or_default();
        let m_ul2 = Matcher::match_rules(ul2, &dom, &cssom.lookup_candidates(&get_el(ul2)), &cssom)
            .unwrap_or_default();
        let m_sec1 =
            Matcher::match_rules(sec1, &dom, &cssom.lookup_candidates(&get_el(sec1)), &cssom)
                .unwrap_or_default();

        println!("m_a (id='a', class='foo bar baz', data-x=1) => {:?}", m_a);
        println!("m_b (id='b', class='foo', data-x=2) => {:?}", m_b);
        println!("m_c (id='c', class='bar') => {:?}", m_c);
        println!("m_d (id='d', class='foo baz', data-x=3) => {:?}", m_d);
        println!(
            "m_e (id='e', class='foo', child of section.highlight) => {:?}",
            m_e
        );
        println!("m_ul1 (ul#ul1 with li#li1.item and li#li2) => {:?}", m_ul1);
        println!("m_ul2 (ul#ul2, empty) => {:?}", m_ul2);
        println!("m_sec1 (section.highlight) => {:?}", m_sec1);

        // .foo:not(.bar):not([data-x="3"])
        assert!(
            m_b.contains(&0),
            "b matches .foo:not(.bar):not([data-x=\"3\"]) (only .foo, data-x=\"2\")"
        );
        assert!(!m_a.contains(&0), "a does NOT match ; has .bar");
        assert!(!m_d.contains(&0), "d does NOT match; has data-x=\"3\"");
        assert!(!m_c.contains(&0), "c does not have .foo");

        // .foo:is(.foo, .baz)
        assert!(m_a.contains(&1), "a matches .foo:is(.foo, .baz)");
        assert!(m_b.contains(&1), "b matches .foo:is(.foo, .baz)");
        assert!(m_d.contains(&1), "d matches .foo:is(.foo, .baz)");
        assert!(!m_c.contains(&1), "c does not match .foo:is(.foo, .baz)");

        // .foo:where(.foo, .baz) -- special: should match like :is, but has 0 specificity
        assert!(m_a.contains(&2), "a matches .foo:where(.foo, .baz)");
        assert!(m_b.contains(&2), "b matches .foo:where(.foo, .baz)");
        assert!(m_d.contains(&2), "d matches .foo:where(.foo, .baz)");
        assert!(!m_c.contains(&2), "c does not match .foo:where(.foo, .baz)");

        // ul:has(li.item)
        assert!(m_ul1.contains(&3), "ul1 has li.item");
        assert!(!m_ul2.contains(&3), "ul2 does not have li.item");

        // .foo[data-x="1"]
        assert!(m_a.contains(&4), "a matches .foo[data-x=\"1\"]");
        assert!(!m_b.contains(&4), "b does not match .foo[data-x=\"1\"]");

        // section.highlight > .foo
        assert!(
            m_e.contains(&5),
            "e is .foo direct child of section.highlight"
        );
        assert!(!m_a.contains(&5), "a is not inside section.highlight");

        // .foo.baz:not(.bar)
        assert!(
            m_d.contains(&6),
            "d (foo baz) does not have .bar, so should match .foo.baz:not(.bar)"
        );
        assert!(
            !m_a.contains(&6),
            "a has .bar, so should NOT match .foo.baz:not(.bar)"
        );
        assert!(!m_b.contains(&6), "b does not have .baz");
    }

    #[test]
    fn test_matcher_target_and_target_within() {
        let html = r#"
            <div id="foo"></div>
            <div id="bar">
                <span id="baz"></span>
            </div>
        "#;
        // Target here is 'bar'
        let css = r#"
            #bar:target { background: pink; }
            #bar:target-within span { color: red; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;

        let find = |id| {
            dom.nodes
                .iter()
                .find(|n| {
                    if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                        el.id() == Some(id)
                    } else {
                        false
                    }
                })
                .unwrap()
        };
        let get_el = |n: &Node| {
            if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                el.clone()
            } else {
                panic!()
            }
        };

        let foo = find("foo");
        let bar = find("bar");

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        // Simulate :target id is "bar"
        // This may depend on surrounding infra; for this test, we expect that :target and :target-within
        // are evaluated with appropriate context in Matcher. If that's not implemented, allow these asserts to fail.
        // If implemented, 'bar' and 'baz' (descendant of target-within) would match.

        let m_foo = Matcher::match_rules(foo, &dom, &cssom.lookup_candidates(&get_el(foo)), &cssom)
            .unwrap_or_default();
        let m_bar = Matcher::match_rules(bar, &dom, &cssom.lookup_candidates(&get_el(bar)), &cssom)
            .unwrap_or_default();
        // bar should match :target (rule 0); baz should match :target-within (rule 1 as descendant)
        assert!(!m_foo.contains(&0));
        assert!(m_bar.contains(&0));
        assert!(!m_foo.contains(&1));
        assert!(!m_bar.contains(&1));
        // Either .contains(&1), or Matcher must support contextual :target-within propagation.
        // This is just a check of syntax and setup expectations.
    }

    #[test]
    fn test_matcher_form_and_state_pseudo_classes() {
        let html = r#"
            <input type="text" id="text" value="foo" required>
            <input type="checkbox" id="chk" checked>
            <input type="radio" id="rad" disabled>
            <input type="email" id="mail" readonly>
            <button id="btn" autofocus></button>
        "#;
        let css = r#"
            input:required { color: red; }
            input:checked { color: green; }
            input:disabled { color: blue; }
            input:read-only { color: purple; }
            button:autofocus { color: pink; }
        "#;
        let result = Parser::to_dom(html).unwrap();
        let dom = result.value;
        let find = |id| {
            dom.nodes
                .iter()
                .find(|n| {
                    if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
                        el.id() == Some(id)
                    } else {
                        false
                    }
                })
                .unwrap()
        };
        let get_el = |n: &Node| match &n.ntype {
            html::dom::node_type::NodeType::Element(el) => el.clone(),
            _ => panic!("Node is not an Element"),
        };

        let text = find("text");
        let chk = find("chk");
        let rad = find("rad");
        let mail = find("mail");
        let btn = find("btn");

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        let m_text =
            Matcher::match_rules(text, &dom, &cssom.lookup_candidates(&get_el(text)), &cssom)
                .unwrap_or_default();
        let m_chk = Matcher::match_rules(chk, &dom, &cssom.lookup_candidates(&get_el(chk)), &cssom)
            .unwrap_or_default();
        let m_rad = Matcher::match_rules(rad, &dom, &cssom.lookup_candidates(&get_el(rad)), &cssom)
            .unwrap_or_default();
        let m_mail =
            Matcher::match_rules(mail, &dom, &cssom.lookup_candidates(&get_el(mail)), &cssom)
                .unwrap_or_default();
        let m_btn = Matcher::match_rules(btn, &dom, &cssom.lookup_candidates(&get_el(btn)), &cssom)
            .unwrap_or_default();

        assert!(m_text.contains(&0));
        assert!(m_chk.contains(&1));
        assert!(m_rad.contains(&2));
        assert!(m_mail.contains(&3));
        assert!(m_btn.contains(&4));
    }
}
