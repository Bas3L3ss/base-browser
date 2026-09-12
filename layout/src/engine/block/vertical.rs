use css::{ Length, LengthContext };

use crate::{ Dimensions, StyleComponents };

pub(crate) struct VerticalEdges {
    pub(crate) padding_top: f32,
    pub(crate) padding_bottom: f32,
    pub(crate) border_top: f32,
    pub(crate) border_bottom: f32,
    pub(crate) margin_top: f32,
    pub(crate) margin_bottom: f32,
}

impl VerticalEdges {
    pub(crate) fn resolve(
        parent: &Dimensions,
        style: &StyleComponents,
        length_context: &LengthContext
    ) -> Self {
        let padding_top = style.padding.top.to_f32(parent.content.width, None, &length_context);
        let padding_bottom = style.padding.bottom.to_f32(
            parent.content.width,
            None,
            &length_context
        );

        let border_top = style.border.top.to_f32(parent.content.width, None, &length_context);
        let border_bottom = style.border.bottom.to_f32(parent.content.width, None, &length_context);
        let margin_top = match style.margin.top {
            Length::Auto => 0.0,
            _ => style.margin.top.to_f32(parent.content.width, None, &length_context),
        };

        let margin_bottom = match style.margin.bottom {
            Length::Auto => 0.0,
            _ => style.margin.bottom.to_f32(parent.content.width, None, &length_context),
        };

        VerticalEdges {
            padding_top,
            padding_bottom,
            border_top,
            border_bottom,
            margin_top,
            margin_bottom,
        }
    }
}
