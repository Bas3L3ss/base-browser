use std::{todo, unreachable, vec};

use crate::{
    BoxType, LayoutNode,
    LayoutSource::{self, DomNode},
};
use css::{ComputedStyle, display::Display};
use html::{Dom, NodeType, html_tags::HtmlTag};

#[derive(Debug, Clone, PartialEq)]
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
            NodeType::Document | NodeType::Doctype(_) | NodeType::Comment(_) => {
                // No layout boxes for these node types.
                None
            }
            NodeType::Element(el) => {
                if el.tag == HtmlTag::Style || el.tag == HtmlTag::Script || el.tag == HtmlTag::Head
                {
                    return None;
                }

                // Get computed style - assert this exists for every element (per UA stylesheet logic)
                let style = computed_styles
                    .get(node_id)
                    .and_then(|s| s.as_ref())
                    .expect("every element should have a computed style");

                let display = style.display();

                match display {
                    Display::None => return None,
                    Display::Flex => {
                        return Some(LayoutNode::new(DomNode(node_id), BoxType::Unknown, vec![]));
                    } // TODO!
                    Display::Unknown(ref v) => {
                        // Developer bug marker:
                        // todo!("unhandled display variant: {v:?}")
                        return Some(LayoutNode::new(DomNode(node_id), BoxType::Unknown, vec![]));
                    }
                    _ => {
                        // Continue
                    }
                }

                let box_type = match display {
                    Display::Block => BoxType::Block,
                    Display::Inline => BoxType::Inline,
                    Display::None => unreachable!(), // already skipped above
                    Display::InlineBlock => BoxType::Unknown, // Todo
                    Display::ListItem => BoxType::Unknown, // TODO
                    Display::Flex => BoxType::Unknown, // TODO
                    Display::Grid => BoxType::Unknown, // TODO
                    Display::Unknown(_) => BoxType::Unknown, // TODO
                };

                // Recursively build children (retain only Some nodes).
                // TODO: Anonymous box generation for mixed inline/block!
                let children: Vec<LayoutNode> = node
                    .children
                    .iter()
                    .filter_map(|&kid| Self::build_node(kid, dom, computed_styles))
                    .collect();

                let final_children = Self::wrap_anonymous_blocks(children);

                Some(LayoutNode::new(DomNode(node_id), box_type, final_children))
            }
            NodeType::Text(_) => Some(LayoutNode::new(DomNode(node_id), BoxType::Text, vec![])),
        }
    }

    /// Wraps anonymous blocks when a mixture of block and inline children exists.
    /// If both block-level and inline-level children are found, it wraps consecutive inline-level
    /// nodes in anonymous boxes to follow proper CSS box tree structure.
    fn wrap_anonymous_blocks(children: Vec<LayoutNode>) -> Vec<LayoutNode> {
        let has_block = children.iter().any(|c| c.box_type.is_block_level());
        let has_inline = children.iter().any(|c| c.box_type.is_inline_level());

        if has_block && has_inline {
            let mut corrected_children: Vec<LayoutNode> = Vec::with_capacity(children.len());
            let mut temp = LayoutNode::new(LayoutSource::Anonymous, BoxType::Anonymous, Vec::new());

            for child in children {
                if child.box_type.is_block_level() {
                    if !temp.children.is_empty() {
                        corrected_children.push(temp);
                        temp = LayoutNode::new(
                            LayoutSource::Anonymous,
                            BoxType::Anonymous,
                            Vec::new(),
                        );
                    }
                    corrected_children.push(child);
                } else {
                    temp.push(child);
                }
            }
            if !temp.children.is_empty() {
                corrected_children.push(temp);
            }

            corrected_children
        } else {
            children
        }
    }
}
