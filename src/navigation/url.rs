#[derive(Debug)]
pub struct Url {
    pub scheme: Scheme,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

#[derive(Debug)]
pub enum Scheme {
    Http,
    Https,
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
            // Default to http
            (Scheme::Http, without_query)
        };

        let (host_port, mut path) = if let Some(slash_idx) = rest.find('/') {
            let (h, p) = rest.split_at(slash_idx);
            (h, p)
        } else {
            (rest, "")
        };

        let (host, mut port) = if let Some(colon_idx) = host_port.find(':') {
            let (h, p) = host_port.split_at(colon_idx);
            let port_str = &p[1..];
            let port = port_str.parse::<u16>().ok();
            (h.to_string(), port)
        } else {
            (host_port.to_string(), None)
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
            port,
            path: path.to_string(),
            query,
            fragment,
        }
    }
}
