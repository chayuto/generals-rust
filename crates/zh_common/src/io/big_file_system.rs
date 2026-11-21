//! `PKT-450`: EA `.big` (`BIGF`) archive binary parser, precedence loader, and mod loader
//! (`Core/GameEngine/Include/Common/BIGFileSystem.h` &
//! `Core/GameEngine/Source/Common/System/BIGFileSystem.cpp`).

use std::collections::BTreeSet;

use crate::io::archive_file::{ArchiveFile, ArchivedFileInfo};
use crate::io::archive_file_system::{
    ArchiveFileSystem, ArchiveFilter, ArchiveModConfig, FileInstance,
};
use crate::io::file::{get_file_name, FileInfo, FilenameKey, LocalFileHost};
use crate::io::ram_file::RamFile;
use crate::types::{Bool, UnsignedInt, FALSE, TRUE};

pub const BIG_FILE_IDENTIFIER: [u8; 4] = *b"BIGF";
pub const BIG_HEADER_SIZE: usize = 16;
pub const BIG_ENTRY_HEADER_SIZE: usize = 8;
pub const BIG_MAX_PATH: usize = 260;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BigFileHeader {
    pub archive_file_size: UnsignedInt,
    pub num_files: UnsignedInt,
    pub first_entry_offset: UnsignedInt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BigFileEntryHeader {
    pub file_offset: UnsignedInt,
    pub file_size: UnsignedInt,
}

fn read_u32_le(bytes: &[u8]) -> UnsignedInt {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn read_u32_be(bytes: &[u8]) -> UnsignedInt {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn read_null_terminated_bounded(bytes: &[u8], max_len: usize) -> Option<(String, usize)> {
    let limit = bytes.len().min(max_len);
    for (idx, &b) in bytes[..limit].iter().enumerate() {
        if b == 0 {
            let text = String::from_utf8_lossy(&bytes[..idx]).into_owned();
            return Some((text, idx + 1));
        }
    }
    None
}

fn ends_with_ignore_case(s: &str, suffix: &str) -> bool {
    if s.len() < suffix.len() {
        return false;
    }
    s[s.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    if s.len() < prefix.len() {
        return false;
    }
    s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

pub fn is_patch_archive_filename(archive_path: &str) -> bool {
    starts_with_ignore_case(get_file_name(archive_path), "Patch")
}

pub fn is_generalsx_archive_filename(archive_path: &str) -> bool {
    starts_with_ignore_case(get_file_name(archive_path), "generalsx")
}

pub fn is_duplicate_inizh_path(path: &str) -> bool {
    ends_with_ignore_case(path, "Data\\INI\\INIZH.big")
        || ends_with_ignore_case(path, "Data/INI/INIZH.big")
}

pub fn is_nested_zh_generals_subpath(dir: &str, full_path: &str) -> bool {
    if full_path.len() <= dir.len() {
        return false;
    }
    let mut sub = &full_path[dir.len()..];
    while sub.starts_with('/') || sub.starts_with('\\') {
        sub = &sub[1..];
    }
    if starts_with_ignore_case(sub, "ZH_Generals/") || starts_with_ignore_case(sub, "ZH_Generals\\")
    {
        return true;
    }
    sub.contains("/ZH_Generals/")
        || sub.contains("\\ZH_Generals\\")
        || sub.contains("/zh_generals/")
        || sub.contains("\\zh_generals\\")
}

/// Pure-Rust EA `.big` virtual filesystem (`BIGFileSystem`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BigFileSystem {
    vfs: ArchiveFileSystem,
}

impl BigFileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: ArchiveModConfig) -> Self {
        Self {
            vfs: ArchiveFileSystem::with_config(config),
        }
    }

    pub fn vfs(&self) -> &ArchiveFileSystem {
        &self.vfs
    }

    pub fn vfs_mut(&mut self) -> &mut ArchiveFileSystem {
        &mut self.vfs
    }

    pub fn set_config(&mut self, config: ArchiveModConfig) {
        self.vfs.set_config(config);
    }

    /// Parses one raw `.big` byte buffer into an [`ArchiveFile`] (`BIGFileSystem::openArchiveFile`).
    pub fn open_archive_bytes(
        filename: &str,
        bytes: Vec<u8>,
        file_info: FileInfo,
    ) -> Option<ArchiveFile> {
        if bytes.len() < BIG_HEADER_SIZE || bytes[..4] != BIG_FILE_IDENTIFIER {
            return None;
        }
        let _header = BigFileHeader {
            archive_file_size: read_u32_le(&bytes[4..8]),
            num_files: read_u32_be(&bytes[8..12]),
            first_entry_offset: read_u32_be(&bytes[12..16]),
        };

        let archive_file_name = filename.to_ascii_lowercase();
        let mut archive = ArchiveFile::new(filename, "");
        let mut cursor = BIG_HEADER_SIZE;

        for _ in 0.._header.num_files {
            if cursor + BIG_ENTRY_HEADER_SIZE > bytes.len() {
                return None;
            }
            let entry = BigFileEntryHeader {
                file_offset: read_u32_be(&bytes[cursor..cursor + 4]),
                file_size: read_u32_be(&bytes[cursor + 4..cursor + 8]),
            };
            cursor += BIG_ENTRY_HEADER_SIZE;

            let (raw_path, consumed) = read_null_terminated_bounded(&bytes[cursor..], BIG_MAX_PATH)?;
            cursor += consumed;

            let leaf = get_file_name(&raw_path).to_ascii_lowercase();
            if leaf.is_empty() || leaf.contains('*') || leaf.contains('?') {
                continue;
            }

            let split_idx = raw_path.len() - leaf.len();
            let dir_path = &raw_path[..split_idx];

            archive.add_file(
                dir_path,
                ArchivedFileInfo {
                    filename: leaf,
                    archive_filename: archive_file_name.clone(),
                    offset: entry.file_offset,
                    size: entry.file_size,
                },
            );
        }

        archive.attach_bytes(bytes, file_info);
        Some(archive)
    }

    pub fn open_archive_file(
        &self,
        host: &dyn LocalFileHost,
        filename: &str,
    ) -> Option<ArchiveFile> {
        let (bytes, info) = host.read_file_bytes(filename)?;
        Self::open_archive_bytes(filename, bytes, info)
    }

    pub fn close_archive_file(&mut self, filename: &str) {
        self.vfs.remove_archive(filename);
    }

    pub fn load_one_archive_file(
        &mut self,
        host: &dyn LocalFileHost,
        archive_path: &str,
        overwrite: bool,
    ) -> bool {
        if self.vfs.contains_archive_key(archive_path) {
            return false;
        }
        let Some(archive) = self.open_archive_file(host, archive_path) else {
            return false;
        };
        self.vfs.insert_archive(archive_path, archive, overwrite)
    }

    /// Scans `dir` matching `file_mask` and mounts base, `Patch*`, and `generalsx*` archives
    /// matching `BIGFileSystem::loadArchiveFilesFromDirectory` (`BIGFileSystem.cpp:900-989`).
    pub fn load_archive_files_from_directory(
        &mut self,
        host: &dyn LocalFileHost,
        dir: &str,
        file_mask: &str,
        overwrite: bool,
        allow_patch_precedence: bool,
    ) -> Bool {
        let found = host.get_file_list_in_directory(dir, file_mask, true);

        let mut patch_list = BTreeSet::new();
        let mut generalsx_list = BTreeSet::new();
        let mut normal_list = BTreeSet::new();
        let mut actually_added = false;

        for path_key in &found {
            let path = path_key.as_str();
            if is_duplicate_inizh_path(path) {
                continue;
            }
            if allow_patch_precedence && is_nested_zh_generals_subpath(dir, path) {
                continue;
            }
            if allow_patch_precedence && is_generalsx_archive_filename(path) {
                generalsx_list.insert(FilenameKey(path.to_string()));
            } else if is_patch_archive_filename(path) {
                patch_list.insert(FilenameKey(path.to_string()));
            } else if !allow_patch_precedence && !overwrite {
                normal_list.insert(FilenameKey(path.to_string()));
            } else if self.load_one_archive_file(host, path, overwrite) {
                actually_added = true;
            }
        }

        for p in &patch_list {
            let patch_ovr = if allow_patch_precedence { true } else { overwrite };
            if self.load_one_archive_file(host, p.as_str(), patch_ovr) {
                actually_added = true;
            }
        }

        if !allow_patch_precedence && !overwrite {
            for n in &normal_list {
                if self.load_one_archive_file(host, n.as_str(), false) {
                    actually_added = true;
                }
            }
        }

        if actually_added && allow_patch_precedence {
            for gx in &generalsx_list {
                let _ = self.load_one_archive_file(host, gx.as_str(), true);
            }
        }

        if actually_added {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn load_mods(&mut self, host: &dyn LocalFileHost) {
        let mod_big = self.vfs.config().mod_big.clone();
        if !mod_big.is_empty() {
            if let Some(archive) = self.open_archive_file(host, &mod_big) {
                let _ = self.vfs.insert_archive(mod_big, archive, true);
            }
        }
        let mod_dir = self.vfs.config().mod_dir.clone();
        if !mod_dir.is_empty() {
            let _ = self.load_archive_files_from_directory(host, &mod_dir, "*.big", true, false);
        }
    }

    pub fn does_file_exist(
        &self,
        filename: &str,
        instance: FileInstance,
        filter: ArchiveFilter,
    ) -> Bool {
        self.vfs.does_file_exist(filename, instance, filter)
    }

    pub fn open_file(
        &self,
        filename: &str,
        instance: FileInstance,
        filter: ArchiveFilter,
    ) -> Option<RamFile> {
        self.vfs.open_file(filename, instance, filter)
    }

    pub fn get_file_info(
        &self,
        filename: &str,
        instance: FileInstance,
        filter: ArchiveFilter,
    ) -> Option<FileInfo> {
        self.vfs.get_file_info(filename, instance, filter)
    }

    pub fn get_file_count(&self, filename: &str, filter: ArchiveFilter) -> FileInstance {
        self.vfs.get_file_count(filename, filter)
    }

    pub fn has_mod_archive_override(&self, filename: &str) -> Bool {
        self.vfs.has_mod_archive_override(filename)
    }

    pub fn get_file_list_in_directory(
        &self,
        original_directory: &str,
        search_name: &str,
    ) -> BTreeSet<FilenameKey> {
        self.vfs
            .get_file_list_in_directory(original_directory, search_name)
    }
}
