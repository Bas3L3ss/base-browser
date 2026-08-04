use css::Edges;

use crate::Rect;

/// The `Dimensions` struct represents the rectangular areas of a CSS box model entity:
/// - `content` is the area where the box's content is rendered.
/// - `padding` is the space between the content and the border.
/// - `border` is the thickness of the border around the padding.
/// - `margin` is the space outside the border, separating the element from others.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: Rect,
    pub border: Rect,
    pub margin: Rect,

    pub padding_edges: Edges<f32>,
    pub border_edges: Edges<f32>,
    pub margin_edges: Edges<f32>,
}
