use std::println;

use anyhow::Result;
use html::dom::{dom::Dom, node_type::NodeType};
use network::resource_loader::resource_loader::ResourceLoader;
use url::url::Url;

use crate::{
    cssom::cssom::CssOm,
    parser::parser::Parser,
    style::{
        cascade::Cascade, inheritence::Inheritance, matcher::Matcher, styled_node::StyledNode,
    },
    tokenizer::tokenizer::Tokenizer,
};

pub struct StyleEngine;

impl StyleEngine {
    pub fn build_cssom(dom: &Dom, loader: &ResourceLoader, base_url: &Url) -> Result<CssOm> {
        let mut cssom = CssOm::default();

        for stylesheet in &dom.style_sheets {
            let source = loader.load_stylesheet(dom, stylesheet, base_url)?;
            let tokens = Tokenizer::from_string(&source);

            let mut parser = Parser::new(&tokens);
            let rules = parser.parse()?;
            cssom.append_stylesheet(rules);
        }

        cssom.build_indexes();

        Ok(cssom)
    }
    pub fn resolve_styles(dom: &Dom, cssom: &CssOm) -> Vec<StyledNode> {
        let mut styled_nodes = Vec::new();

        for (node_id, node) in dom.nodes.iter().enumerate() {
            match &node.ntype {
                NodeType::Element(element) => {
                    let candidates = cssom.lookup_candidates(element);

                    let matching = if candidates.is_empty() {
                        None
                    } else {
                        Some(Matcher::match_rules(node, dom, &candidates, &cssom))
                    };

                    // let mut style =
                    //     Cascade::compute(node, dom, matching.as_ref().map(|v| v.as_slice()));

                    // Inheritance::apply(node, dom, &mut style);

                    // styled_nodes.push(StyledNode::new(node_id, style));
                }

                NodeType::Text(_) => {
                    // let mut style = Cascade::compute(node, dom, None);
                    // Inheritance::apply(node, dom, &mut style);
                    // styled_nodes.push(StyledNode::new(node_id, style));
                }
                _ => {}
            }
        }

        styled_nodes
    }
}
