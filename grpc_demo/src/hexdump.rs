//! Classic `xxd`-style hexdump, shared by the tap's frame printer.

/// Formats `bytes` as hexdump lines, capped at `max` bytes (0 = no cap).
pub fn hexdump(bytes: &[u8], indent: &str, max: usize) -> Vec<String> {
    if bytes.is_empty() {
        return vec![];
    }
    let shown = if max > 0 && bytes.len() > max { &bytes[..max] } else { bytes };
    let mut lines: Vec<String> = shown
        .chunks(16)
        .enumerate()
        .map(|(i, row)| {
            let hex = row
                .chunks(8)
                .map(|g| g.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join("  ");
            let asc: String = row
                .iter()
                .map(|&b| if (32..=126).contains(&b) { b as char } else { '.' })
                .collect();
            format!("{indent}{:04x}  {hex:<49} |{asc}|", i * 16)
        })
        .collect();
    if shown.len() < bytes.len() {
        lines.push(format!("{indent}      ... {} more bytes", bytes.len() - shown.len()));
    }
    lines
}
