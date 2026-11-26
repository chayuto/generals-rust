//! `PKT-258`: `BattleBusSlowDeathBehavior` (`BattleBusSlowDeathBehavior.h`, `.cpp`).
//!
//! Packet: `packets/phase5/PKT-258_BattleBusSlowDeathBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_b.rs`.

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    damage_info_from_die_op, resolve_optional_asset_name, xfer_module_version,
};
use crate::modules::behavior::slow_death_behavior::{SlowDeathBehaviorModuleData, SlowDeathCore};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{
    BehaviorModule, CommandSourceType, DieFacet, ModuleData, SlowDeathFacet, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType, DisabledType};
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Check for colliding with the ground only after this long, to prevent hitting on the way up.
const GROUND_CHECK_DELAY: UnsignedInt = 10;
/// Check for an empty hulk this often instead of every frame.
const EMPTY_HULK_CHECK_DELAY: UnsignedInt = 15;

/// INI template data matching `BattleBusSlowDeathBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct BattleBusSlowDeathBehaviorModuleData {
    pub slow_death: SlowDeathBehaviorModuleData,
    pub fx_start_undeath: Option<String>,
    pub ocl_start_undeath: Option<String>,
    pub fx_hit_ground: Option<String>,
    pub ocl_hit_ground: Option<String>,
    pub throw_force: Real,
    pub percent_damage_to_passengers: Real,
    pub empty_hulk_destruction_delay: UnsignedInt,
}

impl BattleBusSlowDeathBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            slow_death: SlowDeathBehaviorModuleData::new(tag_key, module_name_key),
            fx_start_undeath: None,
            ocl_start_undeath: None,
            fx_hit_ground: None,
            ocl_hit_ground: None,
            throw_force: 1.0,
            percent_damage_to_passengers: 0.0,
            empty_hulk_destruction_delay: 0,
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let asset = |reader: &mut IniReader<'_>,
                     known: &std::collections::BTreeSet<String>,
                     out: &mut Option<String>| {
            let tok = reader.next_token(SEPS_DEFAULT)?;
            *out = resolve_optional_asset_name(tok, known);
            Ok(())
        };
        if field.eq_ignore_ascii_case("FXStartUndeath") {
            Some(asset(reader, parse_ctx.fx_list_names, &mut self.fx_start_undeath))
        } else if field.eq_ignore_ascii_case("OCLStartUndeath") {
            Some(asset(reader, parse_ctx.ocl_names, &mut self.ocl_start_undeath))
        } else if field.eq_ignore_ascii_case("FXHitGround") {
            Some(asset(reader, parse_ctx.fx_list_names, &mut self.fx_hit_ground))
        } else if field.eq_ignore_ascii_case("OCLHitGround") {
            Some(asset(reader, parse_ctx.ocl_names, &mut self.ocl_hit_ground))
        } else if field.eq_ignore_ascii_case("ThrowForce") {
            Some(reader.parse_real(&mut self.throw_force))
        } else if field.eq_ignore_ascii_case("PercentDamageToPassengers") {
            Some(reader.parse_percent_to_real(&mut self.percent_damage_to_passengers))
        } else if field.eq_ignore_ascii_case("EmptyHulkDestructionDelay") {
            Some(reader.parse_duration_unsigned_int(&mut self.empty_hulk_destruction_delay))
        } else {
            self.slow_death.parse_field(field, reader, parse_ctx)
        }
    }
}

impl ModuleData for BattleBusSlowDeathBehaviorModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.slow_death.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.slow_death.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x003 | FacetMask::SLOW_DEATH.0)
    }
}

/// Runtime instance of `BattleBusSlowDeathBehavior` (`PKT-258`).
pub struct BattleBusSlowDeathBehavior {
    pub data: BattleBusSlowDeathBehaviorModuleData,
    pub core: SlowDeathCore,
    /// Set by `onDie`; prevents the alternate processing in `beginSlowDeath`.
    pub is_real_death: Bool,
    /// Controls the lifetime of the alternate (thrown in the air) processing.
    pub is_in_first_death: Bool,
    pub ground_check_frame: UnsignedInt,
    /// If non zero, kill us with penalty damage after this frame.
    pub penalty_death_frame: UnsignedInt,
}

impl BattleBusSlowDeathBehavior {
    pub fn new(data: BattleBusSlowDeathBehaviorModuleData) -> Self {
        Self {
            data,
            core: SlowDeathCore::new(),
            is_real_death: FALSE,
            is_in_first_death: FALSE,
            ground_check_frame: 0,
            penalty_death_frame: 0,
        }
    }

    fn do_effects(
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        fx: Option<&String>,
        ocl: Option<&String>,
    ) {
        let pos = sim
            .object(ctx.owner_slot)
            .map(|o| *o.thing.get_position())
            .unwrap_or_default();
        if let Some(fx) = fx {
            sim.trigger_fx_list(fx, &pos);
        }
        if let Some(ocl) = ocl {
            sim.spawn_ocl(ocl, ctx.owner_slot, &pos, &pos);
        }
    }

    /// `BattleBusSlowDeathBehavior::onDie`.
    fn die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        // Set beforehand because onDie could result in picking us to beginSlowDeath.
        self.is_real_death = TRUE;
        self.is_in_first_death = FALSE;

        if self.core.on_die(&self.data.slow_death, sim, ctx, dmg) != FALSE {
            self.begin(sim, ctx, dmg);
        }
    }

    /// `BattleBusSlowDeathBehavior::beginSlowDeath`.
    fn begin(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.is_real_death != FALSE {
            self.core
                .begin_slow_death(&self.data.slow_death, sim, ctx, dmg);
            return;
        }
        let me = ctx.owner_slot;
        // Intercept with the extra slow death: this was not triggered by a real onDie.
        self.is_in_first_death = TRUE;
        self.ground_check_frame = sim.frame().wrapping_add(GROUND_CHECK_DELAY);

        Self::do_effects(
            sim,
            ctx,
            self.data.fx_start_undeath.as_ref(),
            self.data.ocl_start_undeath.as_ref(),
        );

        if sim.has_ai(me) != FALSE {
            sim.ai_idle(me, CommandSourceType::FromAi);
        }
        if sim.has_physics(me) != FALSE {
            sim.physics_clear_acceleration(me);
            sim.physics_scrub_velocity_2d(me, 0.0);
            let throw_force = Coord3D::new(0.0, 0.0, self.data.throw_force);
            sim.physics_apply_shock(me, &throw_force);
            sim.physics_apply_random_rotation(me);
        }
        sim.contain_process_damage_to_contained(me, self.data.percent_damage_to_passengers);

        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    /// `BattleBusSlowDeathBehavior::update`.
    fn tick(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let me = ctx.owner_slot;
        if self.is_in_first_death != FALSE {
            if self.ground_check_frame < sim.frame() && sim.object_is_above_terrain(me) == FALSE {
                self.is_in_first_death = FALSE;

                Self::do_effects(
                    sim,
                    ctx,
                    self.data.fx_hit_ground.as_ref(),
                    self.data.ocl_hit_ground.as_ref(),
                );
                if sim.object_has_drawable(me) != FALSE {
                    sim.set_model_condition_state(me, ModelConditionFlagType::SecondLife);
                }
                if sim.has_ai(me) != FALSE {
                    sim.ai_idle(me, CommandSourceType::FromAi);
                }
                if sim.has_physics(me) != FALSE {
                    sim.physics_clear_acceleration(me);
                    sim.physics_scrub_velocity_2d(me, 0.0);
                }
                sim.set_disabled(me, DisabledType::Held);

                // We can only sleep if we don't have to watch out for being empty.
                return if self.data.empty_hulk_destruction_delay == 0 {
                    UpdateSleepTime::FOREVER
                } else {
                    UpdateSleepTime::NONE
                };
            }
            return UpdateSleepTime::NONE;
        }
        if self.is_real_death != FALSE {
            return self.core.update_core(&self.data.slow_death, sim, ctx);
        }

        // Neither death is active: stay awake to check for emptiness.
        if sim.object(me).is_none_or(|o| o.contain_slot.is_none()) {
            return UpdateSleepTime::FOREVER;
        }
        let check_again = UpdateSleepTime(EMPTY_HULK_CHECK_DELAY);
        if self.penalty_death_frame != 0 {
            if sim.contain_count(me) > 0 {
                self.penalty_death_frame = 0;
                check_again
            } else if sim.frame() > self.penalty_death_frame {
                // Penalty prevents effects; Extra4 stops this die module from recalling.
                kill_object(sim, me, DamageType::Penalty, DeathType::Extra4);
                UpdateSleepTime::FOREVER
            } else {
                check_again
            }
        } else {
            if sim.contain_count(me) == 0 {
                self.penalty_death_frame = sim
                    .frame()
                    .wrapping_add(self.data.empty_hulk_destruction_delay);
            }
            check_again
        }
    }
}

impl BehaviorModule for BattleBusSlowDeathBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.slow_death.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.on_object_created(sim, ctx);
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some(dmg) = damage_info_from_die_op(&op) {
            self.die(sim, ctx, &dmg);
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.core.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)?;
        xfer.xfer_bool(&mut self.is_real_death)?;
        xfer.xfer_bool(&mut self.is_in_first_death)?;
        xfer.xfer_unsigned_int(&mut self.ground_check_frame)?;
        xfer.xfer_unsigned_int(&mut self.penalty_death_frame)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.load_post_process(sim, ctx);
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

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }

    fn as_slow_death(&self) -> Option<&dyn SlowDeathFacet> {
        Some(self)
    }

    fn as_slow_death_mut(&mut self) -> Option<&mut dyn SlowDeathFacet> {
        Some(self)
    }
}

impl DieFacet for BattleBusSlowDeathBehavior {
    fn is_die_applicable(&self, sim: &dyn SimView, ctx: ModuleContext, dmg: &DamageInfo) -> Bool {
        self.core
            .is_die_applicable(&self.data.slow_death, sim, ctx.owner_slot, dmg)
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.die(sim, ctx, dmg);
    }
}

impl SlowDeathFacet for BattleBusSlowDeathBehavior {
    fn is_die_applicable(&self, sim: &dyn SimView, ctx: ModuleContext, dmg: &DamageInfo) -> Bool {
        self.core
            .is_die_applicable(&self.data.slow_death, sim, ctx.owner_slot, dmg)
    }

    fn get_probability_modifier(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Int {
        self.core
            .get_probability_modifier(&self.data.slow_death, sim, ctx.owner_slot, dmg)
    }

    fn begin_slow_death(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.begin(sim, ctx, dmg);
    }

    fn is_slow_death_in_progress(&self) -> Bool {
        self.core.is_slow_death_activated()
    }
}

impl UpdateFacet for BattleBusSlowDeathBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.tick(sim, ctx)
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

pub fn parse_battle_bus_slow_death_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = BattleBusSlowDeathBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    if data.slow_death.probability_modifier < 1 {
        return Err(IniError::InvalidData);
    }
    Ok(Box::new(data))
}

pub fn create_battle_bus_slow_death_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<BattleBusSlowDeathBehaviorModuleData>()
        .expect("BattleBusSlowDeathBehavior data type mismatch")
        .clone();
    Box::new(BattleBusSlowDeathBehavior::new(typed))
}
