use crate::FontMetrics;
#[derive(Clone, Debug)]
pub struct GlyphRun {
    pub glyphs: Vec<GlyphPosition>,
    pub advance_width: f32,
}

#[derive(Clone, Debug)]
pub struct GlyphPosition {
    // WHAT glyph should I draw?
    pub glyph_id: u16,

    // WHERE should I draw this glyph relative to the cursor?
    pub x_offset: f32,
    pub y_offset: f32,

    // WHERE should the cursor move for the NEXT glyph?
    pub x_advance: f32,
    pub y_advance: f32,
}

impl GlyphRun {
    pub fn run(
        rb_face: &rustybuzz::Face,
        text: &str,
        font_size: f32,
        metrics: &FontMetrics
    ) -> Self {
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();

        let output = rustybuzz::shape(&rb_face, &[], buffer); // CACHE THIS

        let scale = metrics.scale(font_size);

        let infos = output.glyph_infos();
        let positions = output.glyph_positions();

        let mut glyphs = Vec::with_capacity(infos.len());
        let mut advance_width = 0.0;

        for (info, position) in infos.iter().zip(positions.iter()) {
            let x_advance = (position.x_advance as f32) * scale;
            let y_advance = (position.y_advance as f32) * scale;

            let x_offset = (position.x_offset as f32) * scale;
            let y_offset = (position.y_offset as f32) * scale;

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
