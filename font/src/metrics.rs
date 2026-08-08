#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontMetrics {
    pub units_per_em: u16,

    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,

    pub x_height: f32,
}

impl FontMetrics {
    pub fn from_face(face: &ttf_parser::Face) -> Self {
        // Get x-height if present (Unicode 'x' or OS/2 table)
        let x_height = face
            .x_height()
            .map(|x| x as f32)
            .unwrap_or_else(|| face.units_per_em() as f32 * 0.56);

        Self {
            units_per_em: face.units_per_em(),
            ascent: face.ascender().abs() as f32,
            descent: face.descender().abs() as f32,
            line_gap: face.line_gap().abs() as f32,
            x_height,
        }
    }

    pub fn scale(&self, font_size: f32) -> f32 {
        font_size / self.units_per_em as f32
    }

    pub fn ch(&self, face: &ttf_parser::Face, font_size: f32) -> f32 {
        let glyph = face.glyph_index('0');
        let advance = glyph
            .and_then(|g| face.glyph_hor_advance(g))
            .unwrap_or(self.units_per_em as u16);
        advance as f32 * self.scale(font_size)
    }
}
