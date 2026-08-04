use crate::{EdgeSizes, Rect};

/// The `Dimensions` struct represents the rectangular areas of a CSS box model entity:
/// - `content` is the area where the box's content is rendered.
/// - `padding` is the space between the content and the border.
/// - `border` is the thickness of the border around the padding.
/// - `margin` is the space outside the border, separating the element from others.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dimensions {
    /// The content area (inner rectangle).
    pub content: Rect,
    /// Padding around the content.
    pub padding: EdgeSizes,
    /// Border thickness around the padding.
    pub border: EdgeSizes,
    /// Margin area outside the border.
    pub margin: EdgeSizes,
}
