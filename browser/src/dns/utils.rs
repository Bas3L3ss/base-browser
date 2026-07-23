use anyhow::{Result, bail};

pub fn decode_name(bytes: &[u8], mut offset: usize) -> Result<(String, usize)> {
    let mut labels = Vec::new();
    let mut jumped = false;
    let mut total_offset = offset;

    loop {
        if offset >= bytes.len() {
            bail!("Offset out of range while decoding DNS name.");
        }

        let len = bytes[offset];
        if len == 0 {
            offset += 1;
            if !jumped {
                total_offset = offset;
            }
            break;
        }

        // Compression pointer: 2 high bits set
        if len & 0xC0 == 0xC0 {
            if offset + 1 >= bytes.len() {
                bail!("Compression pointer out of bounds");
            }
            let pointer = (((len & 0x3F) as usize) << 8) | bytes[offset + 1] as usize;
            if !jumped {
                total_offset = offset + 2;
            }
            // Mark as jumped so that total_offset is not updated anymore after following the pointer
            jumped = true;
            let (name, _) = decode_name(bytes, pointer)?;
            labels.push(name);
            break;
        } else {
            offset += 1;
            let end = offset + (len as usize);
            if end > bytes.len() {
                bail!("Label length beyond packet bounds");
            }
            let label = std::str::from_utf8(&bytes[offset..end])
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in label"))?;
            labels.push(label.to_string());
            offset = end;
            if !jumped {
                total_offset = offset;
            }
        }
    }
    Ok((labels.join("."), total_offset))
}
