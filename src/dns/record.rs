use anyhow::{Result, bail};

use crate::dns::utils::decode_name;

#[derive(Debug)]
/// Represents a DNS Resource Record as specified in RFC 1035.
pub struct DnsRecord {
    pub record_name: String,
    pub record_type: u16,
    pub record_class: u16,
    pub time_to_live: u32,
    pub rdata_length: u16,

    /// A variable-length string of octets that describes the resource. Format varies by TYPE and CLASS.
    pub resource_data: Vec<u8>,
}

impl DnsRecord {
    /// Parses a DNS Resource Record from the supplied bytes starting at `offset`, returning the DnsRecord and the new offset.
    pub fn from_bytes(bytes: &[u8], offset: usize) -> Result<(DnsRecord, usize)> {
        let (record_name, mut offset) = decode_name(bytes, offset)?;
        if offset + 10 > bytes.len() {
            bail!("RR too short for fixed part");
        }
        let record_type = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        let record_class = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]);
        let time_to_live = u32::from_be_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let rdata_length = u16::from_be_bytes([bytes[offset + 8], bytes[offset + 9]]);
        offset += 10;
        if offset + (rdata_length as usize) > bytes.len() {
            bail!(
                "Not enough bytes for rdata (expected {} bytes)",
                rdata_length
            );
        }
        let resource_data = bytes[offset..offset + (rdata_length as usize)].to_vec();
        offset += rdata_length as usize;

        Ok((
            DnsRecord {
                record_name,
                record_type,
                record_class,
                time_to_live,
                rdata_length,
                resource_data,
            },
            offset,
        ))
    }
}
