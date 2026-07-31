use crate::dom::node_type::NodeType;

pub type NodeID = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: NodeID,
    pub parent: Option<NodeID>,
    pub children: Vec<NodeID>,
    pub ntype: NodeType,
}

impl Node {
    pub fn new(id: NodeID, ntype: NodeType, parent: Option<NodeID>, children: Vec<NodeID>) -> Self {
        Node {
            id,
            parent,
            children,
            ntype,
        }
    }

    pub fn siblings(&self, dom_nodes: &[Node]) -> Option<Vec<NodeID>> {
        let parent_id = self.parent?;
        let parent = dom_nodes.get(parent_id)?;
        let siblings: Vec<NodeID> = parent
            .children
            .iter()
            .filter_map(|&cid| {
                let node = dom_nodes.get(cid)?;
                match node.ntype {
                    NodeType::Element(_) => Some(cid),
                    _ => None,
                }
            })
            .collect();
        Some(siblings)
    }

    pub fn previous_sibling(&self, dom_nodes: &[Node]) -> Option<NodeID> {
        self.parent.and_then(|parent_id| {
            dom_nodes.get(parent_id).and_then(|parent| {
                parent
                    .children
                    .iter()
                    .position(|&id| id == self.id)
                    .and_then(|pos| {
                        if pos > 0 {
                            Some(parent.children[pos - 1])
                        } else {
                            None
                        }
                    })
            })
        })
    }
}
