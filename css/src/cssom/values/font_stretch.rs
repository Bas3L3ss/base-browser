/// Represents the font-stretch property in CSS.
///
/// - Named variants correspond to official CSS font-stretch keywords.
/// - `Percentage(f32)` is for numeric percentage values (e.g. 85%).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStretch {
    UltraCondensed,  // 50%
    ExtraCondensed,  // 62.5%
    Condensed,       // 75%
    SemiCondensed,   // 87.5%
    Normal,          // 100%
    SemiExpanded,    // 112.5%
    Expanded,        // 125%
    ExtraExpanded,   // 150%
    UltraExpanded,   // 200%
    Percentage(f32), // e.g., 90.0 for 90%
}

impl FontStretch {
    /// Creates a FontStretch from a CssValue, matching CSS identifiers or percentages.
    pub fn from_css(value: &crate::CssValue) -> Self {
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident.to_ascii_lowercase().as_str() {
                "ultra-condensed" => FontStretch::UltraCondensed,
                "extra-condensed" => FontStretch::ExtraCondensed,
                "condensed" => FontStretch::Condensed,
                "semi-condensed" => FontStretch::SemiCondensed,
                "normal" => FontStretch::Normal,
                "semi-expanded" => FontStretch::SemiExpanded,
                "expanded" => FontStretch::Expanded,
                "extra-expanded" => FontStretch::ExtraExpanded,
                "ultra-expanded" => FontStretch::UltraExpanded,
                _ => FontStretch::Normal,
            }
        } else if let Some(percent) = value.components.first().and_then(|c| c.as_percentage()) {
            FontStretch::Percentage(percent)
        } else if let Some(number) = value.components.first().and_then(|c| c.as_number()) {
            // Interpret bare number as percentage
            FontStretch::Percentage(number as f32)
        } else {
            FontStretch::Normal
        }
    }

    /// Returns the numeric f32 value of the stretch as a percentage (e.g. 87.5 for 87.5%).
    pub fn to_f32(&self) -> f32 {
        match *self {
            FontStretch::UltraCondensed => 50.0,
            FontStretch::ExtraCondensed => 62.5,
            FontStretch::Condensed => 75.0,
            FontStretch::SemiCondensed => 87.5,
            FontStretch::Normal => 100.0,
            FontStretch::SemiExpanded => 112.5,
            FontStretch::Expanded => 125.0,
            FontStretch::ExtraExpanded => 150.0,
            FontStretch::UltraExpanded => 200.0,
            FontStretch::Percentage(val) => val,
        }
    }
}
