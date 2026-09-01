//! Standalone STUN + rendezvous server.
//!
//!   cargo run --bin stun_server [-- port]      # default port 3478
//!
//! - Answers Binding Requests with the client's reflexive address
//!   (XOR-MAPPED-ADDRESS): "this is what you look like from outside".
//! - Mediates peers: clients REGISTER into a room, and when two peers are in
//!   the same room the server sends each one a PEER-INFO Indication carrying
//!   the other's IP:port. After that the peers talk directly — no traffic
//!   flows through this server.

use std::net::UdpSocket;
use stun_turn_demo::packet_log::{self, Dir};
use stun_turn_demo::rendezvous::Rendezvous;
use stun_turn_demo::{as_v4, stun};

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(3478);
    let sock = UdpSocket::bind(("0.0.0.0", port)).expect("bind failed");
    println!("[stun-server] listening on 0.0.0.0:{port}/udp (STUN binding + room rendezvous)");

    let mut rendezvous = Rendezvous::new();
    let mut buf = [0u8; 2048];
    loop {
        let Ok((n, from)) = sock.recv_from(&mut buf) else { continue };
        let data = &buf[..n];
        if !stun::is_stun_packet(data) {
            continue;
        }
        let from = as_v4(from);
        packet_log::log("stun-server", Dir::Recv, &from.to_string(), data, None);
        let Some(msg) = stun::parse(data) else { continue };
        if !rendezvous.handle("stun-server", &sock, &msg, from) {
            println!("[stun-server] unhandled message type 0x{:04x}", msg.typ);
        }
    }
}
