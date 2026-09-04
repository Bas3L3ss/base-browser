use crate::{FontStretch, FontStyle, FontWeight};

pub struct FontRequest<'a> {
    pub families: &'a [String],
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
}

impl<'a> FontRequest<'a> {
    pub fn new(
        families: &'a [String],
        weight: FontWeight,
        style: FontStyle,
        stretch: FontStretch,
    ) -> Self {
        Self {
            families,
            weight,
            style,
            stretch,
        }
    }
}
