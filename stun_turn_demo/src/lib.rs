pub mod packet_log;
pub mod rendezvous;
pub mod stun;

use std::net::{SocketAddr, SocketAddrV4};

/// The demo is IPv4-only (STUN address attributes here encode family 0x01);
/// normalizes recv_from's SocketAddr.
pub fn as_v4(addr: SocketAddr) -> SocketAddrV4 {
    match addr {
        SocketAddr::V4(a) => a,
        SocketAddr::V6(a) => panic!("IPv6 peer not supported by this demo: {a}"),
    }
}
