//! `PKT-448`: Single `.big` archive directory tree and sub-file extractor (`ArchiveFile`).
//! (`Core/GameEngine/Include/Common/ArchiveFile.h` & `Core/GameEngine/Source/Common/System/ArchiveFile.cpp`).

use std::collections::{BTreeMap, BTreeSet};

use crate::io::file::{FileInfo, FilenameKey};
use crate::io::ram_file::RamFile;
use crate::types::{Bool, UnsignedInt, FALSE, TRUE};

/// Sub-file location descriptor inside an open archive (`ArchivedFileInfo`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArchivedFileInfo {
    pub filename: String,
    pub archive_filename: String,
    pub offset: UnsignedInt,
    pub size: UnsignedInt,
}

/// Nested directory tree for one [`ArchiveFile`] (`DetailedArchivedDirectoryInfo`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DetailedArchivedDirectoryInfo {
    pub directory_name: String,
    pub directories: BTreeMap<String, DetailedArchivedDirectoryInfo>,
    pub files: BTreeMap<String, ArchivedFileInfo>,
}

/// Case-sensitive wildcard matcher matching `SearchStringMatches` (`ArchiveFile.cpp:44-86`).
pub fn search_string_matches(text: &str, pattern: &str) -> bool {
    fn inner(t: &[u8], p: &[u8]) -> bool {
        if t.is_empty() {
            return p.is_empty() || (p[0] == b'*' && p.iter().all(|&b| b == b'*'));
        }
        if p.is_empty() {
            return false;
        }
        let mut ti = 0usize;
        let mut pi = 0usize;
        while pi < p.len() {
            if ti >= t.len() {
                return p[pi..].iter().all(|&b| b == b'*');
            }
            if p[pi] == t[ti] || p[pi] == b'?' {
                ti += 1;
                pi += 1;
            } else if p[pi] == b'*' {
                pi += 1;
                if pi == p.len() {
                    return true;
                }
                while ti < t.len() {
                    if inner(&t[ti..], &p[pi..]) {
                        return true;
                    }
                    ti += 1;
                }
                return pi == p.len();
            } else {
                return false;
            }
        }
        ti == t.len() && pi == p.len()
    }
    inner(text.as_bytes(), pattern.as_bytes())
}

fn next_path_token(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    while start < bytes.len() && (bytes[start] == b'\\' || bytes[start] == b'/') {
        start += 1;
    }
    if start >= bytes.len() {
        return None;
    }
    let mut end = start;
    while end < bytes.len() && bytes[end] != b'\\' && bytes[end] != b'/' {
        end += 1;
    }
    Some((&s[start..end], &s[end..]))
}

/// In-memory directory and payload view for one mounted `.big` archive (`ArchiveFile`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArchiveFile {
    name: String,
    path: String,
    file_info: FileInfo,
    raw_bytes: Option<Vec<u8>>,
    root_directory: DetailedArchivedDirectoryInfo,
}

impl ArchiveFile {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            file_info: FileInfo::default(),
            raw_bytes: None,
            root_directory: DetailedArchivedDirectoryInfo::default(),
        }
    }

    pub fn attach_bytes(&mut self, bytes: Vec<u8>, file_info: FileInfo) {
        self.raw_bytes = Some(bytes);
        self.file_info = file_info;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn raw_bytes(&self) -> Option<&[u8]> {
        self.raw_bytes.as_deref()
    }

    pub fn close(&mut self) {
        self.raw_bytes = None;
    }

    /// Registers one sub-file entry into `m_rootDirectory` matching `ArchiveFile::addFile` (`ArchiveFile.cpp:103-127`).
    pub fn add_file(&mut self, path: &str, file_info: ArchivedFileInfo) {
        let mut dir = &mut self.root_directory;
        let lower = path.to_ascii_lowercase();
        let mut rest = lower.as_str();
        while let Some((tok, next_rest)) = next_path_token(rest) {
            dir = dir
                .directories
                .entry(tok.to_string())
                .or_insert_with(|| DetailedArchivedDirectoryInfo {
                    directory_name: tok.to_string(),
                    ..DetailedArchivedDirectoryInfo::default()
                });
            rest = next_rest;
        }
        dir.files.insert(file_info.filename.clone(), file_info);
    }

    pub fn get_archived_file_info(&self, filename: &str) -> Option<&ArchivedFileInfo> {
        let mut dir = &self.root_directory;
        let lower = filename.to_ascii_lowercase();
        let mut tok_opt = next_path_token(&lower);
        while let Some((tok, rest)) = tok_opt {
            if rest.is_empty() {
                return dir.files.get(tok);
            }
            // Also check C++ termination condition: `while (!tokenizer.isEmpty())` where trailing separators make `rest` non-empty but empty of tokens.
            dir = dir.directories.get(tok)?;
            tok_opt = next_path_token(rest);
        }
        None
    }

    pub fn get_file_info(&self, filename: &str) -> Option<FileInfo> {
        let archived = self.get_archived_file_info(filename)?;
        Some(FileInfo {
            size_high: 0,
            size_low: archived.size as i32,
            timestamp_high: self.file_info.timestamp_high,
            timestamp_low: self.file_info.timestamp_low,
        })
    }

    pub fn get_file_info_into(&self, filename: &str, out: &mut FileInfo) -> Bool {
        if let Some(fi) = self.get_file_info(filename) {
            *out = fi;
            TRUE
        } else {
            FALSE
        }
    }

    pub fn open_file(&self, filename: &str) -> Option<RamFile> {
        let archived = self.get_archived_file_info(filename)?;
        let bytes = self.raw_bytes.as_deref()?;
        let mut ram = RamFile::new();
        if ram.open_from_archive(
            Some(bytes),
            &archived.filename,
            archived.offset as i32,
            archived.size as i32,
        ) == FALSE
        {
            return None;
        }
        Some(ram)
    }

    /// Populates `out` matching `ArchiveFile::getFileListInDirectory` (`ArchiveFile.cpp:129-180`).
    pub fn get_file_list_in_directory(
        &self,
        original_directory: &str,
        search_name: &str,
        out: &mut BTreeSet<FilenameKey>,
    ) {
        let mut dir = &self.root_directory;
        let lower = original_directory.to_ascii_lowercase();
        let mut rest = lower.as_str();
        while let Some((tok, next_rest)) = next_path_token(rest) {
            let Some(child) = dir.directories.get(tok) else {
                return;
            };
            dir = child;
            rest = next_rest;
        }
        Self::collect_dir(dir, original_directory, search_name, out);
    }

    fn collect_dir(
        dir: &DetailedArchivedDirectoryInfo,
        current_directory: &str,
        search_name: &str,
        out: &mut BTreeSet<FilenameKey>,
    ) {
        for sub in dir.directories.values() {
            let mut next_dir = current_directory.to_string();
            if !next_dir.is_empty() && !next_dir.ends_with('\\') {
                next_dir.push('\\');
            }
            next_dir.push_str(&sub.directory_name);
            Self::collect_dir(sub, &next_dir, search_name, out);
        }
        for file_info in dir.files.values() {
            if search_string_matches(&file_info.filename, search_name) {
                let mut full = current_directory.to_string();
                if !full.is_empty() && !full.ends_with('\\') {
                    full.push('\\');
                }
                full.push_str(&file_info.filename);
                out.insert(FilenameKey(full));
            }
        }
    }
}
