use std::todo;

use css::{ComputedStyle, LengthContext};

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

pub struct LayoutEngine {
    context: LayoutContext,
}

impl LayoutEngine {
    pub fn layout(
        tree: &mut LayoutTree,
        viewport: Dimensions,
        computed_style: &[Option<ComputedStyle>],
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

        let ctx = LayoutContext {
            viewport_height: viewport.content.height,
            viewport_width: viewport.content.width,
            root_font_size,
            current_font_size: root_font_size,
            ch_width: root_font_size * 0.5,
            ex_height: root_font_size * 0.5,
        };

        let mut engine = LayoutEngine { context: ctx };
        engine.layout_node(root, &viewport, computed_style);
    }

    fn layout_node(
        &mut self,
        node: &mut LayoutNode,
        containing_block: &Dimensions,
        computed_style: &[Option<ComputedStyle>],
    ) {
        match node.box_type {
            BoxType::Block => self.layout_block(node, containing_block, computed_style),
            BoxType::Anonymous => self.layout_block(node, containing_block, computed_style),

            BoxType::Inline => todo!(),
            BoxType::Text => todo!(),

            _ => todo!(),
        }
    }
    fn layout_block(
        &mut self,
        node: &mut LayoutNode,
        parent: &Dimensions,
        computed_style: &[Option<ComputedStyle>],
    ) {
        match node.source {
            LayoutSource::Anonymous => {
                todo!()
            }
            LayoutSource::DomNode(node_id) => {
                let style = computed_style[node_id]
                    .as_ref()
                    .expect("Computed style for node is missing");

                let length_context = LengthContext {
                    font_size: self.context.current_font_size,
                    root_font_size: self.context.root_font_size,
                    ex_height: self.context.ex_height,
                    ch_width: self.context.ch_width,
                    viewport_width: self.context.viewport_width,
                    viewport_height: self.context.viewport_height,
                };

                let style_components = StyleComponents::new(&style);

                Self::calculate_width(node, parent, &style_components, &length_context);

                Self::calculate_x(node, parent);

                // Self::calculate_height(node, parent, &style_components, &length_context);

                if let Some(new_font_size) = style_components.font_size {
                    self.context.current_font_size =
                        new_font_size.to_f32(parent.content.width, None, &length_context);
                }
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

        let layout = HorizontalLayout::solve_horizontal_constraints(
            parent.content.width,
            &edges,
            &specified,
        );
        layout.apply(node, &edges);
    }

    // fn calculate_height(
    //     node: &mut LayoutNode,
    //     parent: &Dimensions,
    //     style: &StyleComponents,
    //     Length_context: &LengthContext,
    // ) {
    //     match style.box_sizing {
    //         BoxSizing::BorderBox => {}
    //         BoxSizing::ContentBox => {}
    //     }
    // }
}
