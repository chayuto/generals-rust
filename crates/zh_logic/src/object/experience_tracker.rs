//! `PKT-003`: `ExperienceTracker` (`GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ExperienceTracker.cpp`).
//!
//! Target module for `WP-1.2b` (`PKT-003`).

use zh_common::types::{
    Bool, Int, ObjectId, Real, Relationship, VeterancyLevel, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError};

/// Decoupled parent object & template access trait (`Pattern 2: Handle Component`).
/// Implemented by `FfiExperienceParent` in `crates/zh_ffi` and by `MockExperienceParent` in pure-Rust tests.
pub trait ExperienceParentAccess {
    fn is_template_trainable(&self) -> Bool;
    fn get_experience_required(&self, level: Int) -> Int;
    fn get_experience_value(&self, level: Int) -> Int;
    fn get_relationship_with_killer(&self) -> Relationship;
    fn on_veterancy_level_changed(
        &mut self,
        old_level: VeterancyLevel,
        new_level: VeterancyLevel,
        provide_feedback: Bool,
    );
    /// Forwards `scaled_exp_gain` (`experienceGain * m_experienceScalar`) to `sink_id` if the sink object exists in `TheGameLogic`.
    /// Returns `true` if `sinkPointer != nullptr` (and therefore forwarded), or `false` if the sink object was not found.
    fn forward_experience_to_sink(
        &mut self,
        sink_id: ObjectId,
        scaled_exp_gain: Int,
        can_scale_for_bonus: Bool,
    ) -> bool;
    /// Forwards `setExperienceAndLevel(experience_in, provide_feedback)` to `sink_id` if the sink object exists.
    /// Returns `true` if `sinkPointer != nullptr`, or `false` if the sink object was not found.
    fn forward_set_exp_and_level_to_sink(
        &mut self,
        sink_id: ObjectId,
        experience_in: Int,
        provide_feedback: Bool,
    ) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExperienceTrackerState {
    pub current_level: VeterancyLevel,
    pub current_experience: Int,
    pub experience_sink: ObjectId,
    pub experience_scalar: Real,
    pub is_trainable: Bool,
}

impl Default for ExperienceTrackerState {
    fn default() -> Self {
        Self {
            current_level: VeterancyLevel::Regular,
            current_experience: 0,
            experience_sink: ObjectId::INVALID,
            experience_scalar: 1.0,
            is_trainable: FALSE,
        }
    }
}

impl ExperienceTrackerState {
    pub fn new(parent: &dyn ExperienceParentAccess) -> Self {
        let mut s = Self::default();
        s.reset_trainable(parent);
        s
    }

    #[inline]
    pub const fn get_veterancy_level(&self) -> VeterancyLevel {
        self.current_level
    }

    #[inline]
    pub const fn get_current_experience(&self) -> Int {
        self.current_experience
    }

    #[inline]
    pub const fn is_trainable(&self) -> Bool {
        self.is_trainable
    }

    #[inline]
    pub fn set_trainable(&mut self, trainable: Bool) {
        self.is_trainable = trainable;
    }

    pub fn reset_trainable(&mut self, _parent: &dyn ExperienceParentAccess) {
        // Pending PKT-003 translation in WP-1.2b
    }

    pub fn is_accepting_experience_points(&self) -> Bool {
        // Pending PKT-003 translation in WP-1.2b
        FALSE
    }

    #[inline]
    pub fn set_experience_sink(&mut self, sink: ObjectId) {
        self.experience_sink = sink;
    }

    #[inline]
    pub const fn get_experience_sink(&self) -> ObjectId {
        self.experience_sink
    }

    #[inline]
    pub const fn get_experience_scalar(&self) -> Real {
        self.experience_scalar
    }

    #[inline]
    pub fn set_experience_scalar(&mut self, scalar: Real) {
        self.experience_scalar = scalar;
    }

    pub fn get_experience_value(&self, _parent: &dyn ExperienceParentAccess) -> Int {
        // Pending PKT-003 translation in WP-1.2b
        0
    }

    pub fn set_min_veterancy_level(
        &mut self,
        _parent: &mut dyn ExperienceParentAccess,
        _new_level: VeterancyLevel,
        _provide_feedback: Bool,
    ) {
        // Pending PKT-003 translation in WP-1.2b
    }

    pub fn set_veterancy_level(
        &mut self,
        _parent: &mut dyn ExperienceParentAccess,
        _new_level: VeterancyLevel,
        _provide_feedback: Bool,
    ) {
        // Pending PKT-003 translation in WP-1.2b
    }

    pub fn gain_exp_for_level(
        &mut self,
        _parent: &mut dyn ExperienceParentAccess,
        _levels_to_gain: Int,
        _can_scale_for_bonus: Bool,
    ) -> Bool {
        // Pending PKT-003 translation in WP-1.2b
        FALSE
    }

    pub fn can_gain_exp_for_level(&self, _levels_to_gain: Int) -> Bool {
        // Pending PKT-003 translation in WP-1.2b
        FALSE
    }

    pub fn add_experience_points(
        &mut self,
        _parent: &mut dyn ExperienceParentAccess,
        _experience_gain: Int,
        _can_scale_for_bonus: Bool,
    ) {
        // Pending PKT-003 translation in WP-1.2b
    }

    pub fn set_experience_and_level(
        &mut self,
        _parent: &mut dyn ExperienceParentAccess,
        _experience_in: Int,
        _provide_feedback: Bool,
    ) {
        // Pending PKT-003 translation in WP-1.2b
    }
}

impl Snapshot for ExperienceTrackerState {
    fn crc(&self, _xfer: &mut dyn Xfer) {
        // Pending PKT-003 translation in WP-1.2b
    }

    fn xfer(&mut self, _xfer: &mut dyn Xfer) -> Result<(), XferError> {
        // Pending PKT-003 translation in WP-1.2b
        let _ = TRUE;
        Ok(())
    }

    fn load_post_process(&mut self) {}
}
