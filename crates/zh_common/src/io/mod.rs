//! `WP-7.2`: Pure-Rust readers for retail data containers.
//!
//! - `compression` (`WP-7.2b`): the EA legacy codecs behind `CompressionManager`
//!   (RefPack `EAR`, BTree `EAB`, Huffman `EAH`).
//! - `data_chunk` (`PKT-451`): EA tagged chunk container reader/writer (`CkMp`)
//!   used by `.map`, `.wak`, and retail binary assets.

pub mod compression;
pub mod data_chunk;
