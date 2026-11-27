//! W3D chunk stream (`WWLib/chunkio.h`) and the record cursor used by the readers.
//!
//! Every byte a reader does not consume is recorded in [`Report`]: nothing is
//! skipped silently.

/// One chunk: `u32` id, `u32` size (top bit set when it holds sub-chunks).
#[derive(Clone, Copy, Debug)]
pub struct Chunk<'a> {
    pub id: u32,
    pub has_sub_chunks: bool,
    pub data: &'a [u8],
}

/// A chunk no reader knows. `path` is the ids of its parents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    pub path: Vec<u32>,
    pub id: u32,
    pub size: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub skipped: Vec<Skipped>,
    /// Overflowing or truncated chunks and bytes left unread in a known chunk.
    pub damaged: Vec<String>,
    /// Chunks seen, at every depth.
    pub chunks: u64,
    /// Bytes after a channel's values that the engine seeks over.
    pub padding: u64,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.skipped.is_empty() && self.damaged.is_empty()
    }

    pub fn skip(&mut self, path: &[u32], c: &Chunk) {
        self.skipped.push(Skipped {
            path: path.to_vec(),
            id: c.id,
            size: c.data.len(),
        });
    }
}

/// Splits `data` into chunks. A chunk that runs past `data` ends the walk and is
/// recorded as damage, as are trailing bytes too short for a header.
pub fn chunks<'a>(data: &'a [u8], path: &[u32], report: &mut Report) -> Vec<Chunk<'a>> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < data.len() {
        if data.len() - at < 8 {
            report.damaged.push(format!(
                "{path:x?}: {} bytes after the last chunk",
                data.len() - at
            ));
            break;
        }
        let id = le_u32(&data[at..]);
        let raw = le_u32(&data[at + 4..]);
        let size = (raw & 0x7fff_ffff) as usize;
        at += 8;
        if size > data.len() - at {
            report.damaged.push(format!(
                "{path:x?}: chunk {id:#x} of {size} bytes overruns its parent"
            ));
            break;
        }
        report.chunks += 1;
        out.push(Chunk {
            id,
            has_sub_chunks: raw >> 31 != 0,
            data: &data[at..at + size],
        });
        at += size;
    }
    out
}

fn le_u32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// Little-endian record reader.
#[derive(Clone, Copy)]
pub struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}

/// A read ran past the end of its chunk or file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Truncated;

pub type Read<T> = Result<T, Truncated>;

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Cursor { data, at: 0 }
    }
    pub fn remaining(&self) -> usize {
        self.data.len() - self.at
    }
    pub fn position(&self) -> usize {
        self.at
    }
    pub fn bytes(&mut self, n: usize) -> Read<&'a [u8]> {
        if n > self.remaining() {
            return Err(Truncated);
        }
        let s = &self.data[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    pub fn array<const N: usize>(&mut self) -> Read<[u8; N]> {
        let mut out = [0; N];
        out.copy_from_slice(self.bytes(N)?);
        Ok(out)
    }
    pub fn u8(&mut self) -> Read<u8> {
        Ok(self.array::<1>()?[0])
    }
    pub fn u16(&mut self) -> Read<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    pub fn u32(&mut self) -> Read<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    pub fn i32(&mut self) -> Read<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }
    pub fn f32(&mut self) -> Read<f32> {
        Ok(f32::from_bits(self.u32()?))
    }
    pub fn floats<const N: usize>(&mut self) -> Read<[f32; N]> {
        let mut out = [0.0; N];
        for v in &mut out {
            *v = self.f32()?;
        }
        Ok(out)
    }
    pub fn uints<const N: usize>(&mut self) -> Read<[u32; N]> {
        let mut out = [0; N];
        for v in &mut out {
            *v = self.u32()?;
        }
        Ok(out)
    }
    /// A fixed-width name field, cut at the first NUL.
    pub fn name(&mut self, width: usize) -> Read<String> {
        Ok(c_string(self.bytes(width)?))
    }
}

/// Bytes up to the first NUL, one `char` per byte.
pub fn c_string(b: &[u8]) -> String {
    b.iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as char)
        .collect()
}

/// Reads fixed-size records until the chunk is used up. Left-over bytes are damage.
pub fn records<'a, T>(
    data: &'a [u8],
    what: &str,
    report: &mut Report,
    mut read: impl FnMut(&mut Cursor<'a>) -> Read<T>,
) -> Vec<T> {
    let mut c = Cursor::new(data);
    let mut out = Vec::new();
    while c.remaining() != 0 {
        match read(&mut c) {
            Ok(v) => out.push(v),
            Err(Truncated) => {
                report.damaged.push(format!("{what}: partial record"));
                break;
            }
        }
    }
    out
}

/// Records damage if `c` has bytes left.
pub fn finish(c: &Cursor, what: &str, report: &mut Report) {
    if c.remaining() != 0 {
        report
            .damaged
            .push(format!("{what}: {} bytes unread", c.remaining()));
    }
}
