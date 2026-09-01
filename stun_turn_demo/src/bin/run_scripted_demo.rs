//! The fully-automated walkthrough: STUN server, TURN server, signaling
//! server, scripted peer B, scripted peer A — all as separate processes on
//! localhost, sharing this terminal.
//!
//!   cargo run --bin run_scripted_demo

use std::process::{Child, Command};
use std::time::Duration;

fn spawn(bin: &str, args: &[&str]) -> Child {
    // Sibling binaries live next to this one in target/<profile>/.
    let path = std::env::current_exe()
        .expect("current_exe failed")
        .with_file_name(bin);
    Command::new(&path)
        .args(args)
        .spawn()
        .unwrap_or_else(|e| panic!("failed to start {} ({e}) — run `cargo build` first", path.display()))
}

fn main() {
    let mut servers: Vec<Child> = ["stun_server", "turn_server", "signaling_server"]
        .iter()
        .map(|s| spawn(s, &[]))
        .collect();
    std::thread::sleep(Duration::from_millis(500));

    let mut peer_b = spawn("scripted_peer", &["B"]);
    std::thread::sleep(Duration::from_millis(300));
    let mut peer_a = spawn("scripted_peer", &["A"]);

    let _ = peer_a.wait();
    let _ = peer_b.wait();
    std::thread::sleep(Duration::from_millis(500));

    for server in &mut servers {
        let _ = server.kill();
        let _ = server.wait();
    }
    println!("\n[demo] finished");
}
