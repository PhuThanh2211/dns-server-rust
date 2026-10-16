#[allow(unused_imports)]
use std::net::UdpSocket;

#[derive(Debug, Default)]
struct DnsHeader {
    id: u16,        // Packet Identifier
    qr: bool,       // Query/Repsonse Indicator
    opcode: u8,     // Operation code 4 bits
    aa: bool,       // Authoritative Answer
    tc: bool,       // Truncation
    rd: bool,       // Recursion Desired
    ra: bool,       // Recursion Available
    z: u8,          // Reserved 3 bits
    rcode: u8,      // Response Code 4 bits
    qdcount: u16,   // Question Count
    ancount: u16,   // Answer Record Count
    nscount: u16,   // Authority Record Count
    arcount: u16,   // Additional Record Count
}

impl DnsHeader {

    // Turn the struct into the 12 raw bytes that go on the network:
    fn to_bytes(&self) -> [u8; 12] {
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

    fn from_bytes(buf: &[u8]) -> DnsHeader {
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

struct DnsQuestion {
    name: String,   // "google.com"
    qtype: u16,     // 1 = A https://www.rfc-editor.org/info/rfc1035/#section-3.2.2
    qclass: u16,    // 1 = IN https://www.rfc-editor.org/info/rfc1035/#section-3.2.4
}

impl DnsQuestion {
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = encode_name(&self.name);
        out.extend_from_slice(&self.qtype.to_be_bytes());
        out.extend_from_slice(&self.qclass.to_be_bytes());
        out
    }
}

struct DnsRecord {
    name: String,
    rtype: u16,
    class: u16,
    ttl: u32,
    data: Vec<u8>,   // RDATA; 4 bytes for an A record
}

impl DnsRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = encode_name(&self.name);
        out.extend_from_slice(&self.rtype.to_be_bytes());
        out.extend_from_slice(&self.class.to_be_bytes());
        out.extend_from_slice(&self.ttl.to_be_bytes());
        out.extend_from_slice(&(self.data.len() as u16).to_be_bytes());
        out.extend_from_slice(&self.data);
        out
    }
}

struct DnsMessage {
    header: DnsHeader,
    questions: Vec<DnsQuestion>,
    answers: Vec<DnsRecord>,
}

impl DnsMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.header.to_bytes());

        for q in &self.questions {
            out.extend_from_slice(&q.to_bytes());
        }

        for a in &self.answers {
            out.extend_from_slice(&a.to_bytes());
        }

        out
    }
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                if size < 12 {
                    continue; // Too short to be an DNS header
                }
                let req = DnsHeader::from_bytes(&buf[..size]);

                let msg = DnsMessage {
                    header: DnsHeader {
                        id: req.id,
                        qr: true,
                        opcode: req.opcode,
                        rd: req.rd,
                        rcode: if req.opcode == 0 { 0 } else { 4 },
                        qdcount: 1,
                        ancount: 1,
                        ..Default::default()
                    },
                    questions: vec![DnsQuestion {
                        name: "codecrafters.io".to_string(),
                        qtype: 1,
                        qclass: 1,
                    }],
                    answers: vec![DnsRecord {
                        name: "codecrafters.io".to_string(),
                        rtype: 1,
                        class: 1,
                        ttl: 60,
                        data: vec![8, 8, 8, 8],
                    }],
                };

                udp_socket
                    .send_to(&msg.to_bytes(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}

fn encode_name(name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for label in name.split('.') {
        out.push(label.len() as u8);            // <length>
        out.extend_from_slice(label.as_bytes());// <content>
    }
    out.push(0);
    out
}