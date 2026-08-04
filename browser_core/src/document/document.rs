use anyhow::Result;
use css::{ComputedStyle, CssOm, StyleContext, StyleEngine};
use html::{Dom, Parser};
use layout::{Dimensions, LayoutEngine, LayoutTree, Rect};
use network::{HttpClient, ResourceLoader};
use url::url::Url;

pub struct Document {
    pub url: Url,
    pub dom: Dom,
    pub cssom: CssOm,
    pub computed_styles: Vec<Option<ComputedStyle>>,
    pub layout_tree: LayoutTree,
}

impl Document {
    pub fn new(ua_cssom: &CssOm, url: Url) -> Result<Self> {
        let http = HttpClient::new();
        let loader = ResourceLoader::new(http);

        let html = loader.load_html(&url)?;
        let parsed_dom = Parser::to_dom(&html)?;
        let context = StyleContext::new(&url, ua_cssom);
        let style_engine = StyleEngine::new(context);
        let cssom = style_engine.build_cssom(&parsed_dom.value, &loader)?;
        let computed_styles = style_engine.resolve_styles(&parsed_dom.value, &cssom);
        let mut layout_tree = LayoutTree::build(&parsed_dom.value, &computed_styles);
        LayoutEngine::layout(
            &mut layout_tree,
            Dimensions {
                content: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                },
                ..Default::default()
            },
            &computed_styles,
        );

        Ok(Self {
            url,
            dom: parsed_dom.value,
            cssom,
            computed_styles,
            layout_tree,
        })
    }
}
