//! `PKT-394`: Value-owned overridable chain container (`GeneralsMD/Code/GameEngine/Include/Common/Overridable.h`).

/// Deterministic, value-owned overridable definition chain matching `Overridable`
/// (`map.ini` scenario overrides layered over base INI definitions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverridableEntry<T> {
    base: T,
    base_is_override: bool,
    overrides: Vec<T>,
}

impl<T> OverridableEntry<T> {
    #[inline]
    pub const fn new(base: T) -> Self {
        Self {
            base,
            base_is_override: false,
            overrides: Vec::new(),
        }
    }

    #[inline]
    pub const fn new_override(base: T) -> Self {
        Self {
            base,
            base_is_override: true,
            overrides: Vec::new(),
        }
    }

    #[inline]
    pub const fn is_override(&self) -> bool {
        self.base_is_override
    }

    #[inline]
    pub fn mark_as_override(&mut self) {
        self.base_is_override = true;
    }

    #[inline]
    pub fn override_count(&self) -> usize {
        self.overrides.len()
    }

    #[inline]
    pub const fn get_base(&self) -> &T {
        &self.base
    }

    #[inline]
    pub fn get_base_mut(&mut self) -> &mut T {
        &mut self.base
    }

    #[inline]
    pub fn get_next_override(&self) -> Option<&T> {
        self.overrides.first()
    }

    /// Returns the most recent override, or the base entry if no overrides were attached.
    #[inline]
    pub fn get_final_override(&self) -> &T {
        self.overrides.last().unwrap_or(&self.base)
    }

    #[inline]
    pub fn get_final_override_mut(&mut self) -> &mut T {
        self.overrides.last_mut().unwrap_or(&mut self.base)
    }

    /// Appends a new override to the chain tail and returns a mutable reference to it.
    #[inline]
    pub fn append_override(&mut self, value: T) -> &mut T {
        self.overrides.push(value);
        self.overrides.last_mut().expect("pushed override element")
    }

    /// Removes scenario overrides matching `Overridable::deleteOverrides()`.
    ///
    /// Returns `false` if this entry's root was marked as an override (map-only template) so the
    /// owning store can drop it; otherwise clears `overrides` and returns `true`.
    #[inline]
    pub fn delete_overrides(&mut self) -> bool {
        self.overrides.clear();
        !self.base_is_override
    }
}
