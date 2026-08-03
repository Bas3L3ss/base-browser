use anyhow::Result;
use html::{Dom, NodeType};
use network::ResourceLoader;
use std::vec;
use url::Url;
use utils::profile_scope;

use crate::{
    Cascade, CascadeSource, ComputedStyle, CssOm, Inheritance, Matcher, Origin, Parser, Tokenizer,
};

pub struct StyleContext<'a> {
    pub url: &'a Url,
    pub ua_cssom: &'a CssOm,
    // later:
    // viewport
    // media query values
    // visited link checker
}

impl<'a> StyleContext<'a> {
    pub fn new(url: &'a Url, ua_cssom: &'a CssOm) -> Self {
        Self { url, ua_cssom }
    }
}

pub struct StyleEngine<'a> {
    context: StyleContext<'a>,
}

impl<'a> StyleEngine<'a> {
    pub fn new(context: StyleContext<'a>) -> Self {
        Self { context }
    }
    pub fn build_cssom(&self, dom: &Dom, loader: &ResourceLoader) -> Result<CssOm> {
        profile_scope!("Build CSSOM");
        let mut cssom = CssOm::default();

        for stylesheet in &dom.style_sheets {
            let source = loader.load_stylesheet(dom, stylesheet, self.context.url)?;
            let tokens = Tokenizer::from_string(&source);

            let mut parser = Parser::new(&tokens);
            let rules = parser.parse()?;
            cssom.append_stylesheet(rules);
        }

        cssom.build_indexes();

        Ok(cssom)
    }
    pub fn resolve_styles(&self, dom: &Dom, cssom: &CssOm) -> Vec<Option<ComputedStyle>> {
        profile_scope!("Resolve Styles");

        let mut styled_nodes = vec![None; dom.nodes.len()];

        for node_id in 0..dom.nodes.len() {
            self.resolve(dom, node_id, &mut styled_nodes, cssom);
        }

        styled_nodes
    }

    pub fn resolve(
        &self,
        dom: &Dom,
        node_id: usize,
        styled_nodes: &mut [Option<ComputedStyle>],
        cssom: &CssOm,
    ) {
        let node = &dom.nodes[node_id];

        if styled_nodes[node_id].is_some() {
            return;
        }

        let parent_style_opt: Option<&ComputedStyle> = if let Some(parent_id) = node.parent {
            if let Some(parent_style) = styled_nodes[parent_id].as_ref() {
                Some(parent_style)
            } else {
                self.resolve(dom, parent_id, styled_nodes, cssom);
                styled_nodes[parent_id].as_ref()
            }
        } else {
            None
        };

        match &node.ntype {
            NodeType::Element(element) => {
                let author_candidates = cssom.lookup_candidates(element);
                let ua_candidates = self.context.ua_cssom.lookup_candidates(element);

                let ua_matches =
                    Matcher::match_rules(node, dom, &ua_candidates, &self.context.ua_cssom);

                let author_matches = Matcher::match_rules(node, dom, &author_candidates, &cssom);

                let mut style = Cascade::compute(
                    element,
                    &[
                        CascadeSource {
                            origin: Origin::UserAgent,
                            matches: ua_matches,
                            cssom: &self.context.ua_cssom,
                        },
                        CascadeSource {
                            origin: Origin::Author,
                            matches: author_matches,
                            cssom: &cssom,
                        },
                    ],
                );

                if let Some(_parent_style) = parent_style_opt {
                    Inheritance::apply(node, &mut style, styled_nodes);
                }

                styled_nodes[node.id] = Some(style);
            }

            NodeType::Text(_) => {
                if let Some(parent_style) = parent_style_opt {
                    let style = parent_style.clone();
                    styled_nodes[node.id] = Some(style);
                }
            }

            _ => {}
        }
    }
}
