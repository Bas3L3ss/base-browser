use css::ComputedStyle;
use font::{FontStretch, FontStyle, FontWeight};

#[derive(Debug, Clone, PartialEq)]
pub struct FontComponents {
    pub families: Vec<String>,
    pub weight: FontWeight,
    pub stretch: FontStretch,
    pub style: FontStyle,
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
        let style =
            FontStyle::from_name_angle(f_style.as_str(), f_style.to_angle().unwrap_or(14.0));

        Self {
            families,
            weight,
            stretch,
            style,
        }
    }
}
