use std::todo;

use css::{ BoxSizing, Length, LengthContext, MaxLength, MinLength };

use crate::{
    Dimensions,
    LayoutNode,
    StyleComponents,
    block::horizontal::{ HorizontalEdges, HorizontalLayout, HorizontalSpecified },
};

pub mod horizontal;

pub struct BlockEngine;

impl BlockEngine {
    pub(crate) fn calculate_y(node: &mut Dimensions, cursor_y: f32) {
        node.margin.y = cursor_y;

        node.border.y = node.margin.y + node.margin_edges.top;

        node.padding.y = node.border.y + node.border_edges.top;

        node.content.y = node.padding.y + node.padding_edges.top;
    }

    pub(crate) fn resolve_specified_height(
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

                if style.box_sizing == BoxSizing::BorderBox {
                    h -= node.padding_edges.top + node.padding_edges.bottom;
                    h -= node.border_edges.top + node.border_edges.bottom;
                    h = h.max(0.0);
                }

                h = Self::clamp_height(h, style, parent, length_context);

                node.content.height = h;
                node.is_height_definite = true;
            }
        }
    }

    pub(crate) fn calculate_x(node: &mut Dimensions, parent: &Dimensions) {
        node.margin.x = parent.content.x;

        node.border.x = node.margin.x + node.margin_edges.left;

        node.padding.x = node.border.x + node.border_edges.left;

        node.content.x = node.padding.x + node.padding_edges.left;
    }

    pub(crate) fn calculate_width(
        node: &mut Dimensions,
        parent: &Dimensions,
        style: &StyleComponents,
        length_context: &LengthContext
    ) {
        let edges = HorizontalEdges::resolve_horizontal_edges(parent, style, length_context);

        let specified = HorizontalSpecified::resolve_horizontal_specified(
            parent,
            style,
            length_context,
            &edges
        );

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

        let layout = HorizontalLayout::solve_horizontal_constraints(
            parent.content.width,
            &edges,
            &specified,
            min_width,
            max_width
        );
        layout.apply(node, &edges);
    }

    pub(crate) fn finalize_box_height(node: &mut Dimensions) {
        node.padding.height =
            node.content.height + node.padding_edges.top + node.padding_edges.bottom;

        node.border.height = node.padding.height + node.border_edges.top + node.border_edges.bottom;

        node.margin.height = node.border.height + node.margin_edges.top + node.margin_edges.bottom;
    }

    pub(crate) fn resolve_auto_height(children: &mut Vec<LayoutNode>, dimensions: &mut Dimensions) {
        dimensions.content.height = children.iter().map(BlockEngine::margin_box_height).sum();

        dimensions.is_height_definite = true;
    }

    pub(crate) fn margin_box_height(node: &LayoutNode) -> f32 {
        match node {
            | LayoutNode::Block { dimensions, .. }
            | LayoutNode::InlineBlock { dimensions, .. }
            | LayoutNode::AnonymousBlock { dimensions, .. } => dimensions.margin.height,
            d => {
                dbg!(d);
                todo!()
            }
        }
    }

    pub(crate) fn finalize_height(
        children: &mut Vec<LayoutNode>,
        dimensions: &mut Dimensions,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext
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

        Self::finalize_box_height(dimensions);
    }

    pub(crate) fn clamp_height(
        h: f32,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext
    ) -> f32 {
        let min_height = match &style.min_height {
            MinLength::Auto => 0.0,
            MinLength::Length(length) =>
                match length {
                    Length::Percent(_) if !parent.is_height_definite => 0.0,
                    _ => length.to_f32(parent.content.height, None, length_context),
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
