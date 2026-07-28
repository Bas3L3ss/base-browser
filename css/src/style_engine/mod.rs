use html::dom::dom::Dom;
use network::resource_loader::resource_loader::ResourceLoader;
use url::url::Url;

use crate::{cssom::cssom::CssOm, parser::parser::Parser, tokenizer::tokenizer::Tokenizer};

pub struct StyleEngine;

impl StyleEngine {
    pub fn build_cssom(
        dom: &Dom,
        loader: &ResourceLoader,
        base_url: &Url,
    ) -> anyhow::Result<CssOm> {
        let mut cssom = CssOm::new();

        for stylesheet in &dom.style_sheets {
            let source = loader.load_stylesheet(dom, stylesheet, base_url)?;
            let tokens = Tokenizer::from_string(&source);

            let mut parser = Parser::new(&tokens);
            let rules = parser.parse()?;
            cssom.append_stylesheet(rules);
        }

        Ok(cssom)
    }
}
