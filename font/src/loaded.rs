use std::sync::Arc;
use swash::{ FontRef };

use crate::FontError;

#[derive(Clone)]
pub struct LoadedFont {
    pub id: u16,
    /// The parsed TTF face using ttf_parser for font metrics, glyph outlines etc.
    pub face: ttf_parser::Face<'static>,
    /// Parsed face for shaping
    pub rb_face: rustybuzz::Face<'static>,
    /// Used by swash rasterizer
    pub font_ref: FontRef<'static>,
}

impl LoadedFont {
    pub fn from_bytes(data: Vec<u8>, id: u16) -> Result<Self, FontError> {
        let data: Arc<[u8]> = data.into();
        let leaked: &[u8] = {
            let slice_ref: &[u8] = &*data;
            Box::leak(slice_ref.to_owned().into_boxed_slice())
        };

        let face = ttf_parser::Face
            ::parse(leaked, 0)
            .map_err(|e| FontError::ParseError(e.to_string()))?;

        let rb_face = rustybuzz::Face
            ::from_slice(leaked, 0)
            .ok_or_else(||
                FontError::ParseError("failed to parse font for rustybuzz::Face".into())
            )?;

        let font_ref = FontRef::from_index(leaked, 0).ok_or_else(||
            FontError::ParseError("Swash failed to parse font".into())
        )?;

        Ok(Self {
            id,
            face,
            rb_face,
            font_ref,
        })
    }
}
