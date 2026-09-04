use core::fmt;
use std::write;

/// Holds contextual information for resolving CSS length units like `em`, `rem`, `ex`, `ch`, `vw`, `vh`, etc.
/// Used in length conversion to absolute values (usually pixels).
#[derive(Default)]
pub struct LengthContext {
    /// The computed font size (in px) of the current element. Used for `em` units.
    pub font_size: f32,
    /// The computed font size (in px) of the root element. Used for `rem` units.
    pub root_font_size: f32,

    /// The x-height (height of lowercase "x") of the element's font, in px. Used for `ex` units.
    pub ex_height: f32,
    /// The width of the "0" (zero) glyph in the current font, in px. Used for `ch` units.
    pub ch_width: f32,

    /// The width of the viewport in px. Used for `vw`, `vmin`, `vmax` units.
    pub viewport_width: f32,
    /// The height of the viewport in px. Used for `vh`, `vmin`, `vmax` units.
    pub viewport_height: f32,
}

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
    // Time units
    S,  // Seconds
    Ms, // Milliseconds
    Unknown(String),
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Unit::Px => "px",
            Unit::Em => "em",
            Unit::Rem => "rem",
            Unit::Pt => "pt",
            Unit::Pc => "pc",
            Unit::In => "in",
            Unit::Cm => "cm",
            Unit::Mm => "mm",
            Unit::Q => "q",
            Unit::Ex => "ex",
            Unit::Ch => "ch",
            Unit::Vw => "vw",
            Unit::Vh => "vh",
            Unit::Vmin => "vmin",
            Unit::Vmax => "vmax",
            Unit::S => "s",
            Unit::Ms => "ms",
            Unit::Unknown(u) => u,
        };
        write!(f, "{}", s)
    }
}

// Implement FromStr to allow parsing from &str (will call from_str)
impl std::str::FromStr for Unit {
    type Err = ();

    fn from_str(s: &str) -> Result<Unit, Self::Err> {
        Ok(Unit::from_str(s))
    }
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
            "s" => Unit::S,
            "ms" => Unit::Ms,
            _ => Unit::Unknown(s.to_string()),
        }
    }

    /// Try to convert a numeric value in this Unit to px (CSS pixels).
    /// For absolute units, uses fixed conversions.
    /// For relative units, uses the fields in the context.
    ///
    /// Returns Some(px) for convertible units, None otherwise.
    pub fn to_px(&self, value: f32, ctx: &LengthContext) -> Option<f32> {
        match self {
            Unit::Px => Some(value),
            Unit::In => Some(value * 96.0),
            Unit::Cm => Some(value * 96.0 / 2.54),
            Unit::Mm => Some(value * 96.0 / 25.4),
            Unit::Q => Some(value * 96.0 / 101.6), // 1q = 1/40th of a cm
            Unit::Pt => Some(value * 96.0 / 72.0),
            Unit::Pc => Some(value * 16.0), // 1pc = 12pt = 16px

            // Relative to element font size
            Unit::Em => Some(value * ctx.font_size),

            // Root em
            Unit::Rem => Some(value * ctx.root_font_size),

            // Ex: x-height of font (approx 0.5em if ex_height not provided)
            Unit::Ex => {
                let ex = if ctx.ex_height > 0.0 {
                    ctx.ex_height
                } else {
                    ctx.font_size * 0.5
                };
                Some(value * ex)
            }

            // Ch: width of "0" glyph (approx 0.5em if not provided)
            Unit::Ch => {
                let ch = if ctx.ch_width > 0.0 {
                    ctx.ch_width
                } else {
                    ctx.font_size * 0.5
                };
                Some(value * ch)
            }

            // Viewport units
            Unit::Vw => Some(value * ctx.viewport_width / 100.0),
            Unit::Vh => Some(value * ctx.viewport_height / 100.0),
            Unit::Vmin => {
                let min = ctx.viewport_width.min(ctx.viewport_height);
                Some(value * min / 100.0)
            }
            Unit::Vmax => {
                let max = ctx.viewport_width.max(ctx.viewport_height);
                Some(value * max / 100.0)
            }

            // Not convertible
            Unit::S | Unit::Ms | Unit::Unknown(_) => None,
        }
    }
}
