//! `PKT-204`: Base `CrateCollideData`, `CrateCollide`, and `CrateCollideImpl`
//! (`CrateCollide.h` / `CrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::collide_module::CollideModule;
use crate::modules::module::{ModuleData, ModuleSnapshotHeader};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::{KindOfMaskType, KindOfType, KINDOF_NAMES};
#[cfg(not(feature = "retail-crc"))]
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::science::{ScienceType, SCIENCE_INVALID};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, Relationship, UnsignedInt, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// Retail Zero Hour preserves `PRESERVE_MULTI_CRATE_PICKUP == 0`.
pub const PRESERVE_MULTI_CRATE_PICKUP: i32 = 0;

/// Sabotage feedback FX category (`CrateCollide::SabotageVictimType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SabotageVictimType {
    Generic = 0,
    CommandCenter = 1,
    FakeBuilding = 2,
    InternetCenter = 3,
    MilitaryFactory = 4,
    PowerPlant = 5,
    Superweapon = 6,
    SupplyCenter = 7,
    DropZone = 8,
}

/// Shared INI template configuration parsed for `CrateCollideModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct CrateCollideData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub kindof: KindOfMaskType,
    pub kindofnot: KindOfMaskType,
    pub is_forbid_owner_player: Bool,
    pub is_building_pickup: Bool,
    pub is_human_only_pickup: Bool,
    pub allow_multi_pickup: Bool,
    pub pickup_science: ScienceType,
    pub execute_fx: Option<String>,
    pub execution_animation_template: String,
    pub execute_animation_display_time_in_seconds: Real,
    pub execute_animation_z_rise_per_second: Real,
    pub execute_animation_fades: Bool,
}

impl CrateCollideData {
    #[inline]
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            kindof: KindOfMaskType::new(),
            kindofnot: KindOfMaskType::new(),
            is_forbid_owner_player: FALSE,
            is_building_pickup: FALSE,
            is_human_only_pickup: FALSE,
            allow_multi_pickup: Bool::from(PRESERVE_MULTI_CRATE_PICKUP != 0),
            pickup_science: SCIENCE_INVALID,
            execute_fx: None,
            execution_animation_template: String::new(),
            execute_animation_display_time_in_seconds: 0.0,
            execute_animation_z_rise_per_second: 0.0,
            execute_animation_fades: TRUE,
        }
    }

    /// Parses one `CrateCollideModuleData` INI field, returning `None` if `field` belongs to a subclass.
    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &mut ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("RequiredKindOf") {
            Some(self.kindof.parse_from_ini(reader, &KINDOF_NAMES))
        } else if field.eq_ignore_ascii_case("ForbiddenKindOf") {
            Some(self.kindofnot.parse_from_ini(reader, &KINDOF_NAMES))
        } else if field.eq_ignore_ascii_case("ForbidOwnerPlayer") {
            Some(reader.parse_bool(&mut self.is_forbid_owner_player))
        } else if field.eq_ignore_ascii_case("BuildingPickup") {
            Some(reader.parse_bool(&mut self.is_building_pickup))
        } else if field.eq_ignore_ascii_case("HumanOnly") {
            Some(reader.parse_bool(&mut self.is_human_only_pickup))
        } else if field.eq_ignore_ascii_case("AllowMultiPickup") {
            Some(reader.parse_bool(&mut self.allow_multi_pickup))
        } else if field.eq_ignore_ascii_case("PickupScience") {
            let res = (|| {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    self.pickup_science = SCIENCE_INVALID;
                } else {
                    self.pickup_science = parse_ctx
                        .science_store
                        .friend_lookup_science(tok, parse_ctx.name_keys)?;
                }
                Ok(())
            })();
            Some(res)
        } else if field.eq_ignore_ascii_case("ExecuteFX") {
            let res = (|| {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    self.execute_fx = None;
                } else {
                    let resolved = parse_ctx
                        .fx_list_names
                        .iter()
                        .find(|n| n.eq_ignore_ascii_case(tok))
                        .cloned()
                        .unwrap_or_else(|| tok.to_owned());
                    self.execute_fx = Some(resolved);
                }
                Ok(())
            })();
            Some(res)
        } else if field.eq_ignore_ascii_case("ExecuteAnimation") {
            Some(reader.parse_ascii_string(&mut self.execution_animation_template))
        } else if field.eq_ignore_ascii_case("ExecuteAnimationTime") {
            Some(reader.parse_real(&mut self.execute_animation_display_time_in_seconds))
        } else if field.eq_ignore_ascii_case("ExecuteAnimationZRise") {
            Some(reader.parse_real(&mut self.execute_animation_z_rise_per_second))
        } else if field.eq_ignore_ascii_case("ExecuteAnimationFades") {
            Some(reader.parse_bool(&mut self.execute_animation_fades))
        } else {
            None
        }
    }
}

impl ModuleData for CrateCollideData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::COLLIDE
    }
}

/// Parses `CrateCollideData` base fields and delegates remaining fields to `parse_extra`.
pub fn parse_crate_collide_data_with(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
    mut parse_extra: impl FnMut(
        &str,
        &mut IniReader<'_>,
        &mut ModuleParseContext<'_>,
    ) -> Result<(), IniError>,
) -> Result<CrateCollideData, IniError> {
    let mut base = CrateCollideData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if let Some(res) = base.parse_field(field, reader, parse_ctx) {
            res
        } else {
            parse_extra(field, reader, parse_ctx)
        }
    })?;
    Ok(base)
}

/// Evaluates C++ `CrateCollide::isValidToExecute`.
pub fn is_valid_to_execute(
    data: &CrateCollideData,
    sim: &dyn SimView,
    ctx: ModuleContext,
    other: Option<ObjectSlot>,
) -> Bool {
    let Some(other_slot) = other else {
        return FALSE;
    };
    if sim.is_neutral_controlled(other_slot) != FALSE {
        return FALSE;
    }

    let valid_building_attempt = Bool::from(
        data.is_building_pickup != FALSE
            && sim.object_is_kind_of(other_slot, KindOfType::Structure) != FALSE,
    );

    if sim.object_has_ai(other_slot) == FALSE && valid_building_attempt == FALSE {
        return FALSE;
    }

    if sim.object_is_kind_of_multi(other_slot, &data.kindof, &data.kindofnot) == FALSE {
        return FALSE;
    }

    let Some(other_obj) = sim.object(other_slot) else {
        return FALSE;
    };
    if other_obj.is_effectively_dead() != FALSE {
        return FALSE;
    }

    if sim.object_is_above_terrain(ctx.owner_slot) != FALSE && valid_building_attempt == FALSE {
        return FALSE;
    }

    let Some(owner_obj) = sim.object(ctx.owner_slot) else {
        return FALSE;
    };

    #[cfg(not(feature = "retail-crc"))]
    if owner_obj.is_destroyed() && data.allow_multi_pickup == FALSE {
        return FALSE;
    }

    if data.is_forbid_owner_player != FALSE && owner_obj.team_id == other_obj.team_id {
        return FALSE;
    }

    if data.is_human_only_pickup != FALSE && sim.player_is_human(other_obj.team_id) == FALSE {
        return FALSE;
    }

    if data.pickup_science != SCIENCE_INVALID
        && sim.player_has_science(other_obj.team_id, data.pickup_science) == FALSE
    {
        return FALSE;
    }

    if sim.object_is_kind_of(other_slot, KindOfType::Parachute) != FALSE {
        return FALSE;
    }

    TRUE
}

/// Completes C++ `CrateCollide::onCollide` after `executeCrateBehavior` returns `TRUE`.
pub fn execute_crate_behavior(
    data: &CrateCollideData,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    other: ObjectSlot,
) {
    if let Some(ref fx_name) = data.execute_fx {
        let pos = sim
            .object(other)
            .map(|o| *o.thing.get_position())
            .unwrap_or_default();
        sim.trigger_fx_list(fx_name, &pos);
    }
    sim.destroy_object(ctx.owner_slot);
}

/// Shared `Sabotage*CrateCollide::isValidToExecute` guard after base `CrateCollide::isValidToExecute`.
pub fn sabotage_is_valid_to_execute(
    base: &CrateCollide,
    sim: &dyn SimView,
    ctx: ModuleContext,
    other: Option<ObjectSlot>,
    kind_matches: impl FnOnce(&dyn SimView, ObjectSlot) -> Bool,
) -> Bool {
    if base.is_valid_to_execute(sim, ctx, other) == FALSE {
        return FALSE;
    }
    let Some(other_slot) = other else {
        return FALSE;
    };
    let Some(other_obj) = sim.object(other_slot) else {
        return FALSE;
    };
    if other_obj.is_effectively_dead() != FALSE {
        return FALSE;
    }
    if kind_matches(sim, other_slot) == FALSE {
        return FALSE;
    }
    #[cfg(not(feature = "retail-crc"))]
    if other_obj
        .status
        .test(ObjectStatusType::UnderConstruction.as_i32())
        != FALSE
        || other_obj.status.test(ObjectStatusType::Sold.as_i32()) != FALSE
    {
        return FALSE;
    }
    if sim.player_relationship(ctx.owner_slot, other_slot) != Relationship::Enemies {
        return FALSE;
    }
    TRUE
}

/// Shared AI goal check (`if (ai && ai->getGoalObject() != other) return false;`) in `Sabotage*`.
pub fn sabotage_goal_matches(sim: &dyn SimView, owner: ObjectSlot, other: ObjectSlot) -> Bool {
    if sim.object_has_ai(owner) != FALSE {
        return Bool::from(sim.object_ai_goal_object(owner) == Some(other));
    }
    TRUE
}

/// Shared cash-stealing helper used by `SabotageSupplyCenterCrateCollide` and `SabotageSupplyDropzoneCrateCollide`.
pub fn sabotage_steal_cash(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    other: ObjectSlot,
    desired_amount: UnsignedInt,
) {
    let Some(other_team) = sim.object(other).map(|o| o.team_id) else {
        return;
    };
    let Some(owner_team) = sim.object(owner).map(|o| o.team_id) else {
        return;
    };
    let cash = sim.player_money(other_team).min(desired_amount);
    if cash > 0 {
        let withdrawn = sim.withdraw_player_money(other_team, cash);
        if withdrawn > 0 {
            let deposit = withdrawn.min(Int::MAX as UnsignedInt) as Int;
            let _ = sim.add_player_money(owner_team, deposit);
        }
    }
}

/// Base runtime state and snapshot serialization matching C++ `CrateCollide` (`CrateCollide.cpp`).
#[derive(Debug, Clone)]
pub struct CrateCollide {
    pub base: CollideModule,
    pub data: CrateCollideData,
}

impl CrateCollide {
    #[inline]
    pub fn new(data: CrateCollideData) -> Self {
        Self {
            base: CollideModule::new(data.module_name_key),
            data,
        }
    }

    #[inline]
    pub fn is_valid_to_execute(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
    ) -> Bool {
        is_valid_to_execute(&self.data, sim, ctx, other)
    }

    #[inline]
    pub fn after_execute_success(
        &self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: ObjectSlot,
    ) {
        execute_crate_behavior(&self.data, sim, ctx, other);
    }

    #[inline]
    pub fn do_sabotage_feedback_fx(
        &self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _other: ObjectSlot,
        _victim_type: SabotageVictimType,
    ) {
        // Audio and drawable flash feedback are client-only effects outside headless simulation.
    }

    #[inline]
    pub fn crc(&self, xfer: &mut dyn Xfer) {
        self.base.crc(xfer);
    }

    /// Serializes `CrateCollide::xfer` (`version(1)` + `CollideModule::xfer`).
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        ModuleSnapshotHeader::xfer_crate_collide_header(xfer)
    }

    /// Serializes a concrete subclass with no additional fields (`version(1)` + `CrateCollide::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer(xfer)
    }
}

/// Trait reducing boilerplate across `CrateCollide` subclasses (`Sabotage*`, `HealCrate`, etc.).
pub trait CrateCollideImpl: Send {
    fn crate_base(&self) -> &CrateCollide;
    fn crate_base_mut(&mut self) -> &mut CrateCollide;

    fn is_valid_to_execute(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
    ) -> Bool {
        self.crate_base().is_valid_to_execute(sim, ctx, other)
    }

    fn execute_crate_behavior(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: ObjectSlot,
    ) -> Bool;

    fn is_hijacked_vehicle_crate_collide(&self) -> Bool {
        FALSE
    }

    fn is_sabotage_building_crate_collide(&self) -> Bool {
        FALSE
    }

    fn is_car_bomb_crate_collide(&self) -> Bool {
        FALSE
    }

    fn is_salvage_crate_collide(&self) -> Bool {
        FALSE
    }
}

/// Implements [`BehaviorModule`] and [`CollideFacet`] by delegating to [`CrateCollideImpl`].
macro_rules! impl_crate_collide_behavior {
    ($ty:ty) => {
        impl $crate::modules::module::BehaviorModule for $ty {
            fn module_name_key(&self) -> zh_common::types::NameKeyType {
                $crate::modules::collide::crate_collide::CrateCollideImpl::crate_base(self)
                    .base
                    .module_name_key
            }

            fn crc(&self, xfer: &mut dyn zh_common::xfer::Xfer) {
                $crate::modules::collide::crate_collide::CrateCollideImpl::crate_base(self)
                    .crc(xfer);
            }

            fn xfer(
                &mut self,
                xfer: &mut dyn zh_common::xfer::Xfer,
                _sim: &dyn $crate::sim::SimView,
            ) -> Result<(), zh_common::xfer::XferError> {
                $crate::modules::collide::crate_collide::CrateCollideImpl::crate_base_mut(self)
                    .xfer_subclass(xfer)
            }

            fn as_collide(&self) -> Option<&dyn $crate::modules::module::CollideFacet> {
                Some(self)
            }

            fn as_collide_mut(&mut self) -> Option<&mut dyn $crate::modules::module::CollideFacet> {
                Some(self)
            }
        }

        impl $crate::modules::module::CollideFacet for $ty {
            fn on_collide(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                other: Option<$crate::arena::ObjectSlot>,
                _loc: Option<&zh_common::math::Coord3D>,
                _normal: Option<&zh_common::math::Coord3D>,
            ) {
                if $crate::modules::collide::crate_collide::CrateCollideImpl::is_valid_to_execute(
                    self, sim, ctx, other,
                ) != zh_common::types::FALSE
                {
                    if let Some(other_slot) = other {
                        if $crate::modules::collide::crate_collide::CrateCollideImpl::execute_crate_behavior(
                            self,
                            sim,
                            ctx,
                            other_slot,
                        ) != zh_common::types::FALSE
                        {
                            $crate::modules::collide::crate_collide::CrateCollideImpl::crate_base(
                                self,
                            )
                            .after_execute_success(sim, ctx, other_slot);
                        }
                    }
                }
            }

            fn would_like_to_collide_with(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
                other: $crate::arena::ObjectSlot,
            ) -> zh_common::types::Bool {
                $crate::modules::collide::crate_collide::CrateCollideImpl::is_valid_to_execute(
                    self,
                    sim,
                    ctx,
                    Some(other),
                )
            }

            fn is_hijacked_vehicle_crate_collide(&self) -> zh_common::types::Bool {
                $crate::modules::collide::crate_collide::CrateCollideImpl::is_hijacked_vehicle_crate_collide(self)
            }

            fn is_sabotage_building_crate_collide(&self) -> zh_common::types::Bool {
                $crate::modules::collide::crate_collide::CrateCollideImpl::is_sabotage_building_crate_collide(self)
            }

            fn is_car_bomb_crate_collide(&self) -> zh_common::types::Bool {
                $crate::modules::collide::crate_collide::CrateCollideImpl::is_car_bomb_crate_collide(self)
            }

            fn is_salvage_crate_collide(&self) -> zh_common::types::Bool {
                $crate::modules::collide::crate_collide::CrateCollideImpl::is_salvage_crate_collide(self)
            }
        }
    };
}

pub(crate) use impl_crate_collide_behavior;
