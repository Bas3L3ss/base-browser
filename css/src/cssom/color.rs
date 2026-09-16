use crate::{ CssValue };

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Rgb(f32, f32, f32),
    Rgba(f32, f32, f32, f32),
    Hsl(f32, f32, f32),
    Hsla(f32, f32, f32, f32),
    Hex(String),
    Named(String),
}

impl Color {
    pub fn from_css(value: &CssValue) -> [f32; 4] {
        let s = value.components.first().map(|comp| { comp.to_string() }); // TODO: AVOID THIS

        if let Some(css_string) = s {
            if let Ok(color) = ::csscolorparser::parse(&css_string) {
                let [r, g, b, a] = color.to_rgba8();
                // Convert sRGB to 0.0-1.0, then linear
                let r_lin = Self::srgb_to_linear((r as f32) / 255.0);
                let g_lin = Self::srgb_to_linear((g as f32) / 255.0);
                let b_lin = Self::srgb_to_linear((b as f32) / 255.0);
                let a_lin = (a as f32) / 255.0;
                return [r_lin, g_lin, b_lin, a_lin];
            }
        }

        [0.0, 0.0, 0.0, 0.0]
    }
    fn srgb_to_linear(x: f32) -> f32 {
        if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
    }
}
