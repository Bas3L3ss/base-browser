use std::matches;

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

    /// Returns the (1-based) nth position of this node among its element siblings (including itself), or None if not found.
    /// Counting only siblings of NodeType::Element.
    pub fn nth_element_sibling_index(&self, dom_nodes: &[Node]) -> Option<usize> {
        let parent_id = self.parent?;
        let parent = dom_nodes.get(parent_id)?;
        let mut pos = 0;
        for &cid in &parent.children {
            if let Some(node) = dom_nodes.get(cid) {
                if matches!(node.ntype, NodeType::Element(_)) {
                    pos += 1;
                    if cid == self.id {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    /// Returns the (1-based) nth position of this node among its element siblings (from the end, including itself), or None if not found.
    /// Counting only siblings of NodeType::Element, starting from the end. Uses formula: siblings.len() - front_index + 1.
    pub fn nth_element_sibling_reverse_index(&self, dom_nodes: &[Node]) -> Option<usize> {
        let front = self.nth_element_sibling_index(dom_nodes)?;
        let len = self.element_sibling_count(dom_nodes)?;
        Some(len - front + 1)
    }

    /// Get the sibling Element node IDs (`NodeID`s) of this node (always including self),
    /// filtering only Element nodes. Returns None if no parent or parent node missing.
    pub fn siblings<'a>(&self, dom_nodes: &'a [Node]) -> Option<Vec<NodeID>> {
        let parent_id = self.parent?;
        let parent = dom_nodes.get(parent_id)?;

        // Avoid allocating here if the parent has no children, or if this is not an Element node
        // Returns Vec<NodeID> but allocates only filtered elements
        let count = parent
            .children
            .iter()
            .filter(|&&cid| {
                dom_nodes
                    .get(cid)
                    .map_or(false, |n| matches!(n.ntype, NodeType::Element(_)))
            })
            .count();

        if count == parent.children.len() {
            // All child nodes are Element nodes, can just clone the parent's children vec.
            return Some(parent.children.clone());
        }

        // Only allocate for filtered elements.
        let mut siblings = Vec::with_capacity(count);
        for &cid in &parent.children {
            if let Some(node) = dom_nodes.get(cid) {
                if matches!(node.ntype, NodeType::Element(_)) {
                    siblings.push(cid);
                }
            }
        }

        Some(siblings)
    }

    /// Get the sibling Element node IDs (`NodeID`s) of this node (always including self)
    /// that have the same HTML tag as this node. Returns None if not Element or no parent/parent missing.
    pub fn same_tag_siblings<'a>(&self, dom_nodes: &'a [Node]) -> Option<Vec<NodeID>> {
        use crate::dom::node_type::NodeType;
        let parent_id = self.parent?;
        let parent = dom_nodes.get(parent_id)?;
        let my_tag = match &self.ntype {
            NodeType::Element(el) => &el.tag,
            _ => return None,
        };

        let mut out = Vec::new();
        for &cid in &parent.children {
            if let Some(Node {
                ntype: NodeType::Element(el),
                ..
            }) = dom_nodes.get(cid)
            {
                if &el.tag == my_tag {
                    out.push(cid);
                }
            }
        }
        Some(out)
    }

    /// Returns the count of element siblings (i.e., nodes of NodeType::Element) for this node (including itself).
    /// Returns None if no parent or parent node not found.
    pub fn element_sibling_count(&self, dom_nodes: &[Node]) -> Option<usize> {
        let parent_id = self.parent?;
        let parent = dom_nodes.get(parent_id)?;

        Some(
            parent
                .children
                .iter()
                .filter(|&&cid| {
                    dom_nodes
                        .get(cid)
                        .map_or(false, |n| matches!(n.ntype, NodeType::Element(_)))
                })
                .count(),
        )
    }

    pub fn previous_sibling(&self, dom_nodes: &[Node]) -> Option<NodeID> {
        let parent_id = self.parent?;
        let parent = dom_nodes.get(parent_id)?;

        let mut last_element_sibling: Option<NodeID> = None;
        for &cid in &parent.children {
            if cid == self.id {
                // Found self; return the last seen element sibling before this one
                return last_element_sibling;
            }
            if let Some(node) = dom_nodes.get(cid) {
                if matches!(node.ntype, NodeType::Element(_)) {
                    last_element_sibling = Some(cid);
                }
            }
        }
        None
    }

    /// Returns the (1-based) nth position of this node among its siblings with the same HTML tag (including itself), or None if not found.
    /// Only compares with siblings that are elements and have the same tag (e.g. <div> only with <div>, <span> only with <span>).
    pub fn nth_same_tag_sibling_index(&self, dom_nodes: &[Node]) -> Option<usize> {
        use crate::dom::node_type::NodeType;
        let parent_id = self.parent?;
        let parent = dom_nodes.get(parent_id)?;
        // Only works for Element nodes
        let my_tag = match &self.ntype {
            NodeType::Element(el) => &el.tag,
            _ => return None,
        };
        let mut pos = 0;
        for &cid in &parent.children {
            if let Some(node) = dom_nodes.get(cid) {
                if let NodeType::Element(ref el) = node.ntype {
                    if &el.tag == my_tag {
                        pos += 1;
                        if cid == self.id {
                            return Some(pos);
                        }
                    }
                }
            }
        }
        None
    }

    /// Returns the (1-based) nth position of this node among its siblings with the same HTML tag (from the end, including itself), or None if not found.
    pub fn nth_same_tag_sibling_reverse_index(&self, dom_nodes: &[Node]) -> Option<usize> {
        let front = self.nth_same_tag_sibling_index(dom_nodes)?;
        let len = self.same_tag_sibling_count(dom_nodes)?;
        Some(len - front + 1)
    }

    /// Returns the count of siblings with the same HTML tag as self (including itself).
    /// Returns None if not an element, or no parent, or parent node not found.
    pub fn same_tag_sibling_count(&self, dom_nodes: &[Node]) -> Option<usize> {
        use crate::dom::node_type::NodeType;
        let parent_id = self.parent?;
        let parent = dom_nodes.get(parent_id)?;
        let my_tag = match &self.ntype {
            NodeType::Element(el) => &el.tag,
            _ => return None,
        };
        Some(
            parent
                .children
                .iter()
                .filter(|&&cid| {
                    dom_nodes.get(cid).map_or(
                        false,
                        |n| matches!(&n.ntype, NodeType::Element(el) if &el.tag == my_tag),
                    )
                })
                .count(),
        )
    }
}
