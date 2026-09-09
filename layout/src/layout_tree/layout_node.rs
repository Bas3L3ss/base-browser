use std::{ matches, vec };

use crate::{ Dimensions, text::inline_fragment::InlineFragment };
use html::NodeID;

#[derive(Clone)]
pub enum LayoutNode {
    Block {
        node_id: NodeID,
        dimensions: Dimensions,
        children: Vec<LayoutNode>,
    },
    Inline {
        node_id: NodeID,
        children: Vec<LayoutNode>,
        boxes: Vec<Dimensions>,
    },
    InlineBlock {
        node_id: NodeID,
        dimensions: Dimensions,
        children: Vec<LayoutNode>,
    },
    Text {
        node_id: NodeID,
        fragments: Vec<InlineFragment>,
        height: f32,
    },
    AnonymousBlock {
        dimensions: Dimensions,
        children: Vec<LayoutNode>,
    },
}

impl std::fmt::Debug for LayoutNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutNode::Block { node_id, dimensions, .. } => {
                f.debug_struct("Block")
                    .field("node_id", node_id)
                    .field("dimensions", dimensions)
                    .finish()
            }
            LayoutNode::Inline { node_id, boxes, .. } => {
                f.debug_struct("Inline").field("node_id", node_id).field("boxes", boxes).finish()
            }
            LayoutNode::InlineBlock { node_id, dimensions, .. } => {
                f.debug_struct("InlineBlock")
                    .field("node_id", node_id)
                    .field("dimensions", dimensions)
                    .finish()
            }
            LayoutNode::Text { node_id, fragments, height } => {
                f.debug_struct("Text")
                    .field("node_id", node_id)
                    .field("fragments", fragments)
                    .field("height", height)
                    .finish()
            }
            LayoutNode::AnonymousBlock { dimensions, .. } => {
                f.debug_struct("AnonymousBlock").field("dimensions", dimensions).finish()
            }
        }
    }
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
        LayoutNode::Inline {
            node_id,
            children,
            boxes: vec![],
        }
    }

    pub fn inline_block(
        node_id: NodeID,
        dimensions: Dimensions,
        children: Vec<LayoutNode>
    ) -> Self {
        LayoutNode::InlineBlock {
            node_id,
            dimensions,
            children,
        }
    }

    pub fn text(node_id: NodeID) -> Self {
        LayoutNode::Text {
            node_id,
            fragments: vec![],
            height: 0.0,
        }
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
            LayoutNode::Block { .. } |
                LayoutNode::InlineBlock { .. } |
                LayoutNode::AnonymousBlock { .. }
        )
    }

    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            LayoutNode::Inline { .. } | LayoutNode::InlineBlock { .. } | LayoutNode::Text { .. }
        )
    }

    pub fn node_id(&self) -> Option<NodeID> {
        match self {
            | LayoutNode::Block { node_id, .. }
            | LayoutNode::Inline { node_id, .. }
            | LayoutNode::InlineBlock { node_id, .. }
            | LayoutNode::Text { node_id, .. } => Some(*node_id),
            LayoutNode::AnonymousBlock { .. } => None,
        }
    }

    pub fn children(&self) -> &[LayoutNode] {
        match self {
            | LayoutNode::Block { children, .. }
            | LayoutNode::Inline { children, .. }
            | LayoutNode::InlineBlock { children, .. }
            | LayoutNode::AnonymousBlock { children, .. } => children,
            LayoutNode::Text { .. } => &[],
        }
    }
    pub fn dimensions(&self) -> Option<&Dimensions> {
        match self {
            | LayoutNode::Block { dimensions, .. }
            | LayoutNode::InlineBlock { dimensions, .. }
            | LayoutNode::AnonymousBlock { dimensions, .. } => Some(dimensions),
            _ => None,
        }
    }

    pub fn text_fragments(&self) -> Option<&[crate::text::inline_fragment::InlineFragment]> {
        match self {
            LayoutNode::Text { fragments, .. } => Some(fragments),
            _ => None,
        }
    }
}
