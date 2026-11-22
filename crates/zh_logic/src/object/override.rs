//! `PKT-395`: Slot/key-based `OVERRIDE<T>` handle (`GeneralsMD/Code/GameEngine/Include/Common/Override.h`).

use super::overridable::OverridableEntry;

/// Read/write store lookup adapter resolving an [`Override<K>`] key to its [`OverridableEntry<T>`].
pub trait OverrideStore<K, T> {
    fn get_overridable(&self, key: K) -> Option<&OverridableEntry<T>>;
    fn get_overridable_mut(&mut self, key: K) -> Option<&mut OverridableEntry<T>>;
}

/// Deterministic key wrapper mirroring C++ `OVERRIDE<T>` without raw pointers or `Rc`/`RefCell`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Override<K> {
    key: Option<K>,
}

impl<K: Copy> Override<K> {
    #[inline]
    pub const fn null() -> Self {
        Self { key: None }
    }

    #[inline]
    pub const fn new(key: Option<K>) -> Self {
        Self { key }
    }

    #[inline]
    pub const fn from_key(key: K) -> Self {
        Self { key: Some(key) }
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        self.key.is_none()
    }

    /// Returns the base key without resolving overrides (`getNonOverloadedPointer()`).
    #[inline]
    pub const fn get_non_overloaded_key(&self) -> Option<K> {
        self.key
    }

    /// Returns the un-overridden base entry from `store`.
    #[inline]
    pub fn get_non_overloaded<'a, T, S: OverrideStore<K, T> + ?Sized>(
        &self,
        store: &'a S,
    ) -> Option<&'a T> {
        self.key
            .and_then(|k| store.get_overridable(k))
            .map(OverridableEntry::get_base)
    }

    /// Resolves through `get_final_override()` (`operator->` / `operator*`).
    #[inline]
    pub fn get_final<'a, T, S: OverrideStore<K, T> + ?Sized>(
        &self,
        store: &'a S,
    ) -> Option<&'a T> {
        self.key
            .and_then(|k| store.get_overridable(k))
            .map(OverridableEntry::get_final_override)
    }

    #[inline]
    pub fn get_final_mut<'a, T, S: OverrideStore<K, T> + ?Sized>(
        &self,
        store: &'a mut S,
    ) -> Option<&'a mut T> {
        self.key
            .and_then(|k| store.get_overridable_mut(k))
            .map(OverridableEntry::get_final_override_mut)
    }
}
