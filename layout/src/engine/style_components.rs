use css::{BoxSizing, ComputedStyle, Edges, Length};

pub(crate) struct StyleComponents {
    pub(crate) box_sizing: BoxSizing,
    pub(crate) margin: Edges<Length>,
    pub(crate) padding: Edges<Length>,
    pub(crate) border: Edges<Length>,
    pub(crate) width: Length,
    pub(crate) height: Length,
    pub(crate) min_width: Length,
    pub(crate) max_width: Length,
    pub(crate) min_height: Length,
    pub(crate) max_height: Length,
    pub(crate) offset: Edges<Length>,
    pub(crate) font_size: Option<Length>,
}

impl StyleComponents {
    pub(crate) fn new(style: &ComputedStyle) -> Self {
        let (min_width, max_width) = style.min_max_width();
        let (min_height, max_height) = style.min_max_height();
        let (left, top, right, bottom) = style.offset();
        let font_size = style.font_size();
        Self {
            box_sizing: style.box_sizing(),
            margin: style.margin(),
            padding: style.padding(),
            border: style.border_width(),
            width: style.width(),
            height: style.height(),
            min_width: min_width,
            max_width: max_width,
            min_height: min_height,
            max_height: max_height,
            offset: Edges {
                left: left,
                top: top,
                right: right,
                bottom: bottom,
            },
            font_size: font_size,
        }
    }
}
