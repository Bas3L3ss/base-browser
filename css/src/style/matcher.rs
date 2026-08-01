use std::{collections::HashSet, format, matches};

use html::{
    dom::{dom::Dom, dom_element::DomElement, node::Node, node_type::NodeType},
    tokenizer::html_tags::HtmlTag,
};

use crate::cssom::{
    component_value::ComponentValue,
    css_indexes::RuleRef,
    cssom::CssOm,
    rule::CssRule,
    selector::{
        AttrSelectorOperator, Combinator, ComplexSelector, CompoundSelector, PseudoArg,
        SimpleSelector,
    },
};
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    AnPlusB(i32, i32), // (a, b)
    Number(i32),
    Ident(String),
}

pub struct Matcher {}
impl Matcher {
    pub(crate) fn match_rules(
        node: &Node,
        dom: &Dom,
        candidates: &[RuleRef],
        cssom: &CssOm,
    ) -> Option<Vec<u32>> {
        let mut matcher = Matcher {};

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
                if matcher.match_compound(node, complex_selector.rightmost(), dom) {
                    matched.insert(candidate.rule_index);
                }
            } else {
                if matcher.matches_complex_selector(
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

    pub fn matches_complex_selector(
        &mut self,
        node: &Node,
        dom: &Dom,
        complex_selector: &ComplexSelector,
        rest_index: isize,
    ) -> bool {
        if rest_index < 0 {
            return self.match_compound(node, &complex_selector.first, dom);
        }

        let (ref combinator, ref right_compound) = complex_selector.rest[rest_index as usize];

        if !self.match_compound(node, right_compound, dom) {
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
                    if self.match_compound(parent, left_compound, dom) {
                        return self.matches_complex_selector(
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
                    if self.match_compound(parent, left_compound, dom)
                        && parent.children.contains(&node.id)
                    {
                        return self.matches_complex_selector(
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
                    if self.match_compound(sibling, left_compound, dom) {
                        return self.matches_complex_selector(
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
                    if self.match_compound(sibling, left_compound, dom) {
                        return self.matches_complex_selector(
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

    fn match_compound(
        &mut self,
        node: &Node,
        compound_selector: &CompoundSelector,
        dom: &Dom,
    ) -> bool {
        let matched = match &node.ntype {
            NodeType::Element(element) => compound_selector
                .simple_selectors
                .iter()
                .all(|selector| self.match_simple(element, selector, node, dom)),
            _ => false,
        };
        matched
    }

    fn match_simple(
        &mut self,
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
                self.match_attribute_operators(value, operator, attr_val)
            }),
            SimpleSelector::Universal => true,
            SimpleSelector::PseudoClass(name, pseudo_args) => {
                self.match_pseudo_class(name, pseudo_args.as_ref(), element, node, dom)
            }
            SimpleSelector::PseudoElement(_) => true,
        }
    }

    fn match_pseudo_class(
        &mut self,
        name: &str,
        pseudo_args: Option<&PseudoArg>,
        element: &DomElement,
        node: &Node,
        dom: &Dom,
    ) -> bool {
        match name {
            "has" => {
                if let Some(PseudoArg::SelectorList(list)) = pseudo_args {
                    list.iter().any(|complex_selector| {
                        self.match_has_selector(node, dom, complex_selector)
                    })
                } else {
                    false
                }
            }

            "not" => {
                if let Some(PseudoArg::SelectorList(selectors)) = pseudo_args {
                    selectors.iter().all(|complex_selector| {
                        if complex_selector.rest.is_empty() {
                            !self.match_compound(node, complex_selector.rightmost(), dom)
                        } else {
                            !self.matches_complex_selector(
                                node,
                                dom,
                                complex_selector,
                                (complex_selector.rest.len() - 1) as isize,
                            )
                        }
                    })
                } else {
                    false
                }
            }
            "is" | "where" => {
                if let Some(PseudoArg::SelectorList(selectors)) = pseudo_args {
                    selectors.iter().any(|complex_selector| {
                        if complex_selector.rest.is_empty() {
                            self.match_compound(node, complex_selector.rightmost(), dom)
                        } else {
                            self.matches_complex_selector(
                                node,
                                dom,
                                complex_selector,
                                (complex_selector.rest.len() - 1) as isize,
                            )
                        }
                    })
                } else {
                    false
                }
            }
            "lang" => {
                let actual_lang = {
                    let mut cur_node = Some(node);
                    let mut found_lang = None;
                    while let Some(n) = cur_node {
                        if let NodeType::Element(el) = &n.ntype {
                            if let Some(lang_val) = &el.attributes.lang {
                                if !lang_val.trim().is_empty() {
                                    found_lang = Some(lang_val.as_str());
                                    break;
                                }
                            }
                        }
                        cur_node = n.parent.and_then(|pidx| dom.nodes.get(pidx as usize));
                    }
                    found_lang
                };

                if let Some(actual_lang) = actual_lang {
                    if let Some(PseudoArg::Values(args)) = pseudo_args {
                        // There may be multiple Ident values in :lang(...), match if any one matches.
                        for value in args {
                            if let ComponentValue::Ident(wanted) = value {
                                if self.css_lang_matches(actual_lang, wanted) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            }
            "dir" => {
                let mut cur_node = Some(node);
                let mut actual_dir = None;

                while let Some(n) = cur_node {
                    if let NodeType::Element(el) = &n.ntype {
                        if let Some(dir) = &el.attributes.dir {
                            actual_dir = Some(dir.as_str());
                            break;
                        }
                    }
                    cur_node = n.parent.and_then(|id| dom.nodes.get(id as usize));
                }

                if let Some(PseudoArg::Values(args)) = pseudo_args {
                    if args.len() != 1 {
                        return false;
                    }
                    if let Some(ComponentValue::Ident(wanted)) = args.first() {
                        if wanted != "rtl" && wanted != "ltr" {
                            return false;
                        }

                        match actual_dir {
                            Some("rtl") | Some("ltr") => {
                                if let Some(actual_dir_str) = actual_dir.as_ref() {
                                    if actual_dir_str.eq_ignore_ascii_case(wanted) {
                                        return true;
                                    }
                                }
                            }
                            Some("auto") => {
                                if wanted.eq_ignore_ascii_case("ltr") {
                                    return true;
                                }
                            }
                            _ => {
                                return false;
                            }
                        }
                    }
                }

                false
            }

            "target" => false,        // TODO
            "target-within" => false, // TODO

            "root" => matches!(element.tag, HtmlTag::Html),

            "required" => element.attributes.required.unwrap_or(false),

            "checked" => match element.tag {
                HtmlTag::Input => {
                    let t = element.attributes.type_attr.as_deref().unwrap_or("");
                    matches!(t, "checkbox" | "radio") && element.attributes.checked.unwrap_or(false)
                }
                HtmlTag::Option => element.attributes.selected.unwrap_or(false),
                _ => false,
            },

            "disabled" => element.attributes.disabled.unwrap_or(false),

            "enabled" => match element.tag {
                HtmlTag::Input
                | HtmlTag::Button
                | HtmlTag::Select
                | HtmlTag::Textarea
                | HtmlTag::Option
                | HtmlTag::Optgroup
                | HtmlTag::Fieldset => !element.attributes.disabled.unwrap_or(false),
                _ => false,
            },

            "optional" => match element.tag {
                HtmlTag::Input | HtmlTag::Select | HtmlTag::Textarea => {
                    !element.attributes.required.unwrap_or(false)
                }
                _ => false,
            },

            "placeholder-shown" => match element.tag {
                HtmlTag::Input | HtmlTag::Textarea => {
                    element.attributes.placeholder.is_some()
                        && element
                            .attributes
                            .value
                            .as_deref()
                            .map_or(true, |v| v.is_empty())
                }
                _ => false,
            },

            "indeterminate" => false, // TODO: Implement when reactive JS state is available

            "read-only" => match element.tag {
                HtmlTag::Input | HtmlTag::Textarea => element.attributes.readonly.unwrap_or(false),
                _ => false,
            },

            "read-write" => match element.tag {
                HtmlTag::Input | HtmlTag::Textarea => !element.attributes.readonly.unwrap_or(false),
                _ => false,
            },

            // Spec: The :default pseudo-class matches the default button or choice in a form, e.g., checked radio, checked checkbox, selected <option>.
            // This requires form state. Not implemented for now.
            "default" => false, // TODO: Implement properly with form controls and state

            // TODO: Properly implement valid/invalid/in-range/out-of-range once form validation is available
            "valid" => false,   // TODO: Implement with real constraint/form state
            "invalid" => false, // TODO: Implement with real constraint/form state

            // Numeric range support for <input type=number> and <input type=range>
            // *min* and *max* should come from custom (element.attributes.custom), not from struct fields
            "in-range" => match element.tag {
                HtmlTag::Input => {
                    let input_type = element
                        .attributes
                        .type_attr
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if input_type == "number" || input_type == "range" {
                        if let Some(value_str) = element.attributes.value.as_ref() {
                            if let Ok(value) = value_str.parse::<f64>() {
                                let min = element
                                    .attributes
                                    .custom
                                    .as_ref()
                                    .and_then(|c| c.get("min"))
                                    .and_then(|s| s.parse::<f64>().ok());
                                let max = element
                                    .attributes
                                    .custom
                                    .as_ref()
                                    .and_then(|c| c.get("max"))
                                    .and_then(|s| s.parse::<f64>().ok());
                                if let Some(min) = min {
                                    if value < min {
                                        return false;
                                    }
                                }
                                if let Some(max) = max {
                                    if value > max {
                                        return false;
                                    }
                                }
                                return true;
                            }
                        }
                    }
                    false
                }
                _ => false,
            },
            "out-of-range" => match element.tag {
                HtmlTag::Input => {
                    let input_type = element
                        .attributes
                        .type_attr
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if input_type == "number" || input_type == "range" {
                        if let Some(value_str) = element.attributes.value.as_ref() {
                            if let Ok(value) = value_str.parse::<f64>() {
                                let min = element
                                    .attributes
                                    .custom
                                    .as_ref()
                                    .and_then(|c| c.get("min"))
                                    .and_then(|s| s.parse::<f64>().ok());
                                let max = element
                                    .attributes
                                    .custom
                                    .as_ref()
                                    .and_then(|c| c.get("max"))
                                    .and_then(|s| s.parse::<f64>().ok());
                                if let Some(min) = min {
                                    if value < min {
                                        return true;
                                    }
                                }
                                if let Some(max) = max {
                                    if value > max {
                                        return true;
                                    }
                                }
                                return false;
                            }
                        }
                    }
                    false
                }
                _ => false,
            },

            "empty" => node.children.iter().all(|&child_id| {
                if let Some(child_node) = dom.nodes.get(child_id as usize) {
                    match &child_node.ntype {
                        html::dom::node_type::NodeType::Comment(_) => true, // Ignore comments (do not disqualify :empty)
                        html::dom::node_type::NodeType::Text(_) => false, // Any text node, including whitespace, means NOT empty
                        html::dom::node_type::NodeType::Element(_) => false,
                        html::dom::node_type::NodeType::Document => false,
                        html::dom::node_type::NodeType::Doctype(_) => false,
                    }
                } else {
                    true
                }
            }),

            "first-child" => matches!(node.nth_element_sibling_index(&dom.nodes), Some(1)),
            "last-child" => matches!(node.nth_element_sibling_reverse_index(&dom.nodes), Some(1)),
            "only-child" => matches!(node.element_sibling_count(&dom.nodes), Some(1)),
            "nth-child" => {
                let args = match pseudo_args {
                    Some(PseudoArg::Values(arg)) => match self.parse_an_plus_b(arg.as_slice()) {
                        Some(args) => args,
                        None => return false,
                    },
                    _ => return false,
                };

                let self_index = match node.nth_element_sibling_index(&dom.nodes) {
                    Some(idx) => idx as i32,
                    None => return false,
                };

                self.is_nth_child_match(self_index, args.0, args.1)
            }
            "nth-last-child" => {
                let args = match pseudo_args {
                    Some(PseudoArg::Values(arg)) => match self.parse_an_plus_b(arg.as_slice()) {
                        Some(args) => args,
                        None => return false,
                    },
                    _ => return false,
                };

                let self_reverse_index = match node.nth_element_sibling_reverse_index(&dom.nodes) {
                    Some(idx) => idx as i32,
                    None => return false,
                };

                self.is_nth_child_match(self_reverse_index, args.0, args.1)
            }
            "nth-of-type" => {
                let args = match pseudo_args {
                    Some(PseudoArg::Values(arg)) => match self.parse_an_plus_b(arg.as_slice()) {
                        Some(args) => args,
                        None => return false,
                    },
                    _ => return false,
                };

                let self_index = match node.nth_same_tag_sibling_index(&dom.nodes) {
                    Some(idx) => idx as i32,
                    None => return false,
                };

                self.is_nth_child_match(self_index, args.0, args.1)
            }
            "nth-last-of-type" => {
                let args = match pseudo_args {
                    Some(PseudoArg::Values(arg)) => match self.parse_an_plus_b(arg.as_slice()) {
                        Some(args) => args,
                        None => return false,
                    },
                    _ => return false,
                };

                let self_reverse_index = match node.nth_same_tag_sibling_reverse_index(&dom.nodes) {
                    Some(idx) => idx as i32,
                    None => return false,
                };
                self.is_nth_child_match(self_reverse_index, args.0, args.1)
            }
            "first-of-type" => matches!(node.nth_same_tag_sibling_index(&dom.nodes), Some(1)),
            "last-of-type" => {
                matches!(node.nth_same_tag_sibling_reverse_index(&dom.nodes), Some(1))
            }
            "only-of-type" => matches!(node.same_tag_sibling_count(&dom.nodes), Some(1)),

            _ => false,
        }
    }

    /// Matches a :has() selector starting from a node, using a clear API instead of magic numbers.
    fn match_has_selector(
        &mut self,
        node: &Node,
        dom: &Dom,
        complex_selector: &ComplexSelector,
    ) -> bool {
        // If there's an explicit "first" segment, start by seeking a descendant that matches the first compound.
        if !complex_selector.first.is_empty() {
            self.relative_seek_first(node, dom, complex_selector)
        } else if !complex_selector.rest.is_empty() {
            // Otherwise, begin matching with the first combinator and right compound.
            self.match_relative_selector(node, dom, complex_selector, 0)
        } else {
            false
        }
    }

    /// Recursively seek any descendant (depth first) where leftmost compound matches, then attempt rest.
    fn relative_seek_first(
        &mut self,
        node: &Node,
        dom: &Dom,
        complex_selector: &ComplexSelector,
    ) -> bool {
        for &child_idx in &node.children {
            let child = &dom.nodes[child_idx];
            if self.match_compound(child, &complex_selector.first, dom) {
                // On a matching descendant, attempt to match the remainder of the selector from here.
                if self.match_relative_selector(child, dom, complex_selector, 0) {
                    return true;
                }
            }
            // Continue seeking on all descendants.
            if self.relative_seek_first(child, dom, complex_selector) {
                return true;
            }
        }
        false
    }

    /// Matches a complex relative selector starting from the current node and the specified phase of matching.
    /// Replaces magic -1/0 rest_index, uses phase (SeekingFirst or Matching(index)).
    fn match_relative_selector(
        &mut self,
        node: &Node,
        dom: &Dom,
        complex_selector: &ComplexSelector,
        rest_idx: usize,
    ) -> bool {
        if rest_idx >= complex_selector.rest.len() {
            return true;
        }
        let (combinator, right_compound) = &complex_selector.rest[rest_idx];

        Self::for_each_combinator_candidate(node, dom, combinator, |cand| {
            if self.match_compound(cand, right_compound, dom) {
                // Advance to next combinator/rest segment if compound matched.
                if self.match_relative_selector(cand, dom, complex_selector, rest_idx + 1) {
                    return true;
                }
            }
            false
        })
    }

    /// Abstracts combinator traversal and candidate discovery for relative selector matching.
    fn for_each_combinator_candidate<F>(
        node: &Node,
        dom: &Dom,
        combinator: &Combinator,
        mut f: F,
    ) -> bool
    where
        F: FnMut(&Node) -> bool,
    {
        match combinator {
            Combinator::AdjacentSibling => {
                if let Some(siblings) = node.siblings(&dom.nodes) {
                    if let Some(pos) = siblings.iter().position(|&idx| idx == node.id) {
                        if pos + 1 < siblings.len() {
                            let next_sibling = &dom.nodes[siblings[pos + 1]];
                            if f(next_sibling) {
                                return true;
                            }
                        }
                    }
                }
            }
            Combinator::GeneralSibling => {
                if let Some(siblings) = node.siblings(&dom.nodes) {
                    if let Some(pos) = siblings.iter().position(|&idx| idx == node.id) {
                        // For each following sibling (after this node)
                        for sib_idx in siblings.iter().skip(pos + 1) {
                            let sib_node = &dom.nodes[*sib_idx];
                            if f(sib_node) {
                                return true;
                            }
                        }
                    }
                }
            }
            Combinator::Child => {
                for &child_idx in &node.children {
                    let child = &dom.nodes[child_idx];
                    if f(child) {
                        return true;
                    }
                }
            }
            Combinator::Descendant => {
                // Traverse all descendants—DFS.
                let mut stack: Vec<&Node> =
                    node.children.iter().map(|&idx| &dom.nodes[idx]).collect();
                while let Some(desc) = stack.pop() {
                    if f(desc) {
                        return true;
                    }
                    // Expand for further descendants
                    stack.extend(desc.children.iter().map(|&idx| &dom.nodes[idx]));
                }
            }
        }
        false
    }

    fn match_attribute_operators(
        &mut self,
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

    /// Parses a sequence of component values representing an An+B microsyntax (for :nth-child, :nth-last-child, etc).
    /// Returns Some((a, b)) if parsed successfully, or None if not valid.
    ///
    /// Accepts forms:
    ///   n           -> (1, 0)
    ///   -n          -> (-1, 0)
    ///   2n+1        -> (2, 1)
    ///   2n-1        -> (2, -1)
    ///   odd         -> (2, 1)
    ///   even        -> (2, 0)
    ///   3           -> (0, 3)
    fn parse_an_plus_b(&mut self, vals: &[ComponentValue]) -> Option<(i32, i32)> {
        use ComponentValue::*;

        if vals.is_empty() {
            return None;
        }

        match &vals[0] {
            Ident(s) => {
                let lower = s.to_ascii_lowercase();
                match lower.as_str() {
                    "odd" => {
                        if vals.len() == 1 {
                            Some((2, 1))
                        } else {
                            None
                        }
                    }
                    "even" => {
                        if vals.len() == 1 {
                            Some((2, 0))
                        } else {
                            None
                        }
                    }
                    "n" => {
                        // "n", or "n" + "+b"/"-b"
                        let a = 1;
                        if vals.len() == 1 {
                            return Some((a, 0));
                        }
                        // "n" "+" Number   or  "n" "-" Number
                        if let Some(Delim(sign)) = vals.get(1) {
                            if let Some(Number(b)) = vals.get(2) {
                                match sign {
                                    '+' => Some((a, *b as i32)),
                                    '-' => Some((a, -(*b as i32))),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "-n" => {
                        // handle "-n", "-n+3", "-n-2"
                        let a = -1;
                        if vals.len() == 1 {
                            return Some((a, 0));
                        }
                        if let Some(Delim(sign)) = vals.get(1) {
                            if let Some(Number(b)) = vals.get(2) {
                                match sign {
                                    '+' => Some((a, *b as i32)),
                                    '-' => Some((a, -(*b as i32))),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    ident if ident.starts_with("n-") && ident.len() > 2 => {
                        // e.g. "n-2" or "n-0"
                        if let Ok(b) = ident[2..].parse::<i32>() {
                            Some((1, -b))
                        } else {
                            None
                        }
                    }
                    ident if ident.starts_with("-n-") && ident.len() > 3 => {
                        // e.g. "-n-5"
                        if let Ok(b) = ident[3..].parse::<i32>() {
                            Some((-1, -b))
                        } else {
                            None
                        }
                    }
                    "n-0" => Some((1, 0)),
                    _ => None,
                }
            }
            Number(num) => {
                // Accepts a simple number (0n+num or just num, i.e. matches only the Nth child).
                if vals.len() == 1 {
                    return Some((0, *num as i32));
                }
                // Might be a number, followed by Ident("n") or Dimension
                match vals.get(1) {
                    // 2n, 2n+1, 2n-1 (tokenizable as: Number, Ident("n"/"-n")).
                    Some(Ident(n_ident)) => {
                        let mut a = *num as i32;
                        let lower = n_ident.to_ascii_lowercase();
                        if lower == "n" {
                            // [Number, Ident("n")] => "an"
                            if vals.len() == 2 {
                                return Some((a, 0));
                            }
                            if let Some(Delim(sign)) = vals.get(2) {
                                if let Some(Number(b)) = vals.get(3) {
                                    match sign {
                                        '+' => Some((a, *b as i32)),
                                        '-' => Some((a, -(*b as i32))),
                                        _ => None,
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else if lower == "-n" {
                            a = -*num as i32;
                            if vals.len() == 2 {
                                return Some((a, 0));
                            }
                            if let Some(Delim(sign)) = vals.get(2) {
                                if let Some(Number(b)) = vals.get(3) {
                                    match sign {
                                        '+' => Some((a, *b as i32)),
                                        '-' => Some((a, -(*b as i32))),
                                        _ => None,
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    // [Number, Dimension] e.g. 2n, 2n+1, 2n-1, if tokenizer builds these as a Dimension with unit "n"
                    Some(Dimension { value, unit }) if unit.to_string() == "n" => {
                        let a = *value as i32;
                        // [Number, Dimension] only
                        if vals.len() == 2 {
                            return Some((a, 0));
                        }
                        // [Number, Dimension, Delim, Number]
                        if let Some(Delim(sign)) = vals.get(2) {
                            if let Some(Number(b)) = vals.get(3) {
                                match sign {
                                    '+' => Some((a, *b as i32)),
                                    '-' => Some((a, -(*b as i32))),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Dimension { value, unit } if unit.to_string() == "n" => {
                // e.g. 2n, 2n+1, 2n-1
                let a = *value as i32;
                if vals.len() == 1 {
                    return Some((a, 0));
                }
                if let Some(Delim(sign)) = vals.get(1) {
                    if let Some(Number(b)) = vals.get(2) {
                        match sign {
                            '+' => Some((a, *b as i32)),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else if let Some(Number(b)) = vals.get(1) {
                    Some((a, *b as i32))
                } else {
                    None
                }
            }
            Delim(sign) if *sign == '+' => {
                if vals.len() == 1 {
                    return None;
                }
                let a = match vals.get(1)? {
                    Ident(s) if s == "n" => return Some((1, 0)),
                    Number(n) => *n as i32,
                    _ => return None,
                };

                if vals.len() == 2 {
                    return Some((a, 0));
                }
                if let Some(Delim(sign)) = vals.get(2) {
                    if let Some(Number(b)) = vals.get(3) {
                        match sign {
                            '+' => Some((a, *b as i32)),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else if let Some(Number(b)) = vals.get(2) {
                    Some((a, *b as i32))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn is_nth_child_match(&mut self, index: i32, a: i32, b: i32) -> bool {
        // CSS child indices are 1-based.
        if index < 1 {
            return false;
        }

        // a == 0 => matches exactly the b-th child.
        if a == 0 {
            return index == b;
        }

        let diff = index - b;

        // n must be an integer.
        if diff % a != 0 {
            return false;
        }

        // n must be non-negative.
        let n = diff / a;
        n >= 0
    }

    fn css_lang_matches(&mut self, actual: &str, wanted: &str) -> bool {
        actual.eq_ignore_ascii_case(wanted) // TODO 
            || actual
                .strip_prefix(wanted)
                .is_some_and(|rest| rest.starts_with('-'))
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{assert_eq, format, matches, println, vec};

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
                <li id="fifth"></li>
                <li id="sixth"></li>
            </ul>
        "#;
        let css = r#"
            li:nth-child(2) { color: red; }              /* rule 0 */
            li:nth-child(even) { color: blue; }            /* rule 1 */
            li:nth-child(3n) { color: purple; }          /* rule 2 */
            li:nth-child(odd) { color: teal; }          /* rule 3 */
            li:nth-child(2n-1) { color: orange; }        /* rule 4 */
            li:nth-child(3n+2) { color: gold; }          /* rule 5 */
            li:nth-child(4n-2) { color: green; }         /* rule 6 */
            li:nth-child(0n+5) { color: khaki; }         /* rule 7 */
            li:nth-child(5n) { color: navy; }            /* rule 8 */
            li:nth-last-child(2) { color: green; }       /* rule 9 */
            li:nth-last-child(2n) { color: black; }      /* rule 10 */
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
        let fifth = find("fifth");
        let sixth = find("sixth");
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
        let m_fifth = Matcher::match_rules(
            fifth,
            &dom,
            &cssom.lookup_candidates(&get_el(fifth)),
            &cssom,
        )
        .unwrap_or_default();
        let m_sixth = Matcher::match_rules(
            sixth,
            &dom,
            &cssom.lookup_candidates(&get_el(sixth)),
            &cssom,
        )
        .unwrap_or_default();

        // The expected results below are checked manually against the CSS selectors
        // for nth-child and nth-last-child, using the actual positions of the <li> elements.
        // 1-based positions: first=1, second=2, ... sixth=6
        // nth-last-child is also position 1-based from the end (so sixth=1, fifth=2, ... first=6)

        // Each line: [rule 0, ..., rule 10] for that <li> node.
        // Calculation comments updated for accuracy; see https://developer.mozilla.org/en-US/docs/Web/CSS/:nth-child

        assert_eq!(
            vec![
                m_first.contains(&0),  // nth-child(2): false (first = 1)
                m_first.contains(&1),  // nth-child(2n): false (1 is not even)
                m_first.contains(&2),  // nth-child(3n): false (1 mod 3 != 0)
                m_first.contains(&3),  // nth-child(2n+1): true (2*0+1 = 1)
                m_first.contains(&4), // nth-child(2n-1): false (no n where 2n-1==1 except n=1 -> 1, so true)
                m_first.contains(&5), // nth-child(3n+2): false (1 not 2,5,8,11...)
                m_first.contains(&6), // nth-child(4n-2): false
                m_first.contains(&7), // nth-child(0n+5): false (first pos is not 5)
                m_first.contains(&8), // nth-child(5n): false (first pos is not 5, 10, ...)
                m_first.contains(&9), // nth-last-child(2): false (first pos from end is 6, 2nd from end is fifth)
                m_first.contains(&10), // nth-last-child(2n): true (6 is even, so matches, n=3)
            ],
            vec![
                false, false, false, true, true, false, false, false, false, false, true
            ]
        );

        assert_eq!(
            vec![
                m_second.contains(&0),  // nth-child(2): true (second=2)
                m_second.contains(&1),  // nth-child(2n): true (2/2=1, 2 is even)
                m_second.contains(&2),  // nth-child(3n): false (2)
                m_second.contains(&3),  // nth-child(2n+1): false (no n: 2n+1==2)
                m_second.contains(&4), // nth-child(2n-1): true (n=2, 2*2-1=3, no, for n=1, 2*1-1=1, n=2:3, n=2,2*2-1=3, n=2 for 2n-1=2 → n=1.5, not integer, so false)
                m_second.contains(&5), // nth-child(3n+2): true (n=0, 0+2=2; n=0)
                m_second.contains(&6), // nth-child(4n-2): false (n=1,4-2=2, yes)
                m_second.contains(&7), // nth-child(0n+5): false (not position 5)
                m_second.contains(&8), // nth-child(5n): false
                m_second.contains(&9), // nth-last-child(2): false (position from end is 5)
                m_second.contains(&10), // nth-last-child(2n): false (5 is odd)
            ],
            vec![
                true, true, false, false, false, true, true, false, false, false, false
            ]
        );

        assert_eq!(
            vec![
                m_third.contains(&0),  // nth-child(2): false (3 vs 2)
                m_third.contains(&1),  // nth-child(2n): false (3 is odd)
                m_third.contains(&2),  // nth-child(3n): true (n=1)
                m_third.contains(&3),  // nth-child(2n+1): true (n=1,2*1+1=3)
                m_third.contains(&4), // nth-child(2n-1): false (no n where 2n-1=3, n=2 gives 3, ok, so true)
                m_third.contains(&5), // nth-child(3n+2): false (3n+2=2,5,8..., 3 not covered)
                m_third.contains(&6), // nth-child(4n-2): false (n=1,4-2=2)
                m_third.contains(&7), // nth-child(0n+5): false
                m_third.contains(&8), // nth-child(5n): false
                m_third.contains(&9), // nth-last-child(2): false
                m_third.contains(&10), // nth-last-child(2n): true (4 is even),
            ],
            vec![
                false, false, true, true, true, false, false, false, false, false, true
            ]
        );

        assert_eq!(
            vec![
                m_fourth.contains(&0),  // nth-child(2): false
                m_fourth.contains(&1),  // nth-child(2n): true (even position)
                m_fourth.contains(&2),  // nth-child(3n): false (not multiple of 3)
                m_fourth.contains(&3),  // nth-child(2n+1): false (not odd)
                m_fourth.contains(&4),  // nth-child(2n-1): false (n=3 → 2*3-1=5, doesn't match 4)
                m_fourth.contains(&5),  // nth-child(3n+2): false (3*0+2=2, 3*1+2=5, not 4)
                m_fourth.contains(&6),  // nth-child(4n-2): true (n=1.5, 4*1.5-2 = 4)
                m_fourth.contains(&7),  // nth-child(0n+5): false
                m_fourth.contains(&8),  // nth-child(5n): false
                m_fourth.contains(&9),  // nth-last-child(2): false (position 3 from end)
                m_fourth.contains(&10), // nth-last-child(2n): false (3 is odd),
            ],
            vec![
                false, true, false, false, false, false, false, false, false, false, false
            ]
        );

        assert_eq!(
            vec![
                m_fifth.contains(&0),  // nth-child(2): false
                m_fifth.contains(&1),  // nth-child(2n): false (odd)
                m_fifth.contains(&2),  // nth-child(3n): false (not a multiple of 3)
                m_fifth.contains(&3),  // nth-child(2n+1): true (2*2+1=5)
                m_fifth.contains(&4),  // nth-child(2n-1): true (n=3,2*3-1=5)
                m_fifth.contains(&5),  // nth-child(3n+2): false (n=1,3*1+2=5, matches)
                m_fifth.contains(&6),  // nth-child(4n-2): true (n=1.75, not integer)
                m_fifth.contains(&7),  // nth-child(0n+5): true (5)
                m_fifth.contains(&8),  // nth-child(5n): true (5)
                m_fifth.contains(&9),  // nth-last-child(2): true (second from end = fifth)
                m_fifth.contains(&10), // nth-last-child(2n): true (fifth element is 2nd from the end, and 2 is even, so true)
            ],
            vec![
                false, false, false, true, true, true, false, true, true, true, true
            ]
        );

        assert_eq!(
            vec![
                m_sixth.contains(&0),  // nth-child(2): false
                m_sixth.contains(&1),  // nth-child(2n): true (6 is even)
                m_sixth.contains(&2),  // nth-child(3n): true (n=2,3*2=6)
                m_sixth.contains(&3),  // nth-child(2n+1): false (2*2+1=5, no solution for 6)
                m_sixth.contains(&4),  // nth-child(2n-1): false (2*4-1=7)
                m_sixth.contains(&5),  // nth-child(3n+2): false (3*1+2=5, 3*2+2=8)
                m_sixth.contains(&6),  // nth-child(4n-2): true (n=2,4*2-2=6)
                m_sixth.contains(&7),  // nth-child(0n+5): false (not 5)
                m_sixth.contains(&8),  // nth-child(5n): false (6 is not a multiple of 5)
                m_sixth.contains(&9),  // nth-last-child(2): false (first from end == sixth (n=1))
                m_sixth.contains(&10), // nth-last-child(2n): true (n=1, 6th from end = 1, 1 is odd, 2 * some n = 1 is false, but 1st from end so n=1, false, but for nth-last-child(2n): 6th from end is 1, only 2, 4, 6 positions from end, so false)
            ],
            vec![
                false, true, true, false, false, false, true, false, false, false, false
            ]
        );
    }

    // Additional test for :nth-child() edge cases and special forms
    #[test]
    fn test_matcher_nth_child_edge_cases() {
        let html = r#"
            <ol>
                <li id="one"></li>
                <li id="two"></li>
                <li id="three"></li>
                <li id="four"></li>
            </ol>
        "#;
        // Rules: 0=-n, 1=n, 2=n+0, 3=-2n, 4=4n-6, 5=0n+1, 6=0n-1, 7=+n, 8=n-2, 9=99999n-99998, 10=n-0, 11=-n-5, 12=-n+3
        let css = r#"
            li:nth-child(-n) { color: x; }     /* rule 0 -- matches nothing! */
            li:nth-child(n) { color: y; }      /* rule 1 -- matches all */
            li:nth-child(n+0) { color: z; }    /* rule 2 -- matches all */
            li:nth-child(-2n) { color: a; }    /* rule 3 -- matches none */
            li:nth-child(4n-6) { color: b; }   /* rule 4 -- matches pos 2, 6, 10, ... so in this case, pos 2 matches */
            li:nth-child(0n+1) { color: c; }   /* rule 5 -- only first child */
            li:nth-child(0n-1) { color: d; }   /* rule 6 -- impossible; can't match index < 1 */
            li:nth-child(+n) { color: e; }     /* rule 7 -- matches all */
            li:nth-child(n-2) { color: f; }    /* rule 8 -- matches all (n-2 = index, n >= 0, so all indices >= 1 are possible) */
            li:nth-child(99999n-99998) { color: g; } /* rule 9 -- n = 1 → 1, n = 2 → 100000, ... (matches pos=1, 100000, ...) */
            li:nth-child(n-0) { color: h; }   /* rule 10 -- matches all */
            li:nth-child(-n-5) { color: i; }   /* rule 11 -- matches nothing (pos = -n-5, n >= 0 => pos <= -5, but all indices > 0) */
            li:nth-child(-n+3) { color: j; }   /* rule 12 -- matches positions 1, 2, 3 (pos = -n+3, for n=0: 3, n=1: 2, n=2: 1) */
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

        let one = find("one");
        let two = find("two");
        let three = find("three");
        let four = find("four");
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

        let m_one = Matcher::match_rules(one, &dom, &cssom.lookup_candidates(&get_el(one)), &cssom)
            .unwrap_or_default();
        let m_two = Matcher::match_rules(two, &dom, &cssom.lookup_candidates(&get_el(two)), &cssom)
            .unwrap_or_default();
        let m_three = Matcher::match_rules(
            three,
            &dom,
            &cssom.lookup_candidates(&get_el(three)),
            &cssom,
        )
        .unwrap_or_default();
        let m_four =
            Matcher::match_rules(four, &dom, &cssom.lookup_candidates(&get_el(four)), &cssom)
                .unwrap_or_default();

        // Comments: [rule0, rule1, ... rule12]
        // li:nth-child(-n): matches no element (never true)
        // li:nth-child(n): matches any
        // li:nth-child(n+0): matches any
        // li:nth-child(-2n): matches only indices divisible by negative even - which is nothing, by spec
        // li:nth-child(4n-6): matches pos 2, 6, 10, ... so the 2nd element in our 4-element list should match
        // li:nth-child(0n+1): matches exactly the first only
        // li:nth-child(0n-1): never matches (negative/no 0 index)
        // li:nth-child(+n): matches all (equivalent to n)
        // li:nth-child(n-2): matches all -- for any index >= 1, there exists an n
        // li:nth-child(99999n-99998): only element #1 matches in this list
        // li:nth-child(n-0): matches all
        // li:nth-child(-n-5): never matches (all indices > 0, -n-5 <= -5)
        // li:nth-child(-n+3): matches indices 1, 2, 3 only

        // <li id="one"> (pos=1)
        assert_eq!(
            vec![
                m_one.contains(&0),  // -n
                m_one.contains(&1),  // n
                m_one.contains(&2),  // n+0
                m_one.contains(&3),  // -2n
                m_one.contains(&4),  // 4n-6
                m_one.contains(&5),  // 0n+1
                m_one.contains(&6),  // 0n-1
                m_one.contains(&7),  // +n
                m_one.contains(&8),  // n-2
                m_one.contains(&9),  // 99999n-99998
                m_one.contains(&10), // n-0
                m_one.contains(&11), // -n-5
                m_one.contains(&12), // -n+3
            ],
            vec![
                false, // -n
                true,  // n
                true,  // n+0
                false, // -2n
                false, // 4n-6
                true,  // 0n+1
                false, // 0n-1
                true,  // +n
                true,  // n-2 (matches all)
                true,  // 99999n-99998 -- only first element matches!
                true,  // n-0
                false, // -n-5
                true   // -n+3
            ]
        );
        // <li id="two"> (pos=2)
        assert_eq!(
            vec![
                m_two.contains(&0), // -n
                m_two.contains(&1),
                m_two.contains(&2),
                m_two.contains(&3),
                m_two.contains(&4),
                m_two.contains(&5),
                m_two.contains(&6),
                m_two.contains(&7),
                m_two.contains(&8),
                m_two.contains(&9),
                m_two.contains(&10),
                m_two.contains(&11),
                m_two.contains(&12),
            ],
            vec![
                false, // -n
                true,  // n
                true,  // n+0
                false, // -2n
                true,  // 4n-6 (2 = 4*2-6, n=2)
                false, // 0n+1
                false, // 0n-1
                true,  // +n
                true,  // n-2 (matches all)
                false, // 99999n-99998 -- only position 1 should match
                true,  // n-0
                false, // -n-5
                true   // -n+3
            ]
        );
        // <li id="three"> (pos=3)
        assert_eq!(
            vec![
                m_three.contains(&0),
                m_three.contains(&1),
                m_three.contains(&2),
                m_three.contains(&3),
                m_three.contains(&4),
                m_three.contains(&5),
                m_three.contains(&6),
                m_three.contains(&7),
                m_three.contains(&8),
                m_three.contains(&9),
                m_three.contains(&10),
                m_three.contains(&11),
                m_three.contains(&12),
            ],
            vec![
                false, // -n
                true,  // n
                true,  // n+0
                false, // -2n
                false, // 4n-6
                false, // 0n+1
                false, // 0n-1
                true,  // +n
                true,  // n-2 (matches all)
                false, // 99999n-99998
                true,  // n-0
                false, // -n-5
                true   // -n+3 (<=3)
            ]
        );
        // <li id="four"> (pos=4)
        assert_eq!(
            vec![
                m_four.contains(&0),
                m_four.contains(&1),
                m_four.contains(&2),
                m_four.contains(&3),
                m_four.contains(&4),
                m_four.contains(&5),
                m_four.contains(&6),
                m_four.contains(&7),
                m_four.contains(&8),
                m_four.contains(&9),
                m_four.contains(&10),
                m_four.contains(&11),
                m_four.contains(&12),
            ],
            vec![
                false, // -n
                true,  // n
                true,  // n+0
                false, // -2n
                false, // 4n-6
                false, // 0n+1
                false, // 0n-1
                true,  // +n
                true,  // n-2 (matches all)
                false, // 99999n-99998
                true,  // n-0
                false, // -n-5
                false  // -n+3 (4>3)
            ]
        );
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
    fn test_matcher_lang_and_dir_multiple_langs() {
        let html = r#"
            <div id="en" lang="en"></div>
            <div id="en2" lang="en-US"></div>
            <div id="fr" lang="fr"></div>
            <div id="ar" dir="rtl"></div>
        "#;
        let css = r#"
            div:lang(en) { color: red; }
            div:lang(fr, EN) { color: blue; }
            div:lang(en-us) { color: orange; }
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
        let en2 = find("en2");
        let fr = find("fr");
        let ar = find("ar");

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();
        println!("{:#?}", cssom.rules[1]);

        let m_en = Matcher::match_rules(en, &dom, &cssom.lookup_candidates(&get_el(en)), &cssom)
            .unwrap_or_default();
        let m_en2 = Matcher::match_rules(en2, &dom, &cssom.lookup_candidates(&get_el(en2)), &cssom)
            .unwrap_or_default();
        let m_fr = Matcher::match_rules(fr, &dom, &cssom.lookup_candidates(&get_el(fr)), &cssom)
            .unwrap_or_default();
        let m_ar = Matcher::match_rules(ar, &dom, &cssom.lookup_candidates(&get_el(ar)), &cssom)
            .unwrap_or_default();

        // div:lang(en) should match lang="en" and lang="en-US" per selector spec: "en-US" is a subtag of "en"
        assert!(m_en.contains(&0));
        assert!(m_en2.contains(&0));
        // div:lang(fr, EN) matches fr node and en node only for rule index 1 (color: blue)
        assert!(m_fr.contains(&1));
        assert!(m_en.contains(&1));
        // div:lang(en-us) should only match lang="en-US"
        assert!(m_en2.contains(&2));
        // div:dir(rtl) matches "ar" only
        assert!(m_ar.contains(&3));
    }

    // #[test]
    // fn test_matcher_dir_auto_with_text_content_and_nested() {
    //     // Some edge and nested cases for inferring direction with dir="auto"
    //     let html = r#"
    //         <div id="en" dir="auto">Hello!</div>
    //         <!-- Arabic (RTL) content: Arabic "مرحبا" -->
    //         <div id="ar" dir="auto">مرحبا!</div>
    //         <!-- Mixed (LTR then RTL), spec says first strong char is used for dir auto -->
    //         <div id="en_ar" dir="auto">Hello! مرحبا</div>
    //         <div id="ar_en" dir="auto">مرحبا Hello!</div>
    //         <!-- Only neutral characters: fallback LTR -->
    //         <div id="neutral" dir="auto">12345 .,!?</div>
    //         <!-- LTR letter inside nested element -->
    //         <div id="nested_ltr" dir="auto"><span>H</span> مرحبا</div>
    //         <!-- RTL letter inside nested element -->
    //         <div id="nested_rtl" dir="auto"><span>م</span> Hello!</div>
    //         <!-- Deeply nested strong char (LTR) triggers LTR -->
    //         <div id="deep_ltr" dir="auto"><span><b><i>E</i></b></span></div>
    //         <!-- Deeply nested strong char (RTL) triggers RTL -->
    //         <div id="deep_rtl" dir="auto"><span><b><i>م</i></b></span></div>
    //         <!-- No child text, only element nodes (should be LTR fallback) -->
    //         <div id="no_text" dir="auto"><span></span><div></div></div>
    //         <!-- Child with only whitespace -->
    //         <div id="only_space" dir="auto">    </div>
    //         <!-- Outer neutral, inner nested LTR and RTL: only first strong counts -->
    //         <div id="outer_neutral_nested_ltr" dir="auto">123 <span>H</span> <span>م</span></div>
    //         <div id="outer_neutral_nested_rtl" dir="auto">.?! <span>م</span> <span>H</span></div>
    //         <!-- Sibling text nodes interleaved with elements -->
    //         <div id="sib_ltr_first" dir="auto">A<span>م</span></div>
    //         <div id="sib_rtl_first" dir="auto">م<span>A</span></div>
    //     "#;
    //     let css = r#"
    //         [dir="auto"]:dir(ltr) { color: red; }
    //         [dir="auto"]:dir(rtl) { color: blue; }
    //         div:dir(ltr) { color: green; }
    //         div:dir(rtl) { color: orange; }
    //     "#;
    //     let result = Parser::to_dom(html).unwrap();
    //     let dom = result.value;
    //     let find = |id| {
    //         dom.nodes
    //             .iter()
    //             .find(|n| {
    //                 if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
    //                     el.id() == Some(id)
    //                 } else {
    //                     false
    //                 }
    //             })
    //             .expect(&format!("Node with id={id} not found"))
    //     };
    //     let get_el = |n: &Node| {
    //         if let html::dom::node_type::NodeType::Element(el) = &n.ntype {
    //             el.clone()
    //         } else {
    //             panic!()
    //         }
    //     };

    //     let ids_ltr = [
    //         "en",
    //         "en_ar",
    //         "ar_en",
    //         "neutral",
    //         "nested_ltr",
    //         "deep_ltr",
    //         "no_text",
    //         "only_space",
    //         "outer_neutral_nested_ltr",
    //         "sib_ltr_first",
    //     ];
    //     let ids_rtl = [
    //         "ar",
    //         "nested_rtl",
    //         "deep_rtl",
    //         "outer_neutral_nested_rtl",
    //         "sib_rtl_first",
    //     ];

    //     // LTR fallback should also apply to cases with only neutral, or empty
    //     // outer_neutral_nested_ltr: first strong is "H" (LTR)
    //     // outer_neutral_nested_rtl: first strong is "م" (RTL)
    //     let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
    //     let mut parser = crate::parser::parser::Parser::new(&tokens);
    //     let rules = parser.parse().unwrap();
    //     let mut cssom = CssOm {
    //         rules,
    //         indexes: CssIndexes::default(),
    //     };
    //     cssom.build_indexes();

    //     // Should be LTR
    //     // Instead of comparing selector as string, match using enum structure.
    //     // ([dir="auto"]:dir(ltr), and [dir="auto"]:dir(rtl))

    //     // Helper to identify a ComplexSelector representing [dir="auto"]:dir(ltr)
    //     fn is_dir_auto_ltr_selector(sel: &crate::cssom::selector::ComplexSelector) -> bool {
    //         // Only one compound (no combinator)
    //         if !sel.rest.is_empty() {
    //             return false;
    //         }
    //         // Should have two simple selectors:
    //         // 1. Attribute: [dir="auto"]
    //         // 2. Pseudo-class: :dir(ltr)
    //         let simples = &sel.first.simple_selectors;
    //         if simples.len() != 2 {
    //             return false;
    //         }
    //         let mut found_attr = false;
    //         let mut found_pseudo = false;
    //         for s in simples {
    //             match s {
    //                 SimpleSelector::Attribute {
    //                     name,
    //                     value,
    //                     operator,
    //                 } => {
    //                     if name == "dir" && value.as_deref() == Some("auto") && operator.is_none() {
    //                         found_attr = true;
    //                     }
    //                 }
    //                 SimpleSelector::PseudoClass(name, Some(PseudoArg::Values(args))) => {
    //                     // :dir(ltr)
    //                     if name == "dir"
    //                         && args.len() == 1
    //                         && matches!(&args[0], crate::cssom::component_value::ComponentValue::Ident(ident) if ident == "ltr")
    //                     {
    //                         found_pseudo = true;
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         found_attr && found_pseudo
    //     }

    //     // Helper to identify a ComplexSelector representing [dir="auto"]:dir(rtl)
    //     fn is_dir_auto_rtl_selector(sel: &crate::cssom::selector::ComplexSelector) -> bool {
    //         // Only one compound (no combinator)
    //         if !sel.rest.is_empty() {
    //             return false;
    //         }
    //         let simples = &sel.first.simple_selectors;
    //         if simples.len() != 2 {
    //             return false;
    //         }
    //         let mut found_attr = false;
    //         let mut found_pseudo = false;
    //         for s in simples {
    //             match s {
    //                 SimpleSelector::Attribute {
    //                     name,
    //                     value,
    //                     operator,
    //                 } => {
    //                     if name == "dir" && value.as_deref() == Some("auto") && operator.is_none() {
    //                         found_attr = true;
    //                     }
    //                 }
    //                 SimpleSelector::PseudoClass(name, Some(PseudoArg::Values(args))) => {
    //                     // :dir(rtl)
    //                     if name == "dir"
    //                         && args.len() == 1
    //                         && matches!(&args[0], crate::cssom::component_value::ComponentValue::Ident(ident) if ident == "rtl")
    //                     {
    //                         found_pseudo = true;
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         found_attr && found_pseudo
    //     }

    //     // Find the index of the appropriate rules
    //     let ltr_rule_idx = cssom
    //         .rules
    //         .iter()
    //         .position(|r| {
    //             if let crate::cssom::rule::CssRule::Style(sr) = r {
    //                 sr.selectors.iter().any(is_dir_auto_ltr_selector)
    //             } else {
    //                 false
    //             }
    //         })
    //         .expect("Couldn't find [dir=\"auto\"]:dir(ltr) rule");
    //     let rtl_rule_idx = cssom
    //         .rules
    //         .iter()
    //         .position(|r| {
    //             if let crate::cssom::rule::CssRule::Style(sr) = r {
    //                 sr.selectors.iter().any(is_dir_auto_rtl_selector)
    //             } else {
    //                 false
    //             }
    //         })
    //         .expect("Couldn't find [dir=\"auto\"]:dir(rtl) rule");

    //     for id in ids_ltr.iter() {
    //         let node = find(id);
    //         let m =
    //             Matcher::match_rules(node, &dom, &cssom.lookup_candidates(&get_el(node)), &cssom)
    //                 .unwrap_or_default();
    //         assert!(
    //             m.iter().any(|&i| i as usize == ltr_rule_idx),
    //             "Expected {id} to match :dir(ltr)"
    //         );
    //         assert!(
    //             !m.iter().any(|&i| i as usize == rtl_rule_idx),
    //             "Expected {id} NOT to match :dir(rtl)"
    //         );
    //     }

    //     // Should be RTL
    //     for id in ids_rtl.iter() {
    //         let node = find(id);
    //         let m =
    //             Matcher::match_rules(node, &dom, &cssom.lookup_candidates(&get_el(node)), &cssom)
    //                 .unwrap_or_default();
    //         assert!(
    //             m.iter().any(|&i| i as usize == rtl_rule_idx),
    //             "Expected {id} to match :dir(rtl)"
    //         );
    //         assert!(
    //             !m.iter().any(|&i| i as usize == ltr_rule_idx),
    //             "Expected {id} NOT to match :dir(ltr)"
    //         );
    //     }
    // }

    #[test]
    fn test_matcher_not_is_where() {
        let html = r#"
            <div id="a" class="foo bar baz" data-x="1"></div>
            <div id="b" class="foo" data-x="2"></div>
            <div id="c" class="bar"></div>
            <div id="d" class="foo baz" data-x="3"></div>
            <div id="e" class="qux foo" data-x="4"></div>
            <div id="f" class="baz"></div>
            <div id="g" class="foo" data-y="5"></div>
            <div id="h"></div>
        "#;
        let css = r#"
            .foo:not(.bar):not([data-x="3"]) { color: red; }
            .foo:is(.foo, .baz) { color: green; }
            .foo:where(.foo, .baz) { color: gray; }
            .foo:not(.baz, .qux) { color: blue; }
            .foo:not(.baz):not(.qux) { color: orange; }
            .foo:is(.baz, .bar):not([data-x]) { color: purple; }
            .foo:where(.baz, .bar) { color: magenta; }
            .baz:is(.foo, #b) { color: lime; }
            .foo:not(:is(.bar, .baz)) { color: gold; }
            .foo:not(:is(.baz, .qux, .bar)) { color: brown; }
            .foo:is(.qux, .nonexistent) { color: pink; }
            .foo:where(.qux, .nonexistent) { color: olive; }
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
                .expect(&format!("Node with id={id} not found"))
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
        let f = find("f");
        let g = find("g");
        let h = find("h");

        let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(css);
        let mut parser = crate::parser::parser::Parser::new(&tokens);
        let rules = parser.parse().unwrap();
        let mut cssom = CssOm {
            rules,
            indexes: CssIndexes::default(),
        };
        cssom.build_indexes();

        let match_for = |node: &Node| {
            Matcher::match_rules(node, &dom, &cssom.lookup_candidates(&get_el(node)), &cssom)
                .unwrap_or_default()
        };
        let m_a = match_for(a);
        let m_b = match_for(b);
        let m_c = match_for(c);
        let m_d = match_for(d);
        let m_e = match_for(e);
        let m_f = match_for(f);
        let m_g = match_for(g);
        let m_h = match_for(h);

        // .foo:not(.bar):not([data-x="3"]) { color: red; }
        assert!(
            m_b.contains(&0),
            "b matches .foo:not(.bar):not([data-x=\"3\"]) (only .foo, data-x=\"2\")"
        );
        assert!(!m_a.contains(&0), "a does NOT match rule 0; has .bar");
        assert!(
            !m_d.contains(&0),
            "d does NOT match rule 0; has data-x=\"3\""
        );
        assert!(!m_c.contains(&0), "c does not have .foo");
        assert!(
            m_e.contains(&0),
            "e should match rule 0 (foo, not bar, not [data-x=3])"
        );
        assert!(
            m_g.contains(&0),
            "g should match rule 0 (foo, not bar, not [data-x=3])"
        );
        assert!(!m_f.contains(&0), "f has baz, but no foo");

        // .foo:is(.foo, .baz) { color: green; }
        assert!(m_a.contains(&1), "a matches .foo:is(.foo, .baz)");
        assert!(m_b.contains(&1), "b matches .foo:is(.foo, .baz)");
        assert!(m_d.contains(&1), "d matches .foo:is(.foo, .baz)");
        assert!(m_e.contains(&1), "e matches .foo:is(.foo, .baz)");
        assert!(m_g.contains(&1), "g matches .foo:is(.foo, .baz)");
        assert!(!m_c.contains(&1), "c does not match .foo:is(.foo, .baz)");
        assert!(!m_f.contains(&1), "f has baz but not foo");
        assert!(!m_h.contains(&1), "h has none");

        // .foo:where(.foo, .baz) { color: gray; }
        assert!(m_a.contains(&2), "a matches .foo:where(.foo, .baz)");
        assert!(m_b.contains(&2), "b matches .foo:where(.foo, .baz)");
        assert!(m_d.contains(&2), "d matches .foo:where(.foo, .baz)");
        assert!(m_e.contains(&2), "e matches .foo:where(.foo, .baz)");
        assert!(m_g.contains(&2), "g matches .foo:where(.foo, .baz)");
        assert!(!m_c.contains(&2), "c does not match .foo:where(.foo, .baz)");
        assert!(!m_f.contains(&2), "f has baz but not foo");
        assert!(!m_h.contains(&2), "h matches nothing");

        // .foo:not(.baz, .qux) { color: blue; }
        assert!(
            m_b.contains(&3),
            "b should match rule 3 (.foo:not(.baz, .qux))"
        );
        assert!(!m_a.contains(&3), "a has .baz");
        assert!(!m_d.contains(&3), "d has .baz");
        assert!(!m_e.contains(&3), "e has .qux");
        assert!(m_g.contains(&3), "g matches rule 3");
        assert!(!m_c.contains(&3), "c is not .foo");

        // .foo:not(.baz):not(.qux) { color: orange; }
        assert!(
            m_b.contains(&4),
            "b matches rule 4 (.foo:not(.baz):not(.qux))"
        );
        assert!(
            m_g.contains(&4),
            "g matches rule 4 (.foo:not(.baz):not(.qux))"
        );
        assert!(!m_a.contains(&4), "a has .baz");
        assert!(!m_e.contains(&4), "e has .qux");
        assert!(!m_d.contains(&4), "d has .baz");
        assert!(!m_c.contains(&4), "c no foo");

        // .foo:is(.baz, .bar):not([data-x]) { color: purple; }
        assert!(!m_a.contains(&5), "a has data-x");
        assert!(!m_b.contains(&5), "b has data-x");
        assert!(!m_d.contains(&5), "d has data-x");
        assert!(!m_e.contains(&5), "e has data-x");
        assert!(!m_g.contains(&5), "g has data-y but not data-x");
        assert!(!m_c.contains(&5), "c no foo");
        assert!(!m_f.contains(&5), "f is baz but not foo, no data-x");
        assert!(!m_h.contains(&5), "h nothing");

        // .foo:where(.baz, .bar) { color: magenta; }
        assert!(m_a.contains(&6), "a matches .foo:where(.baz, .bar)");
        assert!(!m_b.contains(&6), "b not .baz or .bar");
        assert!(m_d.contains(&6), "d matches .foo:where(.baz, .bar)");
        assert!(!m_e.contains(&6), "e matches .foo:where(.baz, .bar)");
        assert!(!m_g.contains(&6), "g not .baz or .bar");
        assert!(!m_c.contains(&6), "c no foo");

        // .baz:is(.foo, #b) { color: lime; }
        assert!(m_a.contains(&7), "a matches .baz:is(.foo, #b)");
        assert!(!m_b.contains(&7), "b not .baz");
        assert!(m_d.contains(&7), "d not .baz");
        assert!(!m_e.contains(&7), "e not .baz");
        assert!(
            !m_f.contains(&7),
            "f is .baz, matches .foo (on a), not itself"
        );
        assert!(!m_g.contains(&7), "g not .baz");

        // .foo:not(:is(.bar, .baz)) { color: gold; }
        assert!(m_b.contains(&8), "b matches .foo:not(:is(.bar, .baz))");
        assert!(m_g.contains(&8), "g matches .foo:not(:is(.bar, .baz))");
        assert!(!m_a.contains(&8), "a is .baz");
        assert!(!m_d.contains(&8), "d is .baz");
        assert!(
            m_e.contains(&8),
            "e is .baz via .baz branch? (actually not), but has .qux"
        );
        assert!(!m_c.contains(&8), "c not .foo");

        // .foo:not(:is(.baz, .qux, .bar)) { color: brown; }
        assert!(
            m_b.contains(&9),
            "b matches .foo:not(:is(.baz, .qux, .bar))"
        );
        assert!(
            m_g.contains(&9),
            "g matches .foo:not(:is(.baz, .qux, .bar))"
        );
        assert!(!m_a.contains(&9), "a matches .baz");
        assert!(!m_d.contains(&9), "d matches .baz");
        assert!(!m_e.contains(&9), "e matches .qux");
        assert!(!m_c.contains(&9), "c not .foo");

        // .foo:is(.qux, .nonexistent) { color: pink; }
        assert!(m_e.contains(&10), "e matches .foo:is(.qux, .nonexistent)");
        assert!(!m_b.contains(&10), "b is foo not qux, not nonexistent");
        assert!(!m_a.contains(&10), "a not qux");
        assert!(!m_d.contains(&10), "d not qux");
        assert!(!m_g.contains(&10), "g not qux");
        assert!(!m_f.contains(&10), "f not foo");

        // .foo:where(.qux, .nonexistent) { color: olive; }
        assert!(
            m_e.contains(&11),
            "e matches .foo:where(.qux, .nonexistent)"
        );
        assert!(!m_b.contains(&11), "b is foo, not qux, not nonexistent");
        assert!(!m_a.contains(&11), "a not qux");
        assert!(!m_d.contains(&11), "d not qux");
        assert!(!m_g.contains(&11), "g not qux");
        assert!(!m_f.contains(&11), "f not foo");

        // not .foo: any selector should not match h
        for i in 0..12 {
            assert!(
                !m_h.contains(&i),
                "h with no classes/attrs must not match any test selector"
            );
        }
    }

    #[test]
    fn test_has_pseudo_class_matrix() {
        let run_test = |html: &str, selectors: &[(&str, &[&str], &[&str])]| {
            let dom_result = Parser::to_dom(html).unwrap();
            let dom = dom_result.value;
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

            for (selector, should_match_ids, should_not_match_ids) in selectors {
                let css = format!(".foo{selector} {{ x: y; }}", selector = selector);
                let tokens = crate::tokenizer::tokenizer::Tokenizer::from_string(&css);
                let mut parser = crate::parser::parser::Parser::new(&tokens);
                let rules = parser.parse().unwrap();
                let mut cssom = CssOm {
                    rules,
                    indexes: CssIndexes::default(),
                };
                cssom.build_indexes();

                for id in *should_match_ids {
                    let node = find(id);
                    let matched = Matcher::match_rules(
                        node,
                        &dom,
                        &cssom.lookup_candidates(&get_el(node)),
                        &cssom,
                    )
                    .unwrap_or_default();
                    assert!(
                        matched.contains(&0),
                        "Selector '{}': node id '{}' should match but did not",
                        selector,
                        id
                    );
                }

                for id in *should_not_match_ids {
                    let node = find(id);
                    let matched = Matcher::match_rules(
                        node,
                        &dom,
                        &cssom.lookup_candidates(&get_el(node)),
                        &cssom,
                    )
                    .unwrap_or_default();
                    assert!(
                        !matched.contains(&0),
                        "Selector '{}': node id '{}' should NOT match but did",
                        selector,
                        id
                    );
                }
            }
        };

        // Group 1 - descendants
        run_test(
            r#"
        <section id="g1">
            <div id="a" class="foo">
                <span class="item"></span>
            </div>
            <div id="b" class="foo">
                <div>
                    <span class="item"></span>
                </div>
            </div>
            <div id="c" class="foo"></div>
        </section>
        "#,
            &[
                (":has(.item)", &["a", "b"], &["c"]),
                (":has(span)", &["a", "b"], &["c"]),
                (":has(.missing)", &[], &["a", "b", "c"]),
            ],
        );
        println!("DONE 1");

        // Group 2 - child combinator
        run_test(
            r#"
        <section id="g2">
            <div id="d" class="foo">
                <span class="item"></span>
            </div>
            <div id="e" class="foo">
                <div>
                    <span class="item"></span>
                </div>
            </div>
            <div id="f" class="foo"></div>
        </section>
        "#,
            &[
                (":has(> .item)", &["d"], &["e", "f"]),
                (":has(> span)", &["d"], &["e", "f"]),
                (":has(> div)", &["e"], &["d", "f"]),
            ],
        );
        println!("DONE 2");

        // Group 3 - adjacent sibling (FIXED)
        run_test(
            r#"
        <section id="g3">
            <div id="g" class="foo"></div>
            <span class="item"></span>
            <div id="h" class="foo"></div>
            <div></div>
            <span class="item"></span>
            <div id="i" class="foo"></div>
        </section>
        "#,
            &[
                (":has(+ .item)", &["g"], &["h", "i"]),
                (":has(+ span)", &["g"], &["h", "i"]),
            ],
        );
        println!("DONE 3");

        // Group 4 - general sibling
        run_test(
            r#"
        <section id="g4">
            <div id="j" class="foo"></div>
            <div></div>
            <span class="item"></span>
            <div id="k" class="foo"></div>
            <div></div>
            <div id="l" class="foo"></div>
            <span></span>
        </section>
        "#,
            &[
                (":has(~ .item)", &["j"], &["k", "l"]),
                (":has(~ span)", &["j", "k", "l"], &[]),
            ],
        );
        println!("DONE 4");

        // Group 5 - complex selector & nested combinators
        run_test(
            r#"
        <section id="g5">
            <div id="m" class="foo">
                <ul>
                    <li class="active"></li>
                </ul>
            </div>
            <div id="n" class="foo">
                <ul>
                    <li></li>
                </ul>
            </div>
            <div id="o" class="foo"></div>
        </section>
        "#,
            &[
                (":has(ul > li.active)", &["m"], &["n", "o"]),
                (":has(ul > li)", &["m", "n"], &["o"]),
                (":has(li.active)", &["m"], &["n", "o"]),
                (":has(> ul > li.active)", &["m"], &["n", "o"]), // Test leading combinator
            ],
        );
        println!("DONE 5");

        // Group 6 - nested pseudos
        run_test(
            r#"
        <section id="g6">
            <div id="p" class="foo">
                <span class="bar"></span>
            </div>
            <div id="q" class="foo">
                <span class="baz"></span>
            </div>
            <div id="r" class="foo">
                <span></span>
            </div>
        </section>
        "#,
            &[
                (":has(:not(.baz))", &["p", "r"], &["q"]),
                (":has(:is(.bar,.baz))", &["p", "q"], &["r"]),
                (":has(:where(.bar,.baz))", &["p", "q"], &["r"]),
            ],
        );
        println!("DONE 6");

        // Group 7 - multiple selectors (comma)
        run_test(
            r#"
        <section id="g7">
            <div id="s" class="foo">
                <span class="bar"></span>
            </div>
            <div id="t" class="foo">
                <div class="baz"></div>
            </div>
            <div id="u" class="foo"></div>
        </section>
        "#,
            &[
                (":has(.bar, .baz)", &["s", "t"], &["u"]),
                (":has(.missing, .baz)", &["t"], &["s", "u"]),
                (":has(.missing1, .missing2)", &[], &["s", "t", "u"]),
            ],
        );
        println!("DONE 7");

        // Group 8 - nested :has()
        run_test(
            r#"
        <section id="g8">
            <div id="v" class="foo">
                <section>
                    <span class="item"></span>
                </section>
            </div>
            <div id="w" class="foo">
                <section></section>
            </div>
        </section>
        "#,
            &[(":has(section:has(.item))", &["v"], &["w"])],
        );
        println!("DONE 8");

        // Group 9 - attributes
        run_test(
            r#"
        <section id="g9">
            <div id="x" class="foo">
                <span data-x="1"></span>
            </div>
            <div id="y" class="foo">
                <span></span>
            </div>
        </section>
        "#,
            &[
                (":has([data-x])", &["x"], &["y"]),
                (":has([data-x=\"1\"])", &["x"], &["y"]),
                (":has([data-x=\"2\"])", &[], &["x", "y"]),
                (":has(> [data-x])", &["x"], &["y"]), // New: attribute with child combinator
            ],
        );
        println!("DONE 9");

        // Group 10 - IDs
        run_test(
            r#"
        <section id="g10">
            <div id="z" class="foo">
                <span id="target"></span>
            </div>
        </section>
        "#,
            &[
                (":has(#target)", &["z"], &[]),
                (":has(#missing)", &[], &["z"]),
            ],
        );
        println!("DONE 10");

        // Group 11 - previous siblings, should not match backward (regression)
        run_test(
            r#"
        <div>
            <span class="item"></span>
            <div id="foo" class="foo"></div>
        </div>
        "#,
            &[
                (":has(~ .item)", &[], &["foo"]), // item comes before foo, not after
            ],
        );
        println!("DONE 11");

        // Group 12 - mixed combinators (fixed!)
        run_test(
            r#"
        <section id="g12">
            <div id="a" class="foo">
                <div>
                    <span class="x"></span>
                </div>
            </div>
            <div id="b" class="foo">
                <div></div>
                <span></span>
            </div>
            <div id="c" class="foo">
                <div></div>
                <span class="y"></span>
            </div>
            <div id="d" class="foo">
                <div>
                    <span class="x"></span>
                </div>
                <span class="y"></span>
            </div>
        </section>
        "#,
            &[
                (":has(div > .x)", &["a", "d"], &["b", "c"]), // Now matches correctly: only a and d have div with child .x
                (":has(div + span)", &["b", "c", "d"], &["a"]),
                (":has(div ~ span)", &["b", "c", "d"], &["a"]),
            ],
        );
        println!("DONE 12");

        // Group 12b - direct/any child regression
        run_test(
            r#"
        <div id="root" class="foo">
            <div class="x"></div>
        </div>
        "#,
            &[(":has(.x)", &["root"], &[]), (":has(> .x)", &["root"], &[])],
        );
        println!("DONE 12b");

        // Group 12c - nested subject for + and ~ combinators (new test, per reviewer recommendation)
        run_test(
            r#"
        <div id="foo" class="foo">
            <section>
                <div></div>
                <span></span>
            </section>
            <section>
                <div></div>
                <div></div>
            </section>
        </div>
        <div id="bar" class="foo">
            <section>
                <div></div>
                <!-- no span sibling -->
            </section>
        </div>
        "#,
            &[
                (":has(div + span)", &["foo"], &["bar"]), // foo has a descendant <div> that is immediately followed by a sibling <span>
                (":has(section > .missing)", &[], &["foo", "bar"]), // optional: tests deep child selector fails if no children match
            ],
        );
        println!("DONE 12c");

        // Group 13 - subject should not count as descendant (regression)
        run_test(
            r#"
        <div id="self" class="foo bar"></div>
        "#,
            &[
                (":has(.bar)", &[], &["self"]), // node itself is not considered its own descendant
            ],
        );
        println!("DONE 13");

        // Group 14 - empty descendants must not match
        run_test(
            r#"
        <div id="empty" class="foo"></div>
        "#,
            &[(":has(*)", &[], &["empty"]), (":has(div)", &[], &["empty"])],
        );
        println!("DONE 14");

        // Group 15 - multiple matching descendants
        run_test(
            r#"
        <div id="multi" class="foo">
            <span class="item"></span>
            <span class="item"></span>
        </div>
        "#,
            &[
                (":has(.item)", &["multi"], &[]), // Should only match once, no error if multiple
            ],
        );
        println!("DONE 15");

        // Group 16 - nested descendant in :has
        run_test(
            r#"
        <div id="outer" class="foo">
            <div>
                <span>
                    <em class="target"></em>
                </span>
            </div>
        </div>
        <div id="isolated" class="foo"></div>
        "#,
            &[
                // outer should match because it has a descendant <em class="target"> via nested :has
                (":has(div span em.target)", &["outer"], &["isolated"]),
            ],
        );
        println!("DONE 16");

        // Group 16b - multiple candidates regression test for :has (backtracking required)
        run_test(
            r#"
        <div id="outer" class="foo">
            <div>
                <span></span>
            </div>
            <div>
                <span>
                    <em class="target"></em>
                </span>
            </div>
        </div>
        <div id="isolated" class="foo"></div>
        "#,
            &[
                // outer should match because the second <div> is a valid candidate, even if the first <div> fails as a path
                (":has(div span em.target)", &["outer"], &["isolated"]),
            ],
        );
        println!("DONE 16b");
    }
}
