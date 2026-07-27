#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px,      // Pixels
    Em,      // Relative to font-size of the element
    Rem,     // Relative to font-size of the root element
    Percent, // Percentage
    Pt,      // Points
    Pc,      // Picas
    In,      // Inches
    Cm,      // Centimeters
    Mm,      // Millimeters
    Q,       // Quarter-millimeters
    Ex,      // x-height of the element's font
    Ch,      // Width of the "0" (zero) glyph
    Vw,      // 1% of the viewport's width
    Vh,      // 1% of the viewport's height
    Vmin,    // 1% of the viewport's smaller dimension
    Vmax,    // 1% of the viewport's larger dimension
}
