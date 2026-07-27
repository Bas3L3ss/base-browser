#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Px,   // Pixels
    Em,   // Relative to font-size of the element
    Rem,  // Relative to font-size of the root element
    Pt,   // Points
    Pc,   // Picas
    In,   // Inches
    Cm,   // Centimeters
    Mm,   // Millimeters
    Q,    // Quarter-millimeters
    Ex,   // x-height of the element's font
    Ch,   // Width of the "0" (zero) glyph
    Vw,   // 1% of the viewport's width
    Vh,   // 1% of the viewport's height
    Vmin, // 1% of the viewport's smaller dimension
    Vmax, // 1% of the viewport's larger dimension
    Unknown(String),
}

impl Unit {
    pub fn from_str(s: &str) -> Unit {
        match s.to_ascii_lowercase().as_str() {
            "px" => Unit::Px,
            "em" => Unit::Em,
            "rem" => Unit::Rem,
            "pt" => Unit::Pt,
            "pc" => Unit::Pc,
            "in" => Unit::In,
            "cm" => Unit::Cm,
            "mm" => Unit::Mm,
            "q" => Unit::Q,
            "ex" => Unit::Ex,
            "ch" => Unit::Ch,
            "vw" => Unit::Vw,
            "vh" => Unit::Vh,
            "vmin" => Unit::Vmin,
            "vmax" => Unit::Vmax,
            _ => Unit::Unknown(s.to_string()),
        }
    }
}
