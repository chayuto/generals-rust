//! Readers for the retail asset files (stage 2 of `docs/FULL_PORT_PLAN.md`):
//! W3D models, TGA and DDS textures and CSF text. Files come as bytes, usually
//! from `zh_common::io::BigFileSystem`.

pub mod chunk;
pub mod csf;
pub mod dds;
pub mod model;
pub mod tga;
pub mod w3d;
pub mod wwmath;
