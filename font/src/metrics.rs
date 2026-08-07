#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontMetrics {
    pub units_per_em: u16,

    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
}

impl FontMetrics {
    pub fn from_face(face: &ttf_parser::Face) -> Self {
        Self {
            units_per_em: face.units_per_em(),
            ascent: face.ascender(),
            descent: face.descender(),
            line_gap: face.line_gap(),
        }
    }
    pub fn scale(&self, font_size: f32) -> f32 {
        font_size / self.units_per_em as f32
    }
}
