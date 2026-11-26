//! `PKT-336`: `HordeUpdateModuleData` and `HordeUpdate`
//! (`HordeUpdate.h`, `HordeUpdate.cpp`), with the
//! `AIUpdateInterface::evaluateMoraleBonus` family it calls.
//!
//! Packet: `packets/phase5/PKT-336_HordeUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt336_370_306_304_leaves.rs`.
//!
//! `joinOrLeaveHorde` has no caller in the reference and is not ported.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::gamelogic::global_data::WeaponBonusConditionType;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, HordeFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::{KindOfMaskType, KindOfType, KINDOF_NAMES};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

const LOGIC_FRAMES_PER_SECOND: UnsignedInt = 30;

const HORDE_ACTION_NAMES: [&str; 2] = ["HORDE", "HORDE_FIXED"];

/// `HordeActionType`. The default is `Horde` in every configuration here:
/// `PRESERVE_PERPETUAL_HORDE_BONUS` is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum HordeActionType {
    Horde = 0,
    HordeFixed = 1,
}

/// `TerrainDecalType` (`Drawable.h`, Zero Hour order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TerrainDecalType {
    DemoralizedObsolete = 0,
    Horde = 1,
    HordeWithNationalismUpgrade = 2,
    HordeVehicle = 3,
    HordeWithNationalismUpgradeVehicle = 4,
    Crate = 5,
    HordeWithFanaticismUpgrade = 6,
    ChemSuit = 7,
    None = 8,
    ShadowTexture = 9,
}

/// INI template data matching `HordeUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct HordeUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub update_rate: UnsignedInt,
    pub kind_of: KindOfMaskType,
    pub min_count: Int,
    pub min_dist: Real,
    pub allies_only: Bool,
    pub exact_match: Bool,
    pub rub_off_radius: Real,
    pub action: HordeActionType,
    pub allowed_nationalism: Bool,
    pub flag_sub_obj_names: Vec<String>,
}

impl HordeUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            update_rate: LOGIC_FRAMES_PER_SECOND,
            kind_of: KindOfMaskType::default(),
            min_count: 0,
            min_dist: 0.0,
            allies_only: TRUE,
            exact_match: FALSE,
            rub_off_radius: 20.0,
            action: HordeActionType::Horde,
            allowed_nationalism: TRUE,
            flag_sub_obj_names: Vec::new(),
        }
    }
}

impl ModuleData for HordeUpdateModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x001 | FacetMask::HORDE_UPDATE.0)
    }
}

/// `Player::hasUpgradeComplete` of a named upgrade by the owner's player;
/// `FALSE` when no such upgrade is defined.
fn owner_has_upgrade(sim: &dyn SimView, owner: ObjectSlot, name: &str) -> bool {
    let Some(team) = sim.object(owner).map(|o| o.team_id) else {
        return false;
    };
    let rules = sim.rules();
    let Some(upgrade) = rules
        .name_keys
        .lookup_key(name)
        .and_then(|key| rules.upgrade_center.find_upgrade_by_key(key))
    else {
        return false;
    };
    sim.player_has_upgrade(team, upgrade.upgrade_mask()) != FALSE
}

fn has_nationalism(sim: &dyn SimView, owner: ObjectSlot) -> bool {
    owner_has_upgrade(sim, owner, "Upgrade_Nationalism")
}

fn has_fanaticism(sim: &dyn SimView, owner: ObjectSlot) -> bool {
    owner_has_upgrade(sim, owner, "Upgrade_Fanaticism")
}

/// `AIUpdateInterface::evaluateNationalismBonusClassic`. Leaving the horde
/// keeps the fanaticism bonus.
fn evaluate_nationalism_bonus_classic(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    in_horde: Bool,
    allow_nationalism: Bool,
) {
    use WeaponBonusConditionType as Bonus;
    if in_horde != FALSE {
        sim.set_weapon_bonus_condition(owner, Bonus::Horde);
    } else {
        sim.clear_weapon_bonus_condition(owner, Bonus::Horde);
    }

    if allow_nationalism != FALSE && has_nationalism(sim, owner) {
        sim.set_weapon_bonus_condition(owner, Bonus::Nationalism);
        if has_fanaticism(sim, owner) {
            sim.set_weapon_bonus_condition(owner, Bonus::Fanaticism);
        } else {
            sim.clear_weapon_bonus_condition(owner, Bonus::Fanaticism);
        }
    } else {
        sim.clear_weapon_bonus_condition(owner, Bonus::Nationalism);
    }
}

/// `AIUpdateInterface::evaluateNationalismBonus`.
#[cfg(not(feature = "retail-crc"))]
fn evaluate_nationalism_bonus(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    in_horde: Bool,
    allow_nationalism: Bool,
) {
    use WeaponBonusConditionType as Bonus;
    if in_horde != FALSE {
        sim.set_weapon_bonus_condition(owner, Bonus::Horde);
        if allow_nationalism != FALSE && has_nationalism(sim, owner) {
            sim.set_weapon_bonus_condition(owner, Bonus::Nationalism);
        }
        if allow_nationalism != FALSE && has_fanaticism(sim, owner) {
            sim.set_weapon_bonus_condition(owner, Bonus::Fanaticism);
        }
    } else {
        sim.clear_weapon_bonus_condition(owner, Bonus::Horde);
        sim.clear_weapon_bonus_condition(owner, Bonus::Nationalism);
        sim.clear_weapon_bonus_condition(owner, Bonus::Fanaticism);
    }
}

/// `AIUpdateInterface::evaluateMoraleBonus` (`ALLOW_DEMORALIZE` is off). It
/// only touches the object, so it lives here until the AI is ported.
pub fn evaluate_morale_bonus(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    in_horde: Bool,
    allow_nationalism: Bool,
    action: HordeActionType,
) {
    match action {
        HordeActionType::Horde => {
            evaluate_nationalism_bonus_classic(sim, owner, in_horde, allow_nationalism);
        }
        #[cfg(not(feature = "retail-crc"))]
        HordeActionType::HordeFixed => {
            evaluate_nationalism_bonus(sim, owner, in_horde, allow_nationalism);
        }
        #[cfg(feature = "retail-crc")]
        HordeActionType::HordeFixed => {}
    }
}

/// Runtime instance of `HordeUpdate`.
pub struct HordeUpdate {
    pub data: HordeUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    /// Neither this nor `true_horde_member` is saved.
    pub last_horde_refresh_frame: UnsignedInt,
    pub in_horde: Bool,
    pub true_horde_member: Bool,
    pub has_flag: Bool,
}

impl HordeUpdate {
    pub fn new(data: HordeUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            last_horde_refresh_frame: 0,
            in_horde: FALSE,
            true_horde_member: FALSE,
            has_flag: FALSE,
        }
    }

    /// `PartitionFilterHordeMember::allow`.
    fn filter_allows(&self, sim: &dyn SimView, owner: ObjectSlot, other: ObjectSlot) -> bool {
        let (Some(me), Some(them)) = (sim.object(owner), sim.object(other)) else {
            return false;
        };
        #[cfg(not(feature = "retail-crc"))]
        if them.is_effectively_dead() != FALSE {
            return false;
        }

        if self.data.exact_match != FALSE {
            let factory = &sim.rules().thing_factory;
            let mine = me.thing.template_override().get_final(factory);
            let theirs = them.thing.template_override().get_final(factory);
            if mine.map(std::ptr::from_ref) != theirs.map(std::ptr::from_ref) {
                return false;
            }
        }

        if sim.horde_member_state(other).is_none() {
            return false;
        }

        if sim.object_is_kind_of_multi(other, &self.data.kind_of, &KindOfMaskType::default())
            == FALSE
        {
            return false;
        }

        if self.data.allies_only != FALSE
            && sim.player_relationship(owner, other) != Relationship::Allies
        {
            return false;
        }

        let off_map = crate::sim::PRIVATE_STATUS_OFF_MAP;
        (me.private_status & off_map) == (them.private_status & off_map)
    }

    /// The membership half of `update`, run on infantry every time and on other
    /// units when the refresh is due.
    fn refresh_membership(&mut self, sim: &dyn SimView, owner: ObjectSlot) {
        let members: Vec<ObjectSlot> = sim
            .find_objects_in_range_bounding_sphere_3d(owner, self.data.min_dist)
            .into_iter()
            .filter(|&other| self.filter_allows(sim, owner, other))
            .collect();

        if members.len() as Int >= self.data.min_count.wrapping_sub(1) {
            self.in_horde = TRUE;
            self.true_horde_member = TRUE;
            return;
        }

        self.in_horde = FALSE;
        self.true_horde_member = FALSE;
        // Close enough to a true member, I am in too.
        let rub_off_sqr = self.data.rub_off_radius * self.data.rub_off_radius;
        let Some(pos) = sim.object(owner).map(|o| *o.thing.get_position()) else {
            return;
        };
        for other in members {
            if sim.horde_member_state(other) != Some(TRUE) {
                continue;
            }
            let Some(there) = sim.object(other).map(|o| *o.thing.get_position()) else {
                continue;
            };
            let (dx, dy) = (there.x - pos.x, there.y - pos.y);
            if dx * dx + dy * dy <= rub_off_sqr {
                self.in_horde = TRUE;
                break;
            }
        }
    }

    /// The terrain decal under the unit. Drawable state only.
    fn update_terrain_decal(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        is_infantry: bool,
        was_in_horde: Bool,
    ) {
        if sim.object_has_drawable(owner) == FALSE {
            return;
        }
        let Some((id, dead, bonus, major_radius)) = sim.object(owner).map(|o| {
            (
                o.id,
                o.is_effectively_dead(),
                o.weapon_bonus_condition,
                o.geometry.get_major_radius(),
            )
        }) else {
            return;
        };
        if dead != FALSE {
            return;
        }

        if sim.draw_icon_ui() != FALSE {
            if self.in_horde != FALSE
                && sim.object_is_kind_of(owner, KindOfType::PortableStructure) == FALSE
            {
                if !is_infantry {
                    let size = 3.5 * major_radius;
                    sim.client().on_terrain_decal_size(id, size, size);
                }
                let has = |c: WeaponBonusConditionType| bonus & (1u32 << c.as_i32()) != 0;
                let decal = if has(WeaponBonusConditionType::Nationalism) {
                    if has(WeaponBonusConditionType::Fanaticism) {
                        TerrainDecalType::HordeWithFanaticismUpgrade
                    } else {
                        TerrainDecalType::HordeWithNationalismUpgrade
                    }
                } else if is_infantry {
                    TerrainDecalType::Horde
                } else {
                    TerrainDecalType::HordeVehicle
                };
                sim.client().on_terrain_decal(id, decal);
            }
        } else {
            sim.client().on_terrain_decal(id, TerrainDecalType::None);
        }

        if was_in_horde == FALSE && self.in_horde != FALSE {
            sim.client().on_terrain_decal_fade_target(id, 1.0, 0.03);
        } else if was_in_horde != FALSE && self.in_horde == FALSE {
            sim.client().on_terrain_decal_fade_target(id, 0.0, -0.03);
        }
    }
}

impl HordeFacet for HordeUpdate {
    fn is_in_horde(&self) -> Bool {
        self.in_horde
    }

    fn has_flag(&self) -> Bool {
        self.has_flag
    }

    fn is_true_horde_member(&self) -> Bool {
        Bool::from(self.true_horde_member != FALSE && self.in_horde != FALSE)
    }

    fn is_allowed_nationalism(&self) -> Bool {
        self.data.allowed_nationalism
    }

    fn horde_action_type(&self) -> HordeActionType {
        self.data.action
    }
}

impl UpdateFacet for HordeUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        if sim.object(owner).is_none() {
            return UpdateSleepTime::FOREVER;
        }
        let was_in_horde = self.in_horde;
        let is_infantry = sim.object_is_kind_of(owner, KindOfType::Infantry) != FALSE;

        if is_infantry
            || sim.frame()
                > self
                    .last_horde_refresh_frame
                    .wrapping_add(self.data.update_rate)
        {
            self.last_horde_refresh_frame = sim.frame();
            self.refresh_membership(sim, owner);
            if sim.has_ai(owner) != FALSE {
                evaluate_morale_bonus(
                    sim,
                    owner,
                    self.in_horde,
                    self.data.allowed_nationalism,
                    self.data.action,
                );
            }
        }

        self.update_terrain_decal(sim, owner, is_infantry, was_in_horde);

        if is_infantry {
            UpdateSleepTime(self.data.update_rate)
        } else {
            UpdateSleepTime::NONE
        }
    }
}

impl BehaviorModule for HordeUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        self.last_horde_refresh_frame = sim.frame();
        // Spread the first looks out so that not every unit scans on one frame.
        let delay = sim.rng_logic_int(1, self.data.update_rate as Int);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(delay as UnsignedInt));
    }

    /// `onDrawableBoundToObject`: hide the flag. The reference binds the
    /// drawable as the object is made; there is no separate hook here.
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if !self.data.flag_sub_obj_names.is_empty() {
            sim.update_sub_objects(ctx.owner_slot, &[], &self.data.flag_sub_obj_names);
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.in_horde)?;
        xfer.xfer_bool(&mut self.has_flag)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_horde(&self) -> Option<&dyn HordeFacet> {
        Some(self)
    }
}

pub fn parse_horde_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = HordeUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("UpdateRate") {
            r.parse_duration_unsigned_int(&mut data.update_rate)
        } else if is("KindOf") {
            data.kind_of.parse_from_ini(r, &KINDOF_NAMES)
        } else if is("Count") {
            r.parse_int(&mut data.min_count)
        } else if is("Radius") {
            r.parse_real(&mut data.min_dist)
        } else if is("RubOffRadius") {
            r.parse_real(&mut data.rub_off_radius)
        } else if is("AlliesOnly") {
            r.parse_bool(&mut data.allies_only)
        } else if is("ExactMatch") {
            r.parse_bool(&mut data.exact_match)
        } else if is("Action") {
            let mut index = 0;
            r.parse_index_list(&mut index, &HORDE_ACTION_NAMES)?;
            data.action = if index == 0 {
                HordeActionType::Horde
            } else {
                HordeActionType::HordeFixed
            };
            Ok(())
        } else if is("FlagSubObjectNames") {
            r.parse_ascii_string_vector(&mut data.flag_sub_obj_names)
        } else if is("AllowedNationalism") {
            r.parse_bool(&mut data.allowed_nationalism)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_horde_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<HordeUpdateModuleData>()
        .expect("HordeUpdate data type mismatch")
        .clone();
    Box::new(HordeUpdate::new(typed))
}
