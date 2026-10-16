use crate::name::{encode_name, parse_name};

#[derive(Debug, Clone, PartialEq)]
pub struct DnsQuestion {
    pub name: String,   // "google.com"
    pub qtype: u16,     // 1 = A https://www.rfc-editor.org/info/rfc1035/#section-3.2.2
    pub qclass: u16,    // 1 = IN https://www.rfc-editor.org/info/rfc1035/#section-3.2.4
}

impl DnsQuestion {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = encode_name(&self.name);
        out.extend_from_slice(&self.qtype.to_be_bytes());
        out.extend_from_slice(&self.qclass.to_be_bytes());
        out
    }

    pub fn from_bytes(buf: &[u8], pos: usize) -> Option<(DnsQuestion, usize)> {
        let (name, pos) = parse_name(buf, pos)?;
        let qtype = u16::from_be_bytes([*buf.get(pos)?, *buf.get(pos + 1)?]);
        let qclass = u16::from_be_bytes([*buf.get(pos + 2)?, *buf.get(pos + 3)?]);
        Some((DnsQuestion { name, qtype, qclass }, pos + 4))
    }
}