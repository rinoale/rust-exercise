//! Logs one packet as a single write so concurrent processes/threads don't
//! interleave mid-dump.

use crate::stun;
use std::io::Write;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    Send,
    Recv,
}

impl Dir {
    fn arrow(self) -> &'static str {
        match self {
            Dir::Send => "--->",
            Dir::Recv => "<---",
        }
    }
}

pub fn log(role: &str, dir: Dir, remote: &str, data: &[u8], note: Option<&str>) {
    let mut out = format!("[{role}] {} {remote}", dir.arrow());
    if let Some(n) = note {
        out.push_str(&format!("   ({n})"));
    }
    out.push('\n');
    if stun::is_stun_packet(data) {
        for l in stun::describe(data) {
            out.push_str("    ");
            out.push_str(&l);
            out.push('\n');
        }
    } else {
        let preview = if data.len() > 64 {
            format!("{:?}... ({} bytes)", String::from_utf8_lossy(&data[..48]), data.len())
        } else {
            format!("{:?}", String::from_utf8_lossy(data))
        };
        out.push_str(&format!("    raw UDP payload (not STUN): {preview}\n"));
    }
    for l in stun::hexdump(data) {
        out.push_str(&l);
        out.push('\n');
    }
    out.push('\n');
    write_atomic(&out);
}

/// One-line form for bulk traffic (file chunks) so transfers don't flood
/// the terminal with hexdumps.
pub fn brief(role: &str, dir: Dir, remote: &str, text: &str) {
    write_atomic(&format!("[{role}] {} {remote}  {text}\n", dir.arrow()));
}

fn write_atomic(s: &str) {
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(s.as_bytes());
    let _ = stdout.flush();
}
