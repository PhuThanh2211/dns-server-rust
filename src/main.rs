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
}

struct DnsMessage {
    header: DnsHeader,
}

impl DnsMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.header.to_bytes());
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
                let msg = DnsMessage {
                    header: DnsHeader {
                        id: 1234,
                        qr: true,
                        ..Default::default()
                    },
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
