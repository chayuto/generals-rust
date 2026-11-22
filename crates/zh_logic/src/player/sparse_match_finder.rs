//! `PKT-408`: Deterministic sparse bit-flag matcher (`Common/SparseMatchFinder.h`).
//!
//! Uses `BTreeMap` with `BitFlags` bit-index lexicographic ordering (matching C++ `MapHelper`)
//! instead of `std::hash_map`, guaranteeing bit-identical tie-breaking and lookup order.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use zh_common::types::{Bool, Int};
use zh_common::xfer::bit_flags::BitFlags;

/// Bit-set interface required by [`SparseMatchFinder`].
pub trait SparseBitSet: Clone + Eq + Ord {
    fn bit_size(&self) -> Int;
    fn bit_test(&self, index: Int) -> Bool;
    fn count_intersection(&self, other: &Self) -> Int;
    fn count_inverse_intersection(&self, other: &Self) -> Int;
}

impl<const NUMBITS: usize> SparseBitSet for BitFlags<NUMBITS> {
    #[inline]
    fn bit_size(&self) -> Int {
        self.size()
    }

    #[inline]
    fn bit_test(&self, index: Int) -> Bool {
        self.test(index)
    }

    #[inline]
    fn count_intersection(&self, other: &Self) -> Int {
        BitFlags::count_intersection(self, other)
    }

    #[inline]
    fn count_inverse_intersection(&self, other: &Self) -> Int {
        BitFlags::count_inverse_intersection(self, other)
    }
}

/// Trait implemented by condition-keyed template sets (`WeaponTemplateSet`, `ArmorTemplateSet`, etc.).
pub trait SparseMatchable<B> {
    fn get_conditions_yes_count(&self) -> Int;
    fn get_nth_conditions_yes(&self, i: Int) -> &B;
}

/// Deterministic memoizing sparse bitset matcher matching `SparseMatchFinder` (`SparseMatchFinder.h`).
#[derive(Debug, Default)]
pub struct SparseMatchFinder<M, B, const NO_COPY: bool = false>
where
    B: SparseBitSet,
{
    best_matches: BTreeMap<B, usize>,
    _marker: PhantomData<fn() -> M>,
}

impl<M, B, const NO_COPY: bool> SparseMatchFinder<M, B, NO_COPY>
where
    B: SparseBitSet,
{
    pub fn new() -> Self {
        Self {
            best_matches: BTreeMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.best_matches.clear();
    }
}

impl<M, B, const NO_COPY: bool> Clone for SparseMatchFinder<M, B, NO_COPY>
where
    B: SparseBitSet,
{
    fn clone(&self) -> Self {
        if NO_COPY {
            Self::new()
        } else {
            Self {
                best_matches: self.best_matches.clone(),
                _marker: PhantomData,
            }
        }
    }
}

impl<M, B, const NO_COPY: bool> PartialEq for SparseMatchFinder<M, B, NO_COPY>
where
    B: SparseBitSet,
{
    fn eq(&self, _other: &Self) -> bool {
        // Memoized cache does not participate in template value equality.
        true
    }
}

impl<M, B, const NO_COPY: bool> SparseMatchFinder<M, B, NO_COPY>
where
    M: SparseMatchable<B>,
    B: SparseBitSet,
{
    /// Scans `v` for the best match using the exact C++ `findBestInfoSlow` loop and tie-breaking.
    pub fn find_best_info_slow_index(v: &[M], bits: &B) -> Option<usize> {
        let mut best_idx: Option<usize> = None;
        let mut best_yes_match: Int = 0;
        let mut best_yes_extraneous_bits: Int = 999;

        for (idx, item) in v.iter().enumerate() {
            let yes_count = item.get_conditions_yes_count();
            for i in (0..yes_count).rev() {
                let yes_flags = item.get_nth_conditions_yes(i);
                let yes_match = bits.count_intersection(yes_flags);
                let yes_extraneous_bits = bits.count_inverse_intersection(yes_flags);
                if (yes_match > best_yes_match)
                    || (yes_match >= best_yes_match
                        && yes_extraneous_bits < best_yes_extraneous_bits)
                {
                    best_idx = Some(idx);
                    best_yes_match = yes_match;
                    best_yes_extraneous_bits = yes_extraneous_bits;
                }
            }
        }
        best_idx
    }

    /// Reads an existing cached match or computes it via `find_best_info_slow_index` without mutating `self`.
    pub fn find_best_info<'a>(&self, v: &'a [M], bits: &B) -> Option<&'a M> {
        if let Some(&cached) = self.best_matches.get(bits) {
            if let Some(m) = v.get(cached) {
                return Some(m);
            }
        }
        Self::find_best_info_slow_index(v, bits).and_then(|idx| v.get(idx))
    }

    /// Returns the index in `v` of the best matching entry, caching the lookup deterministically.
    pub fn find_best_info_index_mut(&mut self, v: &[M], bits: &B) -> Option<usize> {
        if let Some(&cached) = self.best_matches.get(bits) {
            if cached < v.len() {
                return Some(cached);
            }
        }
        let found = Self::find_best_info_slow_index(v, bits);
        if let Some(idx) = found {
            self.best_matches.insert(bits.clone(), idx);
        }
        found
    }

    /// Memoizing version of `findBestInfo`, caching the winning index in `self.best_matches`.
    pub fn find_best_info_mut<'a>(&mut self, v: &'a [M], bits: &B) -> Option<&'a M> {
        self.find_best_info_index_mut(v, bits).and_then(|idx| v.get(idx))
    }
}
