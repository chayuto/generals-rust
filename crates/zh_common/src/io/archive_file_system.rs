//! `PKT-449`: Global archive VFS overlay, precedence ordering, and mod override filtering
//! (`Core/GameEngine/Include/Common/ArchiveFileSystem.h` &
//! `Core/GameEngine/Source/Common/System/ArchiveFileSystem.cpp`).

use std::collections::{BTreeMap, BTreeSet};

use crate::io::archive_file::ArchiveFile;
use crate::io::file::{get_file_name, FileInfo, FilenameKey};
use crate::io::ram_file::RamFile;
use crate::types::{Bool, UnsignedByte, FALSE, TRUE};

pub type FileInstance = UnsignedByte;

/// Archive lookup filter matching `ArchiveFileSystem::ArchiveFilter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum ArchiveFilter {
    #[default]
    All = 0,
    ModOnly = 1,
    NonModOnly = 2,
}

impl ArchiveFilter {
    pub const fn from_i32(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::All),
            1 => Some(Self::ModOnly),
            2 => Some(Self::NonModOnly),
            _ => None,
        }
    }
}

/// Global mod/loose-file configuration injected into [`ArchiveFileSystem`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArchiveModConfig {
    pub mod_big: String,
    pub mod_dir: String,
    pub loose_files_win: bool,
}

/// Overlay directory node mapping filenames to ordered archive keys (`ArchiveFileSystem`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArchivedDirectoryInfo {
    pub path: String,
    pub directory_name: String,
    pub directories: BTreeMap<String, ArchivedDirectoryInfo>,
    /// Ordered list of archive keys in `ArchiveFileSystem::archives` providing this file.
    /// Instance `0` is at index `0` (highest precedence).
    pub files: BTreeMap<String, Vec<String>>,
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

/// Deterministic archive filesystem merging multiple [`ArchiveFile`] instances (`TheArchiveFileSystem`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArchiveFileSystem {
    archives: BTreeMap<String, ArchiveFile>,
    root_directory: ArchivedDirectoryInfo,
    config: ArchiveModConfig,
}

impl ArchiveFileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: ArchiveModConfig) -> Self {
        Self {
            archives: BTreeMap::new(),
            root_directory: ArchivedDirectoryInfo::default(),
            config,
        }
    }

    pub fn config(&self) -> &ArchiveModConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: ArchiveModConfig) {
        self.config = config;
    }

    pub fn contains_archive_key(&self, archive_key: &str) -> bool {
        self.archives.contains_key(archive_key)
    }

    pub fn get_archive(&self, archive_key: &str) -> Option<&ArchiveFile> {
        self.archives.get(archive_key)
    }

    pub fn remove_archive(&mut self, archive_key: &str) -> Option<ArchiveFile> {
        self.archives.remove(archive_key)
    }

    pub fn clear(&mut self) {
        self.archives.clear();
        self.root_directory = ArchivedDirectoryInfo::default();
    }

    /// Registers `archive` under `archive_key` and indexes its entries into the overlay directory tree.
    pub fn insert_archive(
        &mut self,
        archive_key: impl Into<String>,
        archive: ArchiveFile,
        overwrite: bool,
    ) -> bool {
        let key = archive_key.into();
        let mut filenames = BTreeSet::new();
        archive.get_file_list_in_directory("", "*", &mut filenames);
        self.archives.insert(key.clone(), archive);
        self.index_archive_filenames(&key, &filenames, overwrite);
        true
    }

    fn index_archive_filenames(
        &mut self,
        archive_key: &str,
        filenames: &BTreeSet<FilenameKey>,
        overwrite: bool,
    ) {
        for entry in filenames {
            let lower = entry.as_str().to_ascii_lowercase();
            let mut dir = &mut self.root_directory;
            let mut built_path = String::new();
            let mut rest = lower.as_str();
            let mut leaf = "";
            while let Some((tok, next_rest)) = next_path_token(rest) {
                if next_rest.is_empty() {
                    leaf = tok;
                    break;
                }
                built_path.push_str(tok);
                built_path.push('\\');
                dir = dir
                    .directories
                    .entry(tok.to_string())
                    .or_insert_with(|| ArchivedDirectoryInfo {
                        path: built_path.clone(),
                        directory_name: tok.to_string(),
                        ..ArchivedDirectoryInfo::default()
                    });
                rest = next_rest;
            }
            if leaf.is_empty() || leaf.contains('*') || leaf.contains('?') {
                continue;
            }
            let list = dir.files.entry(leaf.to_string()).or_default();
            if overwrite {
                list.insert(0, archive_key.to_string());
            } else {
                list.push(archive_key.to_string());
            }
        }
    }

    fn find_file_archives(&self, filename: &str) -> Option<&[String]> {
        if filename.is_empty() {
            return None;
        }
        let lower = filename.to_ascii_lowercase();
        let mut dir = &self.root_directory;
        let mut tok_opt = next_path_token(&lower);
        while let Some((tok, rest)) = tok_opt {
            if rest.is_empty() {
                return dir.files.get(tok).map(Vec::as_slice);
            }
            dir = dir.directories.get(tok)?;
            tok_opt = next_path_token(rest);
        }
        None
    }

    fn resolve_archive(
        &self,
        filename: &str,
        instance: FileInstance,
        filter: ArchiveFilter,
    ) -> Option<&ArchiveFile> {
        let keys = self.find_file_archives(filename)?;
        let mut current: FileInstance = 0;
        for key in keys {
            let Some(archive) = self.archives.get(key) else {
                continue;
            };
            let is_mod = self.is_mod_archive(archive, Some(filename));
            match filter {
                ArchiveFilter::All => {}
                ArchiveFilter::ModOnly if !is_mod => continue,
                ArchiveFilter::NonModOnly if is_mod => continue,
                _ => {}
            }
            if current == instance {
                return Some(archive);
            }
            current = current.saturating_add(1);
        }
        None
    }

    pub fn does_file_exist(
        &self,
        filename: &str,
        instance: FileInstance,
        filter: ArchiveFilter,
    ) -> Bool {
        if self.resolve_archive(filename, instance, filter).is_some() {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn open_file(
        &self,
        filename: &str,
        instance: FileInstance,
        filter: ArchiveFilter,
    ) -> Option<RamFile> {
        self.resolve_archive(filename, instance, filter)?
            .open_file(filename)
    }

    pub fn get_file_info(
        &self,
        filename: &str,
        instance: FileInstance,
        filter: ArchiveFilter,
    ) -> Option<FileInfo> {
        self.resolve_archive(filename, instance, filter)?
            .get_file_info(filename)
    }

    pub fn get_file_list_in_directory(
        &self,
        original_directory: &str,
        search_name: &str,
    ) -> BTreeSet<FilenameKey> {
        let mut out = BTreeSet::new();
        for archive in self.archives.values() {
            archive.get_file_list_in_directory(original_directory, search_name, &mut out);
        }
        out
    }

    /// Stock loose scripts shipped outside `.big` archives (`ArchiveFileSystem::isStockLooseFile`).
    pub fn is_stock_loose_file(filename: &str) -> Bool {
        if filename.is_empty() {
            return FALSE;
        }
        let base = get_file_name(filename);
        if base.eq_ignore_ascii_case("SkirmishScripts.scb")
            || base.eq_ignore_ascii_case("MultiplayerScripts.scb")
            || base.eq_ignore_ascii_case("Scripts.ini")
        {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_mod_archive(&self, archive: &ArchiveFile, filename: Option<&str>) -> bool {
        if let Some(fname) = filename {
            if Self::is_stock_loose_file(fname) == TRUE {
                return true;
            }
        }
        let archive_name = archive.name();
        let base = get_file_name(archive_name);
        if base.starts_with('!') || base.starts_with('@') {
            return true;
        }
        if !self.config.mod_dir.is_empty() {
            if let Some(pos) = archive_name.find(&self.config.mod_dir) {
                let after = pos + self.config.mod_dir.len();
                let next_byte = archive_name.as_bytes().get(after).copied();
                if matches!(next_byte, None | Some(b'/') | Some(b'\\')) {
                    return true;
                }
            }
        }
        if !self.config.mod_big.is_empty() {
            if archive_name.eq_ignore_ascii_case(&self.config.mod_big) {
                return true;
            }
            let mod_big_base = get_file_name(&self.config.mod_big);
            if base.eq_ignore_ascii_case(mod_big_base) {
                return true;
            }
        }
        false
    }

    pub fn get_file_count(&self, filename: &str, filter: ArchiveFilter) -> FileInstance {
        let Some(keys) = self.find_file_archives(filename) else {
            return 0;
        };
        if filter == ArchiveFilter::All {
            return keys.len() as FileInstance;
        }
        let mut count: FileInstance = 0;
        for key in keys {
            let Some(archive) = self.archives.get(key) else {
                continue;
            };
            let is_mod = self.is_mod_archive(archive, Some(filename));
            match filter {
                ArchiveFilter::ModOnly if is_mod => count = count.saturating_add(1),
                ArchiveFilter::NonModOnly if !is_mod => count = count.saturating_add(1),
                _ => {}
            }
        }
        count
    }

    pub fn has_mod_archive_override(&self, filename: &str) -> Bool {
        if self.config.loose_files_win {
            return FALSE;
        }
        if Self::is_stock_loose_file(filename) == FALSE {
            return FALSE;
        }
        if self.get_file_count(filename, ArchiveFilter::ModOnly) > 0 {
            TRUE
        } else {
            FALSE
        }
    }
}
