//! Standalone TURN server (RFC 5766, simplified: no auth/MESSAGE-INTEGRITY).
//! Also speaks STUN Binding and the rendezvous extension, like real-world
//! servers that serve both roles — so TURN-mode peers only need this one
//! server process.
//!
//!   cargo run --bin turn_server [-- advertised_ip port]   # defaults: 127.0.0.1 3479
//!   VERBOSE=1 cargo run --bin turn_server                 # hexdump bulk data too
//!
//! advertised_ip is the address handed out in XOR-RELAYED-ADDRESS — set it to
//! this machine's LAN/public IP when peers connect from other hosts.
//!
//! Supported TURN messages:
//!   Allocate          -> opens a fresh relay UDP socket for the client,
//!                        returns its address as XOR-RELAYED-ADDRESS
//!   CreatePermission  -> whitelists a peer IP for the allocation
//!   Send Indication   -> client -> TURN: unwrap DATA, forward raw to peer
//!   (relay socket rx) -> peer -> TURN: wrap in Data Indication, forward to
//!                        the allocation's client

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Mutex};
use stun_turn_demo::packet_log::{self, Dir};
use stun_turn_demo::rendezvous::Rendezvous;
use stun_turn_demo::{as_v4, stun};

const BULK: usize = 512; // payloads bigger than this are counted, not hexdumped

#[derive(Default)]
struct Agg {
    n: u64,
    bytes: u64,
}

struct Alloc {
    relay: UdpSocket,
    permissions: Arc<Mutex<Vec<Ipv4Addr>>>,
    agg: Arc<Mutex<Agg>>,
}

fn verbose() -> bool {
    std::env::var("VERBOSE").as_deref() == Ok("1")
}

// File chunks flying through the relay would flood the terminal — log small
// (control/chat) packets in full, count bulk data and print a summary line.
fn log_bulk(role: &str, dir: Dir, remote: &str, data: &[u8], agg: &Mutex<Agg>, note: Option<&str>) {
    if verbose() || data.len() <= BULK {
        packet_log::log(role, dir, remote, data, note);
    } else {
        let mut agg = agg.lock().unwrap();
        agg.n += 1;
        agg.bytes += data.len() as u64;
        if agg.n == 1 || agg.n.is_multiple_of(1000) {
            let suffix = note.map(|n| format!("  ({n})")).unwrap_or_default();
            packet_log::brief(role, dir, remote,
                &format!("bulk data {} bytes ({} pkts / {} bytes relayed so far){suffix}",
                         data.len(), agg.n, agg.bytes));
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let advertised_ip: Ipv4Addr = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(Ipv4Addr::LOCALHOST);
    let port: u16 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(3479);

    let control = UdpSocket::bind(("0.0.0.0", port)).expect("bind failed");
    println!("[turn-server] listening on 0.0.0.0:{port}/udp (relay addresses advertised as {advertised_ip})");

    let mut rendezvous = Rendezvous::new();
    let mut allocations: HashMap<SocketAddrV4, Alloc> = HashMap::new();
    let mut buf = [0u8; 65536];

    loop {
        let Ok((n, from)) = control.recv_from(&mut buf) else { continue };
        let data = &buf[..n];
        let client = as_v4(from);
        let client_key = client.to_string();
        if !stun::is_stun_packet(data) {
            packet_log::log("turn-server", Dir::Recv, &client_key, data, Some("ignored: not STUN"));
            continue;
        }
        let Some(msg) = stun::parse(data) else { continue };

        match msg.typ {
            stun::ALLOCATE_REQUEST => {
                packet_log::log("turn-server", Dir::Recv, &client_key, data, None);
                let relay = UdpSocket::bind(("0.0.0.0", 0)).expect("relay bind failed");
                let relay_port = relay.local_addr().unwrap().port(); // ephemeral port; this becomes the relayed address
                let alloc = Alloc {
                    relay: relay.try_clone().unwrap(),
                    permissions: Arc::new(Mutex::new(vec![])),
                    agg: Arc::new(Mutex::new(Agg::default())),
                };

                // Anything a peer sends to the relayed address gets wrapped in a
                // Data Indication and forwarded to the client that owns the allocation.
                {
                    let permissions = Arc::clone(&alloc.permissions);
                    let agg = Arc::clone(&alloc.agg);
                    let control = control.try_clone().unwrap();
                    std::thread::spawn(move || {
                        let mut buf = [0u8; 65536];
                        while let Ok((n, peer)) = relay.recv_from(&mut buf) {
                            let d = &buf[..n];
                            let peer = as_v4(peer);
                            log_bulk("turn-server", Dir::Recv, &peer.to_string(), d, &agg,
                                     Some(&format!("arrived on relay port {relay_port}")));
                            if !permissions.lock().unwrap().contains(peer.ip()) {
                                println!("[turn-server] DROP: no permission for {} on relay {relay_port}\n", peer.ip());
                                continue;
                            }
                            let ind = stun::build(stun::DATA_INDICATION, &stun::new_txid(), &[
                                (stun::ATTR_XOR_PEER_ADDRESS, stun::xor_address(*peer.ip(), peer.port())),
                                (stun::ATTR_DATA, d.to_vec()),
                            ]);
                            if verbose() || ind.len() <= BULK {
                                packet_log::log("turn-server", Dir::Send, &client.to_string(), &ind,
                                                Some("wrapped as Data Indication for allocation owner"));
                            }
                            let _ = control.send_to(&ind, client);
                        }
                    });
                }

                let resp = stun::build(stun::ALLOCATE_SUCCESS, &msg.txid, &[
                    (stun::ATTR_XOR_RELAYED_ADDRESS, stun::xor_address(advertised_ip, relay_port)),
                    (stun::ATTR_XOR_MAPPED_ADDRESS, stun::xor_address(*client.ip(), client.port())),
                    (stun::ATTR_LIFETIME, 600u32.to_be_bytes().to_vec()),
                ]);
                packet_log::log("turn-server", Dir::Send, &client_key, &resp,
                                Some(&format!("allocated relay {advertised_ip}:{relay_port} for {client_key}")));
                let _ = control.send_to(&resp, client);
                allocations.insert(client, alloc);
            }

            stun::CREATE_PERMISSION_REQ => {
                packet_log::log("turn-server", Dir::Recv, &client_key, data, None);
                let Some(alloc) = allocations.get(&client) else { continue };
                let Some((peer_ip, _)) = msg.attr(stun::ATTR_XOR_PEER_ADDRESS).and_then(stun::unxor_address)
                else { continue };
                alloc.permissions.lock().unwrap().push(peer_ip);
                let resp = stun::build(stun::CREATE_PERMISSION_OK, &msg.txid, &[]);
                packet_log::log("turn-server", Dir::Send, &client_key, &resp,
                                Some(&format!("permission granted for peer IP {peer_ip}")));
                let _ = control.send_to(&resp, client);
            }

            stun::SEND_INDICATION => {
                let Some(alloc) = allocations.get(&client) else { continue };
                log_bulk("turn-server", Dir::Recv, &client_key, data, &alloc.agg, None);
                let Some((peer_ip, peer_port)) = msg.attr(stun::ATTR_XOR_PEER_ADDRESS).and_then(stun::unxor_address)
                else { continue };
                let payload = msg.attr(stun::ATTR_DATA).unwrap_or(&[]);
                let _ = alloc.relay.send_to(payload, (peer_ip, peer_port));
            }

            _ => {
                packet_log::log("turn-server", Dir::Recv, &client_key, data, None);
                if !rendezvous.handle("turn-server", &control, &msg, client) {
                    println!("[turn-server] unhandled message type 0x{:04x}", msg.typ);
                }
            }
        }
    }
}
