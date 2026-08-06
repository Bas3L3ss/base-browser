use anyhow::Result;
use css::{CssOm, CssRule, Origin, Parser, Tokenizer};
use font::FontManager;
use network::{HttpClient, ResourceLoader};

pub struct BrowserShared {
    // CSS
    pub ua_stylesheet: CssOm,
    // Networking
    // dns_cache: DnsCache,
    // http_cache: HttpCache,
    // cookie_store: CookieStore,
    pub resource_loader: ResourceLoader,

    // Resources

    // image_cache: ImageCache,
    pub font_manager: FontManager,
    // History
    // history: HistoryDatabase,

    // Configuration
    // settings: BrowserSettings,
}
impl BrowserShared {
    pub fn new() -> Result<Self> {
        let http = HttpClient::new();
        let loader = ResourceLoader::new(http);

        Ok(Self {
            ua_stylesheet: Self::load_ua_stylesheet()?,
            font_manager: FontManager::default(),
            resource_loader: loader,
        })
    }

    fn load_ua_stylesheet() -> Result<CssOm> {
        let mut cssom = CssOm::default();
        cssom.origin = Origin::UserAgent;
        let source = include_str!("../../resources/ua.css");
        let tokens = Tokenizer::from_string(&source);
        let mut parser = Parser::new(&tokens);
        let rules: Vec<CssRule> = parser.parse()?;
        cssom.append_stylesheet(rules);
        cssom.build_indexes();
        Ok(cssom)
    }
}
