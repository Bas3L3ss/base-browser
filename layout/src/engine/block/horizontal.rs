use css::{BoxSizing, Length, LengthContext};

use crate::{Dimensions, StyleComponents};

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
    pub(crate) fn resolve_width(available: f32, specified: &HorizontalSpecified) -> f32 {
        match specified.width {
            Some(w) => w,
            None => {
                let ml = specified.margin_left.unwrap_or(0.0);
                let mr = specified.margin_right.unwrap_or(0.0);

                (available - ml - mr).max(0.0)
            }
        }
    }

    pub(crate) fn clamp_width(width: f32, min: f32, max: f32) -> f32 {
        width.max(min).min(max)
    }

    pub(crate) fn resolve_margins(
        available: f32,
        width: f32,
        specified: &HorizontalSpecified,
    ) -> (f32, f32) {
        match (specified.margin_left, specified.margin_right) {
            (None, None) => {
                // both margins auto, distribute space equally
                let remain = available - width;
                let margin = remain / 2.0;
                (margin, margin)
            }
            (None, Some(mr)) => {
                // left margin auto, right margin specified
                let right = mr;
                let left = available - width - right;
                (left, right)
            }
            (Some(ml), None) => {
                // right margin auto, left margin specified
                let left = ml;
                let right = available - width - left;
                (left, right)
            }
            (Some(ml), Some(mr)) => {
                // no auto, both margins specified
                (ml, mr)
            }
        }
    }

    pub(crate) fn solve_horizontal_constraints(
        containing_width: f32,
        edges: &HorizontalEdges,
        specified: &HorizontalSpecified,
        min: f32,
        max: f32,
    ) -> Self {
        let available = containing_width
            - edges.padding_left
            - edges.padding_right
            - edges.border_left
            - edges.border_right;

        let resolved_width = Self::resolve_width(available, specified);

        let resolved_width = Self::clamp_width(resolved_width, min, max);

        let (resolved_margin_left, resolved_margin_right) =
            Self::resolve_margins(available, resolved_width, specified);

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
        node: &mut Dimensions,
        edges: &HorizontalEdges,
        margin_left: f32,
        margin_right: f32,
    ) {
        node.padding_edges.left = edges.padding_left;
        node.padding_edges.right = edges.padding_right;

        node.border_edges.left = edges.border_left;
        node.border_edges.right = edges.border_right;

        node.margin_edges.left = margin_left;
        node.margin_edges.right = margin_right;
    }

    pub(crate) fn apply(&self, node: &mut Dimensions, edges: &HorizontalEdges) {
        node.content.width = self.content_width();
        node.padding.width = self.padding_width(edges);
        node.border.width = self.border_width(edges);
        node.margin.width = self.margin_width(edges);

        Self::populate_edges(node, edges, self.margin_left, self.margin_right);
    }
}
