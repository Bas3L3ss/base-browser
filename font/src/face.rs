use crate::{FontSource, LoadedFont};

pub struct FontFace {
    pub family: String,

    pub style: FontStyle,

    pub weight: FontWeight,

    pub stretch: FontStretch,

    pub source: FontSource,

    pub loaded: Option<LoadedFont>,
}

pub struct FontWeight(pub u16);
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy)]
pub struct FontStretch(pub f32);
