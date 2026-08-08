use std::matches;

use crate::{Dimensions, text::inline_fragment::InlineFragment};
use html::NodeID;

#[derive(Debug, Clone)]
pub enum LayoutNode {
    Block {
        node_id: NodeID,
        dimensions: Dimensions,
        children: Vec<LayoutNode>,
    },
    Inline {
        node_id: NodeID,
        children: Vec<LayoutNode>,
        // fragments: Vec<InlineBlock>,
    },
    InlineBlock {
        node_id: NodeID,
        dimensions: Dimensions,
        children: Vec<LayoutNode>,
    },
    Text {
        node_id: NodeID,
        fragments: Vec<InlineFragment>,
    },
    AnonymousBlock {
        dimensions: Dimensions,
        children: Vec<LayoutNode>,
    },
}

impl LayoutNode {
    pub fn block(node_id: NodeID, dimensions: Dimensions, children: Vec<LayoutNode>) -> Self {
        LayoutNode::Block {
            node_id,
            dimensions,
            children,
        }
    }

    pub fn inline(node_id: NodeID, children: Vec<LayoutNode>) -> Self {
        LayoutNode::Inline { node_id, children }
    }

    pub fn inline_block(
        node_id: NodeID,
        dimensions: Dimensions,
        children: Vec<LayoutNode>,
    ) -> Self {
        LayoutNode::InlineBlock {
            node_id,
            dimensions,
            children,
        }
    }

    pub fn text(node_id: NodeID, fragments: Vec<InlineFragment>) -> Self {
        LayoutNode::Text { node_id, fragments }
    }

    pub fn anonymous_block(dimensions: Dimensions, children: Vec<LayoutNode>) -> Self {
        LayoutNode::AnonymousBlock {
            dimensions,
            children,
        }
    }
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            LayoutNode::Block { .. }
                | LayoutNode::InlineBlock { .. }
                | LayoutNode::AnonymousBlock { .. }
        )
    }

    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            LayoutNode::Inline { .. } | LayoutNode::InlineBlock { .. } | LayoutNode::Text { .. }
        )
    }
}
