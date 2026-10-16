use crate::name::encode_name;

pub struct DnsRecord {
    name: String,
    rtype: u16,
    class: u16,
    ttl: u32,
    data: Vec<u8>,   // RDATA; 4 bytes for an A record
}

impl DnsRecord {
    // Builds an "A" record (type 1, class IN) pointing to an IPv4 address.
    pub fn a(name: &str, ttl: u32, ip: [u8; 4]) -> DnsRecord {
        DnsRecord {
            name: name.to_string(),
            rtype: 1,          // A
            class: 1,          // IN
            ttl,
            data: ip.to_vec(), // 4 bytes
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = encode_name(&self.name);
        out.extend_from_slice(&self.rtype.to_be_bytes());
        out.extend_from_slice(&self.class.to_be_bytes());
        out.extend_from_slice(&self.ttl.to_be_bytes());
        out.extend_from_slice(&(self.data.len() as u16).to_be_bytes());
        out.extend_from_slice(&self.data);
        out
    }
}