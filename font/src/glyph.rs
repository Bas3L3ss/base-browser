use std::collections::HashMap;

pub type GlyphId = u16;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Glyph {
    pub advance_width: i16,
    pub left_side_bearing: i16,
    // pub outline: Option<GlyphOutline>, //TODO: For rendering
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cmap {
    unicode_to_glyph: HashMap<char, GlyphId>, // TODO: Replace hashmap with a more compact ds
}

impl Cmap {
    pub fn get(&self, ch: char) -> Option<GlyphId> {
        self.unicode_to_glyph.get(&ch).cloned()
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlyphTable {
    glyphs: Vec<Glyph>,
}

impl GlyphTable {
    pub fn get(&self, id: GlyphId) -> Option<&Glyph> {
        self.glyphs.get(id as usize)
    }
}
