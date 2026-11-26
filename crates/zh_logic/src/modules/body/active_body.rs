//! `PKT-165`: `ActiveBody` (`ActiveBody.h` / `ActiveBody.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar};
use crate::combat::armor::Armor;
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::body::body_module::BodyModuleBase;
use crate::modules::module::{
    BehaviorModule, BodyFacet, MaxHealthChangeType, ModuleData, INVALID_BODY_TIMESTAMP,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{
    is_subdual_damage, ArmorSetFlags, ArmorSetType, DamageType, ARMOR_SET_NAMES,
};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::{kindof_mask_fs, KindOfType};
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{armor_set_flags_to_u8, SimApi, SimView};
use zh_common::ini::{convert_duration_from_msecs_to_frames, IniError, IniReader};
use zh_common::math::{div_safe_f32, real_to_int_ceil};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UnsignedShort, VeterancyLevel,
    XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

const YELLOW_DAMAGE_PERCENT: Real = 0.25;

/// Template configuration parsed for `ActiveBody` and its subclasses (`ActiveBodyModuleData`).
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveBodyModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub max_health: Real,
    pub initial_health: Real,
    pub subdual_damage_cap: Real,
    pub subdual_damage_heal_rate: UnsignedInt,
    pub subdual_damage_heal_amount: Real,
}

impl ActiveBodyModuleData {
    pub const fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            max_health: 0.0,
            initial_health: 0.0,
            subdual_damage_cap: 0.0,
            subdual_damage_heal_rate: 0,
            subdual_damage_heal_amount: 0.0,
        }
    }

    /// Parses fields defined by `ActiveBodyModuleData::buildFieldParse`.
    /// Shared with `UndeadBodyModuleData::buildFieldParse`.
    pub fn parse_field(&mut self, field: &str, ini: &mut IniReader<'_>) -> Result<bool, IniError> {
        if field.eq_ignore_ascii_case("MaxHealth") {
            ini.parse_real(&mut self.max_health)?;
            Ok(true)
        } else if field.eq_ignore_ascii_case("InitialHealth") {
            ini.parse_real(&mut self.initial_health)?;
            Ok(true)
        } else if field.eq_ignore_ascii_case("SubdualDamageCap") {
            ini.parse_real(&mut self.subdual_damage_cap)?;
            Ok(true)
        } else if field.eq_ignore_ascii_case("SubdualDamageHealRate") {
            ini.parse_duration_unsigned_int(&mut self.subdual_damage_heal_rate)?;
            Ok(true)
        } else if field.eq_ignore_ascii_case("SubdualDamageHealAmount") {
            ini.parse_real(&mut self.subdual_damage_heal_amount)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl ModuleData for ActiveBodyModuleData {
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
        FacetMask::BODY
    }

    fn subdual_damage_heal_rate(&self) -> UnsignedInt {
        self.subdual_damage_heal_rate
    }

    fn subdual_damage_heal_amount(&self) -> Real {
        self.subdual_damage_heal_amount
    }
}

pub fn parse_active_body_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = ActiveBodyModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if data.parse_field(field, reader)? {
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

#[inline]
fn calc_damage_state(
    health: Real,
    max_health: Real,
    damaged_thresh: Real,
    really_damaged_thresh: Real,
) -> BodyDamageType {
    let ratio = health / max_health;
    if ratio > damaged_thresh {
        BodyDamageType::Pristine
    } else if ratio > really_damaged_thresh {
        BodyDamageType::Damaged
    } else if ratio > 0.0 {
        BodyDamageType::ReallyDamaged
    } else {
        BodyDamageType::Rubble
    }
}

/// Active damageable object body tracking health, armor, damage state, subdual damage,
/// and damage timestamps (`ActiveBody.h` / `ActiveBody.cpp`).
#[derive(Debug, Clone)]
pub struct ActiveBody {
    pub base: BodyModuleBase,
    pub subdual_damage_cap: Real,
    pub subdual_damage_heal_rate: UnsignedInt,
    pub subdual_damage_heal_amount: Real,
    pub is_immortal: bool,

    pub current_health: Real,
    pub prev_health: Real,
    pub max_health: Real,
    pub initial_health: Real,
    pub current_subdual_damage: Real,

    pub cur_damage_state: BodyDamageType,
    pub next_damage_fx_time: UnsignedInt,
    pub last_damage_fx_done: Int,
    pub last_damage_info: DamageInfo,
    pub last_damage_timestamp: UnsignedInt,
    pub last_healing_timestamp: UnsignedInt,
    pub front_crushed: Bool,
    pub back_crushed: Bool,
    pub last_damage_cleared: Bool,
    pub indestructible: Bool,

    pub particle_systems: Vec<UnsignedInt>,
    pub cur_armor_set_flags: ArmorSetFlags,
}

impl ActiveBody {
    pub fn from_active_data(
        module_name_key: NameKeyType,
        data: &ActiveBodyModuleData,
        is_immortal: bool,
    ) -> Self {
        Self {
            base: BodyModuleBase::new(module_name_key),
            subdual_damage_cap: data.subdual_damage_cap,
            subdual_damage_heal_rate: data.subdual_damage_heal_rate,
            subdual_damage_heal_amount: data.subdual_damage_heal_amount,
            is_immortal,
            current_health: data.initial_health,
            prev_health: data.initial_health,
            max_health: data.max_health,
            initial_health: data.initial_health,
            current_subdual_damage: 0.0,
            cur_damage_state: BodyDamageType::Pristine,
            next_damage_fx_time: 0,
            last_damage_fx_done: -1,
            last_damage_info: DamageInfo::default(),
            last_damage_timestamp: INVALID_BODY_TIMESTAMP,
            last_healing_timestamp: INVALID_BODY_TIMESTAMP,
            front_crushed: FALSE,
            back_crushed: FALSE,
            last_damage_cleared: FALSE,
            indestructible: FALSE,
            particle_systems: Vec::new(),
            cur_armor_set_flags: ArmorSetFlags::new(),
        }
    }

    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let abd = data
            .as_any()
            .downcast_ref::<ActiveBodyModuleData>()
            .expect("ActiveBodyModuleData");
        Box::new(Self::from_active_data(data.module_name_key(), abd, false))
    }

    #[inline]
    pub fn compute_sidecar(&self, subdued: Bool) -> ModuleSidecar {
        ModuleSidecar::Body {
            health: self.current_health,
            max_health: self.max_health,
            prev_health: self.prev_health,
            initial_health: self.initial_health,
            damage_scalar: self.base.damage_scalar,
            damage_state: self.cur_damage_state.as_i32() as u8,
            front_crushed: self.front_crushed,
            back_crushed: self.back_crushed,
            indestructible: self.indestructible,
            current_subdual_damage: self.current_subdual_damage,
            subdued,
            last_damage_ts: self.last_damage_timestamp,
            last_healing_ts: self.last_healing_timestamp,
            armor_set_bits: armor_set_flags_to_u8(&self.cur_armor_set_flags),
        }
    }

    #[inline]
    fn sync_sidecar_to_sim(&self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let sub = self.is_subdued(sim, ctx);
        sim.set_module_sidecar(ctx.mod_slot, self.compute_sidecar(sub));
    }

    fn resolve_armor_and_fx<'a>(
        &self,
        sim: &'a dyn SimView,
        ctx: ModuleContext,
    ) -> (Armor<'a>, Option<&'a str>) {
        let Some(tmpl) = sim
            .object(ctx.owner_slot)
            .and_then(|o| o.thing.template_override().get_final(&sim.rules().thing_factory))
        else {
            return (Armor::none(), None);
        };
        let Some(set) = tmpl.find_armor_template_set(&self.cur_armor_set_flags) else {
            return (Armor::none(), None);
        };
        let armor = Armor::new(set.get_armor_template());
        (armor, set.get_damage_fx_name())
    }

    pub fn set_correct_damage_state(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let gd = &sim.rules().global_data;
        self.cur_damage_state = calc_damage_state(
            self.current_health,
            self.max_health,
            gd.unit_damaged_thresh,
            gd.unit_really_damaged_thresh,
        );

        if self.cur_damage_state == BodyDamageType::Rubble {
            let rubble_info = sim.object(ctx.owner_slot).and_then(|o| {
                let tmpl = o
                    .thing
                    .template_override()
                    .get_final(&sim.rules().thing_factory)?;
                if tmpl.is_kind_of(KindOfType::Structure) == FALSE {
                    return None;
                }
                let tmpl_h = tmpl.get_structure_rubble_height();
                let gd_h = sim.rules().global_data.default_structure_rubble_height;
                Some(if tmpl_h <= 0.0 { gd_h } else { tmpl_h })
            });
            if let Some(rubble_height) = rubble_info {
                if let Some(obj) = sim.object_mut(ctx.owner_slot) {
                    obj.geometry.set_max_height_above_position(rubble_height);
                }
                sim.set_status(ctx.owner_slot, ObjectStatusType::NoCollisions);
            }
        }
    }

    fn do_damage_fx(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        let mut damage_type_to_use = dmg.input.damage_type;
        if dmg.input.damage_fx_override != DamageType::Unresistable {
            damage_type_to_use = dmg.input.damage_fx_override;
        }
        let fx_name_opt = {
            let (_, dfx_name) = self.resolve_armor_and_fx(sim, ctx);
            dfx_name.map(str::to_owned)
        };
        let Some(dfx_name) = fx_name_opt else {
            return;
        };
        let now = sim.frame();
        if damage_type_to_use.as_i32() == self.last_damage_fx_done && self.next_damage_fx_time > now
        {
            return;
        }
        let source_vet = sim
            .find_object(dmg.input.source_id)
            .and_then(|slot| sim.object(slot))
            .map(|o| o.experience_tracker.get_veterancy_level());

        let throttle_opt = {
            let rules = sim.rules();
            rules
                .name_keys
                .lookup_key(&dfx_name)
                .and_then(|k| rules.damage_fx_store.find_damage_fx(k))
                .map(|dfx| dfx.get_damage_fx_throttle_time(damage_type_to_use, source_vet))
        };
        let Some(throttle) = throttle_opt else {
            return;
        };

        self.last_damage_fx_done = damage_type_to_use.as_i32();
        self.next_damage_fx_time = now.wrapping_add(throttle);
        sim.trigger_damage_fx(
            &dfx_name,
            damage_type_to_use,
            dmg.output.actual_damage_dealt,
            dmg.input.source_id,
            ctx.owner_slot,
        );
    }

    fn on_subdual_change(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, is_now_subdued: Bool) {
        let is_projectile = sim
            .object(ctx.owner_slot)
            .and_then(|o| o.thing.template_override().get_final(&sim.rules().thing_factory))
            .is_some_and(|t| t.is_kind_of(KindOfType::Projectile) != FALSE);

        if !is_projectile {
            let (is_internet_center, contain_slot) = sim
                .object(ctx.owner_slot)
                .map(|o| {
                    let ic = o
                        .thing
                        .template_override()
                        .get_final(&sim.rules().thing_factory)
                        .is_some_and(|t| t.is_kind_of(KindOfType::FsInternetCenter) != FALSE);
                    (ic, o.contain_slot)
                })
                .unwrap_or((false, None));

            self.sync_sidecar_to_sim(sim, ctx);
            if is_now_subdued != FALSE {
                sim.set_disabled(ctx.owner_slot, DisabledType::Subdued);
                if let Some(cs) = contain_slot {
                    let _ = sim.dispatch_on_die_stub_noop(cs, |c, s, cc| {
                        c.order_all_passengers_to_idle(s, cc);
                    });
                }
            } else {
                sim.clear_disabled(ctx.owner_slot, DisabledType::Subdued);
                if is_internet_center {
                    if let Some(cs) = contain_slot {
                        let _ = sim.dispatch_on_die_stub_noop(cs, |c, s, cc| {
                            c.order_all_passengers_to_hack_internet(s, cc);
                        });
                    }
                }
            }
        }
    }

    fn is_priority_damager(&self, sim: &dyn SimView, id: ObjectId) -> bool {
        let Some(slot) = sim.find_object(id) else {
            return false;
        };
        let Some(tmpl) = sim
            .object(slot)
            .and_then(|o| o.thing.template_override().get_final(&sim.rules().thing_factory))
        else {
            return false;
        };
        let fs_mask = kindof_mask_fs();
        tmpl.is_kind_of(KindOfType::Vehicle) != FALSE
            || tmpl.is_kind_of(KindOfType::Infantry) != FALSE
            || tmpl.is_any_kind_of(&fs_mask) != FALSE
    }

    fn record_last_damage_info(&mut self, sim: &dyn SimView, dmg: &DamageInfo) {
        let now = sim.frame();
        if self.last_damage_timestamp != now && self.last_damage_timestamp != now.wrapping_sub(1) {
            self.last_damage_info = dmg.clone();
            self.last_damage_cleared = FALSE;
            self.last_damage_timestamp = now;
        } else {
            let src1 = sim.find_object(self.last_damage_info.input.source_id);
            let src2 = sim.find_object(dmg.input.source_id);
            if src2.is_some()
                && (src1.is_none() || self.is_priority_damager(sim, dmg.input.source_id))
            {
                self.last_damage_info = dmg.clone();
                self.last_damage_cleared = FALSE;
                self.last_damage_timestamp = now;
            }
        }
    }

    fn finish_health_damage(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dmg: &mut DamageInfo,
        applied_amount: Real,
        already_handled: bool,
        old_state: BodyDamageType,
    ) {
        if !already_handled {
            self.internal_change_health(sim, ctx, -applied_amount);
        }

        dmg.output.actual_damage_dealt = applied_amount;
        dmg.output.actual_damage_clipped = self.prev_health - self.current_health;

        self.record_last_damage_info(sim, dmg);

        if self.last_damage_info.input.source_id != ObjectId::INVALID {
            sim.record_attacked_by(ctx.owner_slot, self.last_damage_info.input.source_id);
        }

        self.sync_sidecar_to_sim(sim, ctx);

        if self.current_health < self.prev_health {
            sim.dispatch_on_damage(ctx.owner_slot, dmg);
        }

        if self.cur_damage_state != old_state {
            sim.dispatch_on_body_damage_state_change(
                ctx.owner_slot,
                dmg,
                old_state,
                self.cur_damage_state,
            );
        }

        if (self.prev_health / self.max_health) > YELLOW_DAMAGE_PERCENT
            && (self.current_health / self.max_health) < YELLOW_DAMAGE_PERCENT
            && self.current_health > 0.0
        {
            let _ = sim.rng_logic_int(0, 99);
        }

        if self.current_health <= 0.0 && self.prev_health > 0.0 {
            if sim.find_object(dmg.input.source_id).is_some() {
                sim.notify_object_killed(ctx.owner_slot, dmg.input.source_id, dmg);
            }
            sim.dispatch_on_die(ctx.owner_slot, dmg);
        }
    }
}

/// Extension helper avoiding exposing `ModuleArena` internals while calling into an optional
/// sibling `ContainFacet` slot (`SimApi` already exposes `contain_slot`).
trait SimContainForward {
    fn dispatch_on_die_stub_noop<F>(&mut self, contain_slot: crate::arena::ModuleSlot, f: F)
    where
        F: FnOnce(&mut dyn crate::modules::module::ContainFacet, &mut dyn SimApi, ModuleContext);
}

impl<T: SimApi + ?Sized> SimContainForward for T {
    fn dispatch_on_die_stub_noop<F>(&mut self, _contain_slot: crate::arena::ModuleSlot, _f: F)
    where
        F: FnOnce(&mut dyn crate::modules::module::ContainFacet, &mut dyn SimApi, ModuleContext),
    {
        // `ContainModule` lands in `WP-5.3`; passenger commands route via `SimApi` once ported.
    }
}

impl BehaviorModule for ActiveBody {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.set_correct_damage_state(sim, ctx);
        self.sync_sidecar_to_sim(sim, ctx);
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        match op {
            DeferredSelfOp::ChangeHealth { delta, .. } => {
                self.internal_change_health(sim, ctx, delta);
            }
            DeferredSelfOp::ApplyAdjustedDamage {
                amount,
                damage_type,
                death_type,
                damage_fx_override,
                damage_status_type,
                kill,
                source_id,
                source_player_mask,
            } => {
                let mut dmg = DamageInfo::default();
                dmg.input.amount = amount;
                if let Some(dt) = DamageType::from_i32(damage_type as Int) {
                    dmg.input.damage_type = dt;
                }
                if let Some(dth) = crate::rules::combat_types::DeathType::from_i32(death_type as Int)
                {
                    dmg.input.death_type = dth;
                }
                if let Some(fxo) = DamageType::from_i32(damage_fx_override as Int) {
                    dmg.input.damage_fx_override = fxo;
                }
                if let Some(st) = ObjectStatusType::from_i32(damage_status_type as Int) {
                    dmg.input.damage_status_type = st;
                }
                dmg.input.kill = kill;
                dmg.input.source_id = source_id;
                dmg.input.source_player_mask = source_player_mask;

                let old_state = self.cur_damage_state;
                let applied = if kill != FALSE {
                    self.current_health
                } else {
                    amount
                };
                self.finish_health_damage(sim, ctx, &mut dmg, applied, false, old_state);
                self.do_damage_fx(sim, ctx, &dmg);
            }
            DeferredSelfOp::AddSubdualDamage(delta) => {
                self.internal_add_subdual_damage(delta);
                self.sync_sidecar_to_sim(sim, ctx);
            }
            DeferredSelfOp::SetMaxHealth {
                new_max,
                change_type,
            } => {
                let ct = MaxHealthChangeType::from_i32(change_type as Int).unwrap_or_default();
                self.set_max_health(sim, ctx, new_max, ct);
            }
            DeferredSelfOp::SetIndestructible(v) => {
                self.set_indestructible(sim, ctx, v);
            }
            DeferredSelfOp::ApplyDamageScalar(scalar) => {
                self.apply_damage_scalar(scalar);
            }
            DeferredSelfOp::SetCrushed { front, back } => {
                if let Some(f) = front {
                    self.front_crushed = f;
                }
                if let Some(b) = back {
                    self.back_crushed = b;
                }
            }
            DeferredSelfOp::SetArmorSetFlag { flag, set } => {
                if set != FALSE {
                    self.set_armor_set_flag(flag);
                } else {
                    self.clear_armor_set_flag(flag);
                }
                self.sync_sidecar_to_sim(sim, ctx);
            }
            _ => {}
        }
    }

    fn sync_sidecar(&self) -> ModuleSidecar {
        #[cfg(feature = "retail-crc")]
        let subdued = Bool::from(self.max_health <= self.current_subdual_damage);
        #[cfg(not(feature = "retail-crc"))]
        let subdued = Bool::from(self.max_health <= self.current_subdual_damage);
        self.compute_sidecar(subdued)
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer_base(xfer)?;

        xfer.xfer_real(&mut self.current_health)?;
        xfer.xfer_real(&mut self.current_subdual_damage)?;
        xfer.xfer_real(&mut self.prev_health)?;
        xfer.xfer_real(&mut self.max_health)?;
        xfer.xfer_real(&mut self.initial_health)?;

        let mut ds_raw = self.cur_damage_state.as_i32();
        xfer.xfer_enum_i32(&mut ds_raw)?;
        self.cur_damage_state = BodyDamageType::from_i32(ds_raw).ok_or(XferError::InvalidData)?;

        xfer.xfer_unsigned_int(&mut self.next_damage_fx_time)?;
        xfer.xfer_enum_i32(&mut self.last_damage_fx_done)?;

        self.last_damage_info.xfer_with_lookup(xfer, |name| {
            sim.rules().thing_factory.find_template(name).is_some()
        })?;

        xfer.xfer_unsigned_int(&mut self.last_damage_timestamp)?;
        xfer.xfer_unsigned_int(&mut self.last_healing_timestamp)?;
        xfer.xfer_bool(&mut self.front_crushed)?;
        xfer.xfer_bool(&mut self.back_crushed)?;
        xfer.xfer_bool(&mut self.last_damage_cleared)?;
        xfer.xfer_bool(&mut self.indestructible)?;

        let mut ps_count: UnsignedShort = self.particle_systems.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut ps_count)?;
        if xfer.mode() == XferMode::Save || xfer.mode() == XferMode::Crc {
            for ps_id in &mut self.particle_systems {
                xfer.xfer_unsigned_int(ps_id)?;
            }
        } else {
            if !self.particle_systems.is_empty() {
                return Err(XferError::ListNotEmpty);
            }
            for _ in 0..ps_count {
                let mut id: UnsignedInt = 0;
                xfer.xfer_unsigned_int(&mut id)?;
                self.particle_systems.push(id);
            }
        }

        self.cur_armor_set_flags.xfer_named(xfer, &ARMOR_SET_NAMES)?;
        Ok(())
    }

    fn load_post_process(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}

    fn as_body(&self) -> Option<&dyn BodyFacet> {
        Some(self)
    }

    fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> {
        Some(self)
    }
}

impl BodyFacet for ActiveBody {
    fn estimate_damage(&self, sim: &dyn SimView, ctx: ModuleContext, dmg: &DamageInfo) -> Real {
        let (armor, _) = self.resolve_armor_and_fx(sim, ctx);

        if is_subdual_damage(dmg.input.damage_type.as_i32()) && self.can_be_subdued() == FALSE {
            return 0.0;
        }

        if dmg.input.damage_type == DamageType::KillGarrisoned {
            if let Some(cs) = sim.object(ctx.owner_slot).and_then(|o| o.contain_slot) {
                if let Some(c) = sim.module_sidecar_contain(cs) {
                    if c.count > 0 {
                        return 1.0;
                    }
                }
            }
            return 0.0;
        }

        if dmg.input.damage_type == DamageType::Sniper {
            if let Some(obj) = sim.object(ctx.owner_slot) {
                let is_struct = obj
                    .thing
                    .template_override()
                    .get_final(&sim.rules().thing_factory)
                    .is_some_and(|t| t.is_kind_of(KindOfType::Structure) != FALSE);
                if is_struct && obj.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE
                {
                    return 0.0;
                }
            }
        }

        armor.adjust_damage(dmg.input.damage_type, dmg.input.amount)
    }

    fn attempt_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        if self.indestructible != FALSE {
            return;
        }
        dmg.output.actual_damage_dealt = 0.0;
        dmg.output.actual_damage_clipped = 0.0;

        let Some(obj_ref) = sim.object(ctx.owner_slot) else {
            return;
        };
        if obj_ref.is_effectively_dead() != FALSE {
            return;
        }
        let is_vehicle = obj_ref
            .thing
            .template_override()
            .get_final(&sim.rules().thing_factory)
            .is_some_and(|t| t.is_kind_of(KindOfType::Vehicle) != FALSE);
        let ai_slot = obj_ref.ai_slot;

        if let Some(d_slot) = sim.find_object(dmg.input.source_id) {
            if let Some(d_tmpl) = sim
                .object(d_slot)
                .and_then(|o| o.thing.template_override().get_final(&sim.rules().thing_factory))
            {
                dmg.input.source_template = d_tmpl.get_name().to_owned();
            }
        }

        let mut already_handled = false;
        let mut allow_modifier = true;
        let mut amount = {
            let (armor, _) = self.resolve_armor_and_fx(sim, ctx);
            armor.adjust_damage(dmg.input.damage_type, dmg.input.amount)
        };

        match dmg.input.damage_type {
            DamageType::Healing => {
                if dmg.input.kill == FALSE {
                    self.attempt_healing(sim, ctx, dmg);
                }
                return;
            }
            DamageType::KillPilot => {
                if is_vehicle {
                    self.sync_sidecar_to_sim(sim, ctx);
                    sim.set_disabled(ctx.owner_slot, DisabledType::Unmanned);
                    if let Some(_ai) = ai_slot {
                        // AI idle dispatched when AIUpdateModule ports in WP-5.2+.
                    }
                }
                already_handled = true;
                allow_modifier = false;
            }
            DamageType::KillGarrisoned => {
                already_handled = true;
                allow_modifier = false;
            }
            DamageType::Status => {
                let real_frames = convert_duration_from_msecs_to_frames(amount);
                let frames_ceil = real_to_int_ceil(real_frames) as Real;
                self.sync_sidecar_to_sim(sim, ctx);
                sim.trigger_status_damage(ctx.owner_slot, dmg.input.damage_status_type, frames_ceil);
                already_handled = true;
                allow_modifier = false;
            }
            _ => {}
        }

        if is_subdual_damage(dmg.input.damage_type.as_i32()) {
            if self.can_be_subdued() == FALSE {
                return;
            }
            let was_subdued = self.is_subdued(sim, ctx);
            self.internal_add_subdual_damage(amount);
            let now_subdued = Bool::from(self.max_health <= self.current_subdual_damage);
            already_handled = true;
            allow_modifier = false;

            if was_subdued != now_subdued {
                self.on_subdual_change(sim, ctx, now_subdued);
            }
            self.sync_sidecar_to_sim(sim, ctx);
            sim.notify_subdual_damage(ctx.owner_slot, amount);
        }

        if allow_modifier && dmg.input.damage_type != DamageType::Unresistable {
            amount *= self.base.damage_scalar;
        }

        if amount > 0.0 || dmg.input.kill != FALSE {
            let old_state = self.cur_damage_state;
            if dmg.input.kill != FALSE {
                amount = self.current_health;
            }
            self.finish_health_damage(sim, ctx, dmg, amount, already_handled, old_state);
        }

        self.do_damage_fx(sim, ctx, dmg);

        if sim.ai_enable_repulsors() != FALSE {
            let can_repulse = sim
                .object(ctx.owner_slot)
                .and_then(|o| o.thing.template_override().get_final(&sim.rules().thing_factory))
                .is_some_and(|t| t.is_kind_of(KindOfType::CanBeRepulsed) != FALSE);
            if can_repulse {
                sim.set_status(ctx.owner_slot, ObjectStatusType::Repulsor);
            }
        }
    }

    fn attempt_healing(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        healing: &mut DamageInfo,
    ) {
        if healing.input.damage_type != DamageType::Healing {
            self.attempt_damage(sim, ctx, healing);
            return;
        }

        let Some(obj) = sim.object(ctx.owner_slot) else {
            return;
        };
        let (is_bridge, is_tower) = obj
            .thing
            .template_override()
            .get_final(&sim.rules().thing_factory)
            .map(|t| {
                (
                    t.is_kind_of(KindOfType::Bridge) != FALSE,
                    t.is_kind_of(KindOfType::BridgeTower) != FALSE,
                )
            })
            .unwrap_or((false, false));
        if !is_bridge && !is_tower && obj.is_effectively_dead() != FALSE {
            return;
        }

        healing.output.actual_damage_dealt = 0.0;
        healing.output.actual_damage_clipped = 0.0;

        let amount = {
            let (armor, _) = self.resolve_armor_and_fx(sim, ctx);
            armor.adjust_damage(healing.input.damage_type, healing.input.amount)
        };

        if amount > 0.0 {
            let old_state = self.cur_damage_state;
            self.internal_change_health(sim, ctx, amount);

            healing.output.actual_damage_dealt = amount;
            healing.output.actual_damage_clipped = self.prev_health - self.current_health;

            self.last_damage_info = healing.clone();
            self.last_damage_cleared = FALSE;
            self.last_healing_timestamp = sim.frame();

            self.sync_sidecar_to_sim(sim, ctx);

            if self.current_health > self.prev_health {
                sim.dispatch_on_healing(ctx.owner_slot, healing);
            }

            if self.cur_damage_state != old_state {
                sim.dispatch_on_body_damage_state_change(
                    ctx.owner_slot,
                    healing,
                    old_state,
                    self.cur_damage_state,
                );
            }
        }

        self.do_damage_fx(sim, ctx, healing);
    }

    fn get_health(&self) -> Real {
        self.current_health
    }

    fn get_max_health(&self) -> Real {
        self.max_health
    }

    fn get_previous_health(&self) -> Real {
        self.prev_health
    }

    fn get_initial_health(&self) -> Real {
        self.initial_health
    }

    fn set_initial_health(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        initial_percent: Int,
    ) {
        self.prev_health = self.current_health;
        let factor = (initial_percent as Real) / 100.0;
        let new_health = factor * self.initial_health;
        self.internal_change_health(sim, ctx, new_health - self.current_health);
    }

    fn set_max_health(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_max: Real,
        change_type: MaxHealthChangeType,
    ) {
        let prev_max_health = self.max_health;
        self.max_health = new_max;
        self.initial_health = new_max;

        match change_type {
            MaxHealthChangeType::PreserveRatio => {
                let ratio = div_safe_f32(self.current_health, prev_max_health);
                let new_health = new_max * ratio;
                self.internal_change_health(sim, ctx, new_health - self.current_health);
            }
            MaxHealthChangeType::AddCurrentHealthToo => {
                self.internal_change_health(sim, ctx, new_max - prev_max_health);
            }
            MaxHealthChangeType::SameCurrentHealth => {}
            MaxHealthChangeType::FullyHeal => {
                self.internal_change_health(sim, ctx, self.max_health - self.current_health);
            }
        }

        if self.current_health > new_max {
            self.internal_change_health(sim, ctx, new_max - self.current_health);
        } else {
            self.sync_sidecar_to_sim(sim, ctx);
        }
    }

    fn get_damage_state(&self) -> BodyDamageType {
        self.cur_damage_state
    }

    fn set_damage_state(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_state: BodyDamageType,
    ) {
        let ratio = match new_state {
            BodyDamageType::Pristine => 1.0,
            BodyDamageType::Damaged => sim.rules().global_data.unit_damaged_thresh,
            BodyDamageType::ReallyDamaged => sim.rules().global_data.unit_really_damaged_thresh,
            BodyDamageType::Rubble => 0.0,
        };
        let mut desired_health = self.max_health * ratio - 1.0;
        if desired_health < 0.0 {
            desired_health = 0.0;
        }
        self.internal_change_health(sim, ctx, desired_health - self.current_health);
        self.set_correct_damage_state(sim, ctx);
        self.sync_sidecar_to_sim(sim, ctx);
    }

    fn set_aflame(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _setting: Bool) {
        self.update_body_particle_systems(sim, ctx);
    }

    fn get_damage_scalar(&self) -> Real {
        self.base.get_damage_scalar()
    }

    fn apply_damage_scalar(&mut self, scalar: Real) {
        self.base.apply_damage_scalar(scalar);
    }

    fn is_indestructible(&self) -> Bool {
        self.indestructible
    }

    fn set_indestructible(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, indestructible: Bool) {
        self.indestructible = indestructible;
        self.sync_sidecar_to_sim(sim, ctx);
    }

    fn internal_change_health(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, mut delta: Real) {
        if self.is_immortal {
            let floor_delta = -self.current_health + 1.0;
            if floor_delta > delta {
                delta = floor_delta;
            }
        }

        self.prev_health = self.current_health;
        self.current_health += delta;

        if self.current_health > self.max_health {
            self.current_health = self.max_health;
        }
        if self.current_health < 0.0 {
            self.current_health = 0.0;
        }

        let old_state = self.cur_damage_state;
        self.set_correct_damage_state(sim, ctx);

        if self.cur_damage_state != old_state {
            let under_construction = sim.object(ctx.owner_slot).is_some_and(|o| {
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE
            });
            if !under_construction {
                self.evaluate_visual_condition(sim, ctx);
            }
        }

        let dead = Bool::from(self.current_health <= 0.0);
        if let Some(obj) = sim.object_mut(ctx.owner_slot) {
            obj.set_effectively_dead(dead);
        }
        self.sync_sidecar_to_sim(sim, ctx);
    }

    fn get_front_crushed(&self) -> Bool {
        self.front_crushed
    }

    fn get_back_crushed(&self) -> Bool {
        self.back_crushed
    }

    fn set_front_crushed(&mut self, crushed: Bool) {
        self.front_crushed = crushed;
    }

    fn set_back_crushed(&mut self, crushed: Bool) {
        self.back_crushed = crushed;
    }

    fn set_armor_set_flag(&mut self, ast: ArmorSetType) {
        self.cur_armor_set_flags.set(ast.as_i32(), 1);
    }

    fn clear_armor_set_flag(&mut self, ast: ArmorSetType) {
        self.cur_armor_set_flags.set(ast.as_i32(), 0);
    }

    fn test_armor_set_flag(&self, ast: ArmorSetType) -> Bool {
        self.cur_armor_set_flags.test(ast.as_i32())
    }

    fn get_last_damage_info(&self) -> Option<&DamageInfo> {
        Some(&self.last_damage_info)
    }

    fn get_last_damage_timestamp(&self) -> Option<UnsignedInt> {
        #[cfg(feature = "retail-crc")]
        {
            Some(self.last_damage_timestamp)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            (self.last_damage_timestamp != INVALID_BODY_TIMESTAMP)
                .then_some(self.last_damage_timestamp)
        }
    }

    fn get_last_healing_timestamp(&self) -> Option<UnsignedInt> {
        #[cfg(feature = "retail-crc")]
        {
            Some(self.last_healing_timestamp)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            (self.last_healing_timestamp != INVALID_BODY_TIMESTAMP)
                .then_some(self.last_healing_timestamp)
        }
    }

    fn get_clearable_last_attacker(&self) -> ObjectId {
        if self.last_damage_cleared != FALSE {
            ObjectId::INVALID
        } else {
            self.last_damage_info.input.source_id
        }
    }

    fn clear_last_attacker(&mut self) {
        self.last_damage_cleared = TRUE;
    }

    fn get_current_subdual_damage(&self) -> Real {
        self.current_subdual_damage
    }

    fn get_subdual_damage_cap(&self) -> Real {
        self.subdual_damage_cap
    }

    fn get_subdual_damage_heal_rate(&self) -> UnsignedInt {
        self.subdual_damage_heal_rate
    }

    fn get_subdual_damage_heal_amount(&self) -> Real {
        self.subdual_damage_heal_amount
    }

    fn has_any_subdual_damage(&self) -> Bool {
        Bool::from(self.current_subdual_damage > 0.0)
    }

    fn is_subdued(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        #[cfg(feature = "retail-crc")]
        {
            let _ = (sim, ctx);
            Bool::from(self.max_health <= self.current_subdual_damage)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            let Some(obj) = sim.object(ctx.owner_slot) else {
                return FALSE;
            };
            let is_proj = obj
                .thing
                .template_override()
                .get_final(&sim.rules().thing_factory)
                .is_some_and(|t| t.is_kind_of(KindOfType::Projectile) != FALSE);
            if is_proj {
                Bool::from(self.max_health <= self.current_subdual_damage)
            } else {
                obj.is_disabled_by_type(DisabledType::Subdued)
            }
        }
    }

    fn can_be_subdued(&self) -> Bool {
        Bool::from(self.subdual_damage_cap > 0.0)
    }

    fn internal_add_subdual_damage(&mut self, delta: Real) {
        self.current_subdual_damage += delta;
        #[cfg(feature = "retail-crc")]
        {
            if self.current_subdual_damage > self.subdual_damage_cap {
                self.current_subdual_damage = self.subdual_damage_cap;
            }
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            if self.current_subdual_damage < 0.0 {
                self.current_subdual_damage = 0.0;
            } else if self.current_subdual_damage > self.subdual_damage_cap {
                self.current_subdual_damage = self.subdual_damage_cap;
            }
        }
    }

    fn on_veterancy_level_changed(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        old_level: VeterancyLevel,
        new_level: VeterancyLevel,
        _provide_feedback: Bool,
    ) {
        if old_level == new_level {
            return;
        }

        let old_idx = old_level.as_i32();
        let new_idx = new_level.as_i32();
        if (0..crate::gamelogic::global_data::LEVEL_COUNT as Int).contains(&old_idx)
            && (0..crate::gamelogic::global_data::LEVEL_COUNT as Int).contains(&new_idx)
        {
            let old_bonus = sim.rules().global_data.health_bonus[old_idx as usize];
            let new_bonus = sim.rules().global_data.health_bonus[new_idx as usize];
            let mult = new_bonus / old_bonus;
            self.set_max_health(
                sim,
                ctx,
                self.max_health * mult,
                MaxHealthChangeType::PreserveRatio,
            );
        }

        match new_level {
            VeterancyLevel::Regular => {
                self.clear_armor_set_flag(ArmorSetType::Veteran);
                self.clear_armor_set_flag(ArmorSetType::Elite);
                self.clear_armor_set_flag(ArmorSetType::Hero);
            }
            VeterancyLevel::Veteran => {
                self.set_armor_set_flag(ArmorSetType::Veteran);
                self.clear_armor_set_flag(ArmorSetType::Elite);
                self.clear_armor_set_flag(ArmorSetType::Hero);
            }
            VeterancyLevel::Elite => {
                self.clear_armor_set_flag(ArmorSetType::Veteran);
                self.set_armor_set_flag(ArmorSetType::Elite);
                self.clear_armor_set_flag(ArmorSetType::Hero);
            }
            VeterancyLevel::Heroic => {
                self.clear_armor_set_flag(ArmorSetType::Veteran);
                self.clear_armor_set_flag(ArmorSetType::Elite);
                self.set_armor_set_flag(ArmorSetType::Hero);
            }
            VeterancyLevel::Invalid => {}
        }
        self.sync_sidecar_to_sim(sim, ctx);
    }

    fn evaluate_visual_condition(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        // `Drawable::reactToBodyDamageStateChange`: these flags are saved and read by logic.
        let mut damage_flags = ModelConditionFlags::default();
        for flag in [
            ModelConditionFlagType::Damaged,
            ModelConditionFlagType::ReallyDamaged,
            ModelConditionFlagType::Rubble,
        ] {
            damage_flags.set(flag.as_i32(), 1);
        }
        let flag = match self.cur_damage_state {
            BodyDamageType::Pristine => None,
            BodyDamageType::Damaged => Some(ModelConditionFlagType::Damaged),
            BodyDamageType::ReallyDamaged => Some(ModelConditionFlagType::ReallyDamaged),
            BodyDamageType::Rubble => Some(ModelConditionFlagType::Rubble),
        };
        let mut new_damage = ModelConditionFlags::default();
        if let Some(flag) = flag {
            new_damage.set(flag.as_i32(), 1);
        }
        sim.clear_and_set_model_condition_flags(ctx.owner_slot, &damage_flags, &new_damage);
        self.update_body_particle_systems(sim, ctx);
    }

    fn update_body_particle_systems(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
}

/// Read-only contain sidecar inspection helper on `SimView` for `ActiveBody::estimate_damage`.
trait SimContainSidecar {
    fn module_sidecar_contain(&self, slot: crate::arena::ModuleSlot) -> Option<ContainSidecarView>;
}

struct ContainSidecarView {
    count: u32,
}

impl<T: SimView + ?Sized> SimContainSidecar for T {
    fn module_sidecar_contain(&self, slot: crate::arena::ModuleSlot) -> Option<ContainSidecarView> {
        match self.module_sidecar(slot) {
            ModuleSidecar::Contain { count, .. } => Some(ContainSidecarView { count }),
            _ => None,
        }
    }
}

/// Forwards every `BodyFacet` method to a wrapped [`ActiveBody`] field, avoiding duplication across
/// `HighlanderBody`, `ImmortalBody`, `StructureBody`, `HiveStructureBody`, and `UndeadBody`.
#[macro_export]
macro_rules! delegate_body_facet_to_inner {
    ($inner:ident) => {
        fn attempt_healing(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
            healing: &mut $crate::combat::damage::DamageInfo,
        ) {
            self.$inner.attempt_healing(sim, ctx, healing);
        }

        fn estimate_damage(
            &self,
            sim: &dyn $crate::sim::SimView,
            ctx: $crate::arena::ModuleContext,
            dmg: &$crate::combat::damage::DamageInfo,
        ) -> zh_common::types::Real {
            self.$inner.estimate_damage(sim, ctx, dmg)
        }

        fn get_health(&self) -> zh_common::types::Real {
            self.$inner.get_health()
        }

        fn get_max_health(&self) -> zh_common::types::Real {
            self.$inner.get_max_health()
        }

        fn get_previous_health(&self) -> zh_common::types::Real {
            self.$inner.get_previous_health()
        }

        fn get_initial_health(&self) -> zh_common::types::Real {
            self.$inner.get_initial_health()
        }

        fn set_initial_health(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
            initial_percent: zh_common::types::Int,
        ) {
            self.$inner.set_initial_health(sim, ctx, initial_percent);
        }

        fn set_max_health(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
            new_max: zh_common::types::Real,
            change_type: $crate::modules::module::MaxHealthChangeType,
        ) {
            self.$inner.set_max_health(sim, ctx, new_max, change_type);
        }

        fn get_damage_state(&self) -> $crate::gamelogic::global_data::BodyDamageType {
            self.$inner.get_damage_state()
        }

        fn set_damage_state(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
            new_state: $crate::gamelogic::global_data::BodyDamageType,
        ) {
            self.$inner.set_damage_state(sim, ctx, new_state);
        }

        fn set_aflame(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
            setting: zh_common::types::Bool,
        ) {
            self.$inner.set_aflame(sim, ctx, setting);
        }

        fn get_damage_scalar(&self) -> zh_common::types::Real {
            self.$inner.get_damage_scalar()
        }

        fn apply_damage_scalar(&mut self, scalar: zh_common::types::Real) {
            self.$inner.apply_damage_scalar(scalar);
        }

        fn is_indestructible(&self) -> zh_common::types::Bool {
            self.$inner.is_indestructible()
        }

        fn set_indestructible(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
            indestructible: zh_common::types::Bool,
        ) {
            self.$inner.set_indestructible(sim, ctx, indestructible);
        }

        fn internal_change_health(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
            delta: zh_common::types::Real,
        ) {
            self.$inner.internal_change_health(sim, ctx, delta);
        }

        fn get_front_crushed(&self) -> zh_common::types::Bool {
            self.$inner.get_front_crushed()
        }

        fn get_back_crushed(&self) -> zh_common::types::Bool {
            self.$inner.get_back_crushed()
        }

        fn set_front_crushed(&mut self, crushed: zh_common::types::Bool) {
            self.$inner.set_front_crushed(crushed);
        }

        fn set_back_crushed(&mut self, crushed: zh_common::types::Bool) {
            self.$inner.set_back_crushed(crushed);
        }

        fn set_armor_set_flag(&mut self, ast: $crate::rules::combat_types::ArmorSetType) {
            self.$inner.set_armor_set_flag(ast);
        }

        fn clear_armor_set_flag(&mut self, ast: $crate::rules::combat_types::ArmorSetType) {
            self.$inner.clear_armor_set_flag(ast);
        }

        fn test_armor_set_flag(
            &self,
            ast: $crate::rules::combat_types::ArmorSetType,
        ) -> zh_common::types::Bool {
            self.$inner.test_armor_set_flag(ast)
        }

        fn get_last_damage_info(&self) -> Option<&$crate::combat::damage::DamageInfo> {
            self.$inner.get_last_damage_info()
        }

        fn get_last_damage_timestamp(&self) -> Option<zh_common::types::UnsignedInt> {
            self.$inner.get_last_damage_timestamp()
        }

        fn get_last_healing_timestamp(&self) -> Option<zh_common::types::UnsignedInt> {
            self.$inner.get_last_healing_timestamp()
        }

        fn get_clearable_last_attacker(&self) -> zh_common::types::ObjectId {
            self.$inner.get_clearable_last_attacker()
        }

        fn clear_last_attacker(&mut self) {
            self.$inner.clear_last_attacker();
        }

        fn get_current_subdual_damage(&self) -> zh_common::types::Real {
            self.$inner.get_current_subdual_damage()
        }

        fn get_subdual_damage_cap(&self) -> zh_common::types::Real {
            self.$inner.get_subdual_damage_cap()
        }

        fn get_subdual_damage_heal_rate(&self) -> zh_common::types::UnsignedInt {
            self.$inner.get_subdual_damage_heal_rate()
        }

        fn get_subdual_damage_heal_amount(&self) -> zh_common::types::Real {
            self.$inner.get_subdual_damage_heal_amount()
        }

        fn has_any_subdual_damage(&self) -> zh_common::types::Bool {
            self.$inner.has_any_subdual_damage()
        }

        fn is_subdued(
            &self,
            sim: &dyn $crate::sim::SimView,
            ctx: $crate::arena::ModuleContext,
        ) -> zh_common::types::Bool {
            self.$inner.is_subdued(sim, ctx)
        }

        fn can_be_subdued(&self) -> zh_common::types::Bool {
            self.$inner.can_be_subdued()
        }

        fn internal_add_subdual_damage(&mut self, delta: zh_common::types::Real) {
            self.$inner.internal_add_subdual_damage(delta);
        }

        fn on_veterancy_level_changed(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
            old_level: zh_common::types::VeterancyLevel,
            new_level: zh_common::types::VeterancyLevel,
            provide_feedback: zh_common::types::Bool,
        ) {
            self.$inner
                .on_veterancy_level_changed(sim, ctx, old_level, new_level, provide_feedback);
        }

        fn evaluate_visual_condition(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
        ) {
            self.$inner.evaluate_visual_condition(sim, ctx);
        }

        fn update_body_particle_systems(
            &mut self,
            sim: &mut dyn $crate::sim::SimApi,
            ctx: $crate::arena::ModuleContext,
        ) {
            self.$inner.update_body_particle_systems(sim, ctx);
        }
    };
}
