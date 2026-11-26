//! `PKT-278`: `JetSlowDeathBehavior` (`JetSlowDeathBehavior.h`, `.cpp`).
//!
//! Packet: `packets/phase5/PKT-278_JetSlowDeathBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_b.rs`.
//!
//! The looping death sound is client audio and is not modelled.

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    damage_info_from_die_op, resolve_optional_asset_name, xfer_module_version,
};
use crate::modules::behavior::slow_death_behavior::{SlowDeathBehaviorModuleData, SlowDeathCore};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, SlowDeathFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `JetSlowDeathBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct JetSlowDeathBehaviorModuleData {
    pub slow_death: SlowDeathBehaviorModuleData,
    pub fx_on_ground_death: Option<String>,
    pub ocl_on_ground_death: Option<String>,
    pub fx_initial_death: Option<String>,
    pub ocl_initial_death: Option<String>,
    pub delay_secondary_from_initial_death: UnsignedInt,
    pub fx_secondary: Option<String>,
    pub ocl_secondary: Option<String>,
    pub fx_hit_ground: Option<String>,
    pub ocl_hit_ground: Option<String>,
    pub delay_final_blow_up_from_hit_ground: UnsignedInt,
    pub fx_final_blow_up: Option<String>,
    pub ocl_final_blow_up: Option<String>,
    pub death_loop_sound: String,
    pub roll_rate: Real,
    pub roll_rate_delta: Real,
    pub pitch_rate: Real,
    pub fall_how_fast: Real,
}

impl JetSlowDeathBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            slow_death: SlowDeathBehaviorModuleData::new(tag_key, module_name_key),
            fx_on_ground_death: None,
            ocl_on_ground_death: None,
            fx_initial_death: None,
            ocl_initial_death: None,
            delay_secondary_from_initial_death: 0,
            fx_secondary: None,
            ocl_secondary: None,
            fx_hit_ground: None,
            ocl_hit_ground: None,
            delay_final_blow_up_from_hit_ground: 0,
            fx_final_blow_up: None,
            ocl_final_blow_up: None,
            death_loop_sound: String::new(),
            roll_rate: 0.0,
            roll_rate_delta: 1.0,
            pitch_rate: 0.0,
            fall_how_fast: 0.0,
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let fx = parse_ctx.fx_list_names;
        let ocl = parse_ctx.ocl_names;
        let asset_field = [
            ("FXOnGroundDeath", fx, &mut self.fx_on_ground_death),
            ("OCLOnGroundDeath", ocl, &mut self.ocl_on_ground_death),
            ("FXInitialDeath", fx, &mut self.fx_initial_death),
            ("OCLInitialDeath", ocl, &mut self.ocl_initial_death),
            ("FXSecondary", fx, &mut self.fx_secondary),
            ("OCLSecondary", ocl, &mut self.ocl_secondary),
            ("FXHitGround", fx, &mut self.fx_hit_ground),
            ("OCLHitGround", ocl, &mut self.ocl_hit_ground),
            ("FXFinalBlowUp", fx, &mut self.fx_final_blow_up),
            ("OCLFinalBlowUp", ocl, &mut self.ocl_final_blow_up),
        ]
        .into_iter()
        .find(|(name, _, _)| field.eq_ignore_ascii_case(name));
        if let Some((_, known, out)) = asset_field {
            return Some(reader.next_token(SEPS_DEFAULT).map(|tok| {
                *out = resolve_optional_asset_name(tok, known);
            }));
        }
        if field.eq_ignore_ascii_case("DelaySecondaryFromInitialDeath") {
            Some(reader.parse_duration_unsigned_int(&mut self.delay_secondary_from_initial_death))
        } else if field.eq_ignore_ascii_case("DelayFinalBlowUpFromHitGround") {
            Some(reader.parse_duration_unsigned_int(&mut self.delay_final_blow_up_from_hit_ground))
        } else if field.eq_ignore_ascii_case("DeathLoopSound") {
            Some(reader.parse_ascii_string(&mut self.death_loop_sound))
        } else if field.eq_ignore_ascii_case("RollRate") {
            Some(reader.parse_real(&mut self.roll_rate))
        } else if field.eq_ignore_ascii_case("RollRateDelta") {
            Some(reader.parse_percent_to_real(&mut self.roll_rate_delta))
        } else if field.eq_ignore_ascii_case("PitchRate") {
            Some(reader.parse_real(&mut self.pitch_rate))
        } else if field.eq_ignore_ascii_case("FallHowFast") {
            Some(reader.parse_percent_to_real(&mut self.fall_how_fast))
        } else {
            self.slow_death.parse_field(field, reader, parse_ctx)
        }
    }
}

impl ModuleData for JetSlowDeathBehaviorModuleData {
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

/// Runtime instance of `JetSlowDeathBehavior` (`PKT-278`).
pub struct JetSlowDeathBehavior {
    pub data: JetSlowDeathBehaviorModuleData,
    pub core: SlowDeathCore,
    /// Frame we died on.
    pub timer_death_frame: UnsignedInt,
    /// Frame we landed on the ground on.
    pub timer_on_ground_frame: UnsignedInt,
    pub roll_rate: Real,
}

impl JetSlowDeathBehavior {
    pub fn new(data: JetSlowDeathBehaviorModuleData) -> Self {
        Self {
            data,
            core: SlowDeathCore::new(),
            timer_death_frame: 0,
            timer_on_ground_frame: 0,
            roll_rate: 0.0,
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

    /// `JetSlowDeathBehavior::onDie`.
    fn die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        let us = ctx.owner_slot;
        let deck_offset = sim.object(us).is_some_and(|o| {
            o.status.test(ObjectStatusType::DeckHeightOffset.as_i32()) != FALSE
        });
        // A jet on the ground only does the ground death.
        if sim.object_is_significantly_above_terrain(us) == FALSE || deck_offset {
            Self::do_effects(
                sim,
                ctx,
                self.data.fx_on_ground_death.as_ref(),
                self.data.ocl_on_ground_death.as_ref(),
            );
            sim.destroy_object(us);
        } else if self.core.on_die(&self.data.slow_death, sim, ctx, dmg) != FALSE {
            self.begin(sim, ctx, dmg);
        }
        sim.clear_status(us, ObjectStatusType::DeckHeightOffset);
    }

    /// `JetSlowDeathBehavior::beginSlowDeath`.
    fn begin(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.core
            .begin_slow_death(&self.data.slow_death, sim, ctx, dmg);

        let us = ctx.owner_slot;
        self.timer_death_frame = sim.frame();
        Self::do_effects(
            sim,
            ctx,
            self.data.fx_initial_death.as_ref(),
            self.data.ocl_initial_death.as_ref(),
        );
        self.roll_rate = self.data.roll_rate;

        // Start falling, and do not allow the jet to turn anymore.
        let gravity = sim.rules().global_data.gravity;
        sim.set_locomotor_max_lift(us, -gravity * (1.0 - self.data.fall_how_fast));
        sim.set_locomotor_max_turn_rate(us, 0.0);
    }

    /// `JetSlowDeathBehavior::update`.
    fn tick(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let _ = self.core.update_core(&self.data.slow_death, sim, ctx);
        if self.core.is_slow_death_activated() == FALSE {
            return UpdateSleepTime::NONE;
        }

        let us = ctx.owner_slot;
        let has_physics = sim.has_physics(us) != FALSE;
        if has_physics {
            sim.physics_set_roll_rate(us, self.roll_rate);
        }
        self.roll_rate *= self.data.roll_rate_delta;

        if self.timer_on_ground_frame == 0 {
            let Some(pos) = sim.object(us).map(|o| *o.thing.get_position()) else {
                return UpdateSleepTime::NONE;
            };
            let layer = sim.get_layer_for_destination(&pos);
            sim.set_object_layer(us, layer);
            let height = if layer == PathfindLayerEnum::Ground {
                sim.object_height_above_terrain(us)
            } else {
                let h = pos.z - sim.get_layer_height(pos.x, pos.y, layer);
                // Slop for bridges: we tend to end up fractionally above them.
                if (0.0..=1.0).contains(&h) {
                    0.0
                } else {
                    h
                }
            };

            // Make sure we crash if we collide with a tree on the way down.
            let mut hit_a_tree = false;
            if has_physics {
                if let Some(tree) = sim.find_object(sim.physics_last_collidee(us)) {
                    hit_a_tree = sim.object_is_kind_of(tree, KindOfType::Shrubbery) != FALSE;
                }
            }

            if height <= 0.0 || hit_a_tree {
                Self::do_effects(
                    sim,
                    ctx,
                    self.data.fx_hit_ground.as_ref(),
                    self.data.ocl_hit_ground.as_ref(),
                );
                self.timer_on_ground_frame = sim.frame();
                if has_physics {
                    sim.physics_set_pitch_rate(us, self.data.pitch_rate);
                }
            }

            if self.timer_death_frame != 0
                && sim.frame().wrapping_sub(self.timer_death_frame)
                    >= self.data.delay_secondary_from_initial_death
            {
                Self::do_effects(
                    sim,
                    ctx,
                    self.data.fx_secondary.as_ref(),
                    self.data.ocl_secondary.as_ref(),
                );
                self.timer_death_frame = 0;
            }
        } else if sim.frame().wrapping_sub(self.timer_on_ground_frame)
            >= self.data.delay_final_blow_up_from_hit_ground
        {
            Self::do_effects(
                sim,
                ctx,
                self.data.fx_final_blow_up.as_ref(),
                self.data.ocl_final_blow_up.as_ref(),
            );
            sim.destroy_object(us);
        }

        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for JetSlowDeathBehavior {
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
        xfer.xfer_unsigned_int(&mut self.timer_death_frame)?;
        xfer.xfer_unsigned_int(&mut self.timer_on_ground_frame)?;
        xfer.xfer_real(&mut self.roll_rate)
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

impl DieFacet for JetSlowDeathBehavior {
    fn is_die_applicable(&self, sim: &dyn SimView, ctx: ModuleContext, dmg: &DamageInfo) -> Bool {
        self.core
            .is_die_applicable(&self.data.slow_death, sim, ctx.owner_slot, dmg)
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.die(sim, ctx, dmg);
    }
}

impl SlowDeathFacet for JetSlowDeathBehavior {
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

impl UpdateFacet for JetSlowDeathBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.tick(sim, ctx)
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

pub fn parse_jet_slow_death_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = JetSlowDeathBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    if data.slow_death.probability_modifier < 1 {
        return Err(IniError::InvalidData);
    }
    Ok(Box::new(data))
}

pub fn create_jet_slow_death_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<JetSlowDeathBehaviorModuleData>()
        .expect("JetSlowDeathBehavior data type mismatch")
        .clone();
    Box::new(JetSlowDeathBehavior::new(typed))
}
