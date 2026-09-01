//! Interactive P2P peer.
//!
//! Direct (STUN) mode — server only mediates, data flows peer-to-peer:
//!   terminal 1:  cargo run --bin stun_server
//!   terminal 2:  cargo run --bin peer -- --name alice --room demo
//!   terminal 3:  cargo run --bin peer -- --name bob   --room demo
//!
//! Relayed (TURN) mode — all data flows through the server's relay ports:
//!   terminal 1:  cargo run --bin turn_server
//!   terminal 2:  cargo run --bin peer -- --name alice --room demo --turn
//!   terminal 3:  cargo run --bin peer -- --name bob   --room demo --turn
//!
//! Handshake: STUN Binding (learn own address) -> [TURN Allocate] ->
//! REGISTER into room -> server sends PEER-INFO with the other side's
//! address -> UDP hole punching -> interactive console:
//!
//!   /send <text>       chat (plain text without a command also sends)
//!   /sendfile <path>   any file — chunked over UDP with retransmission
//!   /status            connection info + stats
//!   /quit

use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufRead;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::os::unix::fs::FileExt;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use stun_turn_demo::packet_log::{self, Dir};
use stun_turn_demo::stun::{self, TxId};
use stun_turn_demo::as_v4;

const CHUNK_SIZE: usize = 1024;
const FRAME_MAGIC: &[u8; 2] = b"P2"; // our tiny app protocol on top of the P2P channel

const HELP: &str = "commands:
  /send <text>       send a chat message (plain text also works)
  /sendfile <path>   send any file (text / image / video ...)
  /status            show connection info and transfer stats
  /quit              exit
";

type Addr = (Ipv4Addr, u16);

#[derive(Clone)]
struct PeerInfo {
    name: String,
    mapped: Addr,
    relay: Option<Addr>,
}

#[derive(Default)]
struct ConnState {
    peer: Option<PeerInfo>,
    mapped: Option<Addr>,
    relay: Option<Addr>,
    connected: bool,
}

struct IncomingTransfer {
    name: String,
    size: u64,
    count: u32,
    chunks: Vec<Option<Vec<u8>>>,
    got: u32,
    t0: Instant,
}

struct OutgoingTransfer {
    file: File,
    size: u64,
    done: bool,
}

struct Peer {
    name: String,
    room: String,
    turn: bool,
    verbose: bool,
    server: SocketAddrV4,
    sock: UdpSocket,
    responses: Mutex<Sender<stun::Msg>>, // success responses, received by request()
    quiet_txids: Mutex<HashSet<TxId>>,
    state: Mutex<ConnState>,
    peer_cv: Condvar,
    incoming: Mutex<HashMap<u32, IncomingTransfer>>,
    outgoing: Mutex<HashMap<u32, OutgoingTransfer>>,
    completed: Mutex<HashSet<u32>>,
    stats: Mutex<HashMap<&'static str, u64>>,
    punch_logged: Mutex<HashSet<Dir>>,
}

fn addr_s(a: Addr) -> String {
    format!("{}:{}", a.0, a.1)
}

impl Peer {
    fn run(self: &Arc<Self>, responses_rx: Receiver<stun::Msg>) {
        {
            let me = Arc::clone(self);
            std::thread::spawn(move || me.recv_loop());
        }
        self.stun_binding(&responses_rx);
        if self.turn {
            self.turn_allocate(&responses_rx);
        }
        self.register(&responses_rx, false);
        self.wait_for_peer(&responses_rx);
        if self.turn {
            self.create_permission(&responses_rx);
        }
        self.punch();
        {
            let me = Arc::clone(self);
            std::thread::spawn(move || me.keepalive_loop());
        }
        self.console();
    }

    // ---------------------------------------------------------- handshake

    fn stun_binding(&self, rx: &Receiver<stun::Msg>) {
        let resp = self.request(rx, stun::BINDING_REQUEST,
            vec![(stun::ATTR_SOFTWARE, format!("rust-demo-{}", self.name).into_bytes())],
            Some("STUN: what is my public address?"), false);
        let mapped = stun::unxor_address(resp.attr(stun::ATTR_XOR_MAPPED_ADDRESS).unwrap_or(&[]))
            .expect("no XOR-MAPPED-ADDRESS in binding response");
        self.state.lock().unwrap().mapped = Some(mapped);
        println!("[{}] my reflexive (public) address: {}", self.name, addr_s(mapped));
    }

    fn turn_allocate(&self, rx: &Receiver<stun::Msg>) {
        let resp = self.request(rx, stun::ALLOCATE_REQUEST, vec![
            (stun::ATTR_REQUESTED_TRANSPORT, vec![17, 0, 0, 0]), // 17 = UDP
            (stun::ATTR_LIFETIME, 600u32.to_be_bytes().to_vec()),
        ], Some("TURN: rent me a relay address"), false);
        let relay = stun::unxor_address(resp.attr(stun::ATTR_XOR_RELAYED_ADDRESS).unwrap_or(&[]))
            .expect("no XOR-RELAYED-ADDRESS in allocate response");
        self.state.lock().unwrap().relay = Some(relay);
        println!("[{}] my relayed address on the TURN server: {}", self.name, addr_s(relay));
    }

    fn register(&self, rx: &Receiver<stun::Msg>, quiet: bool) {
        let mut attrs = vec![(stun::ATTR_USERNAME, format!("{}/{}", self.room, self.name).into_bytes())];
        // In TURN mode we advertise the relay as our candidate: the other peer
        // sends there, and the relay forwards to us wrapped in Data Indications.
        if let Some(relay) = self.state.lock().unwrap().relay {
            attrs.push((stun::ATTR_XOR_RELAYED_ADDRESS, stun::xor_address(relay.0, relay.1)));
        }
        let note = format!("join room '{}' as '{}'", self.room, self.name);
        let resp = self.request(rx, stun::REGISTER_REQUEST, attrs, Some(&note), quiet);
        if !quiet {
            let software = resp.attr(stun::ATTR_SOFTWARE).map(|v| String::from_utf8_lossy(v).into_owned());
            println!("[{}] server: {}", self.name, software.unwrap_or_default());
        }
    }

    fn wait_for_peer(&self, rx: &Receiver<stun::Msg>) {
        if self.state.lock().unwrap().peer.is_some() {
            return;
        }
        println!("[{}] waiting for a peer to join room '{}' ...", self.name, self.room);
        loop {
            {
                let state = self.state.lock().unwrap();
                if state.peer.is_some() {
                    return;
                }
                let (state, _) = self.peer_cv.wait_timeout(state, Duration::from_secs(15)).unwrap();
                if state.peer.is_some() {
                    return;
                }
            }
            // re-register periodically: keeps the NAT mapping to the server alive
            self.register(rx, true);
        }
    }

    fn create_permission(&self, rx: &Receiver<stun::Msg>) {
        let peer = self.state.lock().unwrap().peer.clone().unwrap();
        self.request(rx, stun::CREATE_PERMISSION_REQ,
            vec![(stun::ATTR_XOR_PEER_ADDRESS, stun::xor_address(peer.mapped.0, peer.mapped.1))],
            Some(&format!("TURN: allow {}'s IP to reach my relay", peer.name)), false);
    }

    fn punch(&self) {
        println!("[{}] hole punching toward {} ...", self.name, self.target_s());
        for _ in 0..40 {
            if self.state.lock().unwrap().connected {
                return;
            }
            let hello = self.hello_frame();
            self.send_raw(&hello);
            std::thread::sleep(Duration::from_millis(500));
        }
        if !self.state.lock().unwrap().connected {
            println!("[{}] punch timed out — peer unreachable", self.name);
        }
    }

    fn hello_frame(&self) -> Vec<u8> {
        let hello = frame(b'H', self.name.as_bytes());
        if self.punch_logged.lock().unwrap().insert(Dir::Send) {
            packet_log::log(&self.name, Dir::Send, &self.target_s(), &hello,
                            Some("hole punch hello (repeats until answered)"));
        }
        hello
    }

    /// Sends a request to the server and waits for the matching success response.
    fn request(&self, rx: &Receiver<stun::Msg>, typ: u16, attrs: Vec<(u16, Vec<u8>)>,
               note: Option<&str>, quiet: bool) -> stun::Msg {
        let txid = stun::new_txid();
        let pkt = stun::build(typ, &txid, &attrs);
        if quiet {
            self.quiet_txids.lock().unwrap().insert(txid);
        } else {
            packet_log::log(&self.name, Dir::Send, &self.server.to_string(), &pkt, note);
        }
        let _ = self.sock.send_to(&pkt, self.server);
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(msg) => msg,
            Err(_) => {
                eprintln!("[{}] no response from server {} — is it running?", self.name, self.server);
                std::process::exit(1);
            }
        }
    }

    // ---------------------------------------------------------- receive path

    fn recv_loop(self: Arc<Self>) {
        let mut buf = [0u8; 65536];
        while let Ok((n, from)) = self.sock.recv_from(&mut buf) {
            let data = &buf[..n];
            let from = addr_s({ let a = as_v4(from); (*a.ip(), a.port()) });
            if stun::is_stun_packet(data) {
                let Some(msg) = stun::parse(data) else { continue };
                match msg.typ {
                    stun::PEER_INFO_INDICATION => {
                        packet_log::log(&self.name, Dir::Recv, &from, data,
                                        Some("peer introduction from server"));
                        self.handle_peer_info(&msg);
                    }
                    stun::DATA_INDICATION => {
                        let payload = msg.attr(stun::ATTR_DATA).unwrap_or(&[]).to_vec();
                        self.log_frame(Dir::Recv, &format!("{from} (Data Indication via my relay)"),
                                       data, &payload);
                        self.handle_frame(&payload);
                    }
                    typ if stun::success_response(typ) => {
                        if !self.quiet_txids.lock().unwrap().remove(&msg.txid) {
                            packet_log::log(&self.name, Dir::Recv, &from, data, None);
                        }
                        let _ = self.responses.lock().unwrap().send(msg);
                    }
                    _ => packet_log::log(&self.name, Dir::Recv, &from, data, Some("unexpected")),
                }
            } else {
                self.log_frame(Dir::Recv, &from, data, data);
                self.handle_frame(data);
            }
        }
        // socket closed on shutdown
    }

    fn handle_peer_info(&self, msg: &stun::Msg) {
        let name = String::from_utf8_lossy(msg.attr(stun::ATTR_USERNAME).unwrap_or(&[])).into_owned();
        let Some(mapped) = msg.attr(stun::ATTR_XOR_PEER_ADDRESS).and_then(stun::unxor_address) else { return };
        let relay = msg.attr(stun::ATTR_XOR_RELAYED_ADDRESS).and_then(stun::unxor_address);
        {
            let mut state = self.state.lock().unwrap();
            state.peer = Some(PeerInfo { name: name.clone(), mapped, relay });
            self.peer_cv.notify_all();
        }
        println!("[{}] room '{}': peer '{}' is at {}{} — sending to {}",
                 self.name, self.room, name, addr_s(mapped),
                 relay.map(|r| format!(" (their relay: {})", addr_s(r))).unwrap_or_default(),
                 self.target_s());
    }

    fn handle_frame(&self, data: &[u8]) {
        if data.len() < 3 || &data[..2] != FRAME_MAGIC {
            return;
        }
        let typ = data[2];
        let payload = &data[3..];
        if typ != b'H' {
            self.mark_connected();
        }
        match typ {
            b'H' => {
                // hole punch hello
                if !self.state.lock().unwrap().connected {
                    self.mark_connected();
                    self.send_raw(&frame(b'H', self.name.as_bytes())); // answer so the other side connects too
                }
            }
            b'M' => {
                // chat message
                if let Ok(m) = serde_json::from_slice::<serde_json::Value>(payload) {
                    println!("\n[{}] says: {}",
                             m["from"].as_str().unwrap_or("?"), m["text"].as_str().unwrap_or(""));
                }
            }
            b'P' => self.send_raw(&frame(b'O', b"")), // keepalive ping
            b'O' => {}                                // keepalive pong
            b'F' => self.handle_file_header(payload),
            b'C' => self.handle_chunk(payload),
            b'E' => self.handle_transfer_end(payload),
            b'R' => self.handle_resend_request(payload),
            b'D' => {
                // receiver confirmed: transfer complete
                if let Some(id) = json_id(payload)
                    && let Some(st) = self.outgoing.lock().unwrap().get_mut(&id) {
                        st.done = true;
                    }
            }
            other => eprintln!("[{}] bad frame (unknown type {:?})", self.name, other as char),
        }
    }

    fn mark_connected(&self) {
        let mut state = self.state.lock().unwrap();
        if state.connected {
            return;
        }
        state.connected = true;
        let peer_name = state.peer.as_ref().map(|p| p.name.clone()).unwrap_or_default();
        println!("\n[{}] *** P2P channel to '{}' established{} ***\n",
                 self.name, peer_name,
                 if self.turn { " (via TURN relay)" } else { " (direct, no server in the path)" });
    }

    // ---------------------------------------------------------- app protocol
    //
    // Tiny frame format on top of UDP: "P2" + 1 type byte + payload
    //   H hello/punch   M chat(json)       P/O keepalive ping/pong
    //   F file header(json {id,name,size,chunks})
    //   C chunk(id u32, seq u32, bytes)
    //   E sender finished(json {id})       R resend request(json {id,missing})
    //   D receiver done(json {id})

    fn target(&self) -> Option<Addr> {
        // where we send app data: peer's relay if they have one
        let state = self.state.lock().unwrap();
        state.peer.as_ref().map(|p| p.relay.unwrap_or(p.mapped))
    }

    fn target_s(&self) -> String {
        self.target().map(addr_s).unwrap_or_else(|| "(no peer yet)".into())
    }

    fn send_raw(&self, data: &[u8]) {
        if let Some(t) = self.target() {
            let _ = self.sock.send_to(data, t);
        }
    }

    fn send_text(&self, text: &str) {
        let pkt = frame(b'M', json!({ "from": self.name, "text": text }).to_string().as_bytes());
        self.log_frame(Dir::Send, &self.target_s(), &pkt, &pkt);
        self.send_raw(&pkt);
    }

    fn send_file(&self, path: &str) {
        let Ok(file) = File::open(path) else {
            println!("no such file: {path}");
            return;
        };
        let Ok(meta) = file.metadata() else { return };
        if !meta.is_file() {
            println!("no such file: {path}");
            return;
        }
        let size = meta.len();
        let count = size.div_ceil(CHUNK_SIZE as u64) as u32;
        let id: u32 = rand::random();
        let basename = std::path::Path::new(path)
            .file_name().unwrap_or_default().to_string_lossy().into_owned();
        self.outgoing.lock().unwrap().insert(id, OutgoingTransfer { file, size, done: false });

        let header = frame(b'F',
            json!({ "id": id, "name": basename, "size": size, "chunks": count }).to_string().as_bytes());
        packet_log::log(&self.name, Dir::Send, &self.target_s(), &header, Some("file transfer header"));
        self.send_raw(&header);

        let t0 = Instant::now();
        for seq in 0..count {
            self.send_chunk(id, seq);
            if seq % 64 == 0 {
                std::thread::sleep(Duration::from_millis(1)); // crude pacing so we don't overrun buffers
            }
            if (seq + 1) % 2000 == 0 {
                println!("[{}] sent {}/{} chunks ...", self.name, seq + 1, count);
            }
        }

        // Tell the receiver we're done; it answers R (resend these) or D (done).
        // Header is resent too in case the F packet itself was lost.
        'waiting: for _ in 0..60 {
            if self.transfer_done(id) {
                break;
            }
            self.send_raw(&header);
            self.send_raw(&frame(b'E', json!({ "id": id }).to_string().as_bytes()));
            for _ in 0..5 {
                std::thread::sleep(Duration::from_millis(200));
                if self.transfer_done(id) {
                    break 'waiting;
                }
            }
        }

        if self.transfer_done(id) {
            let secs = t0.elapsed().as_secs_f64().max(0.001);
            let retransmitted = *self.stats.lock().unwrap().get("chunks_retransmitted").unwrap_or(&0);
            println!("[{}] file '{}' delivered: {} bytes in {:.2}s ({:.2} MB/s), {} chunks retransmitted",
                     self.name, basename, size, secs, size as f64 / secs / 1e6, retransmitted);
        } else {
            println!("[{}] transfer was never acknowledged — is the peer still there?", self.name);
        }
        self.outgoing.lock().unwrap().remove(&id);
    }

    fn transfer_done(&self, id: u32) -> bool {
        self.outgoing.lock().unwrap().get(&id).is_some_and(|st| st.done)
    }

    fn send_chunk(&self, id: u32, seq: u32) {
        let outgoing = self.outgoing.lock().unwrap();
        let Some(st) = outgoing.get(&id) else { return };
        let offset = seq as u64 * CHUNK_SIZE as u64;
        let len = (CHUNK_SIZE as u64).min(st.size.saturating_sub(offset)) as usize;
        let mut data = vec![0u8; len];
        if st.file.read_exact_at(&mut data, offset).is_err() {
            return;
        }
        drop(outgoing);
        let mut payload = Vec::with_capacity(8 + len);
        payload.extend_from_slice(&id.to_be_bytes());
        payload.extend_from_slice(&seq.to_be_bytes());
        payload.extend_from_slice(&data);
        self.send_raw(&frame(b'C', &payload));
        *self.stats.lock().unwrap().entry("chunks_sent").or_insert(0) += 1;
    }

    fn handle_file_header(&self, payload: &[u8]) {
        let Ok(h) = serde_json::from_slice::<serde_json::Value>(payload) else { return };
        let (Some(id), Some(size), Some(count)) =
            (h["id"].as_u64(), h["size"].as_u64(), h["chunks"].as_u64())
        else { return };
        let id = id as u32;
        if self.incoming.lock().unwrap().contains_key(&id)
            || self.completed.lock().unwrap().contains(&id) {
            return; // duplicate header
        }
        let name = h["name"].as_str().unwrap_or("unnamed").to_string();
        self.incoming.lock().unwrap().insert(id, IncomingTransfer {
            name: name.clone(),
            size,
            count: count as u32,
            chunks: vec![None; count as usize],
            got: 0,
            t0: Instant::now(),
        });
        let peer_name = self.state.lock().unwrap().peer.as_ref().map(|p| p.name.clone()).unwrap_or_default();
        println!("[{}] incoming file '{}' ({} bytes, {} chunks) from '{}'",
                 self.name, name, size, count, peer_name);
    }

    fn handle_chunk(&self, payload: &[u8]) {
        if payload.len() < 8 {
            return;
        }
        let id = u32::from_be_bytes(payload[..4].try_into().unwrap());
        let seq = u32::from_be_bytes(payload[4..8].try_into().unwrap()) as usize;
        let mut incoming = self.incoming.lock().unwrap();
        let Some(st) = incoming.get_mut(&id) else { return };
        if seq >= st.chunks.len() || st.chunks[seq].is_some() {
            return;
        }
        st.chunks[seq] = Some(payload[8..].to_vec());
        st.got += 1;
        let got = st.got;
        let count = st.count;
        drop(incoming);
        *self.stats.lock().unwrap().entry("chunks_received").or_insert(0) += 1;
        if got % 2000 == 0 {
            println!("[{}] received {got}/{count} chunks ...", self.name);
        }
    }

    fn handle_transfer_end(&self, payload: &[u8]) {
        let Some(id) = json_id(payload) else { return };
        if self.completed.lock().unwrap().contains(&id) {
            // already saved; sender missed our D — resend it
            self.send_raw(&frame(b'D', json!({ "id": id }).to_string().as_bytes()));
            return;
        }
        let missing: Vec<u32> = {
            let incoming = self.incoming.lock().unwrap();
            let Some(st) = incoming.get(&id) else { return };
            (0..st.count).filter(|&i| st.chunks[i as usize].is_none()).collect()
        };
        if missing.is_empty() {
            self.finish_incoming(id);
        } else {
            println!("[{}] {} chunks missing — requesting retransmit", self.name, missing.len());
            let first: Vec<u32> = missing.into_iter().take(800).collect();
            self.send_raw(&frame(b'R', json!({ "id": id, "missing": first }).to_string().as_bytes()));
        }
    }

    fn handle_resend_request(&self, payload: &[u8]) {
        let Ok(req) = serde_json::from_slice::<serde_json::Value>(payload) else { return };
        let Some(id) = req["id"].as_u64().map(|v| v as u32) else { return };
        if !self.outgoing.lock().unwrap().contains_key(&id) {
            return;
        }
        let missing: Vec<u32> = req["missing"].as_array().map(|a| {
            a.iter().filter_map(|v| v.as_u64().map(|v| v as u32)).collect()
        }).unwrap_or_default();
        *self.stats.lock().unwrap().entry("chunks_retransmitted").or_insert(0) += missing.len() as u64;
        for (i, seq) in missing.iter().enumerate() {
            self.send_chunk(id, *seq);
            if i % 64 == 0 {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        self.send_raw(&frame(b'E', json!({ "id": id }).to_string().as_bytes()));
    }

    fn finish_incoming(&self, id: u32) {
        let Some(st) = self.incoming.lock().unwrap().remove(&id) else { return };
        let _ = std::fs::create_dir_all("downloads");
        let mut dest = std::path::PathBuf::from("downloads").join(&st.name);
        if dest.exists() {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            dest = std::path::PathBuf::from("downloads").join(format!("{ts}_{}", st.name));
        }
        let bytes: Vec<u8> = st.chunks.into_iter().flatten().flatten().collect();
        let _ = std::fs::write(&dest, &bytes);
        self.completed.lock().unwrap().insert(id);
        self.send_raw(&frame(b'D', json!({ "id": id }).to_string().as_bytes()));
        let secs = st.t0.elapsed().as_secs_f64().max(0.001);
        println!("[{}] file saved to {} ({} bytes in {:.2}s, {:.2} MB/s)",
                 self.name, dest.display(), st.size, secs, st.size as f64 / secs / 1e6);
    }

    // ---------------------------------------------------------- housekeeping

    fn keepalive_loop(self: Arc<Self>) {
        loop {
            std::thread::sleep(Duration::from_secs(15));
            if self.state.lock().unwrap().connected {
                self.send_raw(&frame(b'P', b"")); // keeps NAT bindings warm
            }
        }
    }

    /// Log app frames; bulk/noisy types (chunks, keepalives, repeated punches)
    /// are skipped unless --verbose.
    fn log_frame(&self, dir: Dir, remote: &str, raw: &[u8], payload: &[u8]) {
        let typ = (payload.len() >= 3 && &payload[..2] == FRAME_MAGIC).then(|| payload[2]);
        if !self.verbose {
            if matches!(typ, Some(b'C') | Some(b'P') | Some(b'O') | Some(b'R')) {
                return;
            }
            if typ == Some(b'H') && !self.punch_logged.lock().unwrap().insert(dir) {
                return;
            }
        }
        packet_log::log(&self.name, dir, remote, raw, None);
    }

    fn show_status(&self) {
        let state = self.state.lock().unwrap();
        let stats = self.stats.lock().unwrap();
        let peer_line = state.peer.as_ref().map(|p| {
            format!("{} @ {}{}", p.name,
                    addr_s(p.relay.unwrap_or(p.mapped)),
                    if p.relay.is_some() { " (their relay)" } else { "" })
        }).unwrap_or_else(|| "(none yet)".into());
        let stats_line = if stats.is_empty() {
            "-".to_string()
        } else {
            stats.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("  ")
        };
        println!("name:       {}   room: {}   mode: {}", self.name, self.room,
                 if self.turn { "TURN relay" } else { "direct P2P" });
        println!("server:     {}", self.server);
        println!("reflexive:  {}", state.mapped.map(addr_s).unwrap_or_else(|| "-".into()));
        println!("my relay:   {}", state.relay.map(addr_s).unwrap_or_else(|| "-".into()));
        println!("peer:       {peer_line}");
        println!("connected:  {}", state.connected);
        println!("stats:      {stats_line}");
    }

    fn console(&self) {
        print!("{HELP}");
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            let line = line.trim();
            match line {
                "" => continue,
                "/quit" | "/q" => break,
                "/status" => self.show_status(),
                "/help" => print!("{HELP}"),
                _ if line.starts_with("/sendfile ") => {
                    self.send_file(line["/sendfile ".len()..].trim())
                }
                _ if line.starts_with("/send ") => self.send_text(&line["/send ".len()..]),
                _ if line.starts_with('/') => print!("unknown command\n{HELP}"),
                _ => self.send_text(line),
            }
        }
        println!("[{}] bye", self.name);
    }
}

fn frame(typ: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(3 + payload.len());
    out.extend_from_slice(FRAME_MAGIC);
    out.push(typ);
    out.extend_from_slice(payload);
    out
}

fn json_id(payload: &[u8]) -> Option<u32> {
    serde_json::from_slice::<serde_json::Value>(payload).ok()?["id"].as_u64().map(|v| v as u32)
}

struct Opts {
    name: String,
    room: String,
    server: Option<String>,
    turn: bool,
    verbose: bool,
}

fn parse_args() -> Opts {
    let mut opts = Opts {
        name: format!("peer-{:04x}", rand::random::<u16>()),
        room: "demo".into(),
        server: None,
        turn: false,
        verbose: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--name" => opts.name = args.next().expect("--name needs a value"),
            "--room" => opts.room = args.next().expect("--room needs a value"),
            "--server" => opts.server = Some(args.next().expect("--server needs a value")),
            "--turn" => opts.turn = true,
            "--verbose" => opts.verbose = true,
            other => {
                eprintln!("unknown option {other}");
                eprintln!("usage: peer [--name NAME] [--room ROOM] [--server HOST:PORT] [--turn] [--verbose]");
                std::process::exit(2);
            }
        }
    }
    opts
}

fn main() {
    let opts = parse_args();
    let default_port = if opts.turn { 3479 } else { 3478 };
    let server_str = opts.server.clone()
        .unwrap_or_else(|| format!("127.0.0.1:{default_port}"));
    let (host, port) = server_str.split_once(':').unwrap_or((server_str.as_str(), "3478"));
    let server = SocketAddrV4::new(
        host.parse().expect("server must be an IPv4 address"),
        port.parse().expect("bad server port"),
    );

    let sock = UdpSocket::bind("0.0.0.0:0").expect("bind failed"); // one socket for STUN, TURN and P2P data
    let (tx, rx) = std::sync::mpsc::channel();
    let peer = Arc::new(Peer {
        name: opts.name,
        room: opts.room,
        turn: opts.turn,
        verbose: opts.verbose,
        server,
        sock,
        responses: Mutex::new(tx),
        quiet_txids: Mutex::new(HashSet::new()),
        state: Mutex::new(ConnState::default()),
        peer_cv: Condvar::new(),
        incoming: Mutex::new(HashMap::new()),
        outgoing: Mutex::new(HashMap::new()),
        completed: Mutex::new(HashSet::new()),
        stats: Mutex::new(HashMap::new()),
        punch_logged: Mutex::new(HashSet::new()),
    });
    peer.run(rx);
}
