use std::todo;

use anyhow::Result;

use crate::LoadedFont;

pub struct FontLoader;

impl FontLoader {
    pub fn load(data: Vec<u8>) -> Result<LoadedFont> {
        // parse woff2/ttf/otf
        // extract metrics
        // build glyph table
        todo!()
    }
}
