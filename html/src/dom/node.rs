use crate::dom::node_type::NodeType;

pub type NodeID = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub parent: Option<NodeID>,
    pub children: Vec<NodeID>,
    pub ntype: NodeType,
}

impl Node {
    pub fn new(ntype: NodeType, parent: Option<NodeID>, children: Vec<NodeID>) -> Self {
        Node {
            parent,
            children,
            ntype,
        }
    }
}
