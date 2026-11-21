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

impl DictValue {
    #[inline]
    fn data_type(&self) -> DictDataType {
        match self {
            Self::Bool(_) => DictDataType::Bool,
            Self::Int(_) => DictDataType::Int,
            Self::Real(_) => DictDataType::Real,
            Self::AsciiString(_) => DictDataType::AsciiString,
            Self::UnicodeString(_) => DictDataType::UnicodeString,
        }
    }
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
            (num_pairs_to_preallocate as usize).min(DICT_MAX_LEN)
        } else {
            0
        };
        Self {
            pairs: Vec::with_capacity(cap),
        }
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    pub fn get_pair_count(&self) -> Int {
        self.pairs.len().min(DICT_MAX_LEN) as Int
    }

    pub fn get_nth_key(&self, n: Int) -> NameKeyType {
        if n < 0 {
            return NameKeyType::INVALID;
        }
        match self.pairs.get(n as usize) {
            Some((key, _)) => *key,
            None => NameKeyType::INVALID,
        }
    }

    pub fn get_type(&self, key: NameKeyType) -> DictDataType {
        match self.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => self.pairs[index].1.data_type(),
            Err(_) => DictDataType::None,
        }
    }

    pub fn known(&self, key: NameKeyType, d: DictDataType) -> Bool {
        if d == DictDataType::None {
            return FALSE;
        }
        if self.get_type(key) == d {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn get_nth_type(&self, n: Int) -> DictDataType {
        if n < 0 {
            return DictDataType::None;
        }
        match self.pairs.get(n as usize) {
            Some((_, value)) => value.data_type(),
            None => DictDataType::None,
        }
    }

    pub fn get_bool(&self, key: NameKeyType, exists: Option<&mut Bool>) -> Bool {
        match self.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => match &self.pairs[index].1 {
                DictValue::Bool(value) => {
                    if let Some(exists) = exists {
                        *exists = TRUE;
                    }
                    *value
                }
                _ => {
                    if let Some(exists) = exists {
                        *exists = FALSE;
                    }
                    FALSE
                }
            },
            Err(_) => {
                if let Some(exists) = exists {
                    *exists = FALSE;
                }
                FALSE
            }
        }
    }

    pub fn get_int(&self, key: NameKeyType, exists: Option<&mut Bool>) -> Int {
        match self.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => match &self.pairs[index].1 {
                DictValue::Int(value) => {
                    if let Some(exists) = exists {
                        *exists = TRUE;
                    }
                    *value
                }
                _ => {
                    if let Some(exists) = exists {
                        *exists = FALSE;
                    }
                    0
                }
            },
            Err(_) => {
                if let Some(exists) = exists {
                    *exists = FALSE;
                }
                0
            }
        }
    }

    pub fn get_real(&self, key: NameKeyType, exists: Option<&mut Bool>) -> Real {
        match self.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => match &self.pairs[index].1 {
                DictValue::Real(value) => {
                    if let Some(exists) = exists {
                        *exists = TRUE;
                    }
                    *value
                }
                _ => {
                    if let Some(exists) = exists {
                        *exists = FALSE;
                    }
                    0.0
                }
            },
            Err(_) => {
                if let Some(exists) = exists {
                    *exists = FALSE;
                }
                0.0
            }
        }
    }

    pub fn get_ascii_string(&self, key: NameKeyType, exists: Option<&mut Bool>) -> String {
        match self.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => match &self.pairs[index].1 {
                DictValue::AsciiString(value) => {
                    if let Some(exists) = exists {
                        *exists = TRUE;
                    }
                    value.clone()
                }
                _ => {
                    if let Some(exists) = exists {
                        *exists = FALSE;
                    }
                    String::new()
                }
            },
            Err(_) => {
                if let Some(exists) = exists {
                    *exists = FALSE;
                }
                String::new()
            }
        }
    }

    pub fn get_unicode_string(&self, key: NameKeyType, exists: Option<&mut Bool>) -> Vec<u16> {
        match self.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => match &self.pairs[index].1 {
                DictValue::UnicodeString(value) => {
                    if let Some(exists) = exists {
                        *exists = TRUE;
                    }
                    value.clone()
                }
                _ => {
                    if let Some(exists) = exists {
                        *exists = FALSE;
                    }
                    Vec::new()
                }
            },
            Err(_) => {
                if let Some(exists) = exists {
                    *exists = FALSE;
                }
                Vec::new()
            }
        }
    }

    pub fn get_nth_bool(&self, n: Int) -> Bool {
        if n < 0 {
            return FALSE;
        }
        match self.pairs.get(n as usize) {
            Some((_, DictValue::Bool(value))) => *value,
            _ => FALSE,
        }
    }

    pub fn get_nth_int(&self, n: Int) -> Int {
        if n < 0 {
            return 0;
        }
        match self.pairs.get(n as usize) {
            Some((_, DictValue::Int(value))) => *value,
            _ => 0,
        }
    }

    pub fn get_nth_real(&self, n: Int) -> Real {
        if n < 0 {
            return 0.0;
        }
        match self.pairs.get(n as usize) {
            Some((_, DictValue::Real(value))) => *value,
            _ => 0.0,
        }
    }

    pub fn get_nth_ascii_string(&self, n: Int) -> String {
        if n < 0 {
            return String::new();
        }
        match self.pairs.get(n as usize) {
            Some((_, DictValue::AsciiString(value))) => value.clone(),
            _ => String::new(),
        }
    }

    pub fn get_nth_unicode_string(&self, n: Int) -> Vec<u16> {
        if n < 0 {
            return Vec::new();
        }
        match self.pairs.get(n as usize) {
            Some((_, DictValue::UnicodeString(value))) => value.clone(),
            _ => Vec::new(),
        }
    }

    pub fn set_bool(&mut self, key: NameKeyType, value: Bool) {
        self.set_value(key, DictValue::Bool(value));
    }

    pub fn set_int(&mut self, key: NameKeyType, value: Int) {
        self.set_value(key, DictValue::Int(value));
    }

    pub fn set_real(&mut self, key: NameKeyType, value: Real) {
        self.set_value(key, DictValue::Real(value));
    }

    pub fn set_ascii_string(&mut self, key: NameKeyType, value: &str) {
        self.set_value(key, DictValue::AsciiString(value.to_owned()));
    }

    pub fn set_unicode_string(&mut self, key: NameKeyType, value: &[u16]) {
        self.set_value(key, DictValue::UnicodeString(value.to_vec()));
    }

    pub fn remove(&mut self, key: NameKeyType) -> Bool {
        match self.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => {
                self.pairs.remove(index);
                TRUE
            }
            Err(_) => FALSE,
        }
    }

    pub fn copy_pair_from(&mut self, that: &Dict, key: NameKeyType) {
        match that.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => {
                let value = that.pairs[index].1.clone();
                self.set_value(key, value);
            }
            Err(_) => {
                self.remove(key);
            }
        }
    }

    fn set_value(&mut self, key: NameKeyType, value: DictValue) {
        match self.pairs.binary_search_by_key(&key.0, |(k, _)| k.0) {
            Ok(index) => self.pairs[index].1 = value,
            Err(index) => self.pairs.insert(index, (key, value)),
        }
    }
}
