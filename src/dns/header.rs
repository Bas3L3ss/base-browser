use crate::dns::flag::Flag;

/// Represents the header section of a DNS message as per RFC 1035.
///
/// The DNS header includes various control fields such as QR, Opcode, AA, TC, RD, RA, Z, and RCODE,
/// all of which are packed into the `flags` field as described in RFC 1035 section 4.1.1.
/// See: https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
///
/// You may extract or manipulate individual flag bits using bitwise operations,
/// but the packed representation is standard for serializing DNS headers over the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub id: u16,

    /// 16-bit packed flags field.
    /// Bits (from MSB to LSB):
    ///  QR | Opcode (4) | AA | TC | RD | RA | Z | RCODE (4)
    pub flag: Flag, // bits include QR, opcode, AA, TC, RD, RA, Z, RCODE

    pub num_questions: u16,
    pub num_answers: u16,
    pub num_authorities: u16,
    pub num_additionals: u16,
}

impl Header {
    pub fn new(
        id: u16,
        flag: Flag,
        num_questions: u16,
        num_answers: u16,
        num_authorities: u16,
        num_additionals: u16,
    ) -> Header {
        Header {
            id,
            flag,
            num_questions,
            num_answers,
            num_authorities,
            num_additionals,
        }
    }

    /// Constructs a Header from a slice of bytes (network byte order).
    /// Returns an error if the input is not at least 12 bytes.
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Header> {
        if bytes.len() < 12 {
            anyhow::bail!(
                "DNS header too short: expected 12 bytes, got {}",
                bytes.len()
            );
        }
        let id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let flag = Flag::from_bytes(&bytes[2..4])?;
        let num_questions = u16::from_be_bytes([bytes[4], bytes[5]]);
        let num_answers = u16::from_be_bytes([bytes[6], bytes[7]]);
        let num_authorities = u16::from_be_bytes([bytes[8], bytes[9]]);
        let num_additionals = u16::from_be_bytes([bytes[10], bytes[11]]);
        Ok(Header {
            id,
            flag,
            num_questions,
            num_answers,
            num_authorities,
            num_additionals,
        })
    }

    /// Writes the header to the given packet in network byte order.
    /// This serializes the entire header, including the packed flags field.
    pub fn write_to(&self, packet: &mut Vec<u8>) {
        packet.extend(&self.id.to_be_bytes());
        packet.extend(&self.flag.to_bytes());
        packet.extend(&self.num_questions.to_be_bytes());
        packet.extend(&self.num_answers.to_be_bytes());
        packet.extend(&self.num_authorities.to_be_bytes());
        packet.extend(&self.num_additionals.to_be_bytes());
    }

    // Helper methods to extract individual flag fields if desired,
    // e.g. qr(), opcode(), z(), rcode(), etc. See RFC for bit layouts.
    // These are optional and can be implemented as needed.
}
