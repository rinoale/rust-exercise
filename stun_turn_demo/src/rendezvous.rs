//! Rendezvous: the "mediation" part of the demo servers.
//!
//! Real STUN/TURN servers don't introduce peers to each other — that's the
//! job of an application signaling server (websockets etc. in WebRTC). Here
//! we fold it into the server with two demo extension messages so a single
//! server process can mediate:
//!
//!   peer ── REGISTER(USERNAME "room/name" [, XOR-RELAYED-ADDRESS]) ──> server
//!   peer <── REGISTER Success(XOR-MAPPED-ADDRESS) ── server
//!   ...when another peer registers in the same room...
//!   peer <── PEER-INFO Indication(USERNAME, XOR-PEER-ADDRESS
//!                                 [, XOR-RELAYED-ADDRESS]) ── server

use crate::packet_log::{self, Dir};
use crate::stun;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

#[derive(Clone)]
struct Entry {
    name: String,
    addr: SocketAddrV4,
    relay: Option<(Ipv4Addr, u16)>,
}

#[derive(Default)]
pub struct Rendezvous {
    rooms: HashMap<String, HashMap<String, Entry>>, // room => { name => entry }
}

impl Rendezvous {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the message was handled.
    pub fn handle(&mut self, role: &str, sock: &UdpSocket, msg: &stun::Msg, from: SocketAddrV4) -> bool {
        match msg.typ {
            stun::BINDING_REQUEST => {
                let resp = stun::build(stun::BINDING_SUCCESS, &msg.txid, &[
                    (stun::ATTR_XOR_MAPPED_ADDRESS, stun::xor_address(*from.ip(), from.port())),
                    (stun::ATTR_SOFTWARE, format!("rust-demo-{role}").into_bytes()),
                ]);
                packet_log::log(role, Dir::Send, &from.to_string(), &resp,
                                Some(&format!("reflexive address is {from}")));
                let _ = sock.send_to(&resp, from);
                true
            }

            stun::REGISTER_REQUEST => {
                let username = msg
                    .attr(stun::ATTR_USERNAME)
                    .map(|v| String::from_utf8_lossy(v).into_owned())
                    .unwrap_or_else(|| "/".into());
                let (room, name) = username.split_once('/').unwrap_or(("", &username));
                let relay = msg.attr(stun::ATTR_XOR_RELAYED_ADDRESS).and_then(stun::unxor_address);
                let entry = Entry { name: name.to_string(), addr: from, relay };

                let room_map = self.rooms.entry(room.to_string()).or_default();
                let known = room_map.contains_key(name);
                room_map.insert(name.to_string(), entry.clone());
                let others: Vec<Entry> =
                    room_map.values().filter(|e| e.name != name).cloned().collect();

                let note = if others.is_empty() {
                    format!("'{name}' registered, waiting for a peer")
                } else {
                    format!("'{name}' registered, introducing peers")
                };
                let resp = stun::build(stun::REGISTER_SUCCESS, &msg.txid, &[
                    (stun::ATTR_XOR_MAPPED_ADDRESS, stun::xor_address(*from.ip(), from.port())),
                    (stun::ATTR_SOFTWARE, note.clone().into_bytes()),
                ]);
                if !known {
                    packet_log::log(role, Dir::Send, &from.to_string(), &resp, Some(&note));
                }
                let _ = sock.send_to(&resp, from);

                if !known {
                    // don't re-announce on keepalive re-registrations
                    for other in &others {
                        send_peer_info(role, sock, other, &entry); // tell existing peer about newcomer
                        send_peer_info(role, sock, &entry, other); // tell newcomer about existing peer
                    }
                }
                true
            }

            _ => false,
        }
    }
}

fn send_peer_info(role: &str, sock: &UdpSocket, to: &Entry, about: &Entry) {
    let mut attrs = vec![
        (stun::ATTR_USERNAME, about.name.clone().into_bytes()),
        (stun::ATTR_XOR_PEER_ADDRESS, stun::xor_address(*about.addr.ip(), about.addr.port())),
    ];
    if let Some((ip, port)) = about.relay {
        attrs.push((stun::ATTR_XOR_RELAYED_ADDRESS, stun::xor_address(ip, port)));
    }
    let pkt = stun::build(stun::PEER_INFO_INDICATION, &stun::new_txid(), &attrs);
    packet_log::log(role, Dir::Send, &to.addr.to_string(), &pkt,
                    Some(&format!("introducing '{}' to '{}'", about.name, to.name)));
    let _ = sock.send_to(&pkt, to.addr);
}
