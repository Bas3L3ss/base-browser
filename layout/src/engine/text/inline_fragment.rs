use font::GlyphRun;

#[derive(Clone)]
pub struct TextFragment {
    pub glyph_run: GlyphRun,

    pub x: f32,
    pub y: f32,

    pub width: f32,

    pub ascent: f32,
    pub descent: f32,
}

#[derive(Clone)]
pub enum InlineFragment {
    Text(TextFragment),
    // Image(ImageFragment),
    // InlineBlock(InlineBlockFragment),
    // ...
}

impl InlineFragment {
    pub fn text(
        glyph_run: GlyphRun,
        x: f32,
        y: f32,
        width: f32,
        ascent: f32,
        descent: f32,
    ) -> TextFragment {
        TextFragment {
            glyph_run,
            x,
            y,
            width,
            ascent,
            descent,
        }
    }
}
