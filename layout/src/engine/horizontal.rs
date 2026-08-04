use css::{BoxSizing, Length, LengthContext};

use crate::{Dimensions, LayoutNode, StyleComponents};

pub(crate) struct HorizontalEdges {
    pub(crate) padding_left: f32,
    pub(crate) padding_right: f32,

    pub(crate) border_left: f32,
    pub(crate) border_right: f32,
}

impl HorizontalEdges {
    pub(crate) fn resolve_horizontal_edges(
        parent: &Dimensions,
        style: &StyleComponents,
        length_context: &LengthContext,
    ) -> Self {
        let padding_left = style
            .padding
            .left
            .to_f32(parent.content.width, None, &length_context);
        let padding_right = style
            .padding
            .right
            .to_f32(parent.content.width, None, &length_context);

        let border_left = style
            .border
            .left
            .to_f32(parent.content.width, None, &length_context);
        let border_right = style
            .border
            .right
            .to_f32(parent.content.width, None, &length_context);

        HorizontalEdges {
            padding_left,
            padding_right,
            border_left,
            border_right,
        }
    }
}

pub(crate) struct HorizontalSpecified {
    width: Option<f32>,
    margin_left: Option<f32>,
    margin_right: Option<f32>,
}
impl HorizontalSpecified {
    pub(crate) fn resolve_horizontal_specified(
        parent: &Dimensions,
        style: &StyleComponents,
        length_context: &LengthContext,
        edges: &HorizontalEdges,
    ) -> Self {
        let margin_left = match style.margin.left {
            Length::Auto => None,
            _ => Some(
                style
                    .margin
                    .left
                    .to_f32(parent.content.width, None, &length_context),
            ),
        };

        let margin_right = match style.margin.right {
            Length::Auto => None,
            _ => Some(
                style
                    .margin
                    .right
                    .to_f32(parent.content.width, None, &length_context),
            ),
        };

        let width = match style.width {
            Length::Auto => None,
            _ => {
                let mut w = style
                    .width
                    .to_f32(parent.content.width, None, &length_context);

                if style.box_sizing == BoxSizing::BorderBox {
                    w -= edges.padding_left;
                    w -= edges.padding_right;
                    w -= edges.border_left;
                    w -= edges.border_right;
                    w = w.max(0.0);
                }

                Some(w)
            }
        };
        HorizontalSpecified {
            width,
            margin_left,
            margin_right,
        }
    }
}

pub(crate) struct HorizontalLayout {
    pub(crate) width: f32,
    pub(crate) margin_left: f32,
    pub(crate) margin_right: f32,
}

impl HorizontalLayout {
    pub(crate) fn solve_horizontal_constraints(
        containing_width: f32,
        edges: &HorizontalEdges,
        specified: &HorizontalSpecified,
    ) -> Self {
        let available = containing_width
            - edges.padding_left
            - edges.padding_right
            - edges.border_left
            - edges.border_right;

        let resolved_width;
        let resolved_margin_left;
        let resolved_margin_right;

        match (
            specified.width,
            specified.margin_left,
            specified.margin_right,
        ) {
            // width:auto
            (None, ml, mr) => {
                let ml = ml.unwrap_or(0.0);
                let mr = mr.unwrap_or(0.0);

                resolved_margin_left = ml;
                resolved_margin_right = mr;

                resolved_width = (available - ml - mr).max(0.0);
            }

            // fixed width, left margin auto
            (Some(w), None, Some(mr)) => {
                resolved_width = w;
                resolved_margin_right = mr;
                resolved_margin_left = available - w - mr;
            }

            // fixed width, right margin auto
            (Some(w), Some(ml), None) => {
                resolved_width = w;
                resolved_margin_left = ml;
                resolved_margin_right = available - w - ml;
            }

            // fixed width, both margins auto
            (Some(w), None, None) => {
                resolved_width = w;

                let remain = available - w;

                resolved_margin_left = remain / 2.0;
                resolved_margin_right = remain / 2.0;
            }

            // nothing auto
            (Some(w), Some(ml), Some(_)) => {
                resolved_width = w;
                resolved_margin_left = ml;

                // CSS 2.1 LTR:
                // any leftover (positive or negative) goes to the right margin
                resolved_margin_right = available - w - ml;
            }
        };

        HorizontalLayout {
            width: resolved_width,
            margin_left: resolved_margin_left,
            margin_right: resolved_margin_right,
        }
    }

    pub fn content_width(&self) -> f32 {
        self.width
    }

    pub fn padding_width(&self, edges: &HorizontalEdges) -> f32 {
        self.content_width() + edges.padding_left + edges.padding_right
    }

    pub fn border_width(&self, edges: &HorizontalEdges) -> f32 {
        self.padding_width(edges) + edges.border_left + edges.border_right
    }

    pub fn margin_width(&self, edges: &HorizontalEdges) -> f32 {
        self.border_width(edges) + self.margin_left + self.margin_right
    }

    fn populate_edges(
        node: &mut LayoutNode,
        edges: &HorizontalEdges,
        margin_left: f32,
        margin_right: f32,
    ) {
        node.dimensions.padding_edges.left = edges.padding_left;
        node.dimensions.padding_edges.right = edges.padding_right;

        node.dimensions.border_edges.left = edges.border_left;
        node.dimensions.border_edges.right = edges.border_right;

        node.dimensions.margin_edges.left = margin_left;
        node.dimensions.margin_edges.right = margin_right;
    }

    pub(crate) fn apply(&self, node: &mut LayoutNode, edges: &HorizontalEdges) {
        node.dimensions.content.width = self.content_width();
        node.dimensions.padding.width = self.padding_width(edges);
        node.dimensions.border.width = self.border_width(edges);
        node.dimensions.margin.width = self.margin_width(edges);

        Self::populate_edges(node, edges, self.margin_left, self.margin_right);
    }
}
