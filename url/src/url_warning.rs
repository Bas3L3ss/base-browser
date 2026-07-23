#[derive(Debug)]
pub enum UrlWarning {
    MissingScheme,
    InvalidPort(String),
    InvalidHost(String),
    MissingHost,
    UnknownScheme(String),
}
