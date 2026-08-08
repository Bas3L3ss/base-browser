pub(crate) use style_components::*;
pub mod font_components;
pub mod style_components;
pub use font_components::*;
pub mod block;
pub mod inline;
pub mod text;

use crate::{
    Dimensions, LayoutNode, LayoutTree,
    block::BlockEngine,
    text::{TextEngine, line::Line},
};
use css::{ComputedStyle, CssOm, LengthContext, LineHeight};
use font::{FontManager, FontMetrics, FontRequest};
use html::{Dom, NodeType};
use network::ResourceLoader;
use std::{eprintln, todo};
use url::Url;

pub(crate) struct LayoutContext {
    viewport_width: f32,
    viewport_height: f32,

    root_font_size: f32,
    current_font_size: f32,
    current_font_weight: u16,

    ex_height: f32,
    ch_width: f32,

    pending_space: bool,

    line_remaining_width: f32,
    current_line: Option<Line>,
}

pub struct LayoutEngine<'a> {
    context: LayoutContext,
    dom: &'a Dom,
    computed_style: &'a [Option<ComputedStyle>],
    font_manager: &'a mut FontManager,
    cssom: &'a CssOm,
    loader: &'a ResourceLoader,
    base_url: &'a Url,
}

impl<'a> LayoutEngine<'a> {
    pub fn layout(
        tree: &mut LayoutTree,
        viewport: Dimensions,
        computed_style: &[Option<ComputedStyle>],
        dom: &Dom,
        cssom: &CssOm,
        font_manager: &mut FontManager,
        loader: &ResourceLoader,
        base_url: &Url,
    ) {
        let root = match tree.root.as_mut() {
            Some(root) => root,
            None => return,
        };

        let (root_font_size, root_font_weight, loaded) = match &root {
            LayoutNode::Block {
                node_id,
                dimensions: _,
                children: _,
            } => {
                let computed_style = computed_style
                    .get(*node_id)
                    .and_then(Option::as_ref)
                    .expect("root must always have style");

                let font_size = computed_style
                    .font_size()
                    .map(|fs| fs.to_f32(0.0, None, &LengthContext::default()))
                    .unwrap_or(16.0);

                let font_weight = computed_style
                    .font_weight()
                    .unwrap_or(css::FontWeight::Normal)
                    .to_u16(400);

                let font_components = FontComponents::new(computed_style, font_weight);

                let loaded_font = font_manager.resolve_font(
                    &cssom.indexes.fonts,
                    &FontRequest::new(
                        &font_components.families,
                        font_components.weight,
                        font_components.style,
                        font_components.stretch,
                    ),
                    base_url,
                    loader,
                );

                (font_size, font_weight, loaded_font)
            }
            _ => return,
        };

        let face = &loaded.face().expect("must always have a valid font");
        let metrics = FontMetrics::from_face(face);

        let context = LayoutContext {
            viewport_height: viewport.content.height,
            viewport_width: viewport.content.width,
            root_font_size,
            current_font_size: root_font_size,
            current_font_weight: root_font_weight,
            pending_space: false,
            line_remaining_width: viewport.content.width,
            ch_width: metrics.ch(&face, root_font_size),
            ex_height: metrics.x_height,
            current_line: None,
        };

        let mut engine = LayoutEngine {
            context,
            dom,
            font_manager,
            computed_style,
            cssom,
            loader,
            base_url,
        };

        engine.layout_node(root, &viewport, viewport.content.y);
    }

    fn layout_node(&mut self, node: &mut LayoutNode, containing_block: &Dimensions, cursor_y: f32) {
        match node {
            LayoutNode::Block { .. } => self.layout_block(node, containing_block, cursor_y),

            LayoutNode::AnonymousBlock { .. } => {
                self.layout_anonymous(node, containing_block, cursor_y)
            }

            LayoutNode::Text { .. } => self.layout_text(node, containing_block, cursor_y),

            _ => todo!(),
        }
    }

    fn layout_text(&mut self, node: &mut LayoutNode, parent: &Dimensions, cursor_y: f32) {
        if let LayoutNode::Text { node_id, fragments } = node {
            let text = match &self.dom.nodes[*node_id].ntype {
                NodeType::Text(text) => text,
                _ => return,
            };

            let style = self.computed_style[*node_id]
                .as_ref()
                .expect("Computed style for node is missing");

            let font_components = FontComponents::new(style, self.context.current_font_weight);

            let length_context = LengthContext {
                font_size: self.context.current_font_size,
                root_font_size: self.context.root_font_size,
                ex_height: self.context.ex_height,
                ch_width: self.context.ch_width,
                viewport_width: self.context.viewport_width,
                viewport_height: self.context.viewport_height,
            };

            let font_size = self.context.current_font_size;

            let request = FontRequest::new(
                &font_components.families,
                font_components.weight,
                font_components.style,
                font_components.stretch,
            );

            let loaded_font = self.font_manager.resolve_font(
                &self.cssom.indexes.fonts,
                &request,
                self.base_url,
                self.loader,
            );

            let face = match loaded_font.face() {
                Ok(face) => face,
                Err(e) => {
                    eprintln!("Font parse error: {:?}", e);
                    return;
                }
            };

            let metrics = FontMetrics::from_face(&face);

            let scale = metrics.scale(font_size);
            let ascent = metrics.ascent * scale;
            let descent = metrics.descent * scale;
            let line_gap = metrics.line_gap * scale;

            let line_height = match style.line_height(font_size) {
                LineHeight::Normal => line_gap + ascent + descent,
                LineHeight::Specified(length) => length.to_f32(font_size, None, &length_context),
            };

            let current_line_should_exist = self.context.current_line.is_some();

            if !current_line_should_exist {
                self.context.current_line = Some(Line::new(line_height, ascent, descent));
                self.context.line_remaining_width = parent.content.width;
            } else {
                if let Some(line) = &self.context.current_line {
                    self.context.line_remaining_width = parent.content.width - line.width;
                }
            }

            let mut cur_line = self.context.current_line.take().unwrap();

            let initial_x = parent.content.x + cur_line.width;

            let mut current_x = initial_x;
            let mut current_y = cursor_y;

            // For both collapsing and non-collapsing, always treat explicit "\n" with preserve_newlines
            // as a hard line break, even when collapsing is disabled.
            let mut slice_start = 0;
            let mut space_pending = self.context.pending_space; // Only relevant for collapse_spaces

            let mut chars = text.char_indices();

            while let Some((idx, ch)) = chars.next() {
                let is_newline = ch == '\n' && font_components.whitespace.preserve_newlines;
                let is_whitespace = ch.is_whitespace();
                // In collapse_spaces mode, whitespace is handled as before, but now with correct newline handling.
                if is_newline {
                    // Emit any text fragment before the \n
                    if slice_start < idx {
                        TextEngine::layout_text_fragments(
                            fragments,
                            &mut self.context,
                            &text[slice_start..idx],
                            &face,
                            font_size,
                            &font_components,
                            &metrics,
                            parent,
                            &mut cur_line,
                            &mut current_x,
                            &mut current_y,
                            ascent,
                            descent,
                            line_height,
                        );
                    }
                    // Discard any pending space, and forcibly break the line.
                    space_pending = false;
                    // Actually break the line.
                    current_y += line_height;
                    current_x = parent.content.x;
                    self.context.line_remaining_width = parent.content.width;
                    cur_line = Line::new(line_height, ascent, descent);

                    slice_start = idx + ch.len_utf8();
                } else if font_components.whitespace.collapse_spaces && is_whitespace {
                    // Emit any text fragment before the whitespace
                    if slice_start < idx {
                        TextEngine::layout_text_fragments(
                            fragments,
                            &mut self.context,
                            &text[slice_start..idx],
                            &face,
                            font_size,
                            &font_components,
                            &metrics,
                            parent,
                            &mut cur_line,
                            &mut current_x,
                            &mut current_y,
                            ascent,
                            descent,
                            line_height,
                        );
                    }
                    // Don't emit "" for whitespace; instead, set pending_space
                    space_pending = true;
                    slice_start = idx + ch.len_utf8();
                } else if !font_components.whitespace.collapse_spaces && is_whitespace {
                    // Handle whitespace, including spaces and newlines (if preserve_newlines is false)
                    if slice_start < idx {
                        TextEngine::layout_text_fragments(
                            fragments,
                            &mut self.context,
                            &text[slice_start..idx],
                            &face,
                            font_size,
                            &font_components,
                            &metrics,
                            parent,
                            &mut cur_line,
                            &mut current_x,
                            &mut current_y,
                            ascent,
                            descent,
                            line_height,
                        );
                    }
                    // Emit the actual whitespace character, as a slice.
                    TextEngine::layout_text_fragments(
                        fragments,
                        &mut self.context,
                        &text[idx..idx + ch.len_utf8()],
                        &face,
                        font_size,
                        &font_components,
                        &metrics,
                        parent,
                        &mut cur_line,
                        &mut current_x,
                        &mut current_y,
                        ascent,
                        descent,
                        line_height,
                    );
                    slice_start = idx + ch.len_utf8();
                } else {
                    // If a space is pending (collapse mode), emit a single space before the nonspace run
                    if font_components.whitespace.collapse_spaces && space_pending {
                        TextEngine::layout_text_fragments(
                            fragments,
                            &mut self.context,
                            " ",
                            &face,
                            font_size,
                            &font_components,
                            &metrics,
                            parent,
                            &mut cur_line,
                            &mut current_x,
                            &mut current_y,
                            ascent,
                            descent,
                            line_height,
                        );
                        space_pending = false;
                    }
                }
            }

            // At the end: if there is remaining non-whitespace, emit it as a final fragment
            if slice_start < text.len() {
                let substr = &text[slice_start..];
                // If collapsing, ignore trailing all-whitespace
                if !font_components.whitespace.collapse_spaces
                    || !substr.chars().all(char::is_whitespace)
                {
                    TextEngine::layout_text_fragments(
                        fragments,
                        &mut self.context,
                        substr,
                        &face,
                        font_size,
                        &font_components,
                        &metrics,
                        parent,
                        &mut cur_line,
                        &mut current_x,
                        &mut current_y,
                        ascent,
                        descent,
                        line_height,
                    );
                }
            }

            self.context.pending_space = if font_components.whitespace.collapse_spaces {
                space_pending
            } else {
                false
            };

            self.context.current_line = Some(cur_line);
        }
    }

    fn layout_block(&mut self, node: &mut LayoutNode, parent: &Dimensions, cursor_y: f32) {
        if let LayoutNode::Block {
            node_id,
            dimensions,
            children,
        } = node
        {
            self.context.current_line = None;
            self.context.pending_space = false;

            let style = self.computed_style[*node_id]
                .as_ref()
                .expect("Computed style for node is missing");

            let old_font_size = self.context.current_font_size;
            let old_font_weight = self.context.current_font_weight;
            let old_available_space = self.context.line_remaining_width;
            let old_ch_width = self.context.ch_width;
            let old_ex_height = self.context.ex_height;

            let style_components = StyleComponents::new(&style);

            let inherited_ctx: LengthContext = LengthContext {
                font_size: self.context.current_font_size,
                root_font_size: self.context.root_font_size,
                ex_height: self.context.ex_height,
                ch_width: self.context.ch_width,
                viewport_width: self.context.viewport_width,
                viewport_height: self.context.viewport_height,
            };

            if let Some(new_font_size) = &style_components.font_size {
                self.context.current_font_size =
                    new_font_size.to_f32(parent.content.width, None, &inherited_ctx);
            }

            if let Some(new_font_weight) = &style.font_weight() {
                self.context.current_font_weight = new_font_weight.to_u16(old_font_weight);
            }

            let font_components = FontComponents::new(style, self.context.current_font_weight);
            let loaded_font = self.font_manager.resolve_font(
                &self.cssom.indexes.fonts,
                &FontRequest::new(
                    &font_components.families,
                    font_components.weight,
                    font_components.style,
                    font_components.stretch,
                ),
                self.base_url,
                self.loader,
            );
            if let Ok(face) = &loaded_font.face() {
                let metrics = FontMetrics::from_face(face);
                self.context.ch_width = metrics.ch(face, self.context.current_font_size);
                self.context.ex_height = metrics.x_height;
            }

            let element_ctx = LengthContext {
                font_size: self.context.current_font_size,
                root_font_size: self.context.root_font_size,
                ex_height: self.context.ex_height,
                ch_width: self.context.ch_width,
                viewport_width: self.context.viewport_width,
                viewport_height: self.context.viewport_height,
            };

            BlockEngine::calculate_width(dimensions, parent, &style_components, &element_ctx);

            self.context.line_remaining_width = dimensions.content.width;

            BlockEngine::calculate_x(dimensions, parent);

            BlockEngine::resolve_specified_height(
                dimensions,
                &style_components,
                parent,
                &element_ctx,
            );

            BlockEngine::calculate_y(dimensions, cursor_y);

            self.layout_descendants(dimensions, children);

            BlockEngine::finalize_height(
                children,
                dimensions,
                &style_components,
                parent,
                &element_ctx,
            );

            self.context.current_font_weight = old_font_weight;
            self.context.current_font_size = old_font_size;
            self.context.line_remaining_width = old_available_space;
            self.context.ch_width = old_ch_width;
            self.context.ex_height = old_ex_height;
        }
    }
    fn layout_anonymous(&mut self, node: &mut LayoutNode, parent: &Dimensions, cursor_y: f32) {
        if let LayoutNode::Block {
            node_id: _,
            dimensions,
            children,
        } = node
        {
            self.context.current_line = None;
            self.context.pending_space = false;

            dimensions.content.width = parent.content.width;

            BlockEngine::calculate_x(dimensions, parent);
            BlockEngine::calculate_y(dimensions, cursor_y);

            self.layout_descendants(dimensions, children);

            BlockEngine::resolve_auto_height(children, dimensions);
            BlockEngine::finalize_box_height(dimensions);
        }
    }

    fn layout_descendants(&mut self, dimensions: &Dimensions, children: &mut Vec<LayoutNode>) {
        let mut cursor_y = dimensions.content.y;

        for child in children {
            self.layout_node(child, &dimensions, cursor_y);

            cursor_y += BlockEngine::margin_box_height(child);
        }
    }
}
