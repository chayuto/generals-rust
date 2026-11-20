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

    pub fn reset_trainable(&mut self, parent: &dyn ExperienceParentAccess) {
        self.is_trainable = parent.is_template_trainable();
    }

    pub fn is_accepting_experience_points(&self) -> Bool {
        if self.is_trainable() != FALSE || self.experience_sink != ObjectId::INVALID {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn get_experience_value(&self, parent: &dyn ExperienceParentAccess) -> Int {
        if parent.get_relationship_with_killer() == Relationship::Allies {
            0
        } else {
            parent.get_experience_value(self.current_level.as_i32())
        }
    }

    pub fn set_min_veterancy_level(
        &mut self,
        parent: &mut dyn ExperienceParentAccess,
        new_level: VeterancyLevel,
        provide_feedback: Bool,
    ) {
        if self.current_level < new_level {
            let old_level = self.current_level;
            self.current_level = new_level;
            self.current_experience = parent.get_experience_required(self.current_level.as_i32());
            parent.on_veterancy_level_changed(old_level, new_level, provide_feedback);
        }
    }

    pub fn set_veterancy_level(
        &mut self,
        parent: &mut dyn ExperienceParentAccess,
        new_level: VeterancyLevel,
        provide_feedback: Bool,
    ) {
        if self.current_level != new_level {
            let old_level = self.current_level;
            self.current_level = new_level;
            self.current_experience = parent.get_experience_required(self.current_level.as_i32());
            parent.on_veterancy_level_changed(old_level, new_level, provide_feedback);
        }
    }

    pub fn gain_exp_for_level(
        &mut self,
        parent: &mut dyn ExperienceParentAccess,
        levels_to_gain: Int,
        can_scale_for_bonus: Bool,
    ) -> Bool {
        let mut new_level = self.current_level.as_i32().wrapping_add(levels_to_gain);
        if new_level > VeterancyLevel::LAST {
            new_level = VeterancyLevel::LAST;
        }
        if new_level > self.current_level.as_i32() {
            let experience_needed = parent.get_experience_required(new_level).wrapping_sub(self.current_experience);
            self.add_experience_points(parent, experience_needed, can_scale_for_bonus);
            TRUE
        } else {
            FALSE
        }
    }

    pub fn can_gain_exp_for_level(&self, levels_to_gain: Int) -> Bool {
        let mut new_level = self.current_level.as_i32().wrapping_add(levels_to_gain);
        if new_level > VeterancyLevel::LAST {
            new_level = VeterancyLevel::LAST;
        }
        if new_level > self.current_level.as_i32() {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn add_experience_points(
        &mut self,
        parent: &mut dyn ExperienceParentAccess,
        experience_gain: Int,
        can_scale_for_bonus: Bool,
    ) {
        if self.experience_sink != ObjectId::INVALID {
            let scaled_for_sink = zh_common::math::cpp_mul_i32_f32(experience_gain, self.experience_scalar);
            if parent.forward_experience_to_sink(self.experience_sink, scaled_for_sink, can_scale_for_bonus) {
                return;
            }
        }

        if self.is_trainable() == FALSE {
            return;
        }

        let old_level = self.current_level;
        let mut amount_to_gain = experience_gain;
        if can_scale_for_bonus != FALSE {
            amount_to_gain = zh_common::math::cpp_mul_i32_f32(amount_to_gain, self.experience_scalar);
        }
        self.current_experience = self.current_experience.wrapping_add(amount_to_gain);

        let mut level_index: Int = 0;
        while (level_index.wrapping_add(1)) < VeterancyLevel::COUNT && self.current_experience >= parent.get_experience_required(level_index.wrapping_add(1)) {
            level_index = level_index.wrapping_add(1);
        }
        self.current_level = VeterancyLevel::from_i32(level_index);

        if old_level != self.current_level {
            parent.on_veterancy_level_changed(old_level, self.current_level, TRUE);
        }
    }

    pub fn set_experience_and_level(
        &mut self,
        parent: &mut dyn ExperienceParentAccess,
        experience_in: Int,
        provide_feedback: Bool,
    ) {
        if self.experience_sink != ObjectId::INVALID {
            if parent.forward_set_exp_and_level_to_sink(self.experience_sink, experience_in, provide_feedback) {
                return;
            }
        }

        if self.is_trainable() == FALSE {
            return;
        }

        let old_level = self.current_level;
        self.current_experience = experience_in;

        let mut level_index: Int = 0;
        while (level_index.wrapping_add(1)) < VeterancyLevel::COUNT && self.current_experience >= parent.get_experience_required(level_index.wrapping_add(1)) {
            level_index = level_index.wrapping_add(1);
        }
        self.current_level = VeterancyLevel::from_i32(level_index);

        if old_level != self.current_level {
            parent.on_veterancy_level_changed(old_level, self.current_level, provide_feedback);
        }
    }
}

impl Snapshot for ExperienceTrackerState {
    fn crc(&self, xfer: &mut dyn Xfer) {
        let mut exp = self.current_experience;
        let _ = xfer.xfer_int(&mut exp);
        let mut level_i32 = self.current_level.as_i32();
        let _ = xfer.xfer_enum_i32(&mut level_i32);
        #[cfg(not(feature = "retail-crc"))]
        {
            let mut trainable = self.is_trainable;
            let _ = xfer.xfer_bool(&mut trainable);
        }
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        #[cfg(feature = "retail-xfer-save")]
        let current_version: zh_common::types::XferVersion = 1;
        #[cfg(not(feature = "retail-xfer-save"))]
        let current_version: zh_common::types::XferVersion = 2;

        let mut version = current_version;
        xfer.xfer_version(&mut version, current_version)?;

        let mut level_i32 = self.current_level.as_i32();
        xfer.xfer_enum_i32(&mut level_i32)?;
        self.current_level = VeterancyLevel::from_i32(level_i32);

        xfer.xfer_int(&mut self.current_experience)?;
        xfer.xfer_object_id(&mut self.experience_sink)?;
        xfer.xfer_real(&mut self.experience_scalar)?;

        if version >= 2 {
            xfer.xfer_bool(&mut self.is_trainable)?;
        }
        Ok(())
    }

    fn load_post_process(&mut self) {}
}
