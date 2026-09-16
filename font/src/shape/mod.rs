pub mod cache;

use std::sync::Arc;

use rustybuzz::GlyphBuffer;

use crate::{ FontMetrics, LoadedFont, shape::cache::{ ShapeCache, ShapeKey } };
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
    pub fn run(
        loaded_font: Arc<LoadedFont>,
        text: &str,
        font_size: f32,
        metrics: &FontMetrics,
        shape_cache: &mut ShapeCache
    ) -> Self {
        let key = ShapeKey::new(text, loaded_font.id, font_size);

        let output: &GlyphBuffer = if let Some(cached) = shape_cache.cache.get(&key) {
            cached
        } else {
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str(text);
            buffer.guess_segment_properties();
            let shaped = rustybuzz::shape(&loaded_font.rb_face, &[], buffer);
            shape_cache.cache.insert(key.clone(), shaped);
            shape_cache.cache.get(&key).expect("Value just inserted should exist")
        };

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
