//! `WP-5.1-A`: Internal `ObjectHelper` modules (`ObjectHelper`, `ObjectDefectionHelper`,
//! `ObjectRepulsorHelper`, `ObjectSMCHelper`, `ObjectWeaponStatusHelper`,
//! `StatusDamageHelper`, `SubdualDamageHelper`, `TempWeaponBonusHelper`).

pub mod object_defection_helper;
pub mod object_helper;
pub mod object_repulsor_helper;
pub mod object_smc_helper;
pub mod object_weapon_status_helper;
pub mod status_damage_helper;
pub mod subdual_damage_helper;
pub mod temp_weapon_bonus_helper;

pub use object_defection_helper::{ObjectDefectionHelper, DEFECTION_DETECTION_TIME_MAX};
pub use object_helper::{ObjectHelper, ObjectHelperBase};
pub use object_repulsor_helper::ObjectRepulsorHelper;
pub use object_smc_helper::ObjectSMCHelper;
pub use object_weapon_status_helper::ObjectWeaponStatusHelper;
pub use status_damage_helper::StatusDamageHelper;
pub use subdual_damage_helper::SubdualDamageHelper;
pub use temp_weapon_bonus_helper::{TempWeaponBonusHelper, WEAPON_BONUS_CONDITION_INVALID};

use crate::arena::{FacetMask, ModuleDataId, ObjectSlot};
use crate::combat::firing_tracker::FiringTracker;
use crate::rules::kind_of::KindOfType;
use crate::scheduler::SleepyUpdatePhase;
use crate::sim::{SimApi, SimContext};
use zh_common::types::{UpdateSleepTime, FALSE};

/// Attaches built-in object helper modules onto `obj_slot` in exact C++ `Object::Object` order
/// prior to INI behavior instantiation (`WP-5.1-A`).
pub fn attach_object_helpers(sim: &mut SimContext, obj_slot: ObjectSlot) {
    let tmpl_opt = sim
        .world
        .objects
        .get(obj_slot)
        .and_then(|o| o.thing.template_override().get_final(&sim.rules.thing_factory));

    let (has_inactive_body, can_be_repulsed, is_shrubbery, can_weapon) = match tmpl_opt {
        Some(t) => {
            let has_inactive_body = t
                .get_behavior_module_info()
                .nuggets()
                .iter()
                .any(|n| n.name == "InactiveBody");
            let can_be_repulsed = t.is_kind_of(KindOfType::CanBeRepulsed) != FALSE;
            let is_shrubbery = t.is_kind_of(KindOfType::Shrubbery) != FALSE;
            let can_weapon = t.can_possibly_have_any_weapon() != FALSE;
            (has_inactive_body, can_be_repulsed, is_shrubbery, can_weapon)
        }
        None => (false, false, false, false),
    };

    let smc_tag = sim.rules.name_keys.name_to_key("ModuleTag_SMCHelper");
    let smc_name = sim.rules.name_keys.name_to_key("ObjectSMCHelper");
    let smc_slot = sim.world.attach_module(
        obj_slot,
        smc_tag,
        ModuleDataId::INVALID,
        FacetMask::UPDATE,
        SleepyUpdatePhase::Normal,
        Box::new(ObjectSMCHelper::new(smc_name)),
    );
    sim.set_wake_frame(smc_slot, UpdateSleepTime::FOREVER);
    if let Some(obj) = sim.world.objects.get_mut(obj_slot) {
        obj.smc_helper_slot = Some(smc_slot);
    }

    if !has_inactive_body {
        let sd_tag = sim.rules.name_keys.name_to_key("ModuleTag_StatusDamageHelper");
        let sd_name = sim.rules.name_keys.name_to_key("StatusDamageHelper");
        let sd_slot = sim.world.attach_module(
            obj_slot,
            sd_tag,
            ModuleDataId::INVALID,
            FacetMask::UPDATE,
            SleepyUpdatePhase::Normal,
            Box::new(StatusDamageHelper::new(sd_name)),
        );
        sim.set_wake_frame(sd_slot, UpdateSleepTime::FOREVER);
        if let Some(obj) = sim.world.objects.get_mut(obj_slot) {
            obj.status_damage_helper_slot = Some(sd_slot);
        }

        let sub_tag = sim.rules.name_keys.name_to_key("ModuleTag_SubdualDamageHelper");
        let sub_name = sim.rules.name_keys.name_to_key("SubdualDamageHelper");
        let sub_slot = sim.world.attach_module(
            obj_slot,
            sub_tag,
            ModuleDataId::INVALID,
            FacetMask::UPDATE,
            SleepyUpdatePhase::Normal,
            Box::new(SubdualDamageHelper::new(sub_name)),
        );
        sim.set_wake_frame(sub_slot, UpdateSleepTime::FOREVER);
        if let Some(obj) = sim.world.objects.get_mut(obj_slot) {
            obj.subdual_damage_helper_slot = Some(sub_slot);
        }
    }

    if sim.world.ai_enable_repulsors != FALSE && can_be_repulsed {
        let rep_tag = sim.rules.name_keys.name_to_key("ModuleTag_RepulsorHelper");
        let rep_name = sim.rules.name_keys.name_to_key("ObjectRepulsorHelper");
        let rep_slot = sim.world.attach_module(
            obj_slot,
            rep_tag,
            ModuleDataId::INVALID,
            FacetMask::UPDATE,
            SleepyUpdatePhase::Normal,
            Box::new(ObjectRepulsorHelper::new(rep_name)),
        );
        sim.set_wake_frame(rep_slot, UpdateSleepTime::FOREVER);
        if let Some(obj) = sim.world.objects.get_mut(obj_slot) {
            obj.repulsor_helper_slot = Some(rep_slot);
        }
    }

    if !is_shrubbery {
        let def_tag = sim.rules.name_keys.name_to_key("ModuleTag_DefectionHelper");
        let def_name = sim.rules.name_keys.name_to_key("ObjectDefectionHelper");
        let def_slot = sim.world.attach_module(
            obj_slot,
            def_tag,
            ModuleDataId::INVALID,
            FacetMask::UPDATE,
            SleepyUpdatePhase::Normal,
            Box::new(ObjectDefectionHelper::new(def_name)),
        );
        sim.set_wake_frame(def_slot, UpdateSleepTime::FOREVER);
        if let Some(obj) = sim.world.objects.get_mut(obj_slot) {
            obj.defection_helper_slot = Some(def_slot);
        }
    }

    if can_weapon {
        let ws_tag = sim.rules.name_keys.name_to_key("ModuleTag_WeaponStatusHelper");
        let ws_name = sim.rules.name_keys.name_to_key("ObjectWeaponStatusHelper");
        let ws_slot = sim.world.attach_module(
            obj_slot,
            ws_tag,
            ModuleDataId::INVALID,
            FacetMask::UPDATE,
            SleepyUpdatePhase::Final,
            Box::new(ObjectWeaponStatusHelper::new(ws_name)),
        );
        sim.set_wake_frame(ws_slot, UpdateSleepTime::NONE);
        if let Some(obj) = sim.world.objects.get_mut(obj_slot) {
            obj.weapon_status_helper_slot = Some(ws_slot);
        }

        // `FiringTracker` sleeps until a shot is reported to it.
        let ft_tag = sim.rules.name_keys.name_to_key("ModuleTag_FiringTrackerHelper");
        let ft_name = sim.rules.name_keys.name_to_key("FiringTracker");
        let ft_slot = sim.world.attach_module(
            obj_slot,
            ft_tag,
            ModuleDataId::INVALID,
            FacetMask::UPDATE,
            SleepyUpdatePhase::Final,
            Box::new(FiringTracker::new(ft_name, sim.world.frame)),
        );
        sim.set_wake_frame(ft_slot, UpdateSleepTime::FOREVER);
        if let Some(obj) = sim.world.objects.get_mut(obj_slot) {
            obj.firing_tracker_slot = Some(ft_slot);
        }

        let tw_tag = sim.rules.name_keys.name_to_key("ModuleTag_TempWeaponBonusHelper");
        let tw_name = sim.rules.name_keys.name_to_key("TempWeaponBonusHelper");
        let tw_slot = sim.world.attach_module(
            obj_slot,
            tw_tag,
            ModuleDataId::INVALID,
            FacetMask::UPDATE,
            SleepyUpdatePhase::Normal,
            Box::new(TempWeaponBonusHelper::new(tw_name)),
        );
        sim.set_wake_frame(tw_slot, UpdateSleepTime::FOREVER);
        if let Some(obj) = sim.world.objects.get_mut(obj_slot) {
            obj.temp_weapon_bonus_helper_slot = Some(tw_slot);
        }
    }
}
