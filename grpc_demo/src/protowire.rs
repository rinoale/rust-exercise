//! Schema-free protobuf wire-format decoder: the bytes tell you the field
//! numbers and wire types, the .proto would tell you the names.

fn wire_name(wire: u64) -> Option<&'static str> {
    match wire {
        0 => Some("VARINT"),
        1 => Some("I64"),
        2 => Some("LEN"),
        5 => Some("I32"),
        _ => None,
    }
}

fn varint(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut val: u64 = 0;
    let mut shift = 0;
    loop {
        let b = *data.get(*pos)?;
        *pos += 1;
        val |= u64::from(b & 0x7f) << shift;
        if b < 0x80 {
            return Some(val);
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }
}

/// Returns printable lines, or None if `data` is not valid protobuf.
pub fn decode(data: &[u8], depth: usize) -> Option<Vec<String>> {
    let mut pos = 0;
    let mut lines = vec![];
    let pad = "  ".repeat(depth);
    while pos < data.len() {
        let tag = varint(data, &mut pos)?;
        let field = tag >> 3;
        let wire = tag & 7;
        let wname = wire_name(wire)?;
        if field == 0 {
            return None;
        }
        match wire {
            0 => {
                let v = varint(data, &mut pos)?;
                lines.push(format!("{pad}field {field:<2} {wname:<7} = {v}"));
            }
            1 => {
                let raw = data.get(pos..pos + 8)?;
                pos += 8;
                let d = f64::from_le_bytes(raw.try_into().unwrap());
                lines.push(format!("{pad}field {field:<2} {wname:<7} = 0x{} (double {d})", hex(raw)));
            }
            5 => {
                let raw = data.get(pos..pos + 4)?;
                pos += 4;
                let f = f32::from_le_bytes(raw.try_into().unwrap());
                lines.push(format!("{pad}field {field:<2} {wname:<7} = 0x{} (float {f})", hex(raw)));
            }
            2 => {
                let len = varint(data, &mut pos)? as usize;
                let raw = data.get(pos..pos + len)?;
                pos += len;
                lines.extend(len_field(&pad, field, raw, depth));
            }
            _ => unreachable!(),
        }
    }
    Some(lines)
}

fn len_field(pad: &str, field: u64, raw: &[u8], depth: usize) -> Vec<String> {
    let head = format!("{pad}field {field:<2} {:<7} ({} bytes)", "LEN", raw.len());
    if let Ok(s) = std::str::from_utf8(raw)
        && !raw.is_empty() && s.chars().all(|c| !c.is_control() || c.is_whitespace()) {
            return vec![format!("{head} = {s:?}")];
        }
    if let Some(nested) = decode(raw, depth + 1) {
        let mut lines = vec![format!("{head} = nested message {{")];
        lines.extend(nested);
        lines.push(format!("{pad}}}"));
        return lines;
    }
    vec![format!("{head} = 0x{}", hex(raw))]
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
