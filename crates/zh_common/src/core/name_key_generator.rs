//! `PKT-064`: Deterministic `NameKeyGenerator` 45,007-bucket string interner translation target.
//!
//! Reference C++ packet: `packets/phase2/PKT-064_NameKeyGenerator.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt064.rs` (`docs/golden_vectors/PKT-064.json`)

#![allow(unused_variables, dead_code)]

use crate::types::{NameKeyType, UnsignedInt};

pub const SOCKET_COUNT: usize = 45007;
pub const NAMEKEY_MAX: i32 = 1 << 23;

pub fn calc_hash_for_string(s: &str) -> UnsignedInt {
    let mut result: u32 = 0;
    for &byte in s.as_bytes() {
        if byte == 0 {
            break;
        }
        result = result
            .wrapping_shl(5)
            .wrapping_add(result)
            .wrapping_add((byte as i8) as u32);
    }
    result
}

pub fn calc_hash_for_lowercase_string(s: &str) -> UnsignedInt {
    let mut result: u32 = 0;
    for &byte in s.as_bytes() {
        if byte == 0 {
            break;
        }
        let lowercase = byte.to_ascii_lowercase();
        result = result
            .wrapping_shl(5)
            .wrapping_add(result)
            .wrapping_add(lowercase as u32);
    }
    result
}

#[derive(Debug, Clone)]
struct Bucket {
    key: NameKeyType,
    name: String,
}

#[derive(Debug, Clone)]
pub struct NameKeyGenerator {
    next_id: UnsignedInt,
    sockets: Vec<Vec<Bucket>>,
}

impl Default for NameKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl NameKeyGenerator {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            sockets: vec![Vec::new(); SOCKET_COUNT],
        }
    }

    pub fn init(&mut self) {
        self.reset();
    }

    pub fn reset(&mut self) {
        for bucket in &mut self.sockets {
            bucket.clear();
        }
        self.next_id = 1;
    }

    pub fn name_to_key(&mut self, name: &str) -> NameKeyType {
        let hash = (calc_hash_for_string(name) % SOCKET_COUNT as u32) as usize;
        if let Some(bucket) = self.sockets[hash].iter().find(|bucket| bucket.name == name) {
            return bucket.key;
        }
        self.create_name_key(hash, name.to_owned())
    }

    pub fn name_to_lowercase_key(&mut self, name: &str) -> NameKeyType {
        let hash = (calc_hash_for_lowercase_string(name) % SOCKET_COUNT as u32) as usize;
        if let Some(bucket) = self
            .sockets[hash]
            .iter()
            .find(|bucket| name.eq_ignore_ascii_case(&bucket.name))
        {
            return bucket.key;
        }
        let lowercase_name = name.to_ascii_lowercase();
        self.create_name_key(hash, lowercase_name)
    }

    pub fn key_to_name(&self, key: NameKeyType) -> &str {
        for bucket_list in &self.sockets {
            if let Some(bucket) = bucket_list.iter().find(|bucket| bucket.key == key) {
                return &bucket.name;
            }
        }
        ""
    }

    fn create_name_key(&mut self, hash: usize, name: String) -> NameKeyType {
        let key = NameKeyType(self.next_id as i32);
        self.next_id = self.next_id.wrapping_add(1);
        self.sockets[hash].push(Bucket { key, name });
        key
    }

    #[cfg(feature = "retail-crc")]
    pub fn sync_name_key_id(&mut self) {
        self.name_to_lowercase_key("Data\\English\\Language9x.ini");
        self.name_to_lowercase_key("Data\\Audio\\Tracks\\English\\GLA_02.mp3");
        self.name_to_lowercase_key("Data\\Audio\\Tracks\\GLA_02.mp3");
    }
}
