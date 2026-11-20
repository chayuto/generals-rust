//! Zero-dependency JSON line/object extractor and hex parser for Gate V2 Golden Test Vectors.

/// Extracts each `{"seed": ...}` object line from a Golden Vector JSON file's `"vectors": [...]` array.
pub fn extract_vector_lines(json_text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    for line in json_text.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if trimmed.starts_with('{') && trimmed.ends_with('}') && trimmed.contains("\"seed") {
            out.push(trimmed);
        }
    }
    out
}

fn find_key_value_slice<'a>(obj_line: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{}\":", key);
    let start = obj_line.find(&pattern)? + pattern.len();
    Some(obj_line[start..].trim_start())
}

pub fn get_str<'a>(obj_line: &'a str, key: &str) -> Option<&'a str> {
    let rest = find_key_value_slice(obj_line, key)?;
    if !rest.starts_with('"') {
        return None;
    }
    let after_quote = &rest[1..];
    let end = after_quote.find('"')?;
    Some(&after_quote[..end])
}

pub fn parse_i64_token(token: &str) -> Option<i64> {
    let bytes = token.trim().as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut idx = 0;
    let mut neg = false;
    if bytes[0] == b'-' {
        neg = true;
        idx += 1;
    }
    if idx >= bytes.len() {
        return None;
    }
    let mut acc: i64 = 0;
    while idx < bytes.len() {
        let b = bytes[idx];
        if !b.is_ascii_digit() {
            break;
        }
        acc = acc.wrapping_mul(10).wrapping_add((b - b'0') as i64);
        idx += 1;
    }
    Some(if neg { acc.wrapping_neg() } else { acc })
}

pub fn get_i32(obj_line: &str, key: &str) -> Option<i32> {
    let rest = find_key_value_slice(obj_line, key)?;
    let end = rest.find([',', '}', ']']).unwrap_or(rest.len());
    parse_i64_token(&rest[..end]).map(|v| v as i32)
}

pub fn get_u32(obj_line: &str, key: &str) -> Option<u32> {
    let rest = find_key_value_slice(obj_line, key)?;
    let end = rest.find([',', '}', ']']).unwrap_or(rest.len());
    parse_i64_token(&rest[..end]).map(|v| v as u32)
}

/// Parses a signed 64-bit integer from JSON.
pub fn get_i64(obj_line: &str, key: &str) -> Option<i64> {
    let rest = find_key_value_slice(obj_line, key)?;
    let end = rest.find([',', '}', ']']).unwrap_or(rest.len());
    parse_i64_token(&rest[..end])
}

/// Parses a variable-length `[i32, ...]` array (including empty `[]`).
pub fn get_i32_vec(obj_line: &str, key: &str) -> Option<Vec<i32>> {
    let rest = find_key_value_slice(obj_line, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    let end = rest.find(']')?;
    let inner = rest[1..end].trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    for part in inner.split(',') {
        out.push(parse_i64_token(part)? as i32);
    }
    Some(out)
}

/// Parses a variable-length `[u16, ...]` array (including empty `[]`).
pub fn get_u16_vec(obj_line: &str, key: &str) -> Option<Vec<u16>> {
    let rest = find_key_value_slice(obj_line, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    let end = rest.find(']')?;
    let inner = rest[1..end].trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    for part in inner.split(',') {
        out.push(parse_i64_token(part)? as u16);
    }
    Some(out)
}

/// Parses an unsigned 64-bit integer (e.g. `f64` bit patterns up to `u64::MAX`).
pub fn get_u64(obj_line: &str, key: &str) -> Option<u64> {
    let rest = find_key_value_slice(obj_line, key)?.trim_start();
    let end = rest.find([',', '}', ']']).unwrap_or(rest.len());
    let token = rest[..end].trim();
    if token.is_empty() || !token.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut acc: u64 = 0;
    for b in token.bytes() {
        acc = acc.checked_mul(10)?.checked_add((b - b'0') as u64)?;
    }
    Some(acc)
}

pub fn get_i32_array<const N: usize>(obj_line: &str, key: &str) -> Option<[i32; N]> {
    let rest = find_key_value_slice(obj_line, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    let end = rest.find(']')?;
    let inner = &rest[1..end];
    let mut out = [0i32; N];
    let mut count = 0;
    for part in inner.split(',') {
        if count < N {
            out[count] = parse_i64_token(part)? as i32;
            count += 1;
        }
    }
    if count == N {
        Some(out)
    } else {
        None
    }
}

pub fn get_u32_array<const N: usize>(obj_line: &str, key: &str) -> Option<[u32; N]> {
    let rest = find_key_value_slice(obj_line, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    let end = rest.find(']')?;
    let inner = &rest[1..end];
    let mut out = [0u32; N];
    let mut count = 0;
    for part in inner.split(',') {
        if count < N {
            out[count] = parse_i64_token(part)? as u32;
            count += 1;
        }
    }
    if count == N {
        Some(out)
    } else {
        None
    }
}

#[inline]
fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub fn parse_hex_u32(s: &str) -> Option<u32> {
    let trimmed = s.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if hex.is_empty() {
        return None;
    }
    let mut acc: u32 = 0;
    for b in hex.bytes() {
        let nib = hex_nibble(b)?;
        acc = (acc << 4) | (nib as u32);
    }
    Some(acc)
}

pub fn get_hex_u32(obj_line: &str, key: &str) -> Option<u32> {
    let s = get_str(obj_line, key)?;
    parse_hex_u32(s)
}

pub fn decode_hex_bytes(hex: &str) -> Option<Vec<u8>> {
    let bytes = hex.trim().as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

pub fn encode_hex_bytes(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0F) as usize] as char);
    }
    out
}
