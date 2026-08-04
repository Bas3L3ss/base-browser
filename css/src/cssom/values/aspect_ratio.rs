use crate::CssValue;

#[derive(Debug, Clone, PartialEq)]
pub enum AspectRatio {
    Auto,
    Ratio(f32, f32),
}

impl AspectRatio {
    pub fn from_css(value: &CssValue) -> Self {
        // CSS aspect-ratio syntax: "aspect-ratio: <number> / <number>;"
        // Parse pattern: [number, "/", number]
        let comps = &value.components;
        if comps.len() == 3 {
            if let (Some(a), Some(b)) = (comps[0].as_number(), comps[2].as_number()) {
                if let Some(div) = comps[1].as_delim() {
                    if div == '/' {
                        return AspectRatio::Ratio(a, b);
                    }
                }
            }
        }
        // Also permit 'auto'
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            if ident == "auto" {
                return AspectRatio::Auto;
            }
        }
        // Fallback
        AspectRatio::Auto
    }
}
