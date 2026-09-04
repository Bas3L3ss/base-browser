use crate::{Angle, AngleUnit, CssValue, Unit};

/// Represents the `font-style` property in CSS.
///
/// - "normal" and "italic" are straightforward keyword variants.
/// - "oblique" may have an optional <angle> (oblique 10deg), or fallback to plain oblique.
#[derive(Debug, Clone, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique(Option<Angle>),
}

impl FontStyle {
    pub fn from_css(value: &CssValue) -> Self {
        let mut iter = value.components.iter().peekable();
        if let Some(component) = iter.next() {
            if let Some(ident) = component.as_ident() {
                match ident.to_ascii_lowercase().as_str() {
                    "normal" => FontStyle::Normal,
                    "italic" => FontStyle::Italic,
                    "oblique" => {
                        // Spec allows: oblique, oblique <angle>
                        if let Some(next) = iter.peek() {
                            if let Some((value, unit)) = next.as_dimension() {
                                if let Unit::Unknown(s) = unit {
                                    let lower = s.to_ascii_lowercase();
                                    let unit_parsed = match lower.as_str() {
                                        "deg" => Some(AngleUnit::Deg),
                                        "rad" => Some(AngleUnit::Rad),
                                        "grad" => Some(AngleUnit::Grad),
                                        "turn" => Some(AngleUnit::Turn),
                                        _ => None,
                                    };
                                    if let Some(angle_unit) = unit_parsed {
                                        FontStyle::Oblique(Some(Angle {
                                            value,
                                            unit: angle_unit,
                                        }))
                                    } else {
                                        FontStyle::Oblique(None)
                                    }
                                } else {
                                    FontStyle::Oblique(None)
                                }
                            } else {
                                FontStyle::Oblique(None)
                            }
                        } else {
                            FontStyle::Oblique(None)
                        }
                    }

                    _ => FontStyle::Normal,
                }
            } else {
                FontStyle::Normal
            }
        } else {
            FontStyle::Normal
        }
    }

    /// Converts Oblique FontStyle to its angle in degrees as f32, or None if not Oblique.
    pub fn to_angle(&self) -> Option<f32> {
        match self {
            FontStyle::Oblique(Some(angle)) => Some(angle.to_degrees()),
            FontStyle::Oblique(None) => None,
            _ => None,
        }
    }

    /// Returns a &str representing the basic style type ("normal", "italic", "oblique")
    pub fn as_str(&self) -> &str {
        match self {
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
            FontStyle::Oblique(_) => "oblique",
        }
    }
}
