use anyhow::{Result, bail};

use crate::dns::enums::{Opcode, Rcode};

/// Represents parsed DNS Flag fields as per RFC 1035 section 4.1.1
/// |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flag {
    pub is_response: bool,         // Query/Response (bit 15)
    pub operation_code: Opcode,    // Operation code (bits 14-11)
    pub is_authoritative: bool,    // Authoritative Answer (bit 10)
    pub is_truncated: bool,        // Truncation (bit 9)
    pub recursion_desired: bool,   // Recursion Desired (bit 8)
    pub recursion_available: bool, // Recursion Available (bit 7)
    pub reserved_z: u8, // Reserved (bits 6-4), always zero in queries and responses per RFC
    pub response_code: Rcode, // Response code (bits 3-0)
}

impl Flag {
    pub fn new() -> Self {
        Self {
            is_response: false,
            operation_code: Opcode::Query,
            is_authoritative: false,
            is_truncated: false,
            recursion_desired: true,
            recursion_available: false,
            reserved_z: 0,
            response_code: Rcode::NoError,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // DNS flags field is 2 bytes, big endian
        if bytes.len() < 2 {
            bail!(
                "DNS flags field requires at least 2 bytes, got {}",
                bytes.len()
            );
        } else {
            let flags = u16::from_be_bytes([bytes[0], bytes[1]]);

            // Extract fields using bit masks and shifts as per RFC 1035 §4.1.1 layout:
            // QR:         0x8000      (1 bit  at 15)
            // Opcode:     0x7800      (4 bits at 14..11)
            // AA:         0x0400      (1 bit  at 10)
            // TC:         0x0200      (1 bit  at 9)
            // RD:         0x0100      (1 bit  at 8)
            // RA:         0x0080      (1 bit  at 7)
            // Z:          0x0070      (3 bits at 6..4)
            // RCODE:      0x000F      (4 bits at 3..0)

            Ok(Flag {
                is_response: (flags & 0x8000) != 0,
                operation_code: Opcode::from(((flags & 0x7800) >> 11) as u8),
                is_authoritative: (flags & 0x0400) != 0,
                is_truncated: (flags & 0x0200) != 0,
                recursion_desired: (flags & 0x0100) != 0,
                recursion_available: (flags & 0x0080) != 0,
                reserved_z: ((flags & 0x0070) >> 4) as u8,
                response_code: Rcode::from((flags & 0x000F) as u8),
            })
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Compose the 16-bit flags field as per RFC 1035 §4.1.1
        // QR:         0x8000      (1 bit  at 15)
        // Opcode:     0x7800      (4 bits at 14..11)
        // AA:         0x0400      (1 bit  at 10)
        // TC:         0x0200      (1 bit  at 9)
        // RD:         0x0100      (1 bit  at 8)
        // RA:         0x0080      (1 bit  at 7)
        // Z:          0x0070      (3 bits at 6..4)
        // RCODE:      0x000F      (4 bits at 3..0)
        let mut flags: u16 = 0;
        if self.is_response {
            flags |= 0x8000;
        }
        flags |= (u8::from(self.operation_code) as u16 & 0x0F) << 11;
        if self.is_authoritative {
            flags |= 0x0400;
        }
        if self.is_truncated {
            flags |= 0x0200;
        }
        if self.recursion_desired {
            flags |= 0x0100;
        }
        if self.recursion_available {
            flags |= 0x0080;
        }
        flags |= ((self.reserved_z & 0x07) as u16) << 4;
        flags |= (u8::from(self.response_code) as u16) & 0x000F;

        flags.to_be_bytes().to_vec()
    }
}
