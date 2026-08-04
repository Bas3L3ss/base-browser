use std::todo;

use css::ComputedStyle;

use crate::{BoxType, Dimensions, LayoutNode, LayoutSource, LayoutTree};

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn layout(
        tree: &mut LayoutTree,
        viewport: Dimensions,
        computed_style: &[Option<ComputedStyle>],
    ) {
        if let Some(root) = tree.root.as_mut() {
            Self::layout_node(root, &viewport, computed_style);
        }
    }
    fn layout_node(
        node: &mut LayoutNode,
        containing_block: &Dimensions,
        computed_style: &[Option<ComputedStyle>],
    ) {
        match node.box_type {
            BoxType::Block => Self::layout_block(node, containing_block, computed_style),
            BoxType::Anonymous => Self::layout_block(node, containing_block, computed_style),

            BoxType::Inline => todo!(),
            BoxType::Text => todo!(),

            _ => todo!(),
        }
    }
    fn layout_block(
        node: &mut LayoutNode,
        parent: &Dimensions,
        computed_style: &[Option<ComputedStyle>],
    ) {
        match node.source {
            LayoutSource::Anonymous => {
                todo!()
            }
            LayoutSource::DomNode(node_id) => {
                let style = computed_style[node_id]
                    .as_ref()
                    .expect("Computed style for node is missing");

                let margin = style.margin();
                let padding = style.padding();
                let border = style.border_width();

                let width = style.width();
                let height = style.height();

                let box_sizing = style.box_sizing();

                let (min_width, max_width) = style.min_max_width();
                let (min_height, max_height) = style.min_max_height();
            }
        }
    }
}
