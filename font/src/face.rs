use crate::{FontSource, LoadedFont};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontWeight(pub u16);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontStretch(pub f32);

#[derive(Debug, Clone, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique(f32),
}

impl FontStyle {
    pub fn from_name_angle(name: &str, angle: f32) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "normal" => FontStyle::Normal,
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique(angle),
            _ => FontStyle::Normal, // fallback to normal if unknown
        }
    }
}

pub struct FontFace {
    pub family: String,

    pub style: FontStyle,

    pub weight: FontWeight,

    pub stretch: FontStretch,

    pub source: FontSource,

    pub loaded: Option<LoadedFont>,
}
