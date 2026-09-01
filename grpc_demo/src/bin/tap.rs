//! tap — a man-in-the-middle TCP proxy that prints the *actual bytes* of a
//! gRPC conversation.
//!
//!   cargo run --bin tap [-- listen_port upstream_host:port]   # default 50052 -> localhost:50051
//!
//! It forwards bytes untouched, and in parallel decodes them as:
//!
//!   TCP bytes
//!     -> HTTP/2 connection preface + frames   (9-byte header: len|type|flags|stream)
//!        -> HEADERS frames  -> HPACK (static/dynamic table + Huffman)  -> :path, grpc-status, ...
//!        -> DATA frames     -> gRPC length-prefixed messages (1-byte flag + 4-byte length)
//!                              -> protobuf wire format (tag = field<<3 | wire_type)
//!
//! Nothing here is gRPC-library magic: it is all just parsing the bytes.
//!
//! Env knobs:  TAP_MAX_HEX=0 (no cap) | N bytes per hexdump (default 96)
//!             TAP_QUIET=1 hides PING / WINDOW_UPDATE / SETTINGS-ACK housekeeping
//!             NO_COLOR=1 to disable ANSI colour

use grpc_demo::hexdump::hexdump;
use grpc_demo::hpack::Hpack;
use grpc_demo::protowire;
use std::collections::HashMap;
use std::io::{IsTerminal, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex, OnceLock};

struct Config {
    max_hex: usize,
    quiet: bool,
    color: bool,
}

fn cfg() -> &'static Config {
    static CFG: OnceLock<Config> = OnceLock::new();
    CFG.get_or_init(|| Config {
        max_hex: std::env::var("TAP_MAX_HEX").ok().and_then(|v| v.parse().ok()).unwrap_or(96),
        quiet: std::env::var("TAP_QUIET").as_deref() == Ok("1"),
        color: std::io::stdout().is_terminal() && std::env::var("NO_COLOR").is_err(),
    })
}

fn c(text: &str, code: u8) -> String {
    if cfg().color { format!("\x1b[{code}m{text}\x1b[0m") } else { text.to_string() }
}
fn dim(t: &str) -> String {
    c(t, 90)
}
fn bold(t: &str) -> String {
    c(t, 1)
}

fn emit(lines: &[String]) {
    static OUT: Mutex<()> = Mutex::new(());
    let _guard = OUT.lock().unwrap();
    let mut stdout = std::io::stdout().lock();
    for line in lines {
        let _ = writeln!(stdout, "{line}");
    }
    let _ = stdout.flush();
}

fn dump(bytes: &[u8]) -> Vec<String> {
    hexdump(bytes, "     ", cfg().max_hex).iter().map(|l| dim(l)).collect()
}

// ---------------------------------------------------------------------------
// HTTP/2 frame parser (one per direction)
// ---------------------------------------------------------------------------
const PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
const TYPES: [&str; 10] = ["DATA", "HEADERS", "PRIORITY", "RST_STREAM", "SETTINGS",
    "PUSH_PROMISE", "PING", "GOAWAY", "WINDOW_UPDATE", "CONTINUATION"];
const ERRORS: [&str; 14] = ["NO_ERROR", "PROTOCOL_ERROR", "INTERNAL_ERROR", "FLOW_CONTROL_ERROR",
    "SETTINGS_TIMEOUT", "STREAM_CLOSED", "FRAME_SIZE_ERROR", "REFUSED_STREAM", "CANCEL",
    "COMPRESSION_ERROR", "CONNECT_ERROR", "ENHANCE_YOUR_CALM", "INADEQUATE_SECURITY",
    "HTTP_1_1_REQUIRED"];

fn setting_name(id: u16) -> String {
    match id {
        1 => "HEADER_TABLE_SIZE".into(),
        2 => "ENABLE_PUSH".into(),
        3 => "MAX_CONCURRENT_STREAMS".into(),
        4 => "INITIAL_WINDOW_SIZE".into(),
        5 => "MAX_FRAME_SIZE".into(),
        6 => "MAX_HEADER_LIST_SIZE".into(),
        65027 => "GRPC_ALLOW_TRUE_BINARY_METADATA".into(),
        other => format!("SETTING({other})"),
    }
}

fn error_name(code: u32) -> String {
    ERRORS.get(code as usize).map(|s| s.to_string()).unwrap_or_else(|| code.to_string())
}

struct H2Parser {
    label: &'static str,
    color: u8,
    buf: Vec<u8>,
    hpack: Hpack,
    preface_done: bool,
    hblock: HashMap<u32, Vec<u8>>, // stream -> accumulated header block (HEADERS + CONTINUATION)
    grpc: HashMap<u32, Vec<u8>>,   // stream -> partial gRPC message buffer
}

impl H2Parser {
    fn new(label: &'static str, color: u8) -> Self {
        H2Parser {
            label,
            color,
            buf: vec![],
            hpack: Hpack::new(),
            preface_done: false,
            hblock: HashMap::new(),
            grpc: HashMap::new(),
        }
    }

    fn arrow(&self) -> String {
        c(self.label, self.color)
    }

    fn push(&mut self, chunk: &[u8]) {
        self.buf.extend_from_slice(chunk);
        if !self.preface_done {
            if self.buf.len() < PREFACE.len() && PREFACE.starts_with(&self.buf) {
                return;
            }
            if self.buf.starts_with(PREFACE) {
                self.buf.drain(..PREFACE.len());
                let mut lines = vec![
                    String::new(),
                    format!("{} {} (24 bytes, sent once by the client)", self.arrow(),
                            bold("HTTP/2 connection preface")),
                ];
                lines.extend(dump(PREFACE));
                emit(&lines);
            }
            self.preface_done = true;
        }

        loop {
            if self.buf.len() < 9 {
                break;
            }
            let len = ((self.buf[0] as usize) << 16) | ((self.buf[1] as usize) << 8) | self.buf[2] as usize;
            if self.buf.len() < 9 + len {
                break;
            }
            let header: Vec<u8> = self.buf[..9].to_vec();
            let payload: Vec<u8> = self.buf[9..9 + len].to_vec();
            self.buf.drain(..9 + len);
            self.frame(&header, &payload);
        }
    }

    fn frame(&mut self, header: &[u8], payload: &[u8]) {
        let ftype = header[3];
        let flags = header[4];
        let stream = u32::from_be_bytes(header[5..9].try_into().unwrap()) & 0x7fff_ffff;
        let name = TYPES
            .get(ftype as usize)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("UNKNOWN({ftype})"));
        if cfg().quiet && (ftype == 6 || ftype == 8 || (ftype == 4 && flags & 1 == 1)) {
            return;
        }

        let mut lines = vec![
            String::new(),
            format!("{} {} stream={} length={} flags=0x{:02x}{}", self.arrow(), bold(&name),
                    stream, payload.len(), flags, flag_names(ftype, flags)),
            dim(&format!("     frame header: {}  = len(3B) type(1B) flags(1B) stream-id(4B)",
                         protowire::hex(header))),
        ];
        if !payload.is_empty() {
            lines.extend(dump(payload));
        }
        lines.extend(self.detail(ftype, flags, stream, payload));
        emit(&lines);
    }

    fn detail(&mut self, ftype: u8, flags: u8, stream: u32, payload: &[u8]) -> Vec<String> {
        match ftype {
            0 => self.data_frame(stream, payload),
            1 => self.headers_frame(stream, flags, payload),
            3 => {
                let code = u32::from_be_bytes(payload[..4].try_into().unwrap());
                vec![format!("     error code: {}", error_name(code))]
            }
            4 => {
                if flags & 1 == 1 {
                    return vec!["     (acknowledgement)".into()];
                }
                payload
                    .chunks_exact(6)
                    .map(|s| {
                        let id = u16::from_be_bytes(s[..2].try_into().unwrap());
                        let val = u32::from_be_bytes(s[2..6].try_into().unwrap());
                        format!("     {:<24} = {}", setting_name(id), val)
                    })
                    .collect()
            }
            7 => {
                let last = u32::from_be_bytes(payload[..4].try_into().unwrap()) & 0x7fff_ffff;
                let code = u32::from_be_bytes(payload[4..8].try_into().unwrap());
                vec![format!("     last-stream-id={} error={} debug={:?}", last, error_name(code),
                             String::from_utf8_lossy(&payload[8..]))]
            }
            8 => {
                let inc = u32::from_be_bytes(payload[..4].try_into().unwrap()) & 0x7fff_ffff;
                vec![format!("     window increment: {inc} bytes")]
            }
            _ => vec![],
        }
    }

    fn headers_frame(&mut self, stream: u32, flags: u8, payload: &[u8]) -> Vec<String> {
        let mut body = payload;
        let mut pad = 0usize;
        if flags & 0x08 == 0x08 {
            pad = body[0] as usize;
            body = &body[1..];
        }
        if flags & 0x20 == 0x20 {
            body = &body[5..]; // strip priority (stream dep + weight)
        }
        if pad > 0 {
            body = &body[..body.len() - pad];
        }

        self.hblock.entry(stream).or_default().extend_from_slice(body);
        if flags & 0x04 != 0x04 {
            return vec!["     (waiting for CONTINUATION frame)".into()];
        }

        let block = self.hblock.remove(&stream).unwrap_or_default();
        let mut lines = vec![format!("     {}", bold("HPACK-decoded headers:"))];
        for (n, v, note) in self.hpack.decode(&block) {
            lines.push(format!("       {:<24} {:<46} {}", format!("{n}:"), format!("{v:?}"), dim(&note)));
        }
        lines
    }

    // A DATA frame payload is a stream of gRPC "length-prefixed messages":
    //   [1 byte compressed-flag][4 byte big-endian length][length bytes of protobuf]
    // A message may straddle several DATA frames, hence the per-stream buffer.
    fn data_frame(&mut self, stream: u32, payload: &[u8]) -> Vec<String> {
        let buf = self.grpc.entry(stream).or_default();
        buf.extend_from_slice(payload);
        let mut lines = vec![];
        while buf.len() >= 5 {
            let compressed = buf[0];
            let len = u32::from_be_bytes(buf[1..5].try_into().unwrap()) as usize;
            if buf.len() < 5 + len {
                break;
            }
            let msg: Vec<u8> = buf[5..5 + len].to_vec();
            buf.drain(..5 + len);
            lines.push(format!("     {}: compressed-flag={} length={} (prefix bytes {:02x} {:08x})",
                               bold("gRPC message frame"), compressed, len, compressed, len));
            lines.push(dim(&format!("     protobuf payload: {}", protowire::hex(&msg))));
            let decoded = if msg.is_empty() {
                vec!["(empty message)".into()]
            } else {
                protowire::decode(&msg, 0).unwrap_or_else(|| vec!["(not decodable as protobuf)".into()])
            };
            lines.extend(decoded.iter().map(|l| format!("       {}", c(l, 32))));
        }
        if !buf.is_empty() {
            lines.push(format!("     ({} bytes buffered, message continues in a later DATA frame)", buf.len()));
        }
        lines
    }
}

fn flag_names(ftype: u8, flags: u8) -> String {
    let mut names: Vec<&str> = vec![];
    match ftype {
        0 => {
            if flags & 0x01 == 0x01 { names.push("END_STREAM"); }
        }
        1 => {
            if flags & 0x01 == 0x01 { names.push("END_STREAM"); }
            if flags & 0x04 == 0x04 { names.push("END_HEADERS"); }
            if flags & 0x08 == 0x08 { names.push("PADDED"); }
            if flags & 0x20 == 0x20 { names.push("PRIORITY"); }
        }
        4 | 6 => {
            if flags & 0x01 == 0x01 { names.push("ACK"); }
        }
        9
            if flags & 0x04 == 0x04 => { names.push("END_HEADERS"); }
        _ => {}
    }
    if names.is_empty() { String::new() } else { format!(" ({})", names.join("|")) }
}

// ---------------------------------------------------------------------------
// the proxy itself
// ---------------------------------------------------------------------------
fn pump(mut src: TcpStream, mut dst: TcpStream, mut parser: H2Parser) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut chunk = [0u8; 65536];
        loop {
            match src.read(&mut chunk) {
                Ok(0) | Err(_) => break, // connection closed
                Ok(n) => {
                    parser.push(&chunk[..n]); // decode a copy
                    if dst.write_all(&chunk[..n]).is_err() { // forward untouched
                        break;
                    }
                }
            }
        }
        let _ = src.shutdown(std::net::Shutdown::Both);
        let _ = dst.shutdown(std::net::Shutdown::Both);
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let listen_port: u16 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(50052);
    let upstream = args.get(2).cloned().unwrap_or_else(|| "localhost:50051".into());

    emit(&[
        bold(&format!("tap: listening on 0.0.0.0:{listen_port} -> {upstream}")),
        dim(&format!("     {} = client to server, {} = server to client", c("C->S", 36), c("S->C", 35))),
        String::new(),
    ]);

    let server = TcpListener::bind(("0.0.0.0", listen_port)).expect("bind failed");
    for client in server.incoming() {
        let Ok(client) = client else { continue };
        let upstream = upstream.clone();
        std::thread::spawn(move || {
            let peer = client.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "?".into());
            let up = match TcpStream::connect(&upstream) {
                Ok(s) => s,
                Err(e) => {
                    emit(&[format!("tap error: cannot reach upstream {upstream}: {e}")]);
                    return;
                }
            };
            emit(&[String::new(), bold(&format!("=== new TCP connection from {peer} ==="))]);

            let a = pump(client.try_clone().unwrap(), up.try_clone().unwrap(), H2Parser::new("C->S", 36));
            let b = pump(up, client, H2Parser::new("S->C", 35));
            let _ = a.join();
            let _ = b.join();
            emit(&[dim("=== connection closed ===")]);
        });
    }
}
