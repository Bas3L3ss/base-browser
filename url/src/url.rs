use std::net::IpAddr;
use std::str::FromStr;

use anyhow::Result;
use input_parser::generic_struct::ParsedOutput;
use input_parser::traits::Parse;
use utils::profile_scope;

use crate::url_warning::UrlWarning;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    pub scheme: Scheme,
    pub host: Host,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Host {
    Ip(IpAddr),
    Domain(String),
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::Ip(ip) => write!(f, "{ip}"),
            Host::Domain(domain) => write!(f, "{domain}"),
        }
    }
}

impl Parse<UrlWarning, &str> for Url {
    fn parse(url_str: &str) -> Result<ParsedOutput<Url, UrlWarning>> {
        profile_scope!("Parse URL");
        let mut warnings = Vec::new();

        // Fragment
        let (without_fragment, fragment) = if let Some(hash_idx) = url_str.find('#') {
            let (left, right) = url_str.split_at(hash_idx);
            (left, Some(right[1..].to_string()))
        } else {
            (url_str, None)
        };

        // Query
        let (without_query, query) = if let Some(q_idx) = without_fragment.find('?') {
            let (left, right) = without_fragment.split_at(q_idx);
            (left, Some(right[1..].to_string()))
        } else {
            (without_fragment, None)
        };

        // Scheme
        let (scheme, rest) = if without_query.starts_with("https://") {
            (Scheme::Https, &without_query["https://".len()..])
        } else if without_query.starts_with("http://") {
            (Scheme::Http, &without_query["http://".len()..])
        } else {
            // No scheme: warn and default to http
            warnings.push(UrlWarning::MissingScheme);
            (Scheme::Https, without_query)
        };

        // Host & Path
        let (host_port, mut path) = if let Some(slash_idx) = rest.find('/') {
            let (h, p) = rest.split_at(slash_idx);
            (h, p)
        } else {
            (rest, "")
        };

        // Host & Port
        let (host_str, mut port) = if host_port.starts_with('[') {
            if let Some(end_bracket_idx) = host_port.find(']') {
                let host_lit = &host_port[..=end_bracket_idx];
                let remainder: &str = &host_port[end_bracket_idx + 1..];
                let port = if remainder.starts_with(':') {
                    match remainder[1..].parse::<u16>() {
                        Ok(n) => Some(n),
                        Err(_) => {
                            warnings.push(UrlWarning::InvalidPort(remainder[1..].to_string()));
                            None
                        }
                    }
                } else {
                    None
                };
                (host_lit, port)
            } else {
                // Missing closing ']' for IPv6 addr, treat entire host_port as host
                warnings.push(UrlWarning::InvalidHost(host_port.to_string()));
                (host_port, None)
            }
        } else {
            if let Some(colon_idx) = host_port.rfind(':') {
                if colon_idx > 0 {
                    let (h, p) = host_port.split_at(colon_idx);
                    let port_str = &p[1..];
                    let port = match port_str.parse::<u16>() {
                        Ok(n) => Some(n),
                        Err(_) => {
                            warnings.push(UrlWarning::InvalidPort(port_str.to_string()));
                            None
                        }
                    };
                    (h, port)
                } else {
                    (host_port, None)
                }
            } else {
                (host_port, None)
            }
        };

        // Host validation
        let host = if host_str.is_empty() {
            warnings.push(UrlWarning::MissingHost);
            Host::Domain("".to_string())
        } else if host_str.starts_with('[') && host_str.ends_with(']') {
            // IPv6 literal, strip brackets
            let ip_str = &host_str[1..host_str.len() - 1];
            match IpAddr::from_str(ip_str) {
                Ok(ip) => Host::Ip(ip),
                Err(_) => {
                    warnings.push(UrlWarning::InvalidHost(ip_str.to_string()));
                    Host::Domain(host_str.to_string())
                }
            }
        } else if let Ok(ip) = IpAddr::from_str(host_str) {
            Host::Ip(ip)
        } else {
            Host::Domain(host_str.to_string())
        };

        // Default port if not specified
        if port.is_none() {
            port = Some(match scheme {
                Scheme::Http => 80,
                Scheme::Https => 443,
            });
        }

        // Path
        if path.is_empty() {
            path = "/";
        }

        Ok(ParsedOutput::new(
            Url {
                scheme,
                host,
                port: port.unwrap_or(0),
                path: path.to_string(),
                query,
                fragment,
            },
            warnings,
        ))
    }
}

// ----------------- UNIT TESTS -------------------

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;
    use input_parser::traits::Parse;

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
}
