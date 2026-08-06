use std::collections::HashMap;

pub struct Glyph {
    pub id: u32,

    pub advance_width: f32,

    pub left_side_bearing: f32,
}

pub struct GlyphTable {
    glyphs: HashMap<char, Glyph>,
}
