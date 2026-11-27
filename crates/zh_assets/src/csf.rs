//! Compiled string files (`GameTextManager::parseCSF`).

use crate::chunk::{c_string, Cursor, Read, Truncated};

const ID_FILE: u32 = u32::from_be_bytes(*b"CSF ");
const ID_LABEL: u32 = u32::from_be_bytes(*b"LBL ");
const ID_STRING: u32 = u32::from_be_bytes(*b"STR ");
const ID_STRING_WITH_WAVE: u32 = u32::from_be_bytes(*b"STRW");
/// `MAX_UITEXT_LENGTH`: the engine's buffers.
const MAX_LENGTH: u32 = 10 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub label: String,
    /// UTF-16 units of the first string, decoded and space-stripped.
    pub text: Vec<u16>,
    /// Speech file name of the first string, empty when it has none.
    pub speech: String,
    /// How many strings the label holds; only the first is kept.
    pub strings: u32,
}

#[derive(Clone, Debug)]
pub struct Csf {
    pub version: u32,
    pub declared_labels: u32,
    pub declared_strings: u32,
    pub language: u32,
    pub entries: Vec<Entry>,
    /// Bytes too few to be another record.
    pub trailing: usize,
}

fn length(c: &mut Cursor) -> Read<usize> {
    let n = c.u32()?;
    if n >= MAX_LENGTH {
        return Err(Truncated);
    }
    Ok(n as usize)
}

pub fn parse(data: &[u8]) -> Read<Csf> {
    let mut c = Cursor::new(data);
    if c.u32()? != ID_FILE {
        return Err(Truncated);
    }
    let version = c.u32()?;
    let declared_labels = c.u32()?;
    let declared_strings = c.u32()?;
    let skip = c.i32()?;
    let language = c.u32()?;
    let mut csf = Csf {
        version,
        declared_labels,
        declared_strings,
        // Version 1 files carry no language: US English.
        language: if version >= 2 { language } else { 0 },
        entries: Vec::new(),
        trailing: 0,
    };
    if skip > 0 {
        c.bytes(skip as usize)?;
    }
    while c.remaining() >= 4 {
        if c.u32()? != ID_LABEL {
            return Err(Truncated);
        }
        let strings = c.u32()?;
        let n = length(&mut c)?;
        let mut entry = Entry {
            label: c_string(c.bytes(n)?),
            text: Vec::new(),
            speech: String::new(),
            strings,
        };
        for num in 0..strings {
            let id = c.u32()?;
            if id != ID_STRING && id != ID_STRING_WITH_WAVE {
                return Err(Truncated);
            }
            let units = length(&mut c)?;
            let raw = c.bytes(units * 2)?;
            if num == 0 {
                // A stored 0 ends the text before decoding, a stored 0xFFFF after.
                let decoded: Vec<u16> = raw
                    .chunks_exact(2)
                    .map(|p| u16::from_le_bytes([p[0], p[1]]))
                    .take_while(|&u| u != 0)
                    .map(|u| !u)
                    .take_while(|&u| u != 0)
                    .collect();
                entry.text = strip_spaces(&decoded);
            }
            if id == ID_STRING_WITH_WAVE {
                let n = length(&mut c)?;
                let wave = c.bytes(n)?;
                if num == 0 && n != 0 {
                    entry.speech = c_string(wave);
                }
            }
        }
        csf.entries.push(entry);
    }
    csf.trailing = c.remaining();
    Ok(csf)
}

/// `GameTextManager::stripSpaces`: drops leading and repeated spaces, and the
/// space before a newline, tab or the end.
fn strip_spaces(text: &[u16]) -> Vec<u16> {
    const SPACE: u16 = b' ' as u16;
    let mut out = Vec::with_capacity(text.len());
    let mut last = 0;
    let mut skip_all = true;
    for &ch in text {
        if ch == SPACE && (last == SPACE || skip_all) {
            continue;
        }
        if ch == u16::from(b'\n') || ch == u16::from(b'\t') {
            if last == SPACE {
                out.pop();
            }
            skip_all = true;
        } else {
            skip_all = false;
        }
        last = ch;
        out.push(ch);
    }
    if last == SPACE {
        out.pop();
    }
    out
}
