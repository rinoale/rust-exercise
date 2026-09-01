//! Tiny "signaling" channel (stands in for what WebRTC apps do over
//! websockets etc.). STUN/TURN only tell a peer its own addresses —
//! the peers still need some out-of-band channel to exchange candidates.
//! This just relays JSON lines between the two connected peers.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn relay(from: TcpStream, mut to: TcpStream) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for line in BufReader::new(from).lines() {
            let Ok(line) = line else { break };
            println!("[signaling] relaying: {line}");
            if writeln!(to, "{line}").is_err() {
                break;
            }
        }
    })
}

fn main() {
    let server = TcpListener::bind("127.0.0.1:5001").expect("bind failed");
    println!("[signaling] listening on 127.0.0.1:5001/tcp (relays candidate info between peers)");

    let (a, _) = server.accept().expect("accept failed");
    let (b, _) = server.accept().expect("accept failed");
    println!("[signaling] both peers connected");

    let t1 = relay(a.try_clone().unwrap(), b.try_clone().unwrap());
    let t2 = relay(b, a);
    let _ = t1.join();
    let _ = t2.join();
}
