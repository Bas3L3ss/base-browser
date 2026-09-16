use std::collections::HashMap;

use rustybuzz::GlyphBuffer;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ShapeKey {
    pub text: String,
    pub font_id: u16,
    pub font_size_64: u32,
}

impl ShapeKey {
    pub fn new(text: &str, font_id: u16, font_size: f32) -> Self {
        Self {
            text: text.to_owned(),
            font_id,
            font_size_64: font_size.to_bits(),
        }
    }
}

pub struct ShapeCache {
    pub cache: HashMap<ShapeKey, GlyphBuffer>,
}

impl ShapeCache {
    pub fn new() -> Self {
        ShapeCache {
            cache: HashMap::new(),
        }
    }
}
