use font::LoadedFont;
use swash::{ scale::{ Render, ScaleContext, Source, StrikeWith }, zeno::{ Format, Vector } };
use swash::scale::image::Image;
use std::collections::HashMap;
use std::hash::{ Hash, Hasher };
use std::sync::Arc;

#[derive(Debug, Clone, Eq)]
struct GlyphCacheKey {
    font_id: u16,
    font_size: u32,
    glyph_id: u16,
    subpixel_x: u8,
    subpixel_y: u8,
}
impl PartialEq for GlyphCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.font_id == other.font_id &&
            self.font_size == other.font_size &&
            self.glyph_id == other.glyph_id &&
            self.subpixel_x == other.subpixel_x &&
            self.subpixel_y == other.subpixel_y
    }
}
impl Hash for GlyphCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_id.hash(state);
        self.font_size.hash(state);
        self.glyph_id.hash(state);
        self.subpixel_x.hash(state);
        self.subpixel_y.hash(state);
    }
}

pub struct GlyphAtlas {
    context: ScaleContext,
    cache: HashMap<GlyphCacheKey, Option<Arc<Image>>>,
}

impl GlyphAtlas {
    pub fn new() -> Self {
        Self {
            context: ScaleContext::new(),
            cache: HashMap::new(),
        }
    }

    fn make_key(
        &self,
        loaded_font: &LoadedFont,
        font_size: f32,
        glyph_id: u16,
        x: f32,
        y: f32
    ) -> GlyphCacheKey {
        // Bin subpixel offsets to 64 possible values (6 bits of precision).
        let subpixel_x = ((x.fract() * 64.0).round() as u8).min(63);
        let subpixel_y = ((y.fract() * 64.0).round() as u8).min(63);
        GlyphCacheKey {
            font_id: loaded_font.id,
            font_size: (font_size * 10.0) as u32, // 1 decimal place
            glyph_id,
            subpixel_x,
            subpixel_y,
        }
    }

    /// Returns rendered glyph image, using a cache.
    /// Instead of returning Image directly, return Option<Arc<Image>> to allow cheap clones and thread safety.
    pub fn get_glpyh_image(
        &mut self,
        font_size: f32,
        loaded_font: &LoadedFont,
        x: f32,
        y: f32,
        glyph_id: u16
    ) -> Option<Arc<Image>> {
        let key = self.make_key(loaded_font, font_size, glyph_id, x, y);
        if let Some(cached) = self.cache.get(&key) {
            return cached.as_ref().map(Arc::clone);
        }
        let font_ref = loaded_font.font_ref;
        let mut scaler = self.context.builder(font_ref).size(font_size).hint(true).build();
        let offset = Vector::new(x.fract(), y.fract());

        let image = Render::new(
            &[Source::ColorOutline(0), Source::ColorBitmap(StrikeWith::BestFit), Source::Outline]
        )
            .format(Format::Subpixel)
            .offset(offset)
            .render(&mut scaler, glyph_id);

        let arc_image = image.map(Arc::new);
        self.cache.insert(key, arc_image.as_ref().map(Arc::clone));
        arc_image
    }

    /// Optional: clear cache if needed externally.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
