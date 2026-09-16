use std::sync::Arc;
use swash::{ CacheKey, FontRef };

use crate::FontError;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoadedFont {
    data: Arc<[u8]>,

    swash_offset: u32,
    swash_key: CacheKey,
}

impl LoadedFont {
    pub fn face(&self) -> Result<ttf_parser::Face<'_>, FontError> {
        ttf_parser::Face::parse(&self.data, 0).map_err(|e| FontError::ParseError(e.to_string()))
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self, FontError> {
        let data: Arc<[u8]> = data.into();

        // Ask Swash to parse the font ONCE.
        let font_ref = FontRef::from_index(&data, 0).ok_or_else(||
            FontError::ParseError("Swash failed to parse font".into())
        )?;

        let swash_offset = font_ref.offset;
        let swash_key = font_ref.key;

        Ok(Self {
            data,
            swash_offset,
            swash_key,
        })
    }

    /// Creates a rustybuzz::Face whose underlying font data is *not* tied to the lifetime of
    /// LoadedFont and may be dropped, so use with caution: don't drop self while Face is alive.
    pub fn to_rb_face(self: &Arc<Self>) -> rustybuzz::Face<'static> {
        let data_arc = Arc::clone(self);
        let leaked: &'static LoadedFont = Box::leak(Box::new(data_arc));
        rustybuzz::Face
            ::from_slice(&leaked.data, 0)
            .expect("failed to parse font for rustybuzz::Face")
    }

    pub fn as_swash_ref(&self) -> FontRef<'_> {
        FontRef {
            data: &self.data,
            offset: self.swash_offset,
            key: self.swash_key,
        }
    }
}
