use crate::BoxType;
use html::NodeID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutNode {
    pub source: LayoutSource,
    pub box_type: BoxType,
    pub children: Vec<LayoutNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutSource {
    DomNode(NodeID),
    Anonymous,
}

impl LayoutNode {
    pub fn new(source: LayoutSource, box_type: BoxType, children: Vec<LayoutNode>) -> Self {
        Self {
            source,
            box_type,
            children,
        }
    }

    /// Pushes a child LayoutNode into this node's children.
    pub fn push(&mut self, child: LayoutNode) {
        self.children.push(child);
    }
}
