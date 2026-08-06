use crate::{glyph::GlyphTable, metrics::FontMetrics};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoadedFont {
    pub data: Vec<u8>,

    pub metrics: FontMetrics,

    pub glyph_table: GlyphTable,
}
