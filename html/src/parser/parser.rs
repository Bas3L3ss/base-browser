use std::vec;

use crate::{
    parser::{element::DomElement, node::Node, node_type::NodeType},
    tokenizer::{attributes::Attributes, html_tags::HtmlTag, token::Token, tokenizer::Tokenizer},
};

pub struct Parser;

impl Parser {
    /// Parses the given HTML string into a DOM tree represented as a vector of Nodes.
    pub fn to_dom(html: &str) -> Vec<Node> {
        let mut tokenizer = Tokenizer::new(html);
        let tokens = tokenizer.tokenize();
        let mut arena = vec![Node::new(NodeType::Document, None, vec![])];
        Self::parse_tokens(&tokens, &mut arena, &mut 0, 0);
        arena
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
    ) {
        let node = Node::new(
            NodeType::Element(DomElement::new(tag.clone(), attributes.clone())),
            Some(parent_id),
            vec![],
        );
        let node_id = Self::push_child(arena, parent_id, node);
        *token_idx += 1;
        Self::parse_tokens(tokens, arena, token_idx, node_id);
    }

    /// Depth-first traversal to consume tokens and build node tree into arena.
    fn parse_tokens(
        tokens: &[Token],
        arena: &mut Vec<Node>,
        token_idx: &mut usize,
        parent_id: usize,
    ) {
        while *token_idx < tokens.len() {
            match &tokens[*token_idx] {
                Token::EOF => break,

                Token::Start(tag, attributes) => {
                    Self::parse_element(tokens, arena, token_idx, parent_id, tag, attributes);
                }

                Token::End(tag) => match &arena[parent_id].ntype {
                    NodeType::Element(element) if element.tag == *tag => {
                        *token_idx += 1;
                        break;
                    }
                    _ => {}
                },

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
