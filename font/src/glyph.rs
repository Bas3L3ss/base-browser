use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Glyph {
    pub id: u32,

    pub advance_width: f32,

    pub left_side_bearing: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlyphTable {
    glyphs: HashMap<char, Glyph>,
}
