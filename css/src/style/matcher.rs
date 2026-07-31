use core::panic;
use std::{collections::HashSet, println};

use html::{
    dom::{dom::Dom, dom_element::DomElement, node::Node, node_type::NodeType},
    tokenizer::html_tags::HtmlTag,
};

use crate::cssom::{
    css_indexes::RuleRef,
    cssom::CssOm,
    rule::CssRule,
    selector::{
        AttrSelectorOperator, Combinator, ComplexSelector, CompoundSelector, SimpleSelector,
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

            println!("{rule:#?}");
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
                if Self::match_compound(node, complex_selector.rightmost()) {
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
            return Self::match_compound(node, &complex_selector.first);
        }

        let (ref combinator, ref right_compound) = complex_selector.rest[rest_index as usize];

        if !Self::match_compound(node, right_compound) {
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
                    if Self::match_compound(parent, left_compound) {
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
                    if Self::match_compound(parent, left_compound)
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
                    if Self::match_compound(sibling, left_compound) {
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
                    if Self::match_compound(sibling, left_compound) {
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

    fn match_compound(node: &Node, compound_selector: &CompoundSelector) -> bool {
        match &node.ntype {
            NodeType::Element(element) => compound_selector
                .simple_selectors
                .iter()
                .all(|selector| Self::match_simple(element, selector)),
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
            } => element.attributes.contains(name).map_or(false, |attr_val| {
                Self::match_attribute_operators(value, operator, attr_val)
            }),
            SimpleSelector::Universal => true,
            _ => false, // TODO: #IDEA 4
        }
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
    fn test_deeply_nested_combinators_selector_matching() {
        // HTML has many tags to allow for combinator chains
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
    fn test_matcher_with_huge_complex_html_and_css_selectors() {
        // Compose a very large HTML document with nested, sibling, and attribute variations.
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
            </div>
        "#;

        // Very complex CSS with many selectors, combinators, pseudo-classes, attributes, and grouping.
        let css = r#"
            /* Type selectors */
            div { color: blue; }
            section.content { border: 1px solid red; }
            ul.list > li.item { font-weight: bold; }
            
            /* Descendant + child selectors */
            div.parent section > article .header { background: yellow; }
            .parent > section > article.depth-0 > header.main-header { padding: 2px; }
            
            /* Attribute selectors */
            [data-test="foo"] { font-size: 120%; }
            [data-role^="call"] { outline: 2px dashed green; }
            [lang|="en"] { text-transform: uppercase; }
            [class~="special"] { text-decoration: underline; }
            
            /* ID selector */
            #h { opacity: 0.8; }

            /* Class and ID combinators */
            .target.z, #h.special { color: orange; }
            .foo.bar { font-style: italic; }
            .article.ancestor > .box { margin: 4px; }

            /* Pseudo-classes */
            ul.list > li:first-child { border-top: 2px solid magenta; }
            ul.list > li:last-child { border-bottom: 2px solid cyan; }
            .item.special { background: pink; }

            /* Complex: grouped selectors */
            #e, .side-span, .deep.child { border-radius: 6px; }
            article > header, article > div { min-height: 20px; }

            /* Pseudo-class argument */
            :is(.span, .target.special, .foo.bar) { letter-spacing: 1px; }
            :where(.item, .aside) { user-select: none; }

            /* Attribute and pseudo-combinators */
            span[title]:not(.side-span):not([id="g"]) { border: 2px dotted brown; }

            /* Universal and descendant + adjacent */
            * > .main-header { box-shadow: 1px 1px gray; }
            .item + .item { letter-spacing: 2px; }

            /* Deep nesting, descendant, and leftovers */
            .parent article .deep.child { color: rebeccapurple; }
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

        // Define expectations (vector of index in cssom.rules matched for each node)
        // Note: rules are ordered as in CSS above
        let expected_matched_rules = [
            // Each entry is a tuple: (id, Vec<rule_indices_that_should_match>)
            ("b", vec![1, 7, 8, 9, 13]), // section.content, [lang|="en"], [class~="special"], .target.z, :where(.item, .aside)
            ("a", vec![0]),              // div { color: blue; }
            ("c", vec![0, 6]),           // div, [data-test="foo"]
            ("d", vec![0, 10]), // div, .article.ancestor > .box doesn't apply, :where(.item, .aside) via ancestor is possible
            ("e", vec![12, 15, 16, 19]), // #e, article > header, * > .main-header, #e (again since grouped), maybe more
            ("f", vec![0, 7, 11, 13, 17]), // div, [data-role^=call], article > div, .article.ancestor > .box, article > div (again, via grouping)
            ("g", vec![14, 18]), // .span.target.z (.is, .span), .item + .item (if applicable)
            ("h", vec![0, 8, 9, 10, 13, 17, 18, 21]), // div, [class~="special"], [lang|="en"], #h, .target.z, :is(), span[title]:not(.side-span):not([id="g"])
            ("i", vec![0, 12]),                       // div, .foo.bar
            ("j", vec![]),                            // ul.list (no ul{} rule)
            ("k", vec![2, 11]),                       // ul.list>li.item, ul.list > li:first-child
            ("l", vec![2, 11, 12]), // ul.list>li.item, .item.special, ul.list > li:last-child
            ("m", vec![2, 12]),     // ul.list>li.item, ul.list > li:last-child (if last)
            ("n", vec![20]),        // .deep.child
            ("o", vec![13]),        // aside, :where(.item, .aside)
            ("p", vec![12]),        // .side-span
            ("q", vec![]),          // footer.footer (no explicit rule)
        ];

        for (id, expected) in expected_matched_rules {
            let (node, el) = get_node_el(id);
            let candidates = cssom.lookup_candidates(el);
            let matches_opt = Matcher::match_rules(node, &dom, &candidates, &cssom);
            println!("{candidates:#?}");
            println!("{node:#?}");

            // The matcher returns Some(indices) if there are matches, None otherwise
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

        // Some concrete spot-checks for pseudo-classes and grouped selectors.
        // #n is span.deep.child
        let (node_n, el_n) = get_node_el("n");
        let candidates_n = cssom.lookup_candidates(el_n);
        let matched_n =
            Matcher::match_rules(node_n, &dom, &candidates_n, &cssom).unwrap_or_default();
        assert!(
            matched_n.contains(&20),
            "Expected rule 20 (.deep.child) to match span#n, but got: {:?}",
            matched_n
        );

        // #h should match [class~="special"], [lang|="en"], #h, .target.z, :is(), span[title]:not(.side-span):not([id="g"])
        let (node_h, el_h) = get_node_el("h");
        let candidates_h = cssom.lookup_candidates(el_h);
        let matched_h =
            Matcher::match_rules(node_h, &dom, &candidates_h, &cssom).unwrap_or_default();
        for expect in &[8u32, 9, 10, 13, 17, 18, 21] {
            assert!(
                matched_h.contains(expect),
                "Expected rule {} to match #h, got: {:?}",
                expect,
                matched_h
            );
        }

        // #f is div[data-role="callout"] under .article.ancestor
        let (node_f, el_f) = get_node_el("f");
        let candidates_f = cssom.lookup_candidates(el_f);
        let matched_f =
            Matcher::match_rules(node_f, &dom, &candidates_f, &cssom).unwrap_or_default();
        assert!(
            matched_f.contains(&7) && matched_f.contains(&17),
            "Expected rule 7 ([data-role^=call]) and 17 (article > div) to match #f, got: {:?}",
            matched_f
        );

        // #l is li.item.special – check for item, special, position in group
        let (node_l, el_l) = get_node_el("l");
        let candidates_l = cssom.lookup_candidates(el_l);
        let matched_l =
            Matcher::match_rules(node_l, &dom, &candidates_l, &cssom).unwrap_or_default();
        for expect in &[2u32, 11, 12] {
            assert!(
                matched_l.contains(expect),
                "Expected rule {} for #l, got: {:?}",
                expect,
                matched_l
            );
        }
    }
}
