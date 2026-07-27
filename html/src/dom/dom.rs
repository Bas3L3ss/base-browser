use crate::dom::{node::Node, stylesheet::StyleSheet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dom {
    pub nodes: Vec<Node>,
    pub style_sheets: Vec<StyleSheet>,
}

impl Dom {
    pub fn new(nodes: Vec<Node>, style_sheets: Vec<StyleSheet>) -> Self {
        Self {
            nodes,
            style_sheets,
        }
    }
}
