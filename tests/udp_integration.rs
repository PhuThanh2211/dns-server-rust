use codecrafters_dns_server::serve;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

// Start a server on a random free port and return its address.
fn start_server() -> SocketAddr {
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = sock.local_addr().unwrap();
    thread::spawn(move || serve(sock));
    addr
}

// Build a query: header + one question (name, type A, class IN).
fn build_query(id: u16, flags: u16, name: &str) -> Vec<u8> {
    let mut q = Vec::new();
    q.extend(id.to_be_bytes());
    q.extend(flags.to_be_bytes());
    q.extend([0, 1, 0, 0, 0, 0, 0, 0]); // QD=1, AN=NS=AR=0
    for label in name.split('.') {
        q.push(label.len() as u8);
        q.extend(label.as_bytes());
    }
    q.push(0);
    q.extend([0, 1, 0, 1]); // type A, class IN
    q
}

// Send a packet and wait up to 2s for the reply.
fn exchange(server: SocketAddr, packet: &[u8]) -> Vec<u8> {
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    client.send_to(packet, server).unwrap();
    let mut buf = [0u8; 512];
    let (n, _) = client.recv_from(&mut buf).expect("no reply");
    buf[..n].to_vec()
}

#[test]
fn standard_query_gets_full_answer() {
    let server = start_server();
    let query = build_query(1234, 0x0100, "codecrafters.io"); // RD=1
    let resp = exchange(server, &query);

    assert_eq!(&resp[0..2], &[0x04, 0xD2]);   // same ID
    assert_eq!(resp[2], 0x81);                // QR=1, opcode=0, RD=1
    assert_eq!(resp[3] & 0x0F, 0);            // RCODE = 0
    assert_eq!(&resp[4..6], &[0, 1]);         // QDCOUNT
    assert_eq!(&resp[6..8], &[0, 1]);         // ANCOUNT

    // Question section echoes the query's question bytes exactly.
    let q_end = query.len();
    assert_eq!(&resp[12..q_end], &query[12..q_end]);

    // The answer ends with RDLENGTH=4 and the IP.
    assert_eq!(&resp[resp.len() - 6..], &[0, 4, 8, 8, 8, 8]);
}

#[test]
fn non_standard_opcode_returns_notimp() {
    let server = start_server();
    let query = build_query(7, 0x1100, "example.com"); // opcode 2, RD=1
    let resp = exchange(server, &query);

    assert_eq!(&resp[0..2], &[0, 7]);
    assert_eq!(resp[2], 0x91);                // QR | opcode 2 | RD
    assert_eq!(resp[3] & 0x0F, 4);            // NOTIMP
}

#[test]
fn each_request_gets_its_own_id() {
    let server = start_server();
    for id in [1u16, 500, 65535] {
        let resp = exchange(server, &build_query(id, 0, "a.io"));
        assert_eq!(u16::from_be_bytes([resp[0], resp[1]]), id);
        assert_eq!(resp[2] & 0x01, 0);        // RD=0 mimicked
    }
}

#[test]
fn malformed_packet_does_not_kill_server() {
    let server = start_server();

    // Too short to be a DNS header: the server should ignore it, so use a short timeout.
    let c = UdpSocket::bind("127.0.0.1:0").unwrap();
    c.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
    c.send_to(&[1, 2, 3], server).unwrap();
    let mut buf = [0u8; 512];
    assert!(c.recv_from(&mut buf).is_err(), "expected no reply");

    // The server must still answer a valid query afterwards.
    let resp = exchange(server, &build_query(9, 0x0100, "ok.io"));
    assert_eq!(&resp[0..2], &[0, 9]);
}