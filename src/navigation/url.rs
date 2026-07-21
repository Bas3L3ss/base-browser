use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug)]
pub struct Url {
    pub scheme: Scheme,
    pub host: Host,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

#[derive(Debug)]
pub enum Scheme {
    Http,
    Https,
}

#[derive(Debug)]
pub enum Host {
    Ip(IpAddr),
    Domain(String),
}

impl Url {
    pub fn parse(url_str: &str) -> Url {
        let (without_fragment, fragment) = if let Some(hash_idx) = url_str.find('#') {
            let (left, right) = url_str.split_at(hash_idx);
            (left, Some(right[1..].to_string()))
        } else {
            (url_str, None)
        };

        let (without_query, query) = if let Some(q_idx) = without_fragment.find('?') {
            let (left, right) = without_fragment.split_at(q_idx);
            (left, Some(right[1..].to_string()))
        } else {
            (without_fragment, None)
        };

        let (scheme, rest) = if without_query.starts_with("https://") {
            (Scheme::Https, &without_query["https://".len()..])
        } else if without_query.starts_with("http://") {
            (Scheme::Http, &without_query["http://".len()..])
        } else {
            // HANDLE ERROR - NO SCHEME SPECIFIED OR WARNING AND TELLING TO DEFAULT TO HTTP
            (Scheme::Http, without_query)
        };

        let (host_port, mut path) = if let Some(slash_idx) = rest.find('/') {
            let (h, p) = rest.split_at(slash_idx);
            (h, p)
        } else {
            (rest, "")
        };

        let (host_str, mut port) = if host_port.starts_with('[') {
            if let Some(end_bracket_idx) = host_port.find(']') {
                let host_lit = &host_port[..=end_bracket_idx];
                let remainder: &str = &host_port[end_bracket_idx + 1..];
                let port = if remainder.starts_with(':') {
                    remainder[1..].parse::<u16>().ok()
                } else {
                    None
                };
                (host_lit, port)
            } else {
                (host_port, None)
            }
        } else {
            if let Some(colon_idx) = host_port.rfind(':') {
                if colon_idx > 0 {
                    let (h, p) = host_port.split_at(colon_idx);
                    let port_str = &p[1..];
                    let port = port_str.parse::<u16>().ok();
                    (h, port)
                } else {
                    (host_port, None)
                }
            } else {
                (host_port, None)
            }
        };

        let host = if host_str.starts_with('[') && host_str.ends_with(']') {
            // It's an IPv6 literal, strip brackets
            let ip_str = &host_str[1..host_str.len() - 1];
            if let Ok(ip) = IpAddr::from_str(ip_str) {
                Host::Ip(ip)
            } else {
                Host::Domain(host_str.to_string())
            }
        } else if let Ok(ip) = IpAddr::from_str(host_str) {
            Host::Ip(ip)
        } else {
            Host::Domain(host_str.to_string())
        };

        if port.is_none() {
            match scheme {
                Scheme::Http => port = Some(80),
                Scheme::Https => port = Some(443),
            }
        }

        if path.is_empty() {
            path = "/"
        }

        Url {
            scheme,
            host,
            port: port.unwrap(),
            path: path.to_string(),
            query,
            fragment,
        }
    }
}
