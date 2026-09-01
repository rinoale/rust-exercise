//! Minimal STUN/TURN wire-format library (RFC 5389 / RFC 5766, no auth),
//! plus two demo-only extension messages for rendezvous (REGISTER/PEER-INFO).
//!
//! STUN message layout:
//!
//!   0                   1                   2                   3
//!   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |0 0|  Message Type (14 bit)  |        Message Length           |
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |                  Magic Cookie = 0x2112A442                    |
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |                 Transaction ID (96 bits)                      |
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |            Attributes (TLV, padded to 4 bytes) ...            |

use std::net::Ipv4Addr;

pub const MAGIC_COOKIE: u32 = 0x2112_A442;

// Standard message types (class + method bits combined, as on the wire)
pub const BINDING_REQUEST: u16 = 0x0001;
pub const BINDING_SUCCESS: u16 = 0x0101;
pub const ALLOCATE_REQUEST: u16 = 0x0003; // TURN
pub const ALLOCATE_SUCCESS: u16 = 0x0103;
pub const CREATE_PERMISSION_REQ: u16 = 0x0008;
pub const CREATE_PERMISSION_OK: u16 = 0x0108;
pub const SEND_INDICATION: u16 = 0x0016; // client -> TURN, "relay this to peer"
pub const DATA_INDICATION: u16 = 0x0017; // TURN -> client, "peer sent you this"

// Demo-only rendezvous extension (NOT standard STUN — plays the role a
// signaling server plays in WebRTC): peers join a room, the server
// introduces them to each other.
pub const REGISTER_REQUEST: u16 = 0x0042; // request class (0b00)
pub const REGISTER_SUCCESS: u16 = 0x0142; // success class
pub const PEER_INFO_INDICATION: u16 = 0x0052; // indication class (0x0010 bit set)

pub fn type_name(typ: u16) -> Option<&'static str> {
    match typ {
        BINDING_REQUEST => Some("STUN Binding Request"),
        BINDING_SUCCESS => Some("STUN Binding Success Response"),
        ALLOCATE_REQUEST => Some("TURN Allocate Request"),
        ALLOCATE_SUCCESS => Some("TURN Allocate Success Response"),
        CREATE_PERMISSION_REQ => Some("TURN CreatePermission Request"),
        CREATE_PERMISSION_OK => Some("TURN CreatePermission Success Response"),
        SEND_INDICATION => Some("TURN Send Indication"),
        DATA_INDICATION => Some("TURN Data Indication"),
        REGISTER_REQUEST => Some("REGISTER Request (demo rendezvous ext.)"),
        REGISTER_SUCCESS => Some("REGISTER Success Response (demo ext.)"),
        PEER_INFO_INDICATION => Some("PEER-INFO Indication (demo ext.)"),
        _ => None,
    }
}

// Attribute types
pub const ATTR_USERNAME: u16 = 0x0006;
pub const ATTR_XOR_PEER_ADDRESS: u16 = 0x0012;
pub const ATTR_DATA: u16 = 0x0013;
pub const ATTR_XOR_RELAYED_ADDRESS: u16 = 0x0016;
pub const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;
pub const ATTR_LIFETIME: u16 = 0x000D;
pub const ATTR_REQUESTED_TRANSPORT: u16 = 0x0019;
pub const ATTR_SOFTWARE: u16 = 0x8022;

pub fn attr_name(typ: u16) -> Option<&'static str> {
    match typ {
        ATTR_USERNAME => Some("USERNAME"),
        ATTR_XOR_PEER_ADDRESS => Some("XOR-PEER-ADDRESS"),
        ATTR_DATA => Some("DATA"),
        ATTR_XOR_RELAYED_ADDRESS => Some("XOR-RELAYED-ADDRESS"),
        ATTR_XOR_MAPPED_ADDRESS => Some("XOR-MAPPED-ADDRESS"),
        ATTR_LIFETIME => Some("LIFETIME"),
        ATTR_REQUESTED_TRANSPORT => Some("REQUESTED-TRANSPORT"),
        ATTR_SOFTWARE => Some("SOFTWARE"),
        _ => None,
    }
}

pub type TxId = [u8; 12];

#[derive(Debug, Clone)]
pub struct Msg {
    pub typ: u16,
    pub txid: TxId,
    pub attrs: Vec<(u16, Vec<u8>)>,
}

impl Msg {
    pub fn attr(&self, typ: u16) -> Option<&[u8]> {
        self.attrs.iter().find(|(t, _)| *t == typ).map(|(_, v)| v.as_slice())
    }
}

pub fn new_txid() -> TxId {
    rand::random()
}

/// attrs: slice of (attr_type, value_bytes)
pub fn build(typ: u16, txid: &TxId, attrs: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut body = vec![];
    for (t, v) in attrs {
        body.extend_from_slice(&t.to_be_bytes());
        body.extend_from_slice(&(v.len() as u16).to_be_bytes());
        body.extend_from_slice(v);
        body.extend_from_slice(&[0u8; 3][..(4 - v.len() % 4) % 4]);
    }
    let mut pkt = vec![];
    pkt.extend_from_slice(&typ.to_be_bytes());
    pkt.extend_from_slice(&(body.len() as u16).to_be_bytes());
    pkt.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
    pkt.extend_from_slice(txid);
    pkt.extend_from_slice(&body);
    pkt
}

pub fn parse(data: &[u8]) -> Option<Msg> {
    if data.len() < 20 {
        return None;
    }
    let typ = u16::from_be_bytes(data[0..2].try_into().unwrap());
    let len = u16::from_be_bytes(data[2..4].try_into().unwrap()) as usize;
    let txid: TxId = data[8..20].try_into().unwrap();
    let mut attrs = vec![];
    let mut off = 20;
    while off < 20 + len {
        let at = u16::from_be_bytes(data.get(off..off + 2)?.try_into().unwrap());
        let alen = u16::from_be_bytes(data.get(off + 2..off + 4)?.try_into().unwrap()) as usize;
        attrs.push((at, data.get(off + 4..off + 4 + alen)?.to_vec()));
        off += 4 + alen + (4 - alen % 4) % 4;
    }
    Some(Msg { typ, txid, attrs })
}

pub fn success_response(typ: u16) -> bool {
    typ & 0x0100 != 0
}

/// Addresses are XOR'd with the magic cookie so NATs that rewrite
/// literal IP addresses in payloads can't corrupt them.
pub fn xor_address(ip: Ipv4Addr, port: u16) -> Vec<u8> {
    let x_port = port ^ (MAGIC_COOKIE >> 16) as u16;
    let x_ip = u32::from(ip) ^ MAGIC_COOKIE;
    let mut out = vec![0u8, 0x01]; // reserved, family=IPv4
    out.extend_from_slice(&x_port.to_be_bytes());
    out.extend_from_slice(&x_ip.to_be_bytes());
    out
}

pub fn unxor_address(bytes: &[u8]) -> Option<(Ipv4Addr, u16)> {
    if bytes.len() < 8 {
        return None;
    }
    let x_port = u16::from_be_bytes(bytes[2..4].try_into().unwrap());
    let x_ip = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
    Some((
        Ipv4Addr::from(x_ip ^ MAGIC_COOKIE),
        x_port ^ (MAGIC_COOKIE >> 16) as u16,
    ))
}

pub fn is_stun_packet(data: &[u8]) -> bool {
    data.len() >= 20 && data[4..8] == MAGIC_COOKIE.to_be_bytes()
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn describe(data: &[u8]) -> Vec<String> {
    let Some(msg) = parse(data) else {
        return vec!["(unparseable STUN packet)".into()];
    };
    let name = type_name(msg.typ)
        .map(str::to_string)
        .unwrap_or_else(|| format!("unknown type 0x{:04x}", msg.typ));
    let mut lines = vec![format!("{name}  ({} bytes, txid={})", data.len(), hex(&msg.txid))];
    for (t, v) in &msg.attrs {
        let aname = attr_name(*t)
            .map(str::to_string)
            .unwrap_or_else(|| format!("attr-0x{t:04x}"));
        lines.push(format!("  {aname:<22} {}", describe_attr(*t, v)));
    }
    lines
}

fn describe_attr(typ: u16, value: &[u8]) -> String {
    match typ {
        ATTR_XOR_MAPPED_ADDRESS | ATTR_XOR_PEER_ADDRESS | ATTR_XOR_RELAYED_ADDRESS => {
            match unxor_address(value) {
                Some((ip, port)) => format!("{ip}:{port}  (xor-encoded on the wire as {})", hex(value)),
                None => format!("(bad address) 0x{}", hex(value)),
            }
        }
        ATTR_DATA => {
            if value.len() > 64 {
                format!("{} bytes: {:?}...", value.len(), String::from_utf8_lossy(&value[..32]))
            } else {
                format!("{:?}", String::from_utf8_lossy(value))
            }
        }
        ATTR_LIFETIME => {
            let secs = value
                .get(..4)
                .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
                .unwrap_or(0);
            format!("{secs} seconds")
        }
        ATTR_REQUESTED_TRANSPORT => {
            let proto = value.first().copied().unwrap_or(0);
            format!("{proto}{}", if proto == 17 { " (UDP)" } else { "" })
        }
        ATTR_SOFTWARE | ATTR_USERNAME => format!("{:?}", String::from_utf8_lossy(value)),
        _ => hex(value),
    }
}

pub fn hexdump(data: &[u8]) -> Vec<String> {
    hexdump_opts(data, "    ", 512)
}

pub fn hexdump_opts(data: &[u8], indent: &str, max_bytes: usize) -> Vec<String> {
    let truncated = data.len() > max_bytes;
    let shown = if truncated { &data[..max_bytes] } else { data };
    let mut lines: Vec<String> = shown
        .chunks(16)
        .enumerate()
        .map(|(i, chunk)| {
            let hex = chunk
                .chunks(8)
                .map(|g| g.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join("  ");
            let ascii: String = chunk
                .iter()
                .map(|&b| if (32..=126).contains(&b) { b as char } else { '.' })
                .collect();
            format!("{indent}{:04x}  {hex:<49} |{ascii}|", i * 16)
        })
        .collect();
    if truncated {
        lines.push(format!("{indent}... ({} more bytes)", data.len() - max_bytes));
    }
    lines
}

pub fn banner(text: &str) {
    print!("\n{}\n== {text}\n{}\n\n", "=".repeat(76), "=".repeat(76));
    use std::io::Write;
    let _ = std::io::stdout().flush();
}
