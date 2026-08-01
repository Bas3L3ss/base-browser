use anyhow::Result;
use css::{cssom::cssom::CssOm, style::engine::StyleEngine};
use html::{dom::dom::Dom, parser::parser::Parser};
use network::{http::client::HttpClient, resource_loader::resource_loader::ResourceLoader};
use std::println;
use url::url::Url;

pub struct Document {
    pub url: Url,
    pub dom: Dom,
    pub cssom: CssOm,
    // layout_tree: Option<LayoutTree>,
}

impl Document {
    pub fn new(url: Url) -> Result<Self> {
        let http = HttpClient::new();
        let loader = ResourceLoader::new(http);

        let html = loader.load_html(&url)?;
        let dom = Parser::to_dom(&html)?;
        let cssom = StyleEngine::build_cssom(&dom.value, &loader, &url)?;
        let styled_dom = StyleEngine::resolve_styles(&dom.value, &cssom);

        println!("{styled_dom:#?}");

        Ok(Self {
            url,
            dom: dom.value,
            cssom,
        })
    }
}
