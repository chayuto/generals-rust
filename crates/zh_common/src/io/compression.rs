//! `WP-7.2b`: EA legacy compression codecs behind `CompressionManager`
//! (`Core/Libraries/Source/Compression/`): RefPack (`"EAR\0"`, `REF_*`), BTree (`"EAB\0"`,
//! `BTREE_*`) and Huffman (`"EAH\0"`, `HUFF_*`).
//!
//! Reference C++ packet: `packets/phase7/WP7_2b_Compression_Bundle.md`
//! Golden Vector test: `crates/zh_common/tests/golden_wp7_2b_compression.rs`
//! (`docs/golden_vectors/WP7_2b_Compression.json`)
//!
//! Contract (the packet's "Port Contract" section has the details):
//! - A compressed blob is the 4-byte magic, the uncompressed length as a little-endian `i32`,
//!   then the codec stream. Codec stream headers (`10fb`, `46fb`, `30fb`) and their size
//!   fields are always big-endian, the portable `ggetm`/`gputm` path. Never port the
//!   `__APPLE__` native-endian branch in `gimex.h`.
//! - Encoders implement only the default mode (`opts == nullptr`), the only mode
//!   `CompressionManager` uses.
//! - Decoders never panic and always terminate on any input. The Huffman decoder may read up
//!   to 4 bytes past the end of `stream`, as `0x00`; any further read is
//!   [`CodecError::Truncated`].
//!
//! Every `todo!()` below is part of the `WP-7.2b` translation task.

#![allow(unused_variables)]

/// Length of the `CompressionManager` blob header: 4-byte magic + little-endian `i32` length.
pub const COMPRESSION_HEADER_LEN: usize = 8;

/// `CompressionType` from `Compression.h`. Discriminants match the C++ enum.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompressionType {
    None = 0,
    RefPack = 1,
    NoxLzh = 2,
    Zlib1 = 3,
    Zlib2 = 4,
    Zlib3 = 5,
    Zlib4 = 6,
    Zlib5 = 7,
    Zlib6 = 8,
    Zlib7 = 9,
    Zlib8 = 10,
    Zlib9 = 11,
    BTree = 12,
    Huff = 13,
}

/// Decode failure. The C++ codecs have no error reporting (they read and write out of bounds
/// instead), so these are Rust-side outcomes for input that has no reference behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    /// The input ended before the stream did, beyond the allowed look-ahead.
    Truncated,
    /// The manager or codec stream header is not one this codec accepts.
    BadHeader,
    /// A recognised header for a variant this port does not implement: Huffman delta or
    /// accelerated streams, `NOXLZH`, or zlib.
    Unsupported,
    /// A back-reference points before the start of the output.
    BadReference,
    /// The decoded length differs from the length the header declares.
    LengthMismatch,
}

/// `CompressionManager::getCompressionType`: identifies a blob by its 4-byte magic.
/// Inputs shorter than [`COMPRESSION_HEADER_LEN`] are [`CompressionType::None`].
pub fn get_compression_type(mem: &[u8]) -> CompressionType {
    todo!("WP-7.2b")
}

/// `CompressionManager::isDataCompressed`.
pub fn is_data_compressed(mem: &[u8]) -> bool {
    todo!("WP-7.2b")
}

/// `CompressionManager::getUncompressedSize`: the header length for any compressed type,
/// otherwise `mem.len()`.
pub fn get_uncompressed_size(mem: &[u8]) -> i32 {
    todo!("WP-7.2b")
}

/// `CompressionManager::compressData` for `RefPack`, `BTree` and `Huff`. Returns the full blob
/// (header + stream). Returns `None` for every other type, and wherever the C++ returns 0.
pub fn compress_data(comp_type: CompressionType, src: &[u8]) -> Option<Vec<u8>> {
    todo!("WP-7.2b")
}

/// `CompressionManager::decompressData` for `RefPack`, `BTree` and `Huff` blobs. Other
/// recognised types return [`CodecError::Unsupported`].
pub fn decompress_data(src: &[u8]) -> Result<Vec<u8>, CodecError> {
    todo!("WP-7.2b")
}

/// RefPack (`refencode.cpp`, `refdecode.cpp`). `stream` is the codec stream after the
/// manager header, starting with the `10fb` / `90fb` type word.
pub mod refpack {
    use super::CodecError;

    /// `REF_is`.
    pub fn is(stream: &[u8]) -> bool {
        todo!("WP-7.2b")
    }

    /// `REF_size`: the uncompressed length from the stream header.
    pub fn size(stream: &[u8]) -> i32 {
        todo!("WP-7.2b")
    }

    /// `REF_encode` with `opts == nullptr` (`maxback = 131072`, `quick = 0`).
    pub fn encode(src: &[u8]) -> Vec<u8> {
        todo!("WP-7.2b")
    }

    /// `REF_decode`.
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        todo!("WP-7.2b")
    }
}

/// BTree (`btreeencode.cpp`, `btreedecode.cpp`). `stream` starts with the `46fb` type word.
pub mod btree {
    use super::CodecError;

    /// `BTREE_is`.
    pub fn is(stream: &[u8]) -> bool {
        todo!("WP-7.2b")
    }

    /// `BTREE_size`.
    pub fn size(stream: &[u8]) -> i32 {
        todo!("WP-7.2b")
    }

    /// `BTREE_encode` with `opts == nullptr`.
    pub fn encode(src: &[u8]) -> Vec<u8> {
        todo!("WP-7.2b")
    }

    /// `BTREE_decode`.
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        todo!("WP-7.2b")
    }
}

/// Huffman (`huffencode.cpp`, `huffdecode.cpp`). `stream` starts with the `30fb` type word.
pub mod huff {
    use super::CodecError;

    /// `HUFF_is`.
    pub fn is(stream: &[u8]) -> bool {
        todo!("WP-7.2b")
    }

    /// `HUFF_size`.
    pub fn size(stream: &[u8]) -> i32 {
        todo!("WP-7.2b")
    }

    /// `HUFF_encode` with `opts == nullptr` (no delta pre-pass).
    pub fn encode(src: &[u8]) -> Vec<u8> {
        todo!("WP-7.2b")
    }

    /// `HUFF_decode` for plain `30fb`/`31fb`/`b0fb`/`b1fb` streams. Delta (`32fb`, `33fb`) and
    /// accelerated (`34fb`, `35fb`) variants return [`CodecError::Unsupported`].
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        todo!("WP-7.2b")
    }
}
