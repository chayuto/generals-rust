//! `WP-7.2`: Pure-Rust readers for retail data containers and virtual filesystem.
//!
//! - `compression` (`WP-7.2b`): the EA legacy codecs behind `CompressionManager`
//!   (RefPack `EAR`, BTree `EAB`, Huffman `EAH`).
//! - `data_chunk` (`PKT-451`): EA tagged chunk container reader/writer (`CkMp`)
//!   used by `.map`, `.wak`, and retail binary assets.
//! - `file` (`PKT-454`): `File` handle flags, seek modes, path comparison/keys, and host traits.
//! - `ram_file` (`PKT-460`): in-memory `RamFile` reader and token scanners.
//! - `archive_file` (`PKT-448`): indexed `.big` archive directory tree and extractor.
//! - `archive_file_system` (`PKT-449`): multi-archive overlay VFS and mod precedence filter.
//! - `big_file_system` (`PKT-450`): `BIGF` binary parser, patch/mod/base precedence loader.

pub mod archive_file;
pub mod archive_file_system;
pub mod big_file_system;
pub mod compression;
pub mod data_chunk;
pub mod file;
pub mod map_object;
pub mod map_reader_writer_info;
pub mod ram_file;
pub mod recorder;

pub use archive_file::{search_string_matches, ArchiveFile, ArchivedFileInfo};
pub use archive_file_system::{
    ArchiveFileSystem, ArchiveFilter, ArchiveModConfig, ArchivedDirectoryInfo, FileInstance,
};
pub use big_file_system::{
    is_duplicate_inizh_path, is_generalsx_archive_filename, is_nested_zh_generals_subpath,
    is_patch_archive_filename, BigFileEntryHeader, BigFileHeader, BigFileSystem,
};
pub use file::{
    compare_path, get_extension, get_file_name, is_path_separator, make_path_key,
    normalize_open_access, wildcard_match_nocase, FileAccess, FileBase, FileInfo, FilenameKey,
    LocalFileHost, MemoryFileHost, OsDiskFileHost, SeekMode,
};
pub use ram_file::RamFile;
