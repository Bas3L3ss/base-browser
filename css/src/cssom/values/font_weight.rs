/// Represents the font-weight property in CSS.
///
/// - The named variants correspond to common font-weights as specified by the CSS standard.
/// - `Specified(u16)` is used for numeric font-weights not represented by a named variant.
/// - `Bolder` and `Lighter` are relative values, typically used in a parent-child context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin,           // 100
    ExtraLight,     // 200
    Light,          // 300
    Normal,         // 400
    Medium,         // 500
    SemiBold,       // 600
    Bold,           // 700
    ExtraBold,      // 800
    Black,          // 900
    Bolder,         // Relative keyword - needs context to resolve
    Lighter,        // Relative keyword - needs context to resolve
    Specified(u16), // Any numeric value (e.g. 350)
}

impl FontWeight {
    /// Creates a FontWeight from a CssValue, matching on identifiers and numbers.
    pub fn from_css(value: &crate::CssValue) -> Self {
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident {
                "thin" => FontWeight::Thin,
                "extralight" | "extra-light" | "ultralight" | "ultra-light" => {
                    FontWeight::ExtraLight
                }
                "light" => FontWeight::Light,
                "normal" => FontWeight::Normal,
                "medium" => FontWeight::Medium,
                "semibold" | "semi-bold" | "demibold" | "demi-bold" => FontWeight::SemiBold,
                "bold" => FontWeight::Bold,
                "extrabold" | "extra-bold" | "ultrabold" | "ultra-bold" => FontWeight::ExtraBold,
                "black" | "heavy" => FontWeight::Black,
                "bolder" => FontWeight::Bolder,
                "lighter" => FontWeight::Lighter,
                _ => {
                    // If identifier does not match known variant, fallback to Normal.
                    // Could be "unset", "inherit", etc.
                    FontWeight::Normal
                }
            }
        } else if let Some(num) = value.components.first().and_then(|c| c.as_number()) {
            match num as u16 {
                100 => FontWeight::Thin,
                200 => FontWeight::ExtraLight,
                300 => FontWeight::Light,
                400 => FontWeight::Normal,
                500 => FontWeight::Medium,
                600 => FontWeight::SemiBold,
                700 => FontWeight::Bold,
                800 => FontWeight::ExtraBold,
                900 => FontWeight::Black,
                n => FontWeight::Specified(n),
            }
        } else {
            FontWeight::Normal
        }
    }

    pub fn to_u16(&self, parent: u16) -> u16 {
        match *self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
            FontWeight::Bolder => Self::resolve_bolder(parent),
            FontWeight::Lighter => Self::resolve_lighter(parent),
            FontWeight::Specified(n) => n,
        }
    }

    fn resolve_bolder(parent: u16) -> u16 {
        match parent {
            0..=349 => 400,
            350..=549 => 700,
            550..=749 => 900,
            _ => 900,
        }
    }

    fn resolve_lighter(parent: u16) -> u16 {
        match parent {
            0..=549 => 100,
            550..=749 => 400,
            _ => 700,
        }
    }
}
