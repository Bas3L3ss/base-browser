use std::collections::HashMap;

use crate::GlyphId;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cmap {
    pub(crate) unicode_to_glyph: HashMap<char, GlyphId>, // TODO: Replace hashmap with a more compact ds
}

impl Cmap {
    pub fn get(&self, ch: char) -> Option<GlyphId> {
        self.unicode_to_glyph.get(&ch).cloned()
    }
}
