pub mod inline_fragment;
pub mod line;

use font::{ FontKey, FontMetrics, GlyphRun };

use crate::{
    Dimensions,
    FontComponents,
    engine::LayoutContext,
    text::{ inline_fragment::InlineFragment, line::Line },
};

pub struct TextEngine;

impl TextEngine {
    pub(crate) fn layout_text_fragments(
        fragments: &mut Vec<InlineFragment>,
        context: &mut LayoutContext,
        text: &str,
        rb_face: &rustybuzz::Face,
        font_size: f32,
        font_components: &FontComponents,
        metrics: &FontMetrics,
        parent: &Dimensions,
        cur_line: &mut Line,
        current_x: &mut f32,
        current_y: &mut f32,
        ascent: f32,
        descent: f32,
        line_height: f32,
        font_key: &FontKey
    ) {
        let mut word_start = 0;
        let text_len = text.len();
        let mut char_indices = text.char_indices();

        while let Some((idx, ch)) = char_indices.next() {
            if ch.is_whitespace() {
                if word_start < idx {
                    let word = &text[word_start..idx];
                    Self::layout_text_fragment(
                        fragments,
                        context,
                        word,
                        rb_face,
                        font_size,
                        font_components,
                        metrics,
                        parent,
                        cur_line,
                        current_x,
                        current_y,
                        ascent,
                        descent,
                        line_height,
                        &font_key
                    );
                }
                // Instead of emitting "" for whitespace, emit the (potentially multi-byte) whitespace char
                Self::layout_text_fragment(
                    fragments,
                    context,
                    &text[idx..idx + ch.len_utf8()],
                    rb_face,
                    font_size,
                    font_components,
                    metrics,
                    parent,
                    cur_line,
                    current_x,
                    current_y,
                    ascent,
                    descent,
                    line_height,
                    &font_key
                );
                word_start = idx + ch.len_utf8();
            }
        }

        // Handle any remaining non-whitespace at the end
        if word_start < text_len {
            let word = &text[word_start..];
            Self::layout_text_fragment(
                fragments,
                context,
                word,
                rb_face,
                font_size,
                font_components,
                metrics,
                parent,
                cur_line,
                current_x,
                current_y,
                ascent,
                descent,
                line_height,
                &font_key
            );
        }
    }

    fn layout_text_fragment(
        fragments: &mut Vec<InlineFragment>,
        context: &mut LayoutContext,
        word: &str,
        rb_face: &rustybuzz::Face,
        font_size: f32,
        font_components: &FontComponents,
        metrics: &FontMetrics,
        parent: &Dimensions,
        cur_line: &mut Line,
        current_x: &mut f32,
        current_y: &mut f32,
        ascent: f32,
        descent: f32,
        line_height: f32,
        font_key: &FontKey
    ) {
        // Only classify as is_collapsible_space if non-empty string and all whitespace, and collapse_spaces is enabled
        let is_collapsible_space =
            !word.is_empty() &&
            font_components.whitespace.collapse_spaces &&
            word.chars().all(char::is_whitespace);

        let glyph_run = GlyphRun::run(rb_face, word, font_size, metrics);

        let advance_width = glyph_run.advance_width;

        let needs_newline =
            advance_width > context.line_remaining_width &&
            cur_line.width > 0.0 &&
            font_components.whitespace.allow_wrap;

        if needs_newline && is_collapsible_space {
            return;
        }

        if needs_newline {
            *current_y += line_height;
            *current_x = parent.content.x;
            context.line_remaining_width = parent.content.width;
            *cur_line = Line::new(line_height, ascent, descent);
        }

        // Avoid emitting fragments for empty substrings
        if word.is_empty() {
            return;
        }

        let baseline_offset = ascent + (line_height - ascent - descent) / 2.0; // TODO: handle inline algorithm more properly
        let fragment = InlineFragment::text(
            glyph_run,
            *current_x,
            *current_y,
            advance_width,
            baseline_offset,
            font_key.clone(),
            font_size
        );
        cur_line.ascent = cur_line.ascent.max(ascent);
        cur_line.descent = cur_line.descent.max(descent);
        cur_line.width += fragment.width;
        fragments.push(InlineFragment::Text(fragment));
        *current_x += advance_width;
        context.line_remaining_width -= advance_width;
    }
}
