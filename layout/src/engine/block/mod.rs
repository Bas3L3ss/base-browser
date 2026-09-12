use css::{ Length, LengthContext, MaxLength, MinLength };

use crate::{
    Dimensions,
    LayoutNode,
    StyleComponents,
    block::{ horizontal::HorizontalEdges, vertical::VerticalEdges },
};
pub mod horizontal;
pub mod vertical;
pub struct BlockEngine;

impl BlockEngine {
    pub(crate) fn calculate_vertical(
        node: &mut Dimensions,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext
    ) {
        match &style.height {
            Length::Auto => {
                node.is_height_definite = false;
            }
            Length::Percent(_) if !parent.is_height_definite => {
                node.is_height_definite = false;
            }
            _ => {
                let mut h = style.height.to_f32(parent.content.height, None, length_context);

                h = Self::clamp_height(h, style, parent, length_context);

                node.content.height = h;
                node.is_height_definite = true;
            }
        }
        let vertical = VerticalEdges::resolve(parent, style, length_context);
        node.margin.top = vertical.margin_top;
        node.margin.bottom = vertical.margin_bottom;
        node.padding.top = vertical.padding_top;
        node.padding.bottom = vertical.padding_bottom;
        node.border.top = vertical.border_top;
        node.border.bottom = vertical.border_bottom;
    }

    pub(crate) fn calculate_x(node: &mut Dimensions, parent: &Dimensions) {
        node.content.x = parent.content.x + node.padding.left;
    }

    pub(crate) fn calculate_y(node: &mut Dimensions, cursor_y: f32) {
        node.content.y = cursor_y + node.padding.top;
    }

    pub(crate) fn calculate_horizontal(
        node: &mut Dimensions,
        parent: &Dimensions,
        style: &StyleComponents,
        length_context: &LengthContext
    ) {
        let edges = HorizontalEdges::resolve(parent, style, length_context);

        let w = match style.width {
            Length::Auto => None,
            _ => {
                let w = style.width.to_f32(parent.content.width, None, &length_context);
                Some(w)
            }
        };

        node.border.left = edges.border_left;
        node.padding.left = edges.padding_left;
        node.border.right = edges.border_right;
        node.padding.right = edges.padding_right;

        let min_width = match &style.min_width {
            MinLength::Auto => 0.0,
            MinLength::Length(length) => length.to_f32(parent.content.width, None, length_context),
        };
        let max_width = (
            match &style.max_width {
                MaxLength::None => f32::INFINITY,
                MaxLength::Length(length) =>
                    length.to_f32(parent.content.width, None, length_context),
            }
        ).max(min_width);

        let mut width = match w {
            None => {
                let ml = edges.margin_left.unwrap_or(0.0);
                let mr = edges.margin_right.unwrap_or(0.0);
                let w = (
                    parent.content.width -
                    node.border.horizontal_sum() -
                    node.padding.horizontal_sum() -
                    ml -
                    mr
                ).max(0.0);

                node.margin.left = ml;
                node.margin.right = mr;

                w
            }
            Some(w) => {
                let available =
                    parent.content.width -
                    node.border.horizontal_sum() -
                    node.padding.horizontal_sum();

                let (ml, mr) = match (edges.margin_left, edges.margin_right) {
                    (None, None) => {
                        // both margins auto, distribute space equally
                        let remain = available - w;
                        let margin = remain / 2.0;
                        (margin, margin)
                    }
                    (None, Some(mr)) => {
                        // left margin auto, right margin specified
                        let right = mr;
                        let left = available - w - right;
                        (left, right)
                    }
                    (Some(ml), None) => {
                        // right margin auto, left margin specified
                        let left = ml;
                        let right = available - w - left;
                        (left, right)
                    }
                    (Some(ml), Some(mr)) => {
                        // no auto, both margins specified
                        (ml, mr)
                    }
                };

                node.margin.left = ml;
                node.margin.right = mr;

                w
            }
        };

        width = width.max(min_width).min(max_width);

        node.content.width = width;

        // let layout = HorizontalLayout::solve_horizontal_constraints(
        //     parent.content.width,
        //     &edges,
        //     &specified,
        //     min_width,
        //     max_width
        // );
        // layout.apply(node, &edges);
    }

    pub(crate) fn finalize_cursor_y(node: &mut Dimensions, cursor_y: &mut f32) {
        *cursor_y +=
            node.margin.bottom + node.padding.bottom + node.content.height + node.border.bottom;
    }

    pub(crate) fn resolve_auto_height(children: &mut Vec<LayoutNode>, dimensions: &mut Dimensions) {
        dimensions.content.height = children.iter().map(BlockEngine::margin_box_height).sum();

        dimensions.is_height_definite = true;
    }

    pub(crate) fn margin_box_height(node: &LayoutNode) -> f32 {
        match node {
            | LayoutNode::Block { dimensions, .. }
            | LayoutNode::InlineBlock { dimensions, .. }
            | LayoutNode::AnonymousBlock { dimensions, .. } =>
                dimensions.margin.top + dimensions.margin.bottom + dimensions.content.height,
            LayoutNode::Text { height, .. } => *height,
            LayoutNode::Inline { .. } => 0.0,
        }
    }

    pub(crate) fn finalize_height(
        children: &mut Vec<LayoutNode>,
        dimensions: &mut Dimensions,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext,
        cursor_y: &mut f32
    ) {
        if !dimensions.is_height_definite {
            Self::resolve_auto_height(children, dimensions);

            dimensions.content.height = Self::clamp_height(
                dimensions.content.height,
                style,
                parent,
                length_context
            );
        }

        Self::finalize_cursor_y(dimensions, cursor_y);
    }

    pub(crate) fn clamp_height(
        h: f32,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext
    ) -> f32 {
        let min_height = match &style.min_height {
            MinLength::Auto => 0.0,
            MinLength::Length(length) => {
                match length {
                    Length::Percent(_) if !parent.is_height_definite => { 0.0 }
                    _ => { length.to_f32(parent.content.height, None, length_context) }
                }
            }
        };
        let max_height = (
            match &style.max_height {
                MaxLength::None => f32::INFINITY,
                MaxLength::Length(length) =>
                    match length {
                        Length::Percent(_) if !parent.is_height_definite => f32::INFINITY,
                        _ => length.to_f32(parent.content.height, None, length_context),
                    }
            }
        ).max(min_height);

        h.max(min_height).min(max_height)
    }
}
