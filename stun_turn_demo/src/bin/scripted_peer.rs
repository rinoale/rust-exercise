//! A peer that connects to another peer, first directly (addresses learned
//! via STUN), then through a TURN relay.
//!
//!   cargo run --bin scripted_peer -- A    # the "offerer": drives the demo, owns the TURN allocation
//!   cargo run --bin scripted_peer -- B    # the "answerer"

use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpStream, UdpSocket};
use stun_turn_demo::packet_log::{self, Dir};
use stun_turn_demo::{as_v4, stun};

const STUN_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::LOCALHOST, 3478);
const TURN_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::LOCALHOST, 3479);

fn sig_send(sig: &mut TcpStream, value: serde_json::Value) {
    writeln!(sig, "{value}").expect("signaling send failed");
}

fn sig_recv(sig: &mut BufReader<TcpStream>) -> serde_json::Value {
    let mut line = String::new();
    sig.read_line(&mut line).expect("signaling recv failed");
    serde_json::from_str(&line).expect("bad signaling JSON")
}

fn addr_of(value: &serde_json::Value) -> (Ipv4Addr, u16) {
    (
        value["ip"].as_str().unwrap().parse().unwrap(),
        value["port"].as_u64().unwrap() as u16,
    )
}

fn main() {
    let role = std::env::args().nth(1).unwrap_or_default();
    if role != "A" && role != "B" {
        eprintln!("usage: scripted_peer A|B");
        std::process::exit(2);
    }
    let me = format!("peer-{role}");
    let turn_s = format!("{}:{}", TURN_ADDR.0, TURN_ADDR.1);

    let sig_stream = TcpStream::connect("127.0.0.1:5001").expect("signaling server not running");
    let mut sig_w = sig_stream.try_clone().unwrap();
    let mut sig_r = BufReader::new(sig_stream);

    // One UDP socket for everything (STUN, P2P, TURN) — matching ports is what
    // makes the reflexive address reusable for hole punching in real NATs.
    let sock = UdpSocket::bind("127.0.0.1:0").expect("bind failed");
    let mut buf = [0u8; 1500];

    // ---------------------------------------------------------------- STEP 1
    // Ask the STUN server "what do I look like from the outside?"
    // B waits for A's candidate first, purely to keep the demo output ordered.
    let mut peer_cand = None;
    if role == "B" {
        peer_cand = Some(sig_recv(&mut sig_r));
    }

    stun::banner(&format!("STEP 1: {me} asks the STUN server for its reflexive address"));

    let req = stun::build(stun::BINDING_REQUEST, &stun::new_txid(),
                          &[(stun::ATTR_SOFTWARE, format!("rust-demo-{me}").into_bytes())]);
    let stun_s = format!("{}:{}", STUN_ADDR.0, STUN_ADDR.1);
    packet_log::log(&me, Dir::Send, &stun_s, &req, None);
    sock.send_to(&req, STUN_ADDR).unwrap();

    let (n, _) = sock.recv_from(&mut buf).unwrap();
    packet_log::log(&me, Dir::Recv, &stun_s, &buf[..n], None);
    let (my_ip, my_port) = stun::parse(&buf[..n])
        .and_then(|m| m.attr(stun::ATTR_XOR_MAPPED_ADDRESS).and_then(stun::unxor_address))
        .expect("no XOR-MAPPED-ADDRESS");
    println!("[{me}] STUN says my reflexive address is {my_ip}:{my_port}");

    // ---------------------------------------------------------------- STEP 2
    // Exchange candidates over the signaling channel (out-of-band; STUN/TURN
    // never do this part for you).
    if role == "A" {
        stun::banner("STEP 2: peers exchange candidates over the signaling channel (TCP, not STUN)");
    }
    sig_send(&mut sig_w, json!({ "role": role, "kind": "candidate", "ip": my_ip.to_string(), "port": my_port }));
    let peer_cand = peer_cand.unwrap_or_else(|| sig_recv(&mut sig_r));
    let (peer_ip, peer_port) = addr_of(&peer_cand);
    println!("[{me}] learned peer candidate {peer_ip}:{peer_port} via signaling");

    // ---------------------------------------------------------------- STEP 3
    // Direct P2P: send straight to the other peer's reflexive address.
    // (Behind real NATs both sides send simultaneously — "hole punching".)
    if role == "A" {
        stun::banner("STEP 3: direct P2P — peers talk to each other, no server in the path");
        let msg = b"PING direct P2P from A";
        packet_log::log(&me, Dir::Send, &format!("{peer_ip}:{peer_port}"), msg, Some("direct to peer"));
        sock.send_to(msg, (peer_ip, peer_port)).unwrap();
        let (n, from) = sock.recv_from(&mut buf).unwrap();
        packet_log::log(&me, Dir::Recv, &as_v4(from).to_string(), &buf[..n], Some("direct from peer"));
    } else {
        let (n, from) = sock.recv_from(&mut buf).unwrap();
        let from = as_v4(from);
        packet_log::log(&me, Dir::Recv, &from.to_string(), &buf[..n], Some("direct from peer"));
        let reply = b"PONG direct P2P from B";
        packet_log::log(&me, Dir::Send, &from.to_string(), reply, Some("direct to peer"));
        sock.send_to(reply, from).unwrap();
    }

    // ---------------------------------------------------------------- STEP 4
    // TURN: when direct P2P is impossible (symmetric NAT etc.), peer A rents a
    // relay address on the TURN server and all traffic flows through it.
    if role == "A" {
        stun::banner("STEP 4: TURN relay — peer-A allocates, then traffic flows via the server");

        // 4a. Allocate a relay address
        let req = stun::build(stun::ALLOCATE_REQUEST, &stun::new_txid(), &[
            (stun::ATTR_REQUESTED_TRANSPORT, vec![17, 0, 0, 0]), // 17 = UDP
            (stun::ATTR_LIFETIME, 600u32.to_be_bytes().to_vec()),
        ]);
        packet_log::log(&me, Dir::Send, &turn_s, &req, None);
        sock.send_to(&req, TURN_ADDR).unwrap();
        let (n, _) = sock.recv_from(&mut buf).unwrap();
        packet_log::log(&me, Dir::Recv, &turn_s, &buf[..n], None);
        let (relay_ip, relay_port) = stun::parse(&buf[..n])
            .and_then(|m| m.attr(stun::ATTR_XOR_RELAYED_ADDRESS).and_then(stun::unxor_address))
            .expect("no XOR-RELAYED-ADDRESS");
        println!("[{me}] TURN gave me relayed address {relay_ip}:{relay_port}\n");

        // 4b. Permit peer B to send to my relay
        let req = stun::build(stun::CREATE_PERMISSION_REQ, &stun::new_txid(), &[
            (stun::ATTR_XOR_PEER_ADDRESS, stun::xor_address(peer_ip, peer_port)),
        ]);
        packet_log::log(&me, Dir::Send, &turn_s, &req, None);
        sock.send_to(&req, TURN_ADDR).unwrap();
        let (n, _) = sock.recv_from(&mut buf).unwrap();
        packet_log::log(&me, Dir::Recv, &turn_s, &buf[..n], None);

        // 4c. Tell B where my relay lives (signaling again)
        sig_send(&mut sig_w, json!({ "role": role, "kind": "relay", "ip": relay_ip.to_string(), "port": relay_port }));

        // 4d. B's message arrives wrapped in a Data Indication
        let (n, _) = sock.recv_from(&mut buf).unwrap();
        packet_log::log(&me, Dir::Recv, &turn_s, &buf[..n], Some("B's data, wrapped by TURN"));
        let msg = stun::parse(&buf[..n]).expect("bad Data Indication");
        let (b_ip, b_port) = msg.attr(stun::ATTR_XOR_PEER_ADDRESS).and_then(stun::unxor_address)
            .expect("no XOR-PEER-ADDRESS");
        println!("[{me}] received via relay from {b_ip}:{b_port}: {:?}\n",
                 String::from_utf8_lossy(msg.attr(stun::ATTR_DATA).unwrap_or(&[])));

        // 4e. Reply through the relay with a Send Indication
        let ind = stun::build(stun::SEND_INDICATION, &stun::new_txid(), &[
            (stun::ATTR_XOR_PEER_ADDRESS, stun::xor_address(b_ip, b_port)),
            (stun::ATTR_DATA, b"REPLY via TURN relay from A".to_vec()),
        ]);
        packet_log::log(&me, Dir::Send, &turn_s, &ind, Some("asking TURN to relay my reply to B"));
        sock.send_to(&ind, TURN_ADDR).unwrap();
    } else {
        let relay = sig_recv(&mut sig_r);
        let (relay_ip, relay_port) = addr_of(&relay);
        let relay_s = format!("{relay_ip}:{relay_port}");
        println!("[{me}] learned A's relay address {relay_s} via signaling\n");

        // B just sends plain UDP to the relayed address — from B's point of view
        // the relay looks exactly like peer A would.
        let msg = b"HELLO via TURN relay from B";
        packet_log::log(&me, Dir::Send, &relay_s, msg, Some("A's relayed address on the TURN server"));
        sock.send_to(msg, (relay_ip, relay_port)).unwrap();

        let (n, from) = sock.recv_from(&mut buf).unwrap();
        packet_log::log(&me, Dir::Recv, &as_v4(from).to_string(), &buf[..n],
                        Some("A's reply, forwarded by the relay"));
    }

    std::thread::sleep(std::time::Duration::from_millis(300)); // let in-flight log output flush before the runner tears down
    println!("[{me}] done");
}
