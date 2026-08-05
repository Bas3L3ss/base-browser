use std::{todo, unreachable};

use css::{BoxSizing, ComputedStyle, Length, LengthContext, MaxLength, MinLength};
use html::{Dom, NodeType, dom};

use crate::{
    BoxType, Dimensions, HorizontalEdges, HorizontalLayout, HorizontalSpecified, LayoutNode,
    LayoutSource, LayoutTree, StyleComponents,
};

struct LayoutContext {
    viewport_width: f32,
    viewport_height: f32,

    root_font_size: f32,
    current_font_size: f32,

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
}

impl<'a> LayoutEngine<'a> {
    pub fn layout(
        tree: &mut LayoutTree,
        viewport: Dimensions,
        computed_style: &[Option<ComputedStyle>],
        dom: &Dom,
    ) {
        let root = match tree.root.as_mut() {
            Some(root) => root,
            None => return,
        };

        let root_font_size = match &root.source {
            LayoutSource::DomNode(node_id) => computed_style
                .get(*node_id)
                .and_then(Option::as_ref)
                .and_then(|computed| {
                    computed
                        .font_size()
                        .map(|fs| fs.to_f32(0.0, None, &LengthContext::default()))
                })
                .unwrap_or(16.0),
            LayoutSource::Anonymous => return,
        };

        let context = LayoutContext {
            viewport_height: viewport.content.height,
            viewport_width: viewport.content.width,
            root_font_size,
            current_font_size: root_font_size,
            ch_width: root_font_size * 0.5,
            ex_height: root_font_size * 0.5,
        };

        let mut engine = LayoutEngine {
            context,
            dom,
            computed_style,
        };
        engine.layout_node(root, &viewport, viewport.content.y);
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
        let node_id = match node.source {
            LayoutSource::DomNode(id) => id,
            _ => unreachable!(),
        };

        let style = self.computed_style[node_id]
            .as_ref()
            .expect("Computed style for node is missing");

        let length_context: LengthContext = LengthContext {
            font_size: self.context.current_font_size,
            root_font_size: self.context.root_font_size,
            ex_height: self.context.ex_height,
            ch_width: self.context.ch_width,
            viewport_width: self.context.viewport_width,
            viewport_height: self.context.viewport_height,
        };

        let font_size = self.context.current_font_size;

        let text = match &self.dom.nodes[node_id].ntype {
            NodeType::Text(text) => text,
            _ => return,
        };
    }

    fn layout_block(&mut self, node: &mut LayoutNode, parent: &Dimensions, cursor_y: f32) {
        match node.source {
            LayoutSource::Anonymous => {
                node.dimensions.content.width = parent.content.width;

                Self::calculate_x(node, parent);
                Self::calculate_y(node, cursor_y);

                self.layout_descendants(node);

                Self::resolve_auto_height(node);
                Self::finalize_box_height(node);
            }
            LayoutSource::DomNode(node_id) => {
                let style = self.computed_style[node_id]
                    .as_ref()
                    .expect("Computed style for node is missing");

                let old_font_size = self.context.current_font_size;
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

                let element_ctx = LengthContext {
                    font_size: self.context.current_font_size,
                    root_font_size: self.context.root_font_size,
                    ex_height: self.context.ex_height,
                    ch_width: self.context.ch_width,
                    viewport_width: self.context.viewport_width,
                    viewport_height: self.context.viewport_height,
                };

                Self::calculate_width(node, parent, &style_components, &element_ctx);

                Self::calculate_x(node, parent);

                Self::resolve_specified_height(node, &style_components, parent, &element_ctx);

                Self::calculate_y(node, cursor_y);

                self.layout_descendants(node);

                Self::finalize_height(node, &style_components, parent, &element_ctx);

                self.context.current_font_size = old_font_size;
            }
        }
    }

    fn calculate_y(node: &mut LayoutNode, cursor_y: f32) {
        node.dimensions.margin.y = cursor_y;

        node.dimensions.border.y = node.dimensions.margin.y + node.dimensions.margin_edges.top;

        node.dimensions.padding.y = node.dimensions.border.y + node.dimensions.border_edges.top;

        node.dimensions.content.y = node.dimensions.padding.y + node.dimensions.padding_edges.top;
    }

    fn layout_descendants(&mut self, node: &mut LayoutNode) {
        let mut cursor_y = node.dimensions.content.y;

        for child in &mut node.children {
            self.layout_node(child, &node.dimensions, cursor_y);

            cursor_y += child.dimensions.margin.height;
        }
    }

    fn resolve_specified_height(
        node: &mut LayoutNode,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext,
    ) {
        match &style.height {
            Length::Auto => {
                node.dimensions.is_height_definite = false;
            }
            Length::Percent(_) if !parent.is_height_definite => {
                node.dimensions.is_height_definite = false;
            }
            _ => {
                let mut h = style
                    .height
                    .to_f32(parent.content.height, None, length_context);

                if style.box_sizing == BoxSizing::BorderBox {
                    h -= node.dimensions.padding_edges.top + node.dimensions.padding_edges.bottom;
                    h -= node.dimensions.border_edges.top + node.dimensions.border_edges.bottom;
                    h = h.max(0.0);
                }

                h = Self::clamp_height(h, style, parent, length_context);

                node.dimensions.content.height = h;
                node.dimensions.is_height_definite = true;
            }
        }
    }

    fn calculate_x(node: &mut LayoutNode, parent: &Dimensions) {
        node.dimensions.margin.x = parent.content.x;

        node.dimensions.border.x = node.dimensions.margin.x + node.dimensions.margin_edges.left;

        node.dimensions.padding.x = node.dimensions.border.x + node.dimensions.border_edges.left;

        node.dimensions.content.x = node.dimensions.padding.x + node.dimensions.padding_edges.left;
    }

    fn calculate_width(
        node: &mut LayoutNode,
        parent: &Dimensions,
        style: &StyleComponents,
        length_context: &LengthContext,
    ) {
        let edges = HorizontalEdges::resolve_horizontal_edges(parent, style, length_context);

        let specified = HorizontalSpecified::resolve_horizontal_specified(
            parent,
            style,
            length_context,
            &edges,
        );

        let min_width = match &style.min_width {
            MinLength::Auto => 0.0,
            MinLength::Length(length) => length.to_f32(parent.content.width, None, length_context),
        };
        let max_width = match &style.max_width {
            MaxLength::None => f32::INFINITY,
            MaxLength::Length(length) => length.to_f32(parent.content.width, None, length_context),
        }
        .max(min_width);

        let layout = HorizontalLayout::solve_horizontal_constraints(
            parent.content.width,
            &edges,
            &specified,
            min_width,
            max_width,
        );
        layout.apply(node, &edges);
    }

    fn finalize_box_height(node: &mut LayoutNode) {
        node.dimensions.padding.height = node.dimensions.content.height
            + node.dimensions.padding_edges.top
            + node.dimensions.padding_edges.bottom;

        node.dimensions.border.height = node.dimensions.padding.height
            + node.dimensions.border_edges.top
            + node.dimensions.border_edges.bottom;

        node.dimensions.margin.height = node.dimensions.border.height
            + node.dimensions.margin_edges.top
            + node.dimensions.margin_edges.bottom;
    }

    fn resolve_auto_height(node: &mut LayoutNode) {
        node.dimensions.content.height = node
            .children
            .iter()
            .map(|child| child.dimensions.margin.height)
            .sum();

        node.dimensions.is_height_definite = true;
    }

    fn finalize_height(
        node: &mut LayoutNode,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext,
    ) {
        if !node.dimensions.is_height_definite {
            Self::resolve_auto_height(node);

            node.dimensions.content.height = Self::clamp_height(
                node.dimensions.content.height,
                style,
                parent,
                length_context,
            );
        }

        Self::finalize_box_height(node);
    }

    fn clamp_height(
        h: f32,
        style: &StyleComponents,
        parent: &Dimensions,
        length_context: &LengthContext,
    ) -> f32 {
        let min_height = match &style.min_height {
            MinLength::Auto => 0.0,
            MinLength::Length(length) => match length {
                Length::Percent(_) if !parent.is_height_definite => 0.0,
                _ => length.to_f32(parent.content.height, None, length_context),
            },
        };
        let max_height = match &style.max_height {
            MaxLength::None => f32::INFINITY,
            MaxLength::Length(length) => match length {
                Length::Percent(_) if !parent.is_height_definite => f32::INFINITY,
                _ => length.to_f32(parent.content.height, None, length_context),
            },
        }
        .max(min_height);

        h.max(min_height).min(max_height)
    }
}
