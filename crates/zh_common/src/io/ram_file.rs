//! `PKT-460`: In-memory read-only file buffer matching `RAMFile`
//! (`Core/GameEngine/Include/Common/RAMFile.h` & `Core/GameEngine/Source/Common/System/RAMFile.cpp`).

use crate::ini::IniReader;
use crate::io::file::{FileAccess, FileBase, LocalFileHost, SeekMode};
use crate::types::{Bool, Int, Real, FALSE, TRUE};

#[inline]
const fn is_ascii_c_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Pure-Rust representation of SAGE `RAMFile`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RamFile {
    base: FileBase,
    data: Option<Vec<u8>>,
    pos: usize,
}

impl Default for RamFile {
    fn default() -> Self {
        Self::new()
    }
}

impl RamFile {
    pub fn new() -> Self {
        Self {
            base: FileBase::new(),
            data: None,
            pos: 0,
        }
    }

    pub fn open(&mut self, host: &dyn LocalFileHost, filename: &str, access: Int) -> Bool {
        let Some((bytes, _info)) = host.read_file_bytes(filename) else {
            return FALSE;
        };
        if self.base.open(filename, access) == FALSE {
            return FALSE;
        }
        self.data = Some(bytes);
        self.pos = 0;
        TRUE
    }

    pub fn open_from_bytes(&mut self, filename: &str, bytes: Vec<u8>, access: Int) -> Bool {
        if self.base.open(filename, access) == FALSE {
            return FALSE;
        }
        self.data = Some(bytes);
        self.pos = 0;
        TRUE
    }

    pub fn open_from_archive(
        &mut self,
        archive_bytes: Option<&[u8]>,
        filename: &str,
        offset: Int,
        size: Int,
    ) -> Bool {
        let Some(src) = archive_bytes else {
            return FALSE;
        };
        if self.base.open(filename, FileAccess::READ | FileAccess::BINARY) == FALSE {
            return FALSE;
        }
        if offset < 0 || size < 0 {
            return FALSE;
        }
        let off = offset as usize;
        let sz = size as usize;
        let Some(end) = off.checked_add(sz) else {
            return FALSE;
        };
        if end > src.len() {
            return FALSE;
        }
        self.data = Some(src[off..end].to_vec());
        self.pos = 0;
        self.base.set_name(filename);
        TRUE
    }

    pub fn close(&mut self) {
        self.data = None;
        self.pos = 0;
        self.base.close();
    }

    pub fn read(&mut self, buffer: Option<&mut [u8]>, bytes: Int) -> Int {
        let Some(ref data) = self.data else {
            return -1;
        };
        let bytes_left = data.len().saturating_sub(self.pos) as isize;
        let requested = bytes as isize;
        let count = if requested > bytes_left {
            bytes_left
        } else {
            requested
        };
        if count > 0 {
            let n = count as usize;
            if let Some(dst) = buffer {
                let copy_len = n.min(dst.len());
                dst[..copy_len].copy_from_slice(&data[self.pos..self.pos + copy_len]);
            }
            self.pos += n;
        }
        count as Int
    }

    pub fn seek(&mut self, pos: Int, mode: SeekMode) -> Int {
        let len = self.data.as_ref().map_or(0isize, |d| d.len() as isize);
        let mut new_pos: isize = match mode {
            SeekMode::Start => pos as isize,
            SeekMode::Current => (self.pos as isize) + (pos as isize),
            SeekMode::End => len + (pos as isize),
        };
        if new_pos < 0 {
            new_pos = 0;
        } else if new_pos > len {
            new_pos = len;
        }
        self.pos = new_pos as usize;
        self.pos as Int
    }

    /// Raw seek preserving C++ behavior when an invalid `seekMode` integer is passed (`-1`).
    pub fn seek_raw(&mut self, pos: Int, mode_raw: Int) -> Int {
        match SeekMode::from_i32(mode_raw) {
            Some(mode) => self.seek(pos, mode),
            None => -1,
        }
    }

    #[inline]
    pub fn size(&self) -> Int {
        self.data.as_ref().map_or(0, |d| d.len() as Int)
    }

    #[inline]
    pub fn position(&self) -> Int {
        self.pos as Int
    }

    #[inline]
    pub fn eof(&self) -> Bool {
        if self.position() == self.size() {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.base.name()
    }

    #[inline]
    pub fn access(&self) -> Int {
        self.base.access()
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        self.base.is_open()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_deref().unwrap_or(&[])
    }

    pub fn read_entire_and_close(mut self) -> Vec<u8> {
        let out = self.data.take().unwrap_or_default();
        self.close();
        out
    }

    /// Scans the next signed decimal integer token matching `RAMFile::scanInt` (`RAMFile.cpp:401-426`).
    pub fn scan_int(&mut self) -> Option<Int> {
        let data = self.data.as_deref()?;
        while self.pos < data.len() && !data[self.pos].is_ascii_digit() && data[self.pos] != b'-'
        {
            self.pos += 1;
        }
        if self.pos >= data.len() {
            self.pos = data.len();
            return None;
        }
        let start = self.pos;
        self.pos += 1;
        while self.pos < data.len() && data[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let token = core::str::from_utf8(&data[start..self.pos]).unwrap_or("");
        if token == "-" || !token.bytes().any(|b| b.is_ascii_digit()) {
            return Some(0);
        }
        let mut neg = false;
        let mut idx = 0usize;
        let tb = token.as_bytes();
        if tb[0] == b'-' {
            neg = true;
            idx = 1;
        }
        let mut acc: u64 = 0;
        for &b in &tb[idx..] {
            acc = acc.saturating_mul(10).saturating_add((b - b'0') as u64);
        }
        let signed = if neg {
            -(acc as i64)
        } else {
            acc as i64
        };
        Some((signed as i32) as Int)
    }

    /// Scans the next floating-point token matching `RAMFile::scanReal` (`RAMFile.cpp:431-460`).
    pub fn scan_real(&mut self) -> Option<Real> {
        let data = self.data.as_deref()?;
        while self.pos < data.len()
            && !data[self.pos].is_ascii_digit()
            && data[self.pos] != b'-'
            && data[self.pos] != b'.'
        {
            self.pos += 1;
        }
        if self.pos >= data.len() {
            self.pos = data.len();
            return None;
        }
        let start = self.pos;
        let mut saw_dec = false;
        loop {
            if data[self.pos] == b'.' {
                saw_dec = true;
            }
            self.pos += 1;
            if self.pos >= data.len() {
                break;
            }
            let b = data[self.pos];
            if !(b.is_ascii_digit() || (b == b'.' && !saw_dec)) {
                break;
            }
        }
        let token = core::str::from_utf8(&data[start..self.pos]).unwrap_or("");
        if !token.bytes().any(|b| b.is_ascii_digit()) {
            return Some(0.0_f32);
        }
        Some(IniReader::scan_real(token).unwrap_or(0.0_f32))
    }

    /// Scans the next whitespace-delimited token matching `RAMFile::scanString` (`RAMFile.cpp:465-484`).
    pub fn scan_string(&mut self) -> Option<String> {
        let data = self.data.as_deref()?;
        while self.pos < data.len() && is_ascii_c_space(data[self.pos]) {
            self.pos += 1;
        }
        if self.pos >= data.len() {
            self.pos = data.len();
            return None;
        }
        let start = self.pos;
        self.pos += 1;
        while self.pos < data.len() && !is_ascii_c_space(data[self.pos]) {
            self.pos += 1;
        }
        Some(String::from_utf8_lossy(&data[start..self.pos]).into_owned())
    }

    /// Reads up to the next `'\n'` matching `RAMFile::nextLine` (`RAMFile.cpp:489-519`).
    pub fn next_line(&mut self, buf_size: Int) -> Vec<u8> {
        let Some(data) = self.data.as_deref() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut i: Int = 0;
        while self.pos < data.len() && data[self.pos] != b'\n' {
            if i < buf_size.saturating_sub(1) {
                out.push(data[self.pos]);
                i += 1;
            }
            self.pos += 1;
        }
        if self.pos < data.len() {
            if i < buf_size {
                out.push(data[self.pos]);
            }
            self.pos += 1;
        }
        out
    }
}
