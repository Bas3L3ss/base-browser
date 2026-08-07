pub(crate) use style_components::*;
pub mod font_components;
pub mod style_components;
pub use font_components::*;
pub mod block;
pub mod inline;
pub mod text;

use css::{ComputedStyle, CssOm, LengthContext};
use font::{FontManager, FontRequest};
use html::{Dom, NodeType};
use network::ResourceLoader;
use std::{todo, unreachable};
use url::Url;

use crate::{BoxType, Dimensions, LayoutNode, LayoutSource, LayoutTree, block::BlockEngine};

struct LayoutContext {
    viewport_width: f32,
    viewport_height: f32,

    root_font_size: f32,
    current_font_size: f32,
    current_font_weight: u16,

    // ex_height and ch_width are generally determined by font metrics
    // of the font in use, which could depend on the system or user agent default settings.
    // However, if we need to store them anywhere for layout or initial context,
    // LayoutContext is a reasonable place for now for use in LengthContext creation,
    // unless you intend to receive them via API or environment.

    // If these are not node-specific but global (browser/os level), it's better to
    // keep them in a config struct or global settings obj.
    //
    // For now (unless a better place is defined in architecture), it's okay to
    // store them here to allow LengthContext construction for the layout engine.
    ex_height: f32,
    ch_width: f32,
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

        let (root_font_size, root_font_weight) = match &root.source {
            LayoutSource::DomNode(node_id) => {
                let maybe_computed = computed_style.get(*node_id).and_then(Option::as_ref);

                let font_size = maybe_computed
                    .and_then(|computed| {
                        computed
                            .font_size()
                            .map(|fs| fs.to_f32(0.0, None, &LengthContext::default()))
                    })
                    .unwrap_or(16.0);

                let font_weight = maybe_computed
                    .map(|computed| {
                        computed
                            .font_weight()
                            .unwrap_or(css::FontWeight::Normal)
                            .to_u16(400)
                    })
                    .unwrap_or(400);

                (font_size, font_weight)
            }
            LayoutSource::Anonymous => return,
        };

        let context = LayoutContext {
            viewport_height: viewport.content.height,
            viewport_width: viewport.content.width,
            root_font_size,
            current_font_size: root_font_size,
            current_font_weight: root_font_weight,
            ch_width: root_font_size * 0.5,
            ex_height: root_font_size * 0.5,
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

    fn layout_descendants(&mut self, node: &mut LayoutNode) {
        let mut cursor_y = node.dimensions.content.y;

        for child in &mut node.children {
            self.layout_node(child, &node.dimensions, cursor_y);

            cursor_y += child.dimensions.margin.height;
        }
    }

    fn layout_node(&mut self, node: &mut LayoutNode, containing_block: &Dimensions, cursor_y: f32) {
        match node.box_type {
            BoxType::Block => self.layout_block(node, containing_block, cursor_y),
            BoxType::Anonymous => self.layout_block(node, containing_block, cursor_y),

            BoxType::Text => self.layout_text(node, containing_block, cursor_y),
            BoxType::Inline => todo!(),

            _ => todo!(),
        }
    }

    fn layout_text(&mut self, node: &mut LayoutNode, parent: &Dimensions, cursor_y: f32) {
        // Get node id and check  type
        let node_id = match node.source {
            LayoutSource::DomNode(id) => id,
            _ => unreachable!(),
        };

        let text = match &self.dom.nodes[node_id].ntype {
            NodeType::Text(text) => text,
            _ => return,
        };

        let style = self.computed_style[node_id]
            .as_ref()
            .expect("Computed style for node is missing");

        // Prepare the length context using the current layout context
        let length_context = LengthContext {
            font_size: self.context.current_font_size,
            root_font_size: self.context.root_font_size,
            ex_height: self.context.ex_height,
            ch_width: self.context.ch_width,
            viewport_width: self.context.viewport_width,
            viewport_height: self.context.viewport_height,
        };

        let font_size = self.context.current_font_size;
        let font_components = FontComponents::new(style, self.context.current_font_weight);
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
    }

    fn layout_block(&mut self, node: &mut LayoutNode, parent: &Dimensions, cursor_y: f32) {
        match node.source {
            LayoutSource::Anonymous => {
                node.dimensions.content.width = parent.content.width;

                BlockEngine::calculate_x(node, parent);
                BlockEngine::calculate_y(node, cursor_y);

                self.layout_descendants(node);

                BlockEngine::resolve_auto_height(node);
                BlockEngine::finalize_box_height(node);
            }
            LayoutSource::DomNode(node_id) => {
                let style = self.computed_style[node_id]
                    .as_ref()
                    .expect("Computed style for node is missing");

                let old_font_size = self.context.current_font_size;
                let old_font_weight = self.context.current_font_weight;
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

                let element_ctx = LengthContext {
                    font_size: self.context.current_font_size,
                    root_font_size: self.context.root_font_size,
                    ex_height: self.context.ex_height,
                    ch_width: self.context.ch_width,
                    viewport_width: self.context.viewport_width,
                    viewport_height: self.context.viewport_height,
                };

                BlockEngine::calculate_width(node, parent, &style_components, &element_ctx);

                BlockEngine::calculate_x(node, parent);

                BlockEngine::resolve_specified_height(
                    node,
                    &style_components,
                    parent,
                    &element_ctx,
                );

                BlockEngine::calculate_y(node, cursor_y);

                self.layout_descendants(node);

                BlockEngine::finalize_height(node, &style_components, parent, &element_ctx);

                self.context.current_font_weight = old_font_weight;
                self.context.current_font_size = old_font_size;
            }
        }
    }
}
