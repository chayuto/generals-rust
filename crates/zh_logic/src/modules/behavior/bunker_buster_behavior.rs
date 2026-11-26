//! `PKT-263`: `BunkerBusterBehaviorModuleData` and `BunkerBusterBehavior`
//! (`BunkerBusterBehavior.h`, `BunkerBusterBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-263_BunkerBusterBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c3.rs`.
//!
//! Not modelled: the seismic simulation (`DO_SEISMIC_SIMULATIONS` is off in the reference build).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

const OCCUPANT_DAMAGE_AMOUNT: Real = 100.0;

/// INI template data matching `BunkerBusterBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct BunkerBusterBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub upgrade_required: String,
    pub detonation_fx: Option<String>,
    pub crash_through_bunker_fx: Option<String>,
    pub crash_through_bunker_fx_frequency: UnsignedInt,
    pub seismic_effect_radius: Real,
    pub seismic_effect_magnitude: Real,
    pub shockwave_weapon_template: String,
    pub occupant_damage_weapon_template: String,
}

impl BunkerBusterBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            upgrade_required: String::new(),
            detonation_fx: None,
            crash_through_bunker_fx: None,
            crash_through_bunker_fx_frequency: 4,
            seismic_effect_radius: 140.0,
            seismic_effect_magnitude: 6.0,
            shockwave_weapon_template: String::new(),
            occupant_damage_weapon_template: String::new(),
        }
    }
}

impl ModuleData for BunkerBusterBehaviorModuleData {
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
        FacetMask(0x003)
    }
}

/// Runtime instance of `BunkerBusterBehavior` (`PKT-263`).
pub struct BunkerBusterBehavior {
    pub data: BunkerBusterBehaviorModuleData,
    /// `m_upgradeRequired`, resolved when the object is created.
    pub upgrade_required: Option<UpgradeMaskType>,
    /// Not saved by the reference.
    pub victim_id: ObjectId,
    pub schedule: UpdateScheduleTracker,
}

impl BunkerBusterBehavior {
    pub fn new(data: BunkerBusterBehaviorModuleData) -> Self {
        Self {
            data,
            upgrade_required: None,
            victim_id: ObjectId::INVALID,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `bustTheBunker`.
    fn bust_the_bunker(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let owner = ctx.owner_slot;
        let Some((owner_id, team)) = sim.object(owner).map(|o| (o.id, o.team_id)) else {
            return;
        };
        if let Some(mask) = &self.upgrade_required {
            if sim.player_has_upgrade(team, mask) == FALSE {
                return;
            }
        }

        let target = sim.find_object(self.victim_id);
        let mut object_for_fx = owner;
        if let Some(target) = target {
            object_for_fx = target;
            if sim.contain_is_bustable(target) != FALSE {
                let occupant = sim
                    .rules()
                    .weapon_store
                    .find_weapon_template(&self.data.occupant_damage_weapon_template)
                    .map(|w| (w.damage_type, w.death_type));
                if let Some((damage_type, death_type)) = occupant {
                    let mut dmg = DamageInfo::default();
                    dmg.input.damage_type = damage_type;
                    dmg.input.death_type = death_type;
                    dmg.input.source_id = owner_id;
                    dmg.input.source_player_mask = sim.player_mask(team);
                    dmg.input.amount = OCCUPANT_DAMAGE_AMOUNT;
                    sim.contain_harm_and_force_exit_all(target, &dmg);
                } else {
                    sim.contain_kill_all(target);
                }
            }
        }

        let Some(pos) = sim.object(object_for_fx).map(|o| *o.thing.get_position()) else {
            return;
        };
        if let Some(fx) = &self.data.detonation_fx {
            sim.trigger_fx_list(fx, &pos);
        }
        let has_shockwave = sim
            .rules()
            .weapon_store
            .find_weapon_template(&self.data.shockwave_weapon_template)
            .is_some();
        if has_shockwave {
            sim.fire_weapon_template(
                object_for_fx,
                &self.data.shockwave_weapon_template,
                None,
                &pos,
            );
        }
    }
}

impl UpdateFacet for BunkerBusterBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        // A smart bomb remembers what its AI was aiming at.
        if sim.has_ai(owner) != FALSE {
            if self.victim_id == ObjectId::INVALID {
                let victim = sim
                    .ai_current_victim(owner)
                    .and_then(|v| sim.object(v))
                    .map(|o| o.id);
                if let Some(id) = victim {
                    self.victim_id = id;
                }
            }

            if sim
                .frame()
                .checked_rem(self.data.crash_through_bunker_fx_frequency)
                == Some(1)
            {
                let killing_self = sim.object(owner).is_some_and(|o| {
                    o.status.test(ObjectStatusType::MissileKillingSelf as Int) != FALSE
                });
                if killing_self {
                    let pos = sim.object(owner).map(|o| *o.thing.get_position());
                    if let (Some(fx), Some(pos)) = (&self.data.crash_through_bunker_fx, pos) {
                        sim.trigger_fx_list(fx, &pos);
                    }
                }
            }
        }
        UpdateSleepTime::NONE
    }
}

impl DieFacet for BunkerBusterBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        // Only bust the bunker if the missile killed itself by reaching its destination.
        #[cfg(not(feature = "retail-crc"))]
        if sim
            .object(ctx.owner_slot)
            .is_none_or(|o| o.status.test(ObjectStatusType::MissileKillingSelf as Int) == FALSE)
        {
            return;
        }
        self.bust_the_bunker(sim, ctx);
    }
}

impl BehaviorModule for BunkerBusterBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        let rules = sim.rules();
        self.upgrade_required = rules
            .upgrade_center
            .lookup_upgrade(&self.data.upgrade_required, &rules.name_keys)
            .map(|u| u.upgrade_mask().clone());
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)
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

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }
}

pub fn parse_bunker_buster_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = BunkerBusterBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("UpgradeRequired") {
            r.parse_ascii_string(&mut data.upgrade_required)
        } else if field.eq_ignore_ascii_case("DetonationFX") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.detonation_fx = resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if field.eq_ignore_ascii_case("CrashThroughBunkerFX") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.crash_through_bunker_fx =
                resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if field.eq_ignore_ascii_case("CrashThroughBunkerFXFrequency") {
            r.parse_duration_unsigned_int(&mut data.crash_through_bunker_fx_frequency)
        } else if field.eq_ignore_ascii_case("SeismicEffectRadius") {
            r.parse_real(&mut data.seismic_effect_radius)
        } else if field.eq_ignore_ascii_case("SeismicEffectMagnitude") {
            r.parse_real(&mut data.seismic_effect_magnitude)
        } else if field.eq_ignore_ascii_case("ShockwaveWeaponTemplate") {
            r.parse_ascii_string(&mut data.shockwave_weapon_template)
        } else if field.eq_ignore_ascii_case("OccupantDamageWeaponTemplate") {
            r.parse_ascii_string(&mut data.occupant_damage_weapon_template)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_bunker_buster_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<BunkerBusterBehaviorModuleData>()
        .expect("BunkerBusterBehavior data type mismatch")
        .clone();
    Box::new(BunkerBusterBehavior::new(typed))
}
