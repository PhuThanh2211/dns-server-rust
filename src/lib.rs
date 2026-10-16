mod header;
mod message;
mod name;
mod question;
mod record;

use std::net::UdpSocket;
use message::DnsMessage;

pub fn handle_query(req: &[u8]) -> Option<Vec<u8>> {
    let msg = DnsMessage::parse(req)?;
    Some(msg.build_response().to_bytes())
}

pub fn serve(sock: UdpSocket) {
    let mut buf = [0u8; 512];
    loop {
        let Ok((n, src)) = sock.recv_from(&mut buf) else { break };
        if let Some(resp) = handle_query(&buf[..n]) {
            let _ = sock.send_to(&resp, src);
        }
    }
}