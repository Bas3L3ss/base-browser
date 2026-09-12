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
    pub padding: Edges<f32>,
    pub border: Edges<f32>,
    pub margin: Edges<f32>,

    pub is_height_definite: bool,
    pub is_border_box: bool,
    pub reserved_border_left: f32,
    pub reserved_padding_left: f32,
}
