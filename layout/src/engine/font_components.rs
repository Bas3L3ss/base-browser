use css::{ComputedStyle, WhiteSpaceBehavior};
use font::{FontStretch, FontStyle, FontWeight};

pub struct FontComponents {
    pub families: Vec<String>,
    pub weight: FontWeight,
    pub stretch: FontStretch,
    pub style: FontStyle,
    pub whitespace: WhiteSpaceBehavior,
}

impl FontComponents {
    pub fn new(style: &ComputedStyle, parent_weight: u16) -> Self {
        let families = style.font_family();
        let weight = FontWeight(
            style
                .font_weight()
                .unwrap_or(css::FontWeight::Normal)
                .to_u16(parent_weight),
        );
        let stretch = FontStretch(style.font_stretch().to_f32());
        let f_style = style.font_style();
        let font_style =
            FontStyle::from_name_angle(f_style.as_str(), f_style.to_angle().unwrap_or(14.0));

        let whitespace = style.white_space();

        Self {
            families,
            weight,
            stretch,
            style: font_style,
            whitespace,
        }
    }
}
