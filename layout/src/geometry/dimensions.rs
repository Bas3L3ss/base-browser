use css::Edges;

use crate::Rect;

/// Box geometry for layout and rendering.
///
/// Layout establishes the content rectangle and edge thicknesses; padding, border, and margin
/// boxes are nested expansions of the content box (not independent positioned layers).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: Edges<f32>,
    pub border: Edges<f32>,
    pub margin: Edges<f32>,

    pub is_height_definite: bool,
}

impl Dimensions {
    pub fn padding_box(&self) -> Rect {
        Rect {
            x: self.content.x - self.padding.left,
            y: self.content.y - self.padding.top,
            width: self.content.width + self.padding.left + self.padding.right,
            height: self.content.height + self.padding.top + self.padding.bottom,
        }
    }

    pub fn border_box(&self) -> Rect {
        let padding = self.padding_box();
        Rect {
            x: padding.x - self.border.left,
            y: padding.y - self.border.top,
            width: padding.width + self.border.left + self.border.right,
            height: padding.height + self.border.top + self.border.bottom,
        }
    }

    pub fn margin_box(&self) -> Rect {
        let border = self.border_box();
        Rect {
            x: border.x - self.margin.left,
            y: border.y - self.margin.top,
            width: border.width + self.margin.left + self.margin.right,
            height: border.height + self.margin.top + self.margin.bottom,
        }
    }

    /// Insets from the containing block's content edge to this box's content edge.
    pub fn content_inset_from_parent(&self) -> Edges<f32> {
        Edges {
            top: self.margin.top + self.border.top + self.padding.top,
            right: self.margin.right + self.border.right + self.padding.right,
            bottom: self.margin.bottom + self.border.bottom + self.padding.bottom,
            left: self.margin.left + self.border.left + self.padding.left,
        }
    }
}
