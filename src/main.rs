#[allow(unused_imports)]
use std::net::UdpSocket;

fn main() {
    let sock = UdpSocket::bind("127.0.0.1:2053").expect("bind");
    codecrafters_dns_server::serve(sock);
}