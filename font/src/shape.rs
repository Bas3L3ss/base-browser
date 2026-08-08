use crate::FontMetrics;
#[derive(Clone, Debug)]
pub struct GlyphRun {
    pub glyphs: Vec<GlyphPosition>,
    pub advance_width: f32,
}

#[derive(Clone, Debug)]
pub struct GlyphPosition {
    pub glyph_id: u16,
    pub x_offset: f32,
    pub y_offset: f32,

    pub x_advance: f32,
    pub y_advance: f32,
}

impl GlyphRun {
    pub fn run(face: &ttf_parser::Face, text: &str, font_size: f32, metrics: &FontMetrics) -> Self {
        let rb_face = rustybuzz::Face::from_face(face.clone());

        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();

        let output = rustybuzz::shape(&rb_face, &[], buffer);

        let scale = metrics.scale(font_size);

        let infos = output.glyph_infos();
        let positions = output.glyph_positions();

        let mut glyphs = Vec::with_capacity(infos.len());
        let mut advance_width = 0.0;

        for (info, position) in infos.iter().zip(positions.iter()) {
            let x_advance = position.x_advance as f32 * scale;
            let y_advance = position.y_advance as f32 * scale;

            let x_offset = position.x_offset as f32 * scale;
            let y_offset = position.y_offset as f32 * scale;

            glyphs.push(GlyphPosition {
                glyph_id: info.glyph_id as u16,
                x_offset,
                y_offset,
                x_advance,
                y_advance,
            });

            advance_width += x_advance;
        }

        Self {
            glyphs,
            advance_width,
        }
    }
}
