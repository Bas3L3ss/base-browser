use anyhow::Result;
use html::dom::dom::Dom;
use network::resource_loader::resource_loader::ResourceLoader;
use url::url::Url;

use crate::{
    cssom::{css_indexes::CssIndexes, cssom::CssOm},
    parser::parser::Parser,
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

        cssom.append_indexes(CssIndexes::build(&cssom));

        Ok(cssom)
    }
}
