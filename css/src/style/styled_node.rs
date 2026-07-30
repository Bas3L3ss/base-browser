use html::dom::node::NodeID;

use crate::style::computed_style::ComputedStyle;

#[derive(Debug, Clone, PartialEq)]
pub struct StyledNode {
    node: NodeID,
    style: ComputedStyle,
}

impl StyledNode {
    pub fn new(node: NodeID, style: ComputedStyle) -> Self {
        Self { node, style }
    }
}
