//! `PKT-038`: Deterministic sorted key-value dictionary (`Core/GameEngine/Source/Common/Dict.cpp`).
//!
//! Reference C++ packet: `packets/phase2/PKT-038_Dict.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt038.rs` (`docs/golden_vectors/PKT-038.json`)

#![allow(unused_variables, dead_code)]

use crate::types::{Bool, Int, NameKeyType, Real, FALSE, TRUE};

pub const DICT_MAX_LEN: usize = 32767;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DictDataType {
    #[default]
    None = -1,
    Bool = 0,
    Int = 1,
    Real = 2,
    AsciiString = 3,
    UnicodeString = 4,
}

impl DictDataType {
    #[inline]
    pub const fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Bool,
            1 => Self::Int,
            2 => Self::Real,
            3 => Self::AsciiString,
            4 => Self::UnicodeString,
            _ => Self::None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DictValue {
    Bool(Bool),
    Int(Int),
    Real(Real),
    AsciiString(String),
    UnicodeString(Vec<u16>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dict {
    pairs: Vec<(NameKeyType, DictValue)>,
}

impl Dict {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn with_capacity(num_pairs_to_preallocate: Int) -> Self {
        let cap = if num_pairs_to_preallocate > 0 {
            num_pairs_to_preallocate as usize
        } else {
            0
        };
        Self {
            pairs: Vec::with_capacity(cap),
        }
    }

    pub fn clear(&mut self) {
        todo!("PKT-038: implement Dict::clear")
    }

    pub fn get_pair_count(&self) -> Int {
        todo!("PKT-038: implement Dict::getPairCount")
    }

    pub fn get_nth_key(&self, n: Int) -> NameKeyType {
        todo!("PKT-038: implement Dict::getNthKey")
    }

    pub fn get_type(&self, key: NameKeyType) -> DictDataType {
        todo!("PKT-038: implement Dict::getType")
    }

    pub fn known(&self, key: NameKeyType, d: DictDataType) -> Bool {
        let _ = (key, d, FALSE, TRUE);
        todo!("PKT-038: implement Dict::known")
    }

    pub fn get_nth_type(&self, n: Int) -> DictDataType {
        todo!("PKT-038: implement Dict::getNthType")
    }

    pub fn get_bool(&self, key: NameKeyType, exists: Option<&mut Bool>) -> Bool {
        let _ = (key, exists);
        todo!("PKT-038: implement Dict::getBool")
    }

    pub fn get_int(&self, key: NameKeyType, exists: Option<&mut Bool>) -> Int {
        let _ = (key, exists);
        todo!("PKT-038: implement Dict::getInt")
    }

    pub fn get_real(&self, key: NameKeyType, exists: Option<&mut Bool>) -> Real {
        let _ = (key, exists);
        todo!("PKT-038: implement Dict::getReal")
    }

    pub fn get_ascii_string(&self, key: NameKeyType, exists: Option<&mut Bool>) -> String {
        let _ = (key, exists);
        todo!("PKT-038: implement Dict::getAsciiString")
    }

    pub fn get_unicode_string(&self, key: NameKeyType, exists: Option<&mut Bool>) -> Vec<u16> {
        let _ = (key, exists);
        todo!("PKT-038: implement Dict::getUnicodeString")
    }

    pub fn get_nth_bool(&self, n: Int) -> Bool {
        todo!("PKT-038: implement Dict::getNthBool")
    }

    pub fn get_nth_int(&self, n: Int) -> Int {
        todo!("PKT-038: implement Dict::getNthInt")
    }

    pub fn get_nth_real(&self, n: Int) -> Real {
        todo!("PKT-038: implement Dict::getNthReal")
    }

    pub fn get_nth_ascii_string(&self, n: Int) -> String {
        todo!("PKT-038: implement Dict::getNthAsciiString")
    }

    pub fn get_nth_unicode_string(&self, n: Int) -> Vec<u16> {
        todo!("PKT-038: implement Dict::getNthUnicodeString")
    }

    pub fn set_bool(&mut self, key: NameKeyType, value: Bool) {
        let _ = (key, value);
        todo!("PKT-038: implement Dict::setBool")
    }

    pub fn set_int(&mut self, key: NameKeyType, value: Int) {
        let _ = (key, value);
        todo!("PKT-038: implement Dict::setInt")
    }

    pub fn set_real(&mut self, key: NameKeyType, value: Real) {
        let _ = (key, value);
        todo!("PKT-038: implement Dict::setReal")
    }

    pub fn set_ascii_string(&mut self, key: NameKeyType, value: &str) {
        let _ = (key, value);
        todo!("PKT-038: implement Dict::setAsciiString")
    }

    pub fn set_unicode_string(&mut self, key: NameKeyType, value: &[u16]) {
        let _ = (key, value);
        todo!("PKT-038: implement Dict::setUnicodeString")
    }

    pub fn remove(&mut self, key: NameKeyType) -> Bool {
        let _ = key;
        todo!("PKT-038: implement Dict::remove")
    }

    pub fn copy_pair_from(&mut self, that: &Dict, key: NameKeyType) {
        let _ = (that, key);
        todo!("PKT-038: implement Dict::copyPairFrom")
    }
}
