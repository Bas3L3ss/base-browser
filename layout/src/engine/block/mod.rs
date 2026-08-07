use css::{BoxSizing, Length, LengthContext, MaxLength, MinLength};

use crate::{
    Dimensions, LayoutNode, StyleComponents,
    block::horizontal::{HorizontalEdges, HorizontalLayout, HorizontalSpecified},
};

pub mod horizontal;

pub struct BlockEngine;

impl BlockEngine {
    pub(crate) fn calculate_y(node: &mut LayoutNode, cursor_y: f32) {
        node.dimensions.margin.y = cursor_y;

        node.dimensions.border.y = node.dimensions.margin.y + node.dimensions.margin_edges.top;

        node.dimensions.padding.y = node.dimensions.border.y + node.dimensions.border_edges.top;

        node.dimensions.content.y = node.dimensions.padding.y + node.dimensions.padding_edges.top;
    }

    pub(crate) fn resolve_specified_height(
        node: &mut LayoutNode,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext,
    ) {
        match &style.height {
            Length::Auto => {
                node.dimensions.is_height_definite = false;
            }
            Length::Percent(_) if !parent.is_height_definite => {
                node.dimensions.is_height_definite = false;
            }
            _ => {
                let mut h = style
                    .height
                    .to_f32(parent.content.height, None, length_context);

                if style.box_sizing == BoxSizing::BorderBox {
                    h -= node.dimensions.padding_edges.top + node.dimensions.padding_edges.bottom;
                    h -= node.dimensions.border_edges.top + node.dimensions.border_edges.bottom;
                    h = h.max(0.0);
                }

                h = Self::clamp_height(h, style, parent, length_context);

                node.dimensions.content.height = h;
                node.dimensions.is_height_definite = true;
            }
        }
    }

    pub(crate) fn calculate_x(node: &mut LayoutNode, parent: &Dimensions) {
        node.dimensions.margin.x = parent.content.x;

        node.dimensions.border.x = node.dimensions.margin.x + node.dimensions.margin_edges.left;

        node.dimensions.padding.x = node.dimensions.border.x + node.dimensions.border_edges.left;

        node.dimensions.content.x = node.dimensions.padding.x + node.dimensions.padding_edges.left;
    }

    pub(crate) fn calculate_width(
        node: &mut LayoutNode,
        parent: &Dimensions,
        style: &StyleComponents,
        length_context: &LengthContext,
    ) {
        let edges = HorizontalEdges::resolve_horizontal_edges(parent, style, length_context);

        let specified = HorizontalSpecified::resolve_horizontal_specified(
            parent,
            style,
            length_context,
            &edges,
        );

        let min_width = match &style.min_width {
            MinLength::Auto => 0.0,
            MinLength::Length(length) => length.to_f32(parent.content.width, None, length_context),
        };
        let max_width = match &style.max_width {
            MaxLength::None => f32::INFINITY,
            MaxLength::Length(length) => length.to_f32(parent.content.width, None, length_context),
        }
        .max(min_width);

        let layout = HorizontalLayout::solve_horizontal_constraints(
            parent.content.width,
            &edges,
            &specified,
            min_width,
            max_width,
        );
        layout.apply(node, &edges);
    }

    pub(crate) fn finalize_box_height(node: &mut LayoutNode) {
        node.dimensions.padding.height = node.dimensions.content.height
            + node.dimensions.padding_edges.top
            + node.dimensions.padding_edges.bottom;

        node.dimensions.border.height = node.dimensions.padding.height
            + node.dimensions.border_edges.top
            + node.dimensions.border_edges.bottom;

        node.dimensions.margin.height = node.dimensions.border.height
            + node.dimensions.margin_edges.top
            + node.dimensions.margin_edges.bottom;
    }

    pub(crate) fn resolve_auto_height(node: &mut LayoutNode) {
        node.dimensions.content.height = node
            .children
            .iter()
            .map(|child| child.dimensions.margin.height)
            .sum();

        node.dimensions.is_height_definite = true;
    }

    pub(crate) fn finalize_height(
        node: &mut LayoutNode,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext,
    ) {
        if !node.dimensions.is_height_definite {
            Self::resolve_auto_height(node);

            node.dimensions.content.height = Self::clamp_height(
                node.dimensions.content.height,
                style,
                parent,
                length_context,
            );
        }

        Self::finalize_box_height(node);
    }

    pub(crate) fn clamp_height(
        h: f32,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext,
    ) -> f32 {
        let min_height = match &style.min_height {
            MinLength::Auto => 0.0,
            MinLength::Length(length) => match length {
                Length::Percent(_) if !parent.is_height_definite => 0.0,
                _ => length.to_f32(parent.content.height, None, length_context),
            },
        };
        let max_height = match &style.max_height {
            MaxLength::None => f32::INFINITY,
            MaxLength::Length(length) => match length {
                Length::Percent(_) if !parent.is_height_definite => f32::INFINITY,
                _ => length.to_f32(parent.content.height, None, length_context),
            },
        }
        .max(min_height);

        h.max(min_height).min(max_height)
    }
}
