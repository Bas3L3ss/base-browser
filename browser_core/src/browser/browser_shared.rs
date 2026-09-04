use anyhow::Result;
use css::{ CssOm, Origin, Parser, tokenizer::tokenizer::Tokenizer };

#[derive(Default)]
pub struct BrowserShared {
    // CSS
    pub ua_stylesheet: CssOm,
    // Networking
    // dns_cache: DnsCache,
    // http_cache: HttpCache,
    // cookie_store: CookieStore,

    // Resources
    // image_cache: ImageCache,
    // font_manager: FontManager,

    // History
    // history: HistoryDatabase,

    // Configuration
    // settings: BrowserSettings,
}
impl BrowserShared {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ua_stylesheet: Self::load_ua_stylesheet()?,
        })
    }

    fn load_ua_stylesheet() -> Result<CssOm> {
        let mut cssom = CssOm::default();
        cssom.origin = Origin::UserAgent;
        let source = include_str!("../../resources/ua.css");
        let tokens = Tokenizer::from_string(&source);
        let mut parser = Parser::new(&tokens);
        let rules = parser.parse()?;
        cssom.append_stylesheet(rules);
        cssom.build_indexes();
        Ok(cssom)
    }
}
