//! `PKT-454`: Abstract file handle, normalized open flags, and cross-platform path ordering
//! (`Common/file.h`, `Core/GameEngine/Source/Common/System/file.cpp`, and path helpers in
//! `Common/FileSystem.h` / `Lib/PathUtil.h`).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::types::{Bool, Int, FALSE, TRUE};

pub type Int64 = i64;

/// Access mode flags matching `File::access`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FileAccess;

impl FileAccess {
    pub const NONE: Int = 0x0000_0000;
    pub const READ: Int = 0x0000_0001;
    pub const WRITE: Int = 0x0000_0002;
    pub const READWRITE: Int = Self::READ | Self::WRITE;
    pub const APPEND: Int = 0x0000_0004;
    pub const CREATE: Int = 0x0000_0008;
    pub const TRUNCATE: Int = 0x0000_0010;
    pub const TEXT: Int = 0x0000_0020;
    pub const BINARY: Int = 0x0000_0040;
    pub const ONLYNEW: Int = 0x0000_0080;
    pub const STREAMING: Int = 0x0000_0100;
    pub const LINEBUF: Int = 0x0000_0200;
    pub const FULLBUF: Int = 0x0000_0400;
}

/// Seek origin matching `File::seekMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SeekMode {
    Start = 0,
    Current = 1,
    End = 2,
}

impl SeekMode {
    pub const fn from_i32(raw: Int) -> Option<Self> {
        match raw {
            0 => Some(Self::Start),
            1 => Some(Self::Current),
            2 => Some(Self::End),
            _ => None,
        }
    }
}

/// Normalizes `File::open` access flags matching `File::open` (`file.cpp:135-174`).
/// Returns `None` if both `STREAMING|WRITE` or `TEXT|BINARY` are set.
pub fn normalize_open_access(mut access: Int) -> Option<Int> {
    if (access & (FileAccess::STREAMING | FileAccess::WRITE))
        == (FileAccess::STREAMING | FileAccess::WRITE)
    {
        return None;
    }
    if (access & (FileAccess::TEXT | FileAccess::BINARY))
        == (FileAccess::TEXT | FileAccess::BINARY)
    {
        return None;
    }
    if (access & (FileAccess::READ | FileAccess::WRITE)) == 0 {
        access |= FileAccess::READ;
    }
    if (access & (FileAccess::READ | FileAccess::APPEND)) == 0 {
        access |= FileAccess::TRUNCATE;
    }
    if (access & (FileAccess::TEXT | FileAccess::BINARY)) == 0 {
        access |= FileAccess::BINARY;
    }
    Some(access)
}

/// File metadata matching `struct FileInfo` (`FileSystem.h:118-135`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FileInfo {
    pub size_high: Int,
    pub size_low: Int,
    pub timestamp_high: Int,
    pub timestamp_low: Int,
}

impl FileInfo {
    #[inline]
    pub const fn size(&self) -> Int64 {
        ((self.size_high as Int64) << 32) | ((self.size_low as u32) as Int64)
    }

    #[inline]
    pub const fn timestamp(&self) -> Int64 {
        ((self.timestamp_high as Int64) << 32) | ((self.timestamp_low as u32) as Int64)
    }
}

/// Shared base state matching `class File` (`file.cpp:101-209`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBase {
    name: String,
    access: Int,
    open: bool,
    delete_on_close: bool,
}

impl Default for FileBase {
    fn default() -> Self {
        Self::new()
    }
}

impl FileBase {
    pub fn new() -> Self {
        Self {
            name: "<no file>".to_string(),
            access: FileAccess::NONE,
            open: false,
            delete_on_close: false,
        }
    }

    pub fn open(&mut self, filename: &str, access: Int) -> Bool {
        if self.open {
            return FALSE;
        }
        let Some(norm) = normalize_open_access(access) else {
            return FALSE;
        };
        self.name = filename.to_string();
        self.access = norm;
        self.open = true;
        TRUE
    }

    pub fn close(&mut self) {
        if self.open {
            self.name = "<no file>".to_string();
            self.open = false;
        }
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        self.open
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    #[inline]
    pub fn access(&self) -> Int {
        self.access
    }

    #[inline]
    pub fn delete_on_close(&mut self) {
        self.delete_on_close = true;
    }

    #[inline]
    pub fn should_delete_on_close(&self) -> bool {
        self.delete_on_close
    }
}

/// Returns true if `ch` is `'/'` or `'\\'` (`PathUtil.h:194-197`).
#[inline]
pub const fn is_path_separator(ch: u8) -> bool {
    ch == b'/' || ch == b'\\'
}

/// Returns the filename slice after the last `'/'` or `'\\'`, matching `getFileName`
/// (`PathUtil.h:250-254`).
pub fn get_file_name(path: &str) -> &str {
    let bytes = path.as_bytes();
    let mut last_sep = None;
    for (idx, &b) in bytes.iter().enumerate() {
        if is_path_separator(b) {
            last_sep = Some(idx);
        }
    }
    match last_sep {
        Some(idx) => &path[idx + 1..],
        None => path,
    }
}

/// Returns the extension starting at `'.'`, ignoring dots in parent directories
/// (`PathUtil.h:257-268`).
pub fn get_extension(path: &str) -> Option<&str> {
    let bytes = path.as_bytes();
    let mut last_dot = None;
    for (idx, &b) in bytes.iter().enumerate() {
        if b == b'.' {
            last_dot = Some(idx);
        } else if is_path_separator(b) {
            last_dot = None;
        }
    }
    last_dot.map(|idx| &path[idx..])
}

#[inline]
const fn to_path_key_byte(b: u8) -> u8 {
    if is_path_separator(b) {
        b'\\'
    } else if b >= b'A' && b <= b'Z' {
        b + (b'a' - b'A')
    } else {
        b
    }
}

/// Converts a path to its canonical key form (`FileSystem::makePathKey`), using `'\\'`
/// as separator, collapsing consecutive separators (matching Unix/macOS C++ `getPathKeyPairEnd`
/// behavior), and lowercasing ASCII letters.
///
/// `buf_cap` includes the C string null terminator, matching `keySize` in C++.
pub fn make_path_key(path: &str, buf_cap: usize) -> Option<String> {
    if buf_cap == 0 {
        return None;
    }
    let bytes = path.as_bytes();
    let max_chars = buf_cap - 1;
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let ch = to_path_key_byte(bytes[i]);
        if ch == 0 {
            break;
        }
        if out.len() == max_chars {
            return None;
        }
        out.push(ch as char);
        i += 1;
        if ch == b'\\' {
            while i < bytes.len() && is_path_separator(bytes[i]) {
                i += 1;
            }
        }
    }
    Some(out)
}

/// Compares two paths canonically matching `FileSystem::comparePath` (`FileSystem.cpp:566-590`).
pub fn compare_path(a: &str, b: &str) -> Ordering {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let mut ia = 0;
    let mut ib = 0;

    loop {
        let ca = if ia < ab.len() {
            to_path_key_byte(ab[ia])
        } else {
            0
        };
        let cb = if ib < bb.len() {
            to_path_key_byte(bb[ib])
        } else {
            0
        };
        if ca != cb {
            return ca.cmp(&cb);
        }
        if ca == 0 {
            return Ordering::Equal;
        }
        ia += 1;
        ib += 1;
        if ca == b'\\' {
            while ia < ab.len() && is_path_separator(ab[ia]) {
                ia += 1;
            }
            while ib < bb.len() && is_path_separator(bb[ib]) {
                ib += 1;
            }
        }
    }
}

/// Case- and separator-insensitive path wrapper matching `std::set<AsciiString, FilenameLess>`.
#[derive(Debug, Clone)]
pub struct FilenameKey(pub String);

impl FilenameKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq for FilenameKey {
    fn eq(&self, other: &Self) -> bool {
        compare_path(&self.0, &other.0) == Ordering::Equal
    }
}

impl Eq for FilenameKey {}

impl PartialOrd for FilenameKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FilenameKey {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_path(&self.0, &other.0)
    }
}

/// Local filesystem interface required by `ArchiveFile` and `BIGFileSystem`.
pub trait LocalFileHost {
    fn read_file_bytes(&self, path: &str) -> Option<(Vec<u8>, FileInfo)>;
    fn get_file_info(&self, path: &str) -> Option<FileInfo>;
    fn does_file_exist(&self, path: &str) -> bool {
        self.get_file_info(path).is_some()
    }
    fn get_file_list_in_directory(
        &self,
        current_dir: &str,
        search_name: &str,
        search_subdirs: bool,
    ) -> BTreeSet<FilenameKey>;
}

/// Case-insensitive wildcard matcher matching `ArchiveFile.cpp`'s directory glob scan (`*`, `?`).
pub fn wildcard_match_nocase(text: &str, pattern: &str) -> bool {
    fn inner(t: &[u8], p: &[u8]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        if p[0] == b'*' {
            let mut rest = &p[1..];
            while rest.first() == Some(&b'*') {
                rest = &rest[1..];
            }
            if rest.is_empty() {
                return true;
            }
            for idx in 0..=t.len() {
                if inner(&t[idx..], rest) {
                    return true;
                }
            }
            return false;
        }
        if t.is_empty() {
            return false;
        }
        if p[0] == b'?' || p[0].to_ascii_lowercase() == t[0].to_ascii_lowercase() {
            inner(&t[1..], &p[1..])
        } else {
            false
        }
    }
    inner(text.as_bytes(), pattern.as_bytes())
}

#[derive(Debug, Clone)]
struct MemoryFileRecord {
    bytes: Vec<u8>,
    info: FileInfo,
}

/// Deterministic in-memory [`LocalFileHost`] used by tests and self-contained headless harnesses.
#[derive(Debug, Clone, Default)]
pub struct MemoryFileHost {
    files: BTreeMap<String, MemoryFileRecord>,
}

impl MemoryFileHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normalize_key(path: &str) -> String {
        let mut out = String::with_capacity(path.len());
        for &b in path.as_bytes() {
            if b == b'\\' {
                out.push('/');
            } else {
                out.push((b as char).to_ascii_lowercase());
            }
        }
        out
    }

    pub fn add_file(
        &mut self,
        path: &str,
        bytes: Vec<u8>,
        timestamp_high: Int,
        timestamp_low: Int,
    ) {
        let info = FileInfo {
            size_high: 0,
            size_low: bytes.len() as Int,
            timestamp_high,
            timestamp_low,
        };
        self.files
            .insert(Self::normalize_key(path), MemoryFileRecord { bytes, info });
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }
}

impl LocalFileHost for MemoryFileHost {
    fn read_file_bytes(&self, path: &str) -> Option<(Vec<u8>, FileInfo)> {
        let rec = self.files.get(&Self::normalize_key(path))?;
        Some((rec.bytes.clone(), rec.info))
    }

    fn get_file_info(&self, path: &str) -> Option<FileInfo> {
        self.files
            .get(&Self::normalize_key(path))
            .map(|r| r.info)
    }

    fn get_file_list_in_directory(
        &self,
        current_dir: &str,
        search_name: &str,
        search_subdirs: bool,
    ) -> BTreeSet<FilenameKey> {
        let mut dir = Self::normalize_key(current_dir);
        while dir.ends_with('/') {
            dir.pop();
        }
        let pat = if search_name.is_empty() {
            "*"
        } else {
            search_name
        };

        let mut out = BTreeSet::new();
        for full in self.files.keys() {
            let rel = if dir.is_empty() {
                full.as_str()
            } else if full.len() > dir.len()
                && full.starts_with(&dir)
                && full.as_bytes()[dir.len()] == b'/'
            {
                &full[dir.len() + 1..]
            } else {
                continue;
            };
            if !search_subdirs && rel.contains('/') {
                continue;
            }
            let leaf = match rel.rfind('/') {
                Some(pos) => &rel[pos + 1..],
                None => rel,
            };
            if wildcard_match_nocase(leaf, pat) {
                out.insert(FilenameKey(full.clone()));
            }
        }
        out
    }
}

/// Safe `std::fs`-backed [`LocalFileHost`] normalizing `'\\'` separators for POSIX and Windows hosts.
#[derive(Debug, Clone, Default)]
pub struct OsDiskFileHost;

impl OsDiskFileHost {
    fn normalize_fs_path(path: &str) -> String {
        path.replace('\\', "/")
    }

    fn resolve_case_insensitive_on_disk(path: &Path) -> Option<std::path::PathBuf> {
        if path.exists() {
            return Some(path.to_path_buf());
        }
        let mut acc = std::path::PathBuf::new();
        for comp in path.components() {
            match comp {
                std::path::Component::Prefix(p) => acc.push(p.as_os_str()),
                std::path::Component::RootDir => acc.push(std::path::MAIN_SEPARATOR_STR),
                std::path::Component::CurDir => acc.push("."),
                std::path::Component::ParentDir => acc.push(".."),
                std::path::Component::Normal(seg) => {
                    let direct = acc.join(seg);
                    if direct.exists() {
                        acc = direct;
                        continue;
                    }
                    let target = seg.to_string_lossy();
                    let read_dir = if acc.as_os_str().is_empty() {
                        std::fs::read_dir(".")
                    } else {
                        std::fs::read_dir(&acc)
                    }
                    .ok()?;
                    let mut matched = None;
                    for entry in read_dir.flatten() {
                        if entry
                            .file_name()
                            .to_string_lossy()
                            .eq_ignore_ascii_case(&target)
                        {
                            matched = Some(entry.path());
                            break;
                        }
                    }
                    acc = matched?;
                }
            }
        }
        Some(acc)
    }

    fn scan_dir_recursive(
        dir: &Path,
        pattern: &str,
        recursive: bool,
        out: &mut BTreeSet<FilenameKey>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut sorted: Vec<_> = entries.flatten().collect();
        sorted.sort_by_key(|e| e.file_name().to_string_lossy().to_ascii_lowercase());
        for entry in sorted {
            let path = entry.path();
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_dir() {
                if recursive {
                    Self::scan_dir_recursive(&path, pattern, true, out);
                }
            } else if ft.is_file() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if wildcard_match_nocase(&name, pattern) {
                    out.insert(FilenameKey(path.to_string_lossy().replace('\\', "/")));
                }
            }
        }
    }
}

impl LocalFileHost for OsDiskFileHost {
    fn read_file_bytes(&self, path: &str) -> Option<(Vec<u8>, FileInfo)> {
        let norm = Self::normalize_fs_path(path);
        let resolved = Self::resolve_case_insensitive_on_disk(Path::new(&norm))?;
        let meta = std::fs::metadata(&resolved).ok()?;
        let bytes = std::fs::read(&resolved).ok()?;
        let len = meta.len();
        let info = FileInfo {
            size_high: ((len >> 32) & 0xffff_ffff) as Int,
            size_low: (len & 0xffff_ffff) as Int,
            timestamp_high: 0,
            timestamp_low: 0,
        };
        Some((bytes, info))
    }

    fn get_file_info(&self, path: &str) -> Option<FileInfo> {
        let norm = Self::normalize_fs_path(path);
        let resolved = Self::resolve_case_insensitive_on_disk(Path::new(&norm))?;
        let meta = std::fs::metadata(&resolved).ok()?;
        let len = meta.len();
        Some(FileInfo {
            size_high: ((len >> 32) & 0xffff_ffff) as Int,
            size_low: (len & 0xffff_ffff) as Int,
            timestamp_high: 0,
            timestamp_low: 0,
        })
    }

    fn get_file_list_in_directory(
        &self,
        current_dir: &str,
        search_name: &str,
        search_subdirs: bool,
    ) -> BTreeSet<FilenameKey> {
        let mut out = BTreeSet::new();
        let dir_str = if current_dir.is_empty() {
            ".".to_string()
        } else {
            Self::normalize_fs_path(current_dir)
        };
        let pat = if search_name.is_empty() {
            "*"
        } else {
            search_name
        };
        if let Some(resolved) = Self::resolve_case_insensitive_on_disk(Path::new(&dir_str)) {
            Self::scan_dir_recursive(&resolved, pat, search_subdirs, &mut out);
        }
        out
    }
}
