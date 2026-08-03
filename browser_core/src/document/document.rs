use anyhow::Result;
use css::{StyleContext, cssom::cssom::CssOm, style::engine::StyleEngine};
use html::{dom::dom::Dom, parser::parser::Parser};
use network::{http::client::HttpClient, resource_loader::resource_loader::ResourceLoader};
use url::url::Url;

pub struct Document {
    pub url: Url,
    pub dom: Dom,
    pub cssom: CssOm,
    // layout_tree: Option<LayoutTree>,
}

impl Document {
    pub fn new(ua_cssom: &CssOm, url: Url) -> Result<Self> {
        let http = HttpClient::new();
        let loader = ResourceLoader::new(http);

        let html = loader.load_html(&url)?;
        let dom = Parser::to_dom(&html)?;
        let context = StyleContext::new(&url, ua_cssom);
        let style_engine = StyleEngine::new(context);
        let cssom = style_engine.build_cssom(&dom.value, &loader)?;
        let styled_dom = style_engine.resolve_styles(&dom.value, &cssom);

        Ok(Self {
            url,
            dom: dom.value,
            cssom,
        })
    }
}
