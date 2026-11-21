//! `WP-7.2`: Pure-Rust readers for retail data containers.
//!
//! - `compression` (`WP-7.2b`): the EA legacy codecs behind `CompressionManager`
//!   (RefPack `EAR`, BTree `EAB`, Huffman `EAH`).
//!
//! `PKT-451` (`DataChunk`, the `.map` chunk container) will land here once it has its own
//! C++ golden-vector oracle.

pub mod compression;
