use std::net::IpAddr;
use std::str::FromStr;

use anyhow::Result;
use input_parser::generic_struct::ParsedOutput;
use input_parser::traits::Parse;
use utils::profile_scope;

use crate::host::Host;
use crate::scheme::Scheme;
use crate::url::Url;
use crate::url_warning::UrlWarning;

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
            // No scheme: warn and default to https
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
