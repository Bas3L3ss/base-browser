use crate::dns::enums::{QueryClass, QueryType};
use crate::dns::utils::decode_name;
use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsQuestion {
    pub name: String,
    pub qtype: QueryType,
    pub qclass: QueryClass,
}

impl DnsQuestion {
    pub fn new(name: String, qtype: QueryType, qclass: QueryClass) -> DnsQuestion {
        DnsQuestion {
            name,
            qtype,
            qclass,
        }
    }

    pub fn write_to(&self, packet: &mut Vec<u8>) {
        for label in self.name.split('.') {
            packet.push(label.len() as u8);
            packet.extend(label.as_bytes());
        }
        packet.push(0);

        let qtype_num: u16 = self.qtype.into();
        packet.extend(&qtype_num.to_be_bytes());

        let qclass_num: u16 = self.qclass.into();
        packet.extend(&qclass_num.to_be_bytes());
    }

    /// Parses a DNS Question section from bytes using the shared decode_name util,
    /// matching the structure seen in src/dns/record.rs for RR parsing.
    pub fn from_bytes(bytes: &[u8], offset: usize) -> Result<(DnsQuestion, usize)> {
        let (name, mut offset) = decode_name(bytes, offset)?;
        if offset + 4 > bytes.len() {
            bail!("Unable to parse qtype/qclass for question; out of bounds");
        }
        let qtype = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        let qclass = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]);
        offset += 4;
        Ok((
            DnsQuestion {
                name,
                qtype: QueryType::from(qtype),
                qclass: QueryClass::from(qclass),
            },
            offset,
        ))
    }
}
