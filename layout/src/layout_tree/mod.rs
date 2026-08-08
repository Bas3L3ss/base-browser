pub mod layout_node;
pub use layout_node::*;

use css::{ComputedStyle, display::Display};
use html::{Dom, NodeType, html_tags::HtmlTag};

#[derive(Debug, Clone)]
pub struct LayoutTree {
    pub root: Option<LayoutNode>,
}

impl LayoutTree {
    pub fn build(dom: &Dom, computed_styles: &[Option<ComputedStyle>]) -> Self {
        let root: Option<LayoutNode> = Self::build_node(dom.document_element, dom, computed_styles);

        LayoutTree { root }
    }

    /// Recursively build a layout tree. Skips nodes (and their subtrees) for display:none,
    /// <style>, <script>, whitespace-only text nodes, etc. Does NOT promote children.
    fn build_node(
        node_id: usize,
        dom: &Dom,
        computed_styles: &[Option<ComputedStyle>],
    ) -> Option<LayoutNode> {
        let node = &dom.nodes[node_id];

        match &node.ntype {
            NodeType::Document | NodeType::Doctype(_) | NodeType::Comment(_) => None,
            NodeType::Element(el) => {
                if el.tag == HtmlTag::Style || el.tag == HtmlTag::Script || el.tag == HtmlTag::Head
                {
                    return None;
                }

                let style = computed_styles
                    .get(node_id)
                    .and_then(|s| s.as_ref())
                    .expect("every element should have a computed style");

                let display = style.display();

                if let Display::None = display {
                    return None;
                }

                // Recursively build children (retain only Some nodes).
                let mut children: Vec<LayoutNode> = node
                    .children
                    .iter()
                    .filter_map(|&kid| Self::build_node(kid, dom, computed_styles))
                    .collect();

                Self::wrap_anonymous_blocks(&mut children);

                // ALLOCATION: Depending on downstream use, repeatedly cloning Default::default() could allocate more than needed.
                let layout_node = match display {
                    Display::Inline => LayoutNode::inline(node_id, children),
                    Display::InlineBlock => {
                        LayoutNode::inline_block(node_id, Default::default(), children)
                    }

                    _ => LayoutNode::block(node_id, Default::default(), children),
                };

                Some(layout_node)
            }
            // ALLOCATION: Always allocates an empty Vec for InlineFragment, even for whitespace or empty texts.
            NodeType::Text(_) => Some(LayoutNode::text(node_id)),
        }
    }

    /// Wraps consecutive inline children in anonymous block boxes when both block-level and inline-level children exist.
    fn wrap_anonymous_blocks(children: &mut Vec<LayoutNode>) {
        let has_block = children.iter().any(LayoutNode::is_block);
        let has_inline = children.iter().any(LayoutNode::is_inline);

        if !has_block || !has_inline {
            return;
        }

        let mut result = Vec::with_capacity(children.len());
        let mut inline_run = Vec::new();

        for child in children.drain(..) {
            if child.is_block() {
                if !inline_run.is_empty() {
                    result.push(LayoutNode::anonymous_block(
                        Default::default(),
                        std::mem::take(&mut inline_run),
                    ));
                }

                result.push(child);
            } else {
                inline_run.push(child);
            }
        }

        if !inline_run.is_empty() {
            result.push(LayoutNode::anonymous_block(Default::default(), inline_run));
        }

        *children = result;
    }
}
