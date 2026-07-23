#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn from_bytes(raw: &[u8]) -> anyhow::Result<Self> {
        let split_pos = raw
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .ok_or_else(|| {
                anyhow::anyhow!("Malformed response: could not find header/body separator")
            })?;

        let header_bytes = &raw[..split_pos];
        let body = raw[(split_pos + 4)..].to_vec();

        let header_str = String::from_utf8_lossy(header_bytes);
        let mut lines = header_str.lines();

        let status_line = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("Malformed response: missing status line"))?;

        let mut status_parts = status_line.splitn(3, ' ');
        let _http_version = status_parts.next().unwrap_or("");
        let status_code: u16 = status_parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Malformed status line: missing status code"))?
            .parse()
            .map_err(|_| anyhow::anyhow!("Malformed status code"))?;

        // Parse headers
        let headers: Vec<(String, String)> = lines
            .filter_map(|line| {
                if let Some(idx) = line.find(':') {
                    let (name, value) = line.split_at(idx);
                    Some((name.trim().to_string(), value[1..].trim().to_string()))
                } else {
                    None
                }
            })
            .collect();

        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }

    pub fn body_as_string(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}
