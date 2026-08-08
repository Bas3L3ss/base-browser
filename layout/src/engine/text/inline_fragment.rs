use font::GlyphRun;

#[derive(Clone, Debug)]
pub struct TextFragment {
    pub glyph_run: GlyphRun,

    pub x: f32,
    pub y: f32,

    pub baseline_offset: f32,
    pub width: f32,
}

#[derive(Clone, Debug)]
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
        baseline_offset: f32,
    ) -> TextFragment {
        TextFragment {
            glyph_run,
            x,
            y,
            width,
            baseline_offset,
        }
    }
}
