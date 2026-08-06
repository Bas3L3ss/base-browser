use std::write;

use crate::host::Host;
use crate::scheme::Scheme;
use anyhow::Result;
use input_parser::traits::Parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    pub scheme: Scheme,
    pub host: Host,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write scheme://host[:port]/path[?query][#fragment]
        write!(f, "{}://{}", self.scheme.as_str(), self.host)?;

        // If port is not the default (scheme), show it.
        let is_default_port = match self.scheme.as_str() {
            "http" => self.port == 80,
            "https" => self.port == 443,
            _ => false,
        };
        if !is_default_port {
            write!(f, ":{}", self.port)?;
        }

        // Path (always starts with /)
        write!(f, "{}", self.path)?;

        // Query
        if let Some(ref q) = self.query {
            write!(f, "?{}", q)?;
        }

        // Fragment
        if let Some(ref frag) = self.fragment {
            write!(f, "#{}", frag)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RelativeUrlType<'a> {
    Absolute(&'a str),       // http:// or https:// (strictly)
    SchemeRelative(&'a str), // //host/path
    RootRelative(&'a str),   // /path
    QueryOnly(&'a str),      // ?query
    FragmentOnly(&'a str),   // #fragment
    Empty,
    Relative(&'a str), // ./xyz or abc.png
}

fn classify_href(href: &str) -> RelativeUrlType<'_> {
    let href = href.trim();
    if href.is_empty() {
        RelativeUrlType::Empty
    } else if href.starts_with("http://") || href.starts_with("https://") {
        RelativeUrlType::Absolute(href)
    } else if href.starts_with("//") {
        RelativeUrlType::SchemeRelative(href)
    } else if href.starts_with('/') {
        RelativeUrlType::RootRelative(href)
    } else if href.starts_with('?') {
        RelativeUrlType::QueryOnly(&href[1..])
    } else if href.starts_with('#') {
        RelativeUrlType::FragmentOnly(&href[1..])
    } else {
        RelativeUrlType::Relative(href)
    }
}

impl Url {
    pub fn resolve(&self, href: &str) -> Result<Self> {
        use RelativeUrlType::*;

        match classify_href(href) {
            // Absolute: e.g. "http://example.com/page", "https://site.org/a/b"
            Absolute(url) => {
                // Only support http/https absolute URLs
                // Example href: "https://example.com/abc?q=1#foo"
                Url::parse(url).map(|x| x.value)
            }
            // SchemeRelative: e.g. "//cdn.example.com/img.png"
            SchemeRelative(url) => {
                // Example href: "//cdn.example.com/resource"
                let scheme_str = self.scheme.as_str();
                let full = format!("{scheme_str}:{}", url);
                Url::parse(&full).map(|x| x.value)
            }
            // RootRelative: e.g. "/images/logo.png"
            RootRelative(path_query_frag) => {
                // Example href: "/foo/bar?x=1#frag"
                // Split path/query/fragment
                let (path, query, fragment) = parse_path_query_fragment(path_query_frag);
                Ok(Url {
                    scheme: self.scheme.clone(),
                    host: self.host.clone(),
                    port: self.port,
                    path,
                    query,
                    fragment,
                })
            }
            // QueryOnly: e.g. "?debug=true"
            QueryOnly(new_query) => {
                // Example href: "?q=test"
                // Replace only query, keep everything else; preserve path, wipe fragment
                Ok(Url {
                    scheme: self.scheme.clone(),
                    host: self.host.clone(),
                    port: self.port,
                    path: self.path.clone(),
                    query: Some(new_query.to_string()),
                    fragment: None,
                })
            }
            // FragmentOnly: e.g. "#section2"
            FragmentOnly(new_fragment) => {
                // Example href: "#footer"
                // Only change fragment, preserve query
                Ok(Url {
                    scheme: self.scheme.clone(),
                    host: self.host.clone(),
                    port: self.port,
                    path: self.path.clone(),
                    query: self.query.clone(),
                    fragment: Some(new_fragment.to_string()),
                })
            }
            // Empty: ""
            Empty => {
                // Example href: ""
                // Empty href means "this page"
                Ok(self.clone())
            }
            // Relative: e.g. "cat.jpg", "./foo", "subdir/page.html", "../img/m.png"
            Relative(relhref) => {
                // Examples: "about.html", "./faq", "../up.html", "dir/file?x=2#bar"
                // Split fragment
                let mut rel = relhref;
                let mut fragment = None;
                if let Some(hash) = rel.find('#') {
                    fragment = Some(rel[hash + 1..].to_string());
                    rel = &rel[..hash];
                }
                // Split query
                let mut query = None;
                if let Some(qmark) = rel.find('?') {
                    query = Some(rel[qmark + 1..].to_string());
                    rel = &rel[..qmark];
                }

                // Compute base directory from self.path
                let mut base_path = self.path.clone();
                let is_directory = base_path.ends_with('/');
                if !is_directory {
                    if let Some(idx) = base_path.rfind('/') {
                        base_path.truncate(idx + 1);
                    } else {
                        base_path = "/".to_string();
                    }
                }
                let combined = format!("{base_path}{rel}");
                let normalized_path = normalize_path(&combined);

                Ok(Url {
                    scheme: self.scheme.clone(),
                    host: self.host.clone(),
                    port: self.port,
                    path: normalized_path,
                    query,
                    fragment,
                })
            }
        }
    }
}

// Helper: splits a string into (path, query, fragment) parts
fn parse_path_query_fragment(input: &str) -> (String, Option<String>, Option<String>) {
    let mut path = input;
    let mut query = None;
    let mut fragment = None;

    // Find fragment first
    if let Some(hash) = path.find('#') {
        fragment = Some(path[hash + 1..].to_string());
        path = &path[..hash];
    }
    // Now query
    if let Some(qmark) = path.find('?') {
        query = Some(path[qmark + 1..].to_string());
        path = &path[..qmark];
    }
    (path.to_string(), query, fragment)
}

// Normalize dot-segments as per RFC 3986 section 5.2.4
// and preserve trailing slash for directories, or if the last segment is "." or ".."
// The key: after normalization, if the input ends with "/" OR the last normalized segment is "" (dir), "." (dir), or ".." (parent dir), output trailing "/"
fn normalize_path(path: &str) -> String {
    let absolute = path.starts_with('/');
    let segments: Vec<&str> = path.split('/').collect();
    let has_trailing = path.ends_with('/') || path.ends_with("/.") || path.ends_with("/..");
    let mut output = Vec::new();
    for seg in segments.iter() {
        match *seg {
            "" | "." => {
                // Skip empty or "."
            }
            ".." => {
                if !output.is_empty() {
                    output.pop();
                }
            }
            s => {
                output.push(s);
            }
        }
    }

    let mut result = String::new();
    if absolute {
        result.push('/');
    }
    result.push_str(&output.join("/"));

    // Should trailing slash be restored?
    // RFC: If 1) original path ends in "/" OR 2) original ends with "/." or "/.." OR 3) after removing dot segments, the intended logical meaning is a directory (".") or parent ("..")
    let needs_trailing_slash = if has_trailing {
        true
    } else {
        // If there were no segments left after normalization, and the original was not absolute (should be treated as "/")
        false
    };

    if needs_trailing_slash && !result.ends_with('/') {
        result.push('/');
    }

    // Special case: if result is empty string, treat as "/"
    if result.is_empty() {
        result.push('/');
    }
    result
}

// ----------------- UNIT TESTS -------------------

#[cfg(test)]
mod tests {
    use std::{assert_eq, net::IpAddr, str::FromStr};

    use crate::url_warning::UrlWarning;

    use super::*;
    use input_parser::{generic_struct::ParsedOutput, traits::Parse};

    fn url(s: &str) -> ParsedOutput<Url, UrlWarning> {
        Url::parse(s).unwrap()
    }

    #[test]
    fn parses_basic_http_url() {
        let out = url("http://example.com/path");
        assert_eq!(out.value.scheme, Scheme::Http);
        assert_eq!(out.value.host, Host::Domain("example.com".to_string()));
        assert_eq!(out.value.port, 80);
        assert_eq!(out.value.path, "/path".to_string());
        assert_eq!(out.value.query, None);
        assert_eq!(out.value.fragment, None);
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn parses_https_url_with_query_and_fragment() {
        let out = url("https://secure.com/secure?foo=bar#frag");
        assert_eq!(out.value.scheme, Scheme::Https);
        assert_eq!(out.value.host, Host::Domain("secure.com".to_string()));
        assert_eq!(out.value.port, 443);
        assert_eq!(out.value.path, "/secure");
        assert_eq!(out.value.query, Some("foo=bar".to_string()));
        assert_eq!(out.value.fragment, Some("frag".to_string()));
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn parses_url_with_custom_port() {
        let out = url("http://foo.com:8080/xyz");
        assert_eq!(out.value.scheme, Scheme::Http);
        assert_eq!(out.value.host, Host::Domain("foo.com".to_string()));
        assert_eq!(out.value.port, 8080);
        assert_eq!(out.value.path, "/xyz");
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn parses_ipv4_address() {
        let out = url("http://127.0.0.1:1234/");
        assert_eq!(
            out.value.host,
            Host::Ip(IpAddr::from_str("127.0.0.1").unwrap())
        );
        assert_eq!(out.value.port, 1234);
        assert_eq!(out.value.path, "/");
    }

    #[test]
    fn parses_ipv6_literal() {
        let out = url("https://[2001:db8::1]/foo");
        assert_eq!(out.value.scheme, Scheme::Https);
        assert_eq!(
            out.value.host,
            Host::Ip(IpAddr::from_str("2001:db8::1").unwrap())
        );
        assert_eq!(out.value.port, 443);
        assert_eq!(out.value.path, "/foo");
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn warns_on_missing_scheme_and_fills_defaults() {
        let out = url("foo.com/blah");
        assert_eq!(out.value.scheme, Scheme::Https);
        assert_eq!(out.value.host, Host::Domain("foo.com".to_string()));
        assert_eq!(out.value.port, 443);
        assert_eq!(out.value.path, "/blah");
        assert!(out.warnings.contains(&UrlWarning::MissingScheme));
    }

    #[test]
    fn warns_on_missing_host() {
        let out = url("http:///abc");
        assert_eq!(out.value.host, Host::Domain("".to_string()));
        assert_eq!(out.value.path, "/abc");
        assert!(out.warnings.contains(&UrlWarning::MissingHost));
    }

    #[test]
    fn warns_on_invalid_port() {
        let out = url("http://foo.com:abc/def");
        assert_eq!(out.value.port, 80); // fallback default
        assert!(
            out.warnings
                .iter()
                .any(|w| matches!(w, UrlWarning::InvalidPort(p) if p == "abc"))
        );
    }

    #[test]
    fn warns_on_invalid_ipv6_host() {
        let out = url("http://[ghij::nutty]/");
        assert!(matches!(out.value.host, Host::Domain(_)));
        assert!(
            out.warnings
                .iter()
                .any(|w| matches!(w, UrlWarning::InvalidHost(msg) if msg == "ghij::nutty"))
        );
    }

    #[test]
    fn default_path_is_slash() {
        let out = url("http://site.com");
        assert_eq!(out.value.path, "/");
    }

    // --- TESTS FOR resolve() ---
    #[test]
    fn resolve_absolute_url() {
        let base = url("https://example.com/store/index.html").value;
        let abs = base.resolve("http://another.com/hi/there").unwrap();
        assert_eq!(abs.scheme, Scheme::Http);
        assert_eq!(abs.host, Host::Domain("another.com".to_string()));
        assert_eq!(abs.path, "/hi/there");
    }

    #[test]
    fn resolve_scheme_relative_url() {
        let base = url("https://example.com/store/index.html").value;
        let abs = base.resolve("//another.com/hi/there").unwrap();
        assert_eq!(abs.scheme, Scheme::Https);
        assert_eq!(abs.host, Host::Domain("another.com".to_string()));
        assert_eq!(abs.path, "/hi/there");
    }

    #[test]
    fn resolve_root_relative() {
        let base = url("https://example.com/store/index.html").value;
        let css = base.resolve("/css/theme.css").unwrap();
        assert_eq!(css.scheme, Scheme::Https);
        assert_eq!(css.host, Host::Domain("example.com".to_string()));
        assert_eq!(css.path, "/css/theme.css");
    }

    #[test]
    fn resolve_sibling_path() {
        let base = url("https://example.com/store/index.html").value;
        let css = base.resolve("theme.css").unwrap();
        assert_eq!(css.path, "/store/theme.css");
    }

    #[test]
    fn resolve_parent_relative() {
        let base = url("https://example.com/store/index.html").value;
        let css = base.resolve("../css/theme.css").unwrap();
        assert_eq!(css.path, "/css/theme.css");
    }

    #[test]
    fn resolve_complex_dot_segments() {
        let base = url("https://example.com/a/b/c/index.html").value;
        let css = base.resolve("../../css/./theme.css").unwrap();
        assert_eq!(css.path, "/a/css/theme.css");
    }

    #[test]
    fn resolve_when_base_has_no_path() {
        let base = url("https://domain.com").value;
        let a = base.resolve("foo.css").unwrap();
        assert_eq!(a.path, "/foo.css");
        let b = base.resolve("/bar.css").unwrap();
        assert_eq!(b.path, "/bar.css");
    }

    #[test]
    fn resolve_query_only_reuses_path() {
        let base = url("https://domain.com/products/index.html").value;
        let a = base.resolve("?page=2").unwrap();
        assert_eq!(a.path, "/products/index.html");
        assert_eq!(a.query, Some("page=2".to_string()));
        assert_eq!(a.fragment, None);
    }

    #[test]
    fn resolve_fragment_only_reuses_path_and_query() {
        let base = url("https://site.com/index.html?a=1").value;
        let b = base.resolve("#part2").unwrap();
        assert_eq!(b.path, "/index.html");
        assert_eq!(b.query, Some("a=1".to_string()));
        assert_eq!(b.fragment, Some("part2".to_string()));
    }

    #[test]
    fn resolve_empty_href_is_idempotent() {
        let base = url("https://site.com/abc?x=2#yyy").value;
        let out = base.resolve("").unwrap();
        assert_eq!(out, base);
    }

    #[test]
    fn resolve_parent_dot_yields_root() {
        // .. from /blog/index.html
        let base = url("https://site.com/blog/index.html").value;
        let out = base.resolve("..").unwrap();
        assert_eq!(out.path, "/");
    }

    #[test]
    fn resolve_current_dir_no_slash() {
        // . from /blog/index.html
        let base = url("https://site.com/blog/index.html").value;
        let out = base.resolve(".").unwrap();
        assert_eq!(out.path, "/blog/");
    }

    #[test]
    fn resolve_current_dir_with_slash() {
        // ./ from /blog/index.html
        let base = url("https://site.com/blog/index.html").value;
        let out = base.resolve("./").unwrap();
        assert_eq!(out.path, "/blog/");
    }

    #[test]
    fn resolve_directory_trailing_slash() {
        // Append child to directory ending slash
        let base = url("https://site.com/dir/").value;
        let out = base.resolve("child/").unwrap();
        assert_eq!(out.path, "/dir/child/");
    }
}
