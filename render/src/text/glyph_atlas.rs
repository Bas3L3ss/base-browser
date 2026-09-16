use font::LoadedFont;
use swash::{ scale::{ Render, ScaleContext, Source, StrikeWith }, zeno::{ Format, Vector } };

pub struct GlyphAtlas {
    context: ScaleContext,
}
impl GlyphAtlas {
    pub fn new() -> Self {
        Self { context: ScaleContext::new() }
    }
    pub fn get_glpyh_image(
        &mut self,
        font_size: f32,
        loaded_font: &LoadedFont,
        x: f32,
        y: f32,
        glyph_id: u16
    ) -> Option<swash::scale::image::Image> {
        let font_ref = loaded_font.as_swash_ref();
        let mut scaler = self.context.builder(font_ref).size(font_size).hint(true).build();
        let offset = Vector::new(x.fract(), y.fract());

        Render::new(
            &[Source::ColorOutline(0), Source::ColorBitmap(StrikeWith::BestFit), Source::Outline]
        )
            // Select a subpixel format
            .format(Format::Subpixel)
            // Apply the fractional offset
            .offset(offset)
            // Render the image
            .render(&mut scaler, glyph_id)
    }
}
