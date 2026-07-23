#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn from_bytes(raw: &[u8]) -> anyhow::Result<Self> {
        // Find the end of headers (\r\n\r\n)
        let split_pos = raw
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .ok_or_else(|| {
                anyhow::anyhow!("Malformed response: could not find header/body separator")
            })?;

        let header_bytes = &raw[..split_pos];
        let mut body_bytes = &raw[(split_pos + 4)..];

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

        // Check for Transfer-Encoding: chunked
        let is_chunked = headers.iter().any(|(name, value)| {
            name.eq_ignore_ascii_case("transfer-encoding") && value.eq_ignore_ascii_case("chunked")
        });

        let body = if is_chunked {
            parse_chunked_body(body_bytes)?
        } else {
            body_bytes.to_vec()
        };

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

// Helper function: Parse chunked transfer encoding body
fn parse_chunked_body(mut data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        // Find the end of the chunk size line
        let pos = data
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| {
                anyhow::anyhow!("Malformed chunked encoding: missing chunk size newline")
            })?;
        let size_str = &data[..pos];
        let size_str = String::from_utf8_lossy(size_str);
        let size = usize::from_str_radix(size_str.trim().split(';').next().unwrap_or(""), 16)
            .map_err(|_| anyhow::anyhow!("Malformed chunk size: {}", size_str))?;
        data = &data[(pos + 2)..]; // Move past the \r\n

        if size == 0 {
            // End of chunks
            // There may be trailer headers here (ending with \r\n), but we can skip.
            // We can try to skip a final "\r\n"
            if data.len() >= 2 && &data[..2] == b"\r\n" {
                data = &data[2..];
            }
            break;
        }

        if data.len() < size {
            return Err(anyhow::anyhow!(
                "Malformed chunked encoding: body too short"
            ));
        }
        body.extend_from_slice(&data[..size]);
        data = &data[size..];

        // Remove "\r\n" after the chunk
        if data.len() < 2 || &data[..2] != b"\r\n" {
            return Err(anyhow::anyhow!(
                "Malformed chunked encoding: missing CRLF after chunk"
            ));
        }
        data = &data[2..];
    }
    Ok(body)
}
