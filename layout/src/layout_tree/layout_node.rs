use crate::BoxType;
use html::NodeID;

pub struct LayoutNode {
    pub node_id: NodeID,
    pub box_type: BoxType,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(node_id: NodeID, box_type: BoxType, children: Vec<LayoutNode>) -> Self {
        Self {
            node_id,
            box_type,
            children,
        }
    }
}
