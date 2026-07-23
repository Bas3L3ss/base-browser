use std::vec;

use crate::{
    parser::{dom_element::DomElement, node::Node, node_type::NodeType},
    tokenizer::{attributes::Attributes, html_tags::HtmlTag, token::Token, tokenizer::Tokenizer},
};

#[derive(Debug)]
pub enum ParserError {
    UnclosedTag { tag: HtmlTag },
    UnexpectedClosingTag { expected: HtmlTag, found: HtmlTag },
}

/// Holds the result of parsing: the DOM tree and any errors found.
#[derive(Debug)]
pub struct ParseResult {
    pub arena: Vec<Node>,
    pub errors: Vec<ParserError>,
}

pub struct Parser;

impl Parser {
    /// Parses the given HTML string into a DOM tree represented as a vector of Nodes,
    /// and collects any parser errors encountered (such as unclosed or unexpected tags).
    pub fn to_dom(html: &str) -> ParseResult {
        let mut tokenizer = Tokenizer::new(html);
        let tokens = tokenizer.tokenize();
        let mut arena = vec![Node::new(NodeType::Document, None, vec![])];
        let mut errors = vec![];
        Self::parse_children(&tokens, &mut arena, &mut 0, 0, &mut errors);
        ParseResult { arena, errors }
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
    ) {
        let node = Node::new(
            NodeType::Element(DomElement::new(tag.clone(), attributes.clone())),
            Some(parent_id),
            vec![],
        );
        let node_id = Self::push_child(arena, parent_id, node);
        *token_idx += 1;
        Self::parse_children(tokens, arena, token_idx, node_id, errors);
    }

    /// Depth-first traversal to consume tokens and build node tree into arena.
    /// Returns errors through the mutable errors vector.
    fn parse_children(
        tokens: &[Token],
        arena: &mut Vec<Node>,
        token_idx: &mut usize,
        parent_id: usize,
        errors: &mut Vec<ParserError>,
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
                        tokens, arena, token_idx, parent_id, tag, attributes, errors,
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
                    Self::push_child(
                        arena,
                        parent_id,
                        Node::new(
                            NodeType::Element(DomElement::new(tag.clone(), attributes.clone())),
                            Some(parent_id),
                            vec![],
                        ),
                    );
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
