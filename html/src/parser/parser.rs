use std::vec;

use anyhow::Result;
use input_parser::generic_struct::ParsedOutput;
use utils::profile_scope;

use crate::{
    dom::{
        dom::Dom, dom_element::DomElement, node::Node, node_type::NodeType,
        style_sheet_ref::StyleSheetRef,
    },
    tokenizer::{attributes::Attributes, html_tags::HtmlTag, token::Token, tokenizer::Tokenizer},
};

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnclosedTag { tag: HtmlTag },
    UnexpectedClosingTag { expected: HtmlTag, found: HtmlTag },
}

pub struct Parser;

impl Parser {
    /// Parses the given HTML string into a DOM tree represented as a vector of Nodes,
    /// and collects any parser errors encountered (such as unclosed or unexpected tags)
    /// as warnings in the ParsedOutput.
    pub fn to_dom(html: &str) -> Result<ParsedOutput<Dom, ParserError>> {
        profile_scope!("HTML to dom");
        let mut tokenizer = Tokenizer::new(html);
        let tokens = tokenizer.tokenize();
        profile_scope!("HTML Parse");
        let mut arena = vec![Node::new(0, NodeType::Document, None, vec![])];
        let mut style_sheets: Vec<StyleSheetRef> = vec![];
        let mut errors = vec![];
        let mut token_idx = 0;
        let mut document_element: Option<usize> = None;
        let mut body_element: Option<usize> = None;
        let mut head_element: Option<usize> = None;

        Self::parse_children_and_record_special_ids(
            &tokens,
            &mut arena,
            &mut token_idx,
            0,
            &mut errors,
            &mut style_sheets,
            &mut document_element,
            &mut body_element,
            &mut head_element,
        );

        Self::recover_document(
            &mut arena,
            &mut document_element,
            &mut head_element,
            &mut body_element,
        );

        Ok(ParsedOutput::new(
            Dom::new(
                arena,
                style_sheets,
                document_element.unwrap(),
                body_element.unwrap(),
                head_element.unwrap(),
            ),
            errors,
        ))
    }

    /// Ensures the DOM has <html>, <head>, and <body>, creates them if missing, and
    /// moves misplaced children as necessary. Handles minimal duplicate merging.
    fn recover_document(
        arena: &mut Vec<Node>,
        document_element: &mut Option<usize>,
        head_element: &mut Option<usize>,
        body_element: &mut Option<usize>,
    ) {
        // Step 1: Ensure <html>
        let html_id = match document_element {
            Some(id) => *id,
            None => {
                let html_node = Node::new(
                    arena.len(),
                    NodeType::Element(DomElement::new(HtmlTag::Html, Attributes::default())),
                    None,
                    vec![],
                );
                let id = Self::push_child(arena, 0, html_node);
                *document_element = Some(id);
                id
            }
        };

        // Step 2: Ensure <head>
        let head_id = match head_element {
            Some(id) => {
                Self::move_child_to_position(arena, html_id, *id, 0);
                *id
            }
            None => {
                let head_node = Node::new(
                    arena.len(),
                    NodeType::Element(DomElement::new(HtmlTag::Head, Attributes::default())),
                    None,
                    vec![],
                );
                let id = Self::insert_child_at(arena, html_id, head_node, 0);
                *head_element = Some(id);
                id
            }
        };

        // Step 3: Ensure <body>
        let body_id = match body_element {
            Some(id) => {
                Self::move_child_to_position(arena, html_id, *id, 1);
                *id
            }
            None => {
                let body_node = Node::new(
                    arena.len(),
                    NodeType::Element(DomElement::new(HtmlTag::Body, Attributes::default())),
                    None,
                    vec![],
                );
                let id = Self::insert_child_at(arena, html_id, body_node, 1);
                *body_element = Some(id);
                id
            }
        };

        // Move misplaced children from Document to their rightful place if not html
        let document_children = arena[0].children.clone();
        for &child_id in &document_children {
            if child_id == html_id {
                continue;
            }
            let node = &arena[child_id];
            match &node.ntype {
                NodeType::Element(el) => match el.tag {
                    HtmlTag::Head => {
                        // Merge duplicate heads: only keep head_id
                        if child_id != head_id {
                            Self::merge_duplicate_into_canonical(arena, child_id, head_id);
                        }
                    }
                    HtmlTag::Body => {
                        if child_id != body_id {
                            Self::merge_duplicate_into_canonical(arena, child_id, body_id);
                        }
                    }
                    HtmlTag::Title
                    | HtmlTag::Base
                    | HtmlTag::Meta
                    | HtmlTag::Link
                    | HtmlTag::Style
                    | HtmlTag::Script
                    | HtmlTag::Noscript
                    | HtmlTag::Template => {
                        // Acceptable in <head>
                        Self::reparent_node(arena, child_id, head_id);
                    }
                    _ => {
                        // Everything else, including <html> and custom elements, goes into <body>
                        Self::reparent_node(arena, child_id, body_id);
                    }
                },
                NodeType::Text(_) | NodeType::Comment(_) => {
                    Self::reparent_node(arena, child_id, body_id);
                }
                NodeType::Document => { /* don't reparent */ }
                NodeType::Doctype(_) => { /* skip for recovery */ }
            }
        }

        // Move misplaced children from <html> to <head> or <body>
        let html_children = arena[html_id].children.clone();
        for &child_id in &html_children {
            if child_id == head_id || child_id == body_id {
                continue;
            }
            let node = &arena[child_id];
            match &node.ntype {
                NodeType::Element(el) => match el.tag {
                    HtmlTag::Head => {
                        if child_id != head_id {
                            Self::merge_duplicate_into_canonical(arena, child_id, head_id);
                        }
                    }
                    HtmlTag::Body => {
                        if child_id != body_id {
                            Self::merge_duplicate_into_canonical(arena, child_id, body_id);
                        }
                    }
                    HtmlTag::Title
                    | HtmlTag::Base
                    | HtmlTag::Meta
                    | HtmlTag::Link
                    | HtmlTag::Style
                    | HtmlTag::Script
                    | HtmlTag::Noscript
                    | HtmlTag::Template => {
                        Self::reparent_node(arena, child_id, head_id);
                    }
                    _ => {
                        Self::reparent_node(arena, child_id, body_id);
                    }
                },
                NodeType::Text(_) | NodeType::Comment(_) => {
                    Self::reparent_node(arena, child_id, body_id);
                }
                NodeType::Document => {}
                NodeType::Doctype(_) => {}
            }
        }
    }

    /// Simple merging: moves all children from `dup_id` into `canonical_id`, then detaches `dup_id` from its parent.
    fn merge_duplicate_into_canonical(arena: &mut Vec<Node>, dup_id: usize, canonical_id: usize) {
        let children = arena[dup_id].children.clone();
        for &kid in &children {
            Self::reparent_node(arena, kid, canonical_id);
        }

        // Remove the duplicate node from its parent (if any)
        if let Some(parent_id) = arena[dup_id].parent {
            let siblings = &mut arena[parent_id].children;
            if let Some(pos) = siblings.iter().position(|&id| id == dup_id) {
                siblings.remove(pos);
                arena[dup_id].parent = None; // Remove parent pointer from detached node (nice to have)
            }
        }
        // We do NOT touch arena[dup_id].children: this is fine for v0 as discussed.
    }

    /// Move a node in the arena to a new parent, updating children and parent pointers.
    fn reparent_node(arena: &mut Vec<Node>, node_id: usize, new_parent_id: usize) {
        if node_id >= arena.len() || new_parent_id >= arena.len() {
            return;
        }
        // Remove node from old parent's children
        if let Some(old_parent_id) = arena[node_id].parent {
            if old_parent_id < arena.len() {
                if let Some(pos) = arena[old_parent_id]
                    .children
                    .iter()
                    .position(|&id| id == node_id)
                {
                    arena[old_parent_id].children.remove(pos);
                }
            }
        }
        // Add node to new parent's children (last position if not present)
        if !arena[new_parent_id].children.contains(&node_id) {
            arena[new_parent_id].children.push(node_id);
        }
        // Update node's parent pointer
        arena[node_id].parent = Some(new_parent_id);
    }

    /// Move a child node to a specific position in the parent's children list.
    /// This will first reparent `child_id` from its old parent to `parent_id`, then
    /// move it to the specified position within parent's children.
    fn move_child_to_position(
        arena: &mut Vec<Node>,
        parent_id: usize,
        child_id: usize,
        pos: usize,
    ) {
        if parent_id >= arena.len() || child_id >= arena.len() {
            return;
        }
        // Always reparent the child to the new parent (removes from previous parent's children)
        Self::reparent_node(arena, child_id, parent_id);

        let children = &mut arena[parent_id].children;

        // Now move node within parent's children to specified position
        if let Some(current_pos) = children.iter().position(|&id| id == child_id) {
            let id = children.remove(current_pos);
            let insert_pos = pos.min(children.len());
            children.insert(insert_pos, id);
        }
    }

    /// Insert a new child node into the arena at a specific position in the parent's children.
    /// Returns the id of the new node.
    fn insert_child_at(
        arena: &mut Vec<Node>,
        parent_id: usize,
        mut node: Node,
        pos: usize,
    ) -> usize {
        let id = node.id;
        node.parent = Some(parent_id);
        arena.push(node);
        let children = &mut arena[parent_id].children;
        let insert_pos = pos.min(children.len());
        children.insert(insert_pos, id);
        id
    }

    fn parse_element_and_record_special_ids(
        tokens: &[Token],
        arena: &mut Vec<Node>,
        token_idx: &mut usize,
        parent_id: usize,
        tag: &HtmlTag,
        attributes: &Attributes,
        errors: &mut Vec<ParserError>,
        style_sheets: &mut Vec<StyleSheetRef>,
        document_element: &mut Option<usize>,
        body_element: &mut Option<usize>,
        head_element: &mut Option<usize>,
    ) {
        let node = Node::new(
            arena.len(),
            NodeType::Element(DomElement::new(tag.clone(), attributes.clone())),
            Some(parent_id),
            vec![],
        );
        let node_id = Self::push_child(arena, parent_id, node);

        // If this is the html/body/head node, record its id
        match tag {
            HtmlTag::Html => {
                if document_element.is_none() {
                    *document_element = Some(node_id);
                }
            }
            HtmlTag::Body => {
                if body_element.is_none() {
                    *body_element = Some(node_id);
                }
            }
            HtmlTag::Head => {
                if head_element.is_none() {
                    *head_element = Some(node_id);
                }
            }
            _ => {}
        }

        // Detect <style> tag and register its node_id as InlineStyle
        if *tag == HtmlTag::Style {
            style_sheets.push(StyleSheetRef::InlineStyle(node_id));
        }

        // Detect <link rel="stylesheet" ...>
        if *tag == HtmlTag::Link {
            let rel_value = attributes.rel.as_ref().map(|v| v.to_ascii_lowercase());
            let href_value = attributes.href.as_ref().map(|v| v.trim().to_string());

            if let (Some(rel), Some(href)) = (rel_value, href_value) {
                if rel == "stylesheet" && !href.is_empty() {
                    style_sheets.push(StyleSheetRef::StyleSheetLink(node_id));
                }
            }
        }

        *token_idx += 1;
        Self::parse_children_and_record_special_ids(
            tokens,
            arena,
            token_idx,
            node_id,
            errors,
            style_sheets,
            document_element,
            body_element,
            head_element,
        );
    }

    fn push_child(arena: &mut Vec<Node>, parent_id: usize, node: Node) -> usize {
        let node_id = node.id;
        arena.push(node);
        arena[parent_id].children.push(node_id);
        node_id
    }

    /// Depth-first traversal to consume tokens and build node tree into arena.
    /// Records <html>, <body>, <head> node ids found during parsing for Dom fields.
    /// Returns errors through the mutable errors vector.
    fn parse_children_and_record_special_ids(
        tokens: &[Token],
        arena: &mut Vec<Node>,
        token_idx: &mut usize,
        parent_id: usize,
        errors: &mut Vec<ParserError>,
        style_sheets: &mut Vec<StyleSheetRef>,
        document_element: &mut Option<usize>,
        body_element: &mut Option<usize>,
        head_element: &mut Option<usize>,
    ) {
        while *token_idx < tokens.len() {
            match &tokens[*token_idx] {
                Token::EOF => {
                    match &arena[parent_id].ntype {
                        NodeType::Document => {
                            // nothing to do at document level on EOF
                        }
                        NodeType::Element(element) => {
                            errors.push(ParserError::UnclosedTag {
                                tag: element.tag.clone(),
                            });
                        }
                        _ => {}
                    }
                    break;
                }

                Token::Start(tag, attributes) => {
                    Self::parse_element_and_record_special_ids(
                        tokens,
                        arena,
                        token_idx,
                        parent_id,
                        tag,
                        attributes,
                        errors,
                        style_sheets,
                        document_element,
                        body_element,
                        head_element,
                    );
                }

                Token::End(tag) => {
                    match &arena[parent_id].ntype {
                        NodeType::Document => {
                            // At document level: stray/orphan closing tag, ignore it.
                            *token_idx += 1;
                            continue;
                        }
                        NodeType::Element(element) => {
                            if element.tag == *tag {
                                // Correctly closing current element.
                                *token_idx += 1;
                                break;
                            } else {
                                // Mismatched close tag within element -- report error.
                                errors.push(ParserError::UnexpectedClosingTag {
                                    expected: element.tag.clone(),
                                    found: tag.clone(),
                                });
                                *token_idx += 1;
                                continue;
                            }
                        }
                        _ => {
                            // Just consume and continue for other node types (should not happen)
                            *token_idx += 1;
                            continue;
                        }
                    }
                }

                Token::SelfClose(tag, attributes) => {
                    let node = Node::new(
                        arena.len(),
                        NodeType::Element(DomElement::new(tag.clone(), attributes.clone())),
                        Some(parent_id),
                        vec![],
                    );
                    let node_id = Self::push_child(arena, parent_id, node);

                    // If this is the html/body/head node, record its id
                    match tag {
                        HtmlTag::Html => {
                            if document_element.is_none() {
                                *document_element = Some(node_id);
                            }
                        }
                        HtmlTag::Body => {
                            if body_element.is_none() {
                                *body_element = Some(node_id);
                            }
                        }
                        HtmlTag::Head => {
                            if head_element.is_none() {
                                *head_element = Some(node_id);
                            }
                        }
                        _ => {}
                    }

                    // Detect <link rel="stylesheet" ... /> self-closing
                    if *tag == HtmlTag::Link {
                        let rel_value = attributes.rel.as_ref().map(|v| v.to_ascii_lowercase());
                        let href_value = attributes.href.as_ref().map(|v| v.trim().to_string());

                        if let (Some(rel), Some(href)) = (rel_value, href_value) {
                            if rel == "stylesheet" && !href.is_empty() {
                                style_sheets.push(StyleSheetRef::StyleSheetLink(node_id));
                            }
                        }
                    }

                    *token_idx += 1;
                }

                Token::Doctype(attribute) => {
                    Self::push_child(
                        arena,
                        parent_id,
                        Node::new(
                            arena.len(),
                            NodeType::Doctype(attribute.clone()),
                            Some(parent_id),
                            vec![],
                        ),
                    );
                    *token_idx += 1;
                }

                Token::Comment(comment) => {
                    Self::push_child(
                        arena,
                        parent_id,
                        Node::new(
                            arena.len(),
                            NodeType::Comment(comment.clone()),
                            Some(parent_id),
                            vec![],
                        ),
                    );
                    *token_idx += 1;
                }

                Token::Content(content) => {
                    Self::push_child(
                        arena,
                        parent_id,
                        Node::new(
                            arena.len(),
                            NodeType::Text(content.clone()),
                            Some(parent_id),
                            vec![],
                        ),
                    );
                    *token_idx += 1;
                }
            }
        }
    }
}

// ----------------- UNIT TESTS -------------------

#[cfg(test)]
mod tests {
    use std::{assert_eq, matches};

    use super::*;
    use crate::dom::style_sheet_ref::StyleSheetRef;
    use crate::{dom::node_type::NodeType, tokenizer::html_tags::HtmlTag};

    fn get_element_tag(node: &Node) -> Option<&HtmlTag> {
        match &node.ntype {
            NodeType::Element(el) => Some(&el.tag),
            _ => None,
        }
    }

    fn get_text_content(node: &Node) -> Option<&str> {
        match &node.ntype {
            NodeType::Text(txt) => Some(txt.as_str()),
            _ => None,
        }
    }

    fn has_element(arena: &[Node], wanted: HtmlTag) -> bool {
        arena
            .iter()
            .any(|n| matches!(&n.ntype, NodeType::Element(el) if el.tag == wanted))
    }

    #[test]
    fn test_parse_simple_html() {
        let html = "<html><body><h1>Hello</h1></body></html>";
        let output = Parser::to_dom(html).unwrap();
        // Should contain html, body, h1 elements
        assert!(has_element(&output.value.nodes, HtmlTag::Html));
        assert!(has_element(&output.value.nodes, HtmlTag::Body));
        assert!(has_element(&output.value.nodes, HtmlTag::Header(1)));
        // Should not have parse errors
        assert!(output.warnings.is_empty());
    }

    #[test]
    fn test_unclosed_tag_reports_error() {
        let html = "<div><span>Unclosed";
        let output = Parser::to_dom(html).unwrap();
        // span is unclosed, so error should be reported
        assert!(
            output
                .warnings
                .iter()
                .any(|err| matches!(err, ParserError::UnclosedTag { tag }
                    if *tag == HtmlTag::Span

                ))
        );
        // div is also unclosed
        assert!(
            output
                .warnings
                .iter()
                .any(|err| matches!(err, ParserError::UnclosedTag { tag }
                    if *tag == HtmlTag::Div
                ))
        );
    }

    #[test]
    fn test_mismatched_close_tag_reports_error() {
        let html = "<ul><li>Item 1</ul>";
        let output = Parser::to_dom(html).unwrap();
        // <ul> closes but <li> not closed - should report a mismatched close error
        assert!(output.warnings.iter().any(
            |err| matches!(err, ParserError::UnexpectedClosingTag { expected, found }
                if *expected == HtmlTag::ListItem && *found == HtmlTag::UnorderedList
            )
        ));
    }

    #[test]
    fn test_self_closing_tag() {
        let html = "<img src='x.jpg'/>";
        let output = Parser::to_dom(html).unwrap();
        assert!(has_element(&output.value.nodes, HtmlTag::Image));
        assert!(output.warnings.is_empty());
    }

    #[test]
    fn test_text_content_extraction() {
        let html = "<div>Hello <b>World</b>!</div>";
        let output = Parser::to_dom(html).unwrap();
        let text_nodes: Vec<&str> = output
            .value
            .nodes
            .iter()
            .filter_map(|n| get_text_content(n))
            .collect();
        assert!(text_nodes.iter().any(|t| t.contains("Hello")));
        assert!(text_nodes.iter().any(|t| t.contains("World")));
        assert!(text_nodes.iter().any(|t| t.contains("!")));
    }

    #[test]
    fn test_comments_and_doctype() {
        let html = "<!DOCTYPE html><!-- a comment --><div></div>";
        let output = Parser::to_dom(html).unwrap();
        let has_doctype = output
            .value
            .nodes
            .iter()
            .any(|n| matches!(&n.ntype, NodeType::Doctype(dt) if dt.contains("html")));
        let has_comment = output
            .value
            .nodes
            .iter()
            .any(|n| matches!(&n.ntype, NodeType::Comment(c) if c.contains("a comment")));
        assert!(has_doctype);
        assert!(has_comment);
    }

    #[test]
    fn test_style_and_link_stylesheet_detection() {
        let html = "<style>.a{}</style><link rel=\"stylesheet\" href=\".\"><div></div>";
        let output = Parser::to_dom(html).unwrap();
        let style_ids: Vec<_> = output.value.style_sheets.iter().collect();
        assert!(
            style_ids
                .iter()
                .any(|s| matches!(s, StyleSheetRef::InlineStyle(_)))
        );
        assert!(
            style_ids
                .iter()
                .any(|s| matches!(s, StyleSheetRef::StyleSheetLink(_)))
        );
    }

    #[test]
    fn test_empty_html_gives_document_node() {
        let html = "";
        let output = Parser::to_dom(html).unwrap();
        // Always contains a single Document node (root)
        assert_eq!(output.value.nodes.len() > 0, true);
        match &output.value.nodes[0].ntype {
            NodeType::Document => {}
            _ => panic!("Root node not a Document"),
        }
        assert!(output.warnings.is_empty());
    }

    #[test]
    fn test_deeply_nested_tags() {
        let html = "<ul><li><span><b>txt</b></span></li></ul>";
        let output = Parser::to_dom(html).unwrap();
        // Should find all these elements
        assert!(has_element(&output.value.nodes, HtmlTag::UnorderedList));
        assert!(has_element(&output.value.nodes, HtmlTag::ListItem));
        assert!(has_element(&output.value.nodes, HtmlTag::Span));
        assert!(has_element(&output.value.nodes, HtmlTag::Bold));
        // And should not have parse errors
        assert!(output.warnings.is_empty());
    }

    #[test]
    fn test_multiple_top_level_elements() {
        let html = "<div>1</div><div>2</div>";
        let output = Parser::to_dom(html).unwrap();
        let divs = output
            .value
            .nodes
            .iter()
            .filter(|n| get_element_tag(n) == Some(&HtmlTag::Div))
            .count();
        assert!(divs >= 2);
        assert!(output.warnings.is_empty());
    }
}
