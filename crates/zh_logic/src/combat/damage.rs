//! `PKT-140`: Damage descriptors and `Snapshot` serialization (`Damage.h` / `Damage.cpp`).

use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::object_status_types::ObjectStatusType;
use zh_common::math::Coord3D;
use zh_common::types::{Bool, ObjectId, Real, UnsignedShort, XferVersion, FALSE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

/// Player bitmask type (`GameCommon.h`: `UnsignedShort`).
pub type PlayerMaskType = UnsignedShort;

/// Bitmask covering all `MAX_PLAYER_COUNT` player slots (`GameCommon.h`: `PLAYERMASK_ALL`).
pub const PLAYERMASK_ALL: PlayerMaskType = 0xFFFF;

/// Massive damage sentinel matching `HUGE_DAMAGE_AMOUNT` in `Damage.h`.
pub const HUGE_DAMAGE_AMOUNT: Real = 999_999.0_f32;

/// Input parameters describing attempted damage (`DamageInfoInput`).
#[derive(Debug, Clone, PartialEq)]
pub struct DamageInfoInput {
    pub source_id: ObjectId,
    /// Template name corresponding to `const ThingTemplate* m_sourceTemplate` (empty = null).
    pub source_template: String,
    pub source_player_mask: PlayerMaskType,
    pub damage_type: DamageType,
    pub damage_status_type: ObjectStatusType,
    pub damage_fx_override: DamageType,
    pub death_type: DeathType,
    pub amount: Real,
    pub kill: Bool,
    pub shock_wave_vector: Coord3D,
    pub shock_wave_amount: Real,
    pub shock_wave_radius: Real,
    pub shock_wave_taper_off: Real,
}

impl Default for DamageInfoInput {
    fn default() -> Self {
        Self {
            source_id: ObjectId::INVALID,
            source_template: String::new(),
            source_player_mask: 0,
            damage_type: DamageType::Explosion,
            damage_status_type: ObjectStatusType::None,
            damage_fx_override: DamageType::Unresistable,
            death_type: DeathType::Normal,
            amount: 0.0_f32,
            kill: FALSE,
            shock_wave_vector: Coord3D {
                x: 0.0_f32,
                y: 0.0_f32,
                z: 0.0_f32,
            },
            shock_wave_amount: 0.0_f32,
            shock_wave_radius: 0.0_f32,
            shock_wave_taper_off: 0.0_f32,
        }
    }
}

impl DamageInfoInput {
    /// Runs `Damage.cpp` v3 serialization, using `has_template` when loading v3 template names.
    pub fn xfer_with_lookup(
        &mut self,
        xfer: &mut dyn Xfer,
        has_template: impl Fn(&str) -> bool,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 3;
        xfer.xfer_version(&mut version, 3)?;

        xfer.xfer_object_id(&mut self.source_id)?;
        xfer.xfer_unsigned_short(&mut self.source_player_mask)?;

        let mut dt_raw = self.damage_type.as_i32();
        xfer.xfer_enum_i32(&mut dt_raw)?;
        self.damage_type = DamageType::from_i32(dt_raw).ok_or(XferError::InvalidData)?;

        if version >= 2 {
            let mut fx_raw = self.damage_fx_override.as_i32();
            xfer.xfer_enum_i32(&mut fx_raw)?;
            self.damage_fx_override =
                DamageType::from_i32(fx_raw).ok_or(XferError::InvalidData)?;
        }

        let mut death_raw = self.death_type.as_i32();
        xfer.xfer_enum_i32(&mut death_raw)?;
        self.death_type = DeathType::from_i32(death_raw).ok_or(XferError::InvalidData)?;

        xfer.xfer_real(&mut self.amount)?;

        if version >= 2 {
            xfer.xfer_bool(&mut self.kill)?;
        }

        if version >= 3 {
            let mut status_raw = self.damage_status_type.as_i32();
            xfer.xfer_enum_i32(&mut status_raw)?;
            self.damage_status_type =
                ObjectStatusType::from_i32(status_raw).ok_or(XferError::InvalidData)?;

            xfer.xfer_coord3d(&mut self.shock_wave_vector)?;
            xfer.xfer_real(&mut self.shock_wave_amount)?;
            xfer.xfer_real(&mut self.shock_wave_radius)?;
            xfer.xfer_real(&mut self.shock_wave_taper_off)?;

            let mut thing_name = self.source_template.clone();
            xfer.xfer_ascii_string(&mut thing_name)?;
            if xfer.mode() == XferMode::Load {
                self.source_template = if !thing_name.is_empty() && has_template(&thing_name) {
                    thing_name
                } else {
                    String::new()
                };
            }
        }

        Ok(())
    }
}

impl Snapshot for DamageInfoInput {
    fn crc(&self, _xfer: &mut dyn Xfer) {
        // Matches `DamageInfoInput::crc` in Damage.h (`{ }`).
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        self.xfer_with_lookup(xfer, |name| !name.is_empty())
    }
}

/// Output results populated after applying damage (`DamageInfoOutput`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DamageInfoOutput {
    pub actual_damage_dealt: Real,
    pub actual_damage_clipped: Real,
    pub no_effect: Bool,
}

impl Snapshot for DamageInfoOutput {
    fn crc(&self, _xfer: &mut dyn Xfer) {
        // Matches `DamageInfoOutput::crc` in Damage.h (`{ }`).
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_real(&mut self.actual_damage_dealt)?;
        xfer.xfer_real(&mut self.actual_damage_clipped)?;
        xfer.xfer_bool(&mut self.no_effect)?;
        Ok(())
    }
}

/// Combined damage request and result (`DamageInfo`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DamageInfo {
    pub input: DamageInfoInput,
    pub output: DamageInfoOutput,
}

impl DamageInfo {
    /// Runs `DamageInfo::xfer` while resolving loaded `source_template` names via `has_template`.
    pub fn xfer_with_lookup(
        &mut self,
        xfer: &mut dyn Xfer,
        has_template: impl Fn(&str) -> bool,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.input.xfer_with_lookup(xfer, &has_template)?;
        self.output.xfer(xfer)?;
        Ok(())
    }
}

impl Snapshot for DamageInfo {
    fn crc(&self, _xfer: &mut dyn Xfer) {
        // Matches `DamageInfo::crc` in Damage.h (`{ }`).
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_snapshot(&mut self.input)?;
        xfer.xfer_snapshot(&mut self.output)?;
        Ok(())
    }
}
