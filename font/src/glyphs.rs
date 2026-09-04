use crate::FontError;

pub type GlyphId = u16;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Glyph {
    pub advance_width: i16,
    pub left_side_bearing: i16,
    // pub outline: Option<GlyphOutline>, //TODO: For rendering
}

impl Glyph {
    pub fn glyph_id(face: ttf_parser::Face<'_>, ch: char) -> Result<GlyphId, FontError> {
        match face.glyph_index(ch) {
            Some(id) => Ok(id.0),
            None => Err(FontError::GlyphNotFound(ch.to_string())),
        }
    }

    pub fn glyph(face: ttf_parser::Face<'_>, id: GlyphId) -> Result<Glyph, FontError> {
        let glyph_id = ttf_parser::GlyphId(id);

        let advance = face
            .glyph_hor_advance(glyph_id)
            .ok_or(FontError::GlyphNotFound(id.to_string()))?;

        let left_side_bearing = face.glyph_hor_side_bearing(glyph_id).unwrap_or(0);

        Ok(Glyph {
            advance_width: advance as i16,
            left_side_bearing: left_side_bearing as i16,
        })
    }
}
