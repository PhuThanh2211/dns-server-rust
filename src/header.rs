#[derive(Debug, Default, Clone, PartialEq)]
pub struct DnsHeader {
    pub id: u16,        // Packet Identifier
    pub qr: bool,       // Query/Repsonse Indicator
    pub opcode: u8,     // Operation code 4 bits
    pub aa: bool,       // Authoritative Answer
    pub tc: bool,       // Truncation
    pub rd: bool,       // Recursion Desired
    pub ra: bool,       // Recursion Available
    pub z: u8,          // Reserved 3 bits
    pub rcode: u8,      // Response Code 4 bits
    pub qdcount: u16,   // Question Count
    pub ancount: u16,   // Answer Record Count
    pub nscount: u16,   // Authority Record Count
    pub arcount: u16,   // Additional Record Count
}

impl DnsHeader {

    // Turn the struct into the 12 raw bytes that go on the network:
    pub fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[0..2].copy_from_slice(&self.id.to_be_bytes());

        // Byte 2: QR(1) OPCODE(4) AA(1) TC(1) RD(1)
        buf[2] = (self.qr as u8) << 7
            | (self.opcode & 0x0F) << 3
            | (self.aa as u8) << 2
            | (self.tc as u8) << 1
            | (self.rd as u8);

        // Byte 3: RA(1) Z(3) RCODE(4)
        buf[3] = (self.ra as u8) << 7
            | (self.z & 0x07) << 4
            | (self.rcode & 0x0F);
        buf[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        buf[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        buf[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        buf[10..12].copy_from_slice(&self.arcount.to_be_bytes());

        buf
    }

    // Turn the 12 received bytes into a struct.
    pub fn from_bytes(buf: &[u8]) -> DnsHeader {
        DnsHeader {
            id: u16::from_be_bytes([buf[0], buf[1]]),
            qr: buf[2] & 0b1000_0000 != 0,
            opcode: (buf[2] >> 3) & 0x0F,
            aa: buf[2] & 0b0000_0100 != 0,
            tc: buf[2] & 0b0000_0010 != 0,
            rd: buf[2] & 0b0000_0001 != 0,
            ra: buf[3] & 0b1000_0000 != 0,
            z: (buf[3] >> 4) & 0x07,
            rcode: buf[3] & 0x0F,
            qdcount: u16::from_be_bytes([buf[4], buf[5]]),
            ancount: u16::from_be_bytes([buf[6], buf[7]]),
            nscount: u16::from_be_bytes([buf[8], buf[9]]),
            arcount: u16::from_be_bytes([buf[10], buf[11]]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_bytes_stage1() {
        let h = DnsHeader { id: 1234, qr: true, ..Default::default() };
        assert_eq!(h.to_bytes(), [0x04,0xD2,0x80,0,0,0,0,0,0,0,0,0]);
    }

    #[test]
    fn roundtrip() {
        let h = DnsHeader { id: 7, qr: true, opcode: 2, rd: true, rcode: 4, qdcount: 1, ..Default::default() };
        assert_eq!(DnsHeader::from_bytes(&h.to_bytes()), h);
    }
}