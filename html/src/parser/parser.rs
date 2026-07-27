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
        let mut arena = vec![Node::new(NodeType::Document, None, vec![])];
        let mut style_sheets: Vec<StyleSheetRef> = vec![];
        let mut errors = vec![];
        let mut token_idx = 0;
        Self::parse_children(
            &tokens,
            &mut arena,
            &mut token_idx,
            0,
            &mut errors,
            &mut style_sheets,
        );
        Ok(ParsedOutput::new(Dom::new(arena, style_sheets), errors))
    }

    fn push_child(arena: &mut Vec<Node>, parent_id: usize, node: Node) -> usize {
        arena.push(node);
        let node_id = arena.len() - 1;
        arena[parent_id].children.push(node_id);
        node_id
    }

    fn parse_element(
        tokens: &[Token],
        arena: &mut Vec<Node>,
        token_idx: &mut usize,
        parent_id: usize,
        tag: &HtmlTag,
        attributes: &Attributes,
        errors: &mut Vec<ParserError>,
        style_sheets: &mut Vec<StyleSheetRef>,
    ) {
        let node = Node::new(
            NodeType::Element(DomElement::new(tag.clone(), attributes.clone())),
            Some(parent_id),
            vec![],
        );
        let node_id = Self::push_child(arena, parent_id, node);

        // Detect <style> tag and register its node_id as InlineStyle
        if *tag == HtmlTag::Style {
            style_sheets.push(StyleSheetRef::InlineStyle(node_id));
        }

        // Detect <link rel="stylesheet" ...>
        if *tag == HtmlTag::Link {
            // rel="stylesheet"
            let rel_value = attributes.rel.as_ref().map(|v| v.to_ascii_lowercase());
            let href_value = attributes.href.as_ref().map(|v| v.trim().to_string());

            if let (Some(rel), Some(href)) = (rel_value, href_value) {
                if rel == "stylesheet" && !href.is_empty() {
                    style_sheets.push(StyleSheetRef::StyleSheetLink(node_id));
                }
            }
        }

        *token_idx += 1;
        Self::parse_children(tokens, arena, token_idx, node_id, errors, style_sheets);
    }

    /// Depth-first traversal to consume tokens and build node tree into arena.
    /// Returns errors through the mutable errors vector.
    fn parse_children(
        tokens: &[Token],
        arena: &mut Vec<Node>,
        token_idx: &mut usize,
        parent_id: usize,
        errors: &mut Vec<ParserError>,
        style_sheets: &mut Vec<StyleSheetRef>,
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
                    Self::parse_element(
                        tokens,
                        arena,
                        token_idx,
                        parent_id,
                        tag,
                        attributes,
                        errors,
                        style_sheets,
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
                        NodeType::Element(DomElement::new(tag.clone(), attributes.clone())),
                        Some(parent_id),
                        vec![],
                    );
                    let node_id = Self::push_child(arena, parent_id, node);

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
                        Node::new(NodeType::Comment(comment.clone()), Some(parent_id), vec![]),
                    );
                    *token_idx += 1;
                }

                Token::Content(content) => {
                    Self::push_child(
                        arena,
                        parent_id,
                        Node::new(NodeType::Text(content.clone()), Some(parent_id), vec![]),
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
