//! HPACK (RFC 7541) decoder — one instance per direction, it is stateful.

use crate::hpack_huffman::HPACK_HUFFMAN;

pub const STATIC_TABLE: [(&str, &str); 62] = [
    ("", ""), // index 0 is unused; the table is 1-based
    (":authority", ""), (":method", "GET"), (":method", "POST"), (":path", "/"),
    (":path", "/index.html"), (":scheme", "http"), (":scheme", "https"), (":status", "200"),
    (":status", "204"), (":status", "206"), (":status", "304"), (":status", "400"),
    (":status", "404"), (":status", "500"), ("accept-charset", ""),
    ("accept-encoding", "gzip, deflate"), ("accept-language", ""), ("accept-ranges", ""),
    ("accept", ""), ("access-control-allow-origin", ""), ("age", ""), ("allow", ""),
    ("authorization", ""), ("cache-control", ""), ("content-disposition", ""),
    ("content-encoding", ""), ("content-language", ""), ("content-length", ""),
    ("content-location", ""), ("content-range", ""), ("content-type", ""), ("cookie", ""),
    ("date", ""), ("etag", ""), ("expect", ""), ("expires", ""), ("from", ""), ("host", ""),
    ("if-match", ""), ("if-modified-since", ""), ("if-none-match", ""), ("if-range", ""),
    ("if-unmodified-since", ""), ("last-modified", ""), ("link", ""), ("location", ""),
    ("max-forwards", ""), ("proxy-authenticate", ""), ("proxy-authorization", ""),
    ("range", ""), ("referer", ""), ("refresh", ""), ("retry-after", ""), ("server", ""),
    ("set-cookie", ""), ("strict-transport-security", ""), ("transfer-encoding", ""),
    ("user-agent", ""), ("vary", ""), ("via", ""), ("www-authenticate", ""),
];

/// Huffman decoding tree, stored as an arena: each node is a pair of edges,
/// each edge either empty, another node (by index), or a decoded symbol.
#[derive(Clone, Copy)]
enum Edge {
    Empty,
    Node(usize),
    Leaf(u16),
}

struct Tree {
    nodes: Vec<[Edge; 2]>,
}

fn build_tree() -> Tree {
    let mut nodes: Vec<[Edge; 2]> = vec![[Edge::Empty, Edge::Empty]];
    for (sym, &(code, bits)) in HPACK_HUFFMAN.iter().enumerate() {
        let mut idx = 0;
        for i in (0..bits).rev() {
            let bit = ((code >> i) & 1) as usize;
            if i == 0 {
                nodes[idx][bit] = Edge::Leaf(sym as u16);
            } else {
                idx = match nodes[idx][bit] {
                    Edge::Node(n) => n,
                    _ => {
                        nodes.push([Edge::Empty, Edge::Empty]);
                        let n = nodes.len() - 1;
                        nodes[idx][bit] = Edge::Node(n);
                        n
                    }
                };
            }
        }
    }
    Tree { nodes }
}

fn huffman_decode(tree: &Tree, bytes: &[u8]) -> String {
    let mut out = Vec::new();
    let mut idx = 0;
    for &byte in bytes {
        for i in (0..8).rev() {
            let bit = ((byte >> i) & 1) as usize;
            match tree.nodes[idx][bit] {
                Edge::Empty => return String::from_utf8_lossy(&out).into_owned(), // malformed
                Edge::Leaf(256) => return String::from_utf8_lossy(&out).into_owned(), // EOS
                Edge::Leaf(sym) => {
                    out.push(sym as u8);
                    idx = 0;
                }
                Edge::Node(n) => idx = n,
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub struct Hpack {
    tree: Tree,
    dynamic: Vec<(String, String)>,
}

impl Default for Hpack {
    fn default() -> Self {
        Self::new()
    }
}

impl Hpack {
    pub fn new() -> Self {
        Hpack { tree: build_tree(), dynamic: vec![] }
    }

    fn lookup(&self, idx: usize) -> (String, String) {
        if (1..=61).contains(&idx) {
            let (n, v) = STATIC_TABLE[idx];
            (n.to_string(), v.to_string())
        } else {
            self.dynamic
                .get(idx.wrapping_sub(62))
                .cloned()
                .unwrap_or(("?".into(), "?".into()))
        }
    }

    /// Decodes one header block into [(name, value, note), ...].
    pub fn decode(&mut self, block: &[u8]) -> Vec<(String, String, String)> {
        let mut headers = vec![];
        let mut pos = 0;
        while pos < block.len() {
            let b = block[pos];
            pos += 1;
            if b & 0x80 == 0x80 {
                // 1xxxxxxx indexed header field
                let Some(idx) = int(block, &mut pos, b, 7) else { break };
                let (n, v) = self.lookup(idx as usize);
                headers.push((n, v, format!("indexed({idx})")));
            } else if b & 0xc0 == 0x40 {
                // 01xxxxxx literal, add to dynamic table
                let Some((n, v)) = self.literal(block, &mut pos, b, 6) else { break };
                self.dynamic.insert(0, (n.clone(), v.clone()));
                self.evict(4096);
                headers.push((n, v, "literal+index".into()));
            } else if b & 0xe0 == 0x20 {
                // 001xxxxx dynamic table size update
                let Some(size) = int(block, &mut pos, b, 5) else { break };
                headers.push(("(dynamic table size update)".into(), size.to_string(), String::new()));
            } else {
                // 0000xxxx / 0001xxxx literal, no index
                let Some((n, v)) = self.literal(block, &mut pos, b, 4) else { break };
                let note = if b & 0x10 == 0x10 { "literal never-index" } else { "literal" };
                headers.push((n, v, note.into()));
            }
        }
        headers
    }

    fn literal(&self, block: &[u8], pos: &mut usize, first: u8, prefix: u8) -> Option<(String, String)> {
        let idx = int(block, pos, first, prefix)?;
        let name = if idx == 0 { self.string(block, pos)? } else { self.lookup(idx as usize).0 };
        let value = self.string(block, pos)?;
        Some((name, value))
    }

    fn string(&self, block: &[u8], pos: &mut usize) -> Option<String> {
        let b = *block.get(*pos)?;
        *pos += 1;
        let huff = b & 0x80 == 0x80;
        let len = int(block, pos, b, 7)? as usize;
        let raw = block.get(*pos..*pos + len)?;
        *pos += len;
        Some(if huff {
            huffman_decode(&self.tree, raw)
        } else {
            String::from_utf8_lossy(raw).into_owned()
        })
    }

    // RFC 7541 4.1: entry size is name + value + 32 bytes of overhead.
    fn evict(&mut self, max: usize) {
        while self.dynamic.iter().map(|(n, v)| n.len() + v.len() + 32).sum::<usize>() > max {
            self.dynamic.pop();
        }
    }
}

/// RFC 7541 5.1 prefixed integer.
fn int(block: &[u8], pos: &mut usize, first: u8, prefix: u8) -> Option<u64> {
    let mask = (1u64 << prefix) - 1;
    let mut val = u64::from(first) & mask;
    if val < mask {
        return Some(val);
    }
    let mut shift = 0;
    loop {
        let b = *block.get(*pos)?;
        *pos += 1;
        val += u64::from(b & 0x7f) << shift;
        if b < 0x80 {
            return Some(val);
        }
        shift += 7;
    }
}
