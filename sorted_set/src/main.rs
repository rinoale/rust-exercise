//! A miniature Redis sorted set (zset), structured the way Redis structures it:
//!
//!   1. a HashMap  member -> score        (answers ZSCORE in O(1))
//!   2. a skip list ordered by (score, member)  (answers ZRANGEBYSCORE in O(log n))
//!
//! Both structures point at the same entries; ZADD/ZREM keep them in sync.
//!
//! The skip list is the part people find mysterious, so `diagram()` draws its
//! lanes. Lane 0 is a plain sorted linked list containing every node. Each
//! higher lane is an "express lane" that skips nodes: at insert time a node
//! flips coins to decide how many lanes it joins (50% chance per extra lane
//! here; Redis uses 25% and up to 32 lanes, we cap at 4 to keep the picture
//! small). A search starts on the top lane, rides it as far as it can without
//! overshooting, then drops down a lane and repeats — O(log n) expected.
//!
//! Simplifications vs. real Redis: no listpack encoding for small sets, no
//! per-link spans (which is what makes ZRANK fast), and removed nodes stay as
//! dead slots in the arena instead of being freed.

use std::collections::HashMap;

const MAX_LEVEL: usize = 4;
const PROMOTE_P: u64 = 2; // 1-in-2 chance to join each further lane

/// Tiny deterministic PRNG so every run prints the same diagram.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Coin-flip promotion: lane count 1 with p=1/2, 2 with 1/4, 3 with 1/8...
    /// (xorshift low bits are weak, so flip a high bit)
    fn random_level(&mut self) -> usize {
        let mut level = 1;
        while level < MAX_LEVEL && (self.next() >> 33).is_multiple_of(PROMOTE_P) {
            level += 1;
        }
        level
    }
}

struct Node {
    member: String,
    score: f64,
    /// forward[l] = arena index of the next node on lane l.
    /// forward.len() is how many lanes this node joined.
    forward: Vec<Option<usize>>,
}

/// Arena-based skip list: nodes own no pointers, only indices into `nodes`.
/// `nodes[0]` is the HEAD sentinel, present on every lane before all data.
struct SkipList {
    nodes: Vec<Node>,
    level: usize, // highest lane currently in use
}

impl SkipList {
    fn new() -> Self {
        let head = Node {
            member: String::new(),
            score: f64::NEG_INFINITY,
            forward: vec![None; MAX_LEVEL],
        };
        SkipList { nodes: vec![head], level: 1 }
    }

    /// Skip-list order is (score, then member) so equal scores still have
    /// one deterministic position — same tie-break as Redis.
    fn comes_before(&self, idx: usize, score: f64, member: &str) -> bool {
        let n = &self.nodes[idx];
        n.score < score || (n.score == score && n.member.as_str() < member)
    }

    /// Walk from HEAD to the last node that sorts before (score, member),
    /// remembering the drop-off point on every lane. Every operation —
    /// insert, remove, range — starts with this same descent.
    fn descend(&self, score: f64, member: &str, trace: Option<&mut Vec<String>>) -> [usize; MAX_LEVEL] {
        let mut path = [0usize; MAX_LEVEL];
        let mut cur = 0;
        let mut lines = Vec::new();
        for l in (0..self.level).rev() {
            let mut moves = Vec::new();
            while let Some(next) = self.nodes[cur].forward[l] {
                if self.comes_before(next, score, member) {
                    moves.push(self.nodes[next].member.clone());
                    cur = next;
                } else {
                    break;
                }
            }
            path[l] = cur;
            lines.push(format!(
                "    lane {l}: {}",
                if moves.is_empty() { "(overshoots immediately, drop down)".to_string() } else { format!("ride to {}", moves.join(" -> ")) }
            ));
        }
        if let Some(t) = trace {
            *t = lines;
        }
        path
    }

    fn insert(&mut self, member: &str, score: f64, rng: &mut XorShift) {
        let mut path = self.descend(score, member, None);
        let lanes = rng.random_level();
        if lanes > self.level {
            for slot in path.iter_mut().take(lanes).skip(self.level) {
                *slot = 0; // new top lanes start from HEAD
            }
            self.level = lanes;
        }
        let idx = self.nodes.len();
        self.nodes.push(Node {
            member: member.to_string(),
            score,
            forward: vec![None; lanes],
        });
        for (l, &prev) in path.iter().enumerate().take(lanes) {
            self.nodes[idx].forward[l] = self.nodes[prev].forward[l];
            self.nodes[prev].forward[l] = Some(idx);
        }
    }

    fn remove(&mut self, member: &str, score: f64) {
        let path = self.descend(score, member, None);
        let Some(target) = self.nodes[path[0]].forward[0] else { return };
        if self.nodes[target].member != member {
            return;
        }
        // path[l] is the node just before target on lane l; bypass it.
        for (l, &prev) in path.iter().enumerate().take(self.nodes[target].forward.len()) {
            self.nodes[prev].forward[l] = self.nodes[target].forward[l];
        }
        while self.level > 1 && self.nodes[0].forward[self.level - 1].is_none() {
            self.level -= 1;
        }
    }

    /// ZRANGEBYSCORE: one O(log n) descent to the first node >= min,
    /// then a plain lane-0 walk until the score passes max.
    fn range_by_score(&self, min: f64, max: f64, trace: Option<&mut Vec<String>>) -> Vec<(String, f64)> {
        let path = self.descend(min, "", trace);
        let mut out = Vec::new();
        let mut cur = self.nodes[path[0]].forward[0];
        while let Some(i) = cur {
            if self.nodes[i].score > max {
                break;
            }
            out.push((self.nodes[i].member.clone(), self.nodes[i].score));
            cur = self.nodes[i].forward[0];
        }
        out
    }

    /// Draw every lane. A node appears on lane l only if it joined it;
    /// otherwise its column is dashes — the "express lane skips it" picture.
    fn diagram(&self) -> String {
        let mut order = Vec::new();
        let mut cur = self.nodes[0].forward[0];
        while let Some(i) = cur {
            order.push(i);
            cur = self.nodes[i].forward[0];
        }
        let labels: Vec<String> = order
            .iter()
            .map(|&i| format!("[{} {}]", self.nodes[i].member, self.nodes[i].score))
            .collect();
        let mut out = String::new();
        for l in (0..self.level).rev() {
            out.push_str(&format!("  lane {l}: HEAD"));
            for (col, &i) in order.iter().enumerate() {
                if self.nodes[i].forward.len() > l {
                    out.push_str(&format!("-->{}", labels[col]));
                } else {
                    out.push_str(&format!("---{}", "-".repeat(labels[col].len())));
                }
            }
            out.push_str("--> None\n");
        }
        out
    }
}

/// The zset itself: the dict and the skip list, kept in sync.
struct SortedSet {
    scores: HashMap<String, f64>,
    list: SkipList,
    rng: XorShift,
}

impl SortedSet {
    fn new() -> Self {
        SortedSet { scores: HashMap::new(), list: SkipList::new(), rng: XorShift(0xBADC0DE) }
    }

    /// ZADD. Updating an existing member means relocating it in the skip
    /// list — remove at the old score, insert at the new one.
    fn zadd(&mut self, member: &str, score: f64) {
        if let Some(old) = self.scores.insert(member.to_string(), score) {
            self.list.remove(member, old);
        }
        self.list.insert(member, score, &mut self.rng);
    }

    /// ZSCORE — the HashMap's entire reason to exist. No list walk.
    fn zscore(&self, member: &str) -> Option<f64> {
        self.scores.get(member).copied()
    }

    fn zrem(&mut self, member: &str) {
        if let Some(score) = self.scores.remove(member) {
            self.list.remove(member, score);
        }
    }

    fn zrange_by_score(&self, min: f64, max: f64, trace: Option<&mut Vec<String>>) -> Vec<(String, f64)> {
        self.list.range_by_score(min, max, trace)
    }
}

fn main() {
    // The pending-expiry scheduler pattern: score = due timestamp.
    let mut zset = SortedSet::new();
    for (member, due) in [
        ("job:email", 905.0),
        ("session:42", 910.0),
        ("job:retry", 925.0),
        ("job:report", 930.0),
        ("token:9", 945.0),
        ("cache:warm", 960.0),
    ] {
        zset.zadd(member, due);
    }

    println!("ZADD six items (score = due timestamp). The skip list lanes:\n");
    println!("{}", zset.list.diagram());
    println!("  Lane 0 holds everything in score order. Higher lanes skip nodes;");
    println!("  which nodes joined which lanes was decided by coin flips at insert.\n");

    println!("ZSCORE session:42 -> {:?}   (HashMap hit, the list is not consulted)\n", zset.zscore("session:42"));

    // First show the descent doing real work: seek a score deep in the list.
    let mut trace = Vec::new();
    let late = zset.zrange_by_score(940.0, 1000.0, Some(&mut trace));
    println!("ZRANGEBYSCORE 940 1000 — the descent rides express lanes, then drops:");
    for line in &trace {
        println!("{line}");
    }
    for (member, score) in &late {
        println!("    found: {member} (score {score})");
    }

    let now = 930.0;
    let due = zset.zrange_by_score(f64::NEG_INFINITY, now, None);
    println!("\nZRANGEBYSCORE -inf {now}  (\"what is due?\"):");
    for (member, score) in &due {
        println!("    due: {member} (score {score})");
    }

    println!("\nProcess the due items, ZREM each, and the lanes heal around them:\n");
    for (member, _) in &due {
        zset.zrem(member);
    }
    println!("{}", zset.list.diagram());

    println!("ZADD token:9 990 (re-schedule an existing member = remove + reinsert):\n");
    zset.zadd("token:9", 990.0);
    println!("{}", zset.list.diagram());
}
