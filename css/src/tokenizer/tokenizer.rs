use std::{print, println, unimplemented};

use html::{
    dom::{node::Node, node_type::NodeType},
    tokenizer::html_tags::HtmlTag,
};
use utils::profile_scope;

use crate::tokenizer::css_token::CssToken;

pub struct Tokenizer<'a> {
    source: &'a Vec<Node>,
    position: usize, // Index in the Vec<Node>
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a Vec<Node>) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<CssToken> {
        profile_scope!("Tokenize CSS");
        let mut tokens = Vec::new();

        while !self.is_eof() {
            if let Some(css_tokens) = self.next_tokens() {
                tokens.extend(css_tokens);
            }
        }
        tokens.push(CssToken::EOF);
        tokens
    }

    /// Peeks at the current node without advancing.
    fn peek(&self) -> Option<&Node> {
        self.source.get(self.position)
    }

    /// Advances and returns the next node, or None if at EOF.
    fn advance(&mut self) -> Option<&Node> {
        let node = self.source.get(self.position);
        if node.is_some() {
            self.position += 1;
        }
        node
    }

    /// Checks if the tokenizer has reached the end of the input.
    fn is_eof(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Consumes the next token from the input stream.
    fn next_tokens(&mut self) -> Option<Vec<CssToken>> {
        let node: &Node = self.advance()?;

        let tokens = match node.ntype {
            NodeType::Element(ref element) if element.tag == HtmlTag::Style => {
                return None;
            }
            NodeType::Element(ref element)
                if element.tag == HtmlTag::Link
                    && element
                        .attributes
                        .rel
                        .as_ref()
                        .map_or(false, |rel| rel.eq_ignore_ascii_case("stylesheet")) =>
            {
                let href = match &element.attributes.href {
                    Some(h) if !h.trim().is_empty() => h,

                    _ => return None,
                };

                return None;
            }
            _ => None,
        };

        tokens
    }

    fn from_string(raw: &str) -> Vec<CssToken> {
        unimplemented!()
    }
}
