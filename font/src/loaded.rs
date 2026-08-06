use crate::{glyph::GlyphTable, metrics::FontMetrics};

pub struct LoadedFont {
    pub data: Vec<u8>,

    pub metrics: FontMetrics,

    pub glyph_table: GlyphTable,
}
