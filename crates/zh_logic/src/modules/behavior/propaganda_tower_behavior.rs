//! `PKT-291`: `PropagandaTowerBehaviorModuleData` and `PropagandaTowerBehavior`
//! (`PropagandaTowerBehavior.h`, `PropagandaTowerBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-291_PropagandaTowerBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c3.rs`.
//!
//! An `UpgradeRequired` that names no upgrade crashes the reference; here it counts as not
//! owned.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::WeaponBonusConditionType;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType, DisabledType};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::upgrade::{UpgradeMaskType, UpgradeType};
use crate::sim::{SimApi, SimView, PRIVATE_STATUS_OFF_MAP};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UnsignedShort,
    UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

const LOGICFRAMES_PER_SECOND_REAL: Real = 30.0;

/// INI template data matching `PropagandaTowerBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct PropagandaTowerBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub scan_radius: Real,
    pub scan_delay_in_frames: UnsignedInt,
    pub auto_heal_percent_per_second: Real,
    pub pulse_fx: Option<String>,
    pub upgrade_required: String,
    pub upgraded_auto_heal_percent_per_second: Real,
    pub upgraded_pulse_fx: Option<String>,
    pub affects_self: Bool,
}

impl PropagandaTowerBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            scan_radius: 1.0,
            scan_delay_in_frames: 100,
            auto_heal_percent_per_second: 0.01,
            pulse_fx: None,
            upgrade_required: String::new(),
            upgraded_auto_heal_percent_per_second: 0.02,
            upgraded_pulse_fx: None,
            affects_self: FALSE,
        }
    }
}

impl ModuleData for PropagandaTowerBehaviorModuleData {
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
        FacetMask(0x003 | FacetMask::PROPAGANDA_TOWER.0)
    }
}

/// Runtime instance of `PropagandaTowerBehavior` (`PKT-291`).
pub struct PropagandaTowerBehavior {
    pub data: PropagandaTowerBehaviorModuleData,
    pub last_scan_frame: UnsignedInt,
    /// `m_insideList`, head first.
    pub inside_list: Vec<ObjectId>,
    /// `m_upgradeRequired`, resolved when the object is created.
    pub upgrade_required: Option<(UpgradeMaskType, UpgradeType)>,
    pub schedule: UpdateScheduleTracker,
}

impl PropagandaTowerBehavior {
    pub fn new(data: PropagandaTowerBehaviorModuleData) -> Self {
        Self {
            data,
            last_scan_frame: 0,
            inside_list: Vec::new(),
            upgrade_required: None,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    fn has_required_upgrade(&self, sim: &dyn SimView, team: TeamId) -> bool {
        self.upgrade_required
            .as_ref()
            .is_some_and(|(mask, _)| sim.player_has_upgrade(team, mask) != FALSE)
    }

    /// `effectLogic`: grant or remove the effect on `obj`.
    fn effect_logic(&self, sim: &mut dyn SimApi, owner: ObjectSlot, obj: ObjectSlot, giving: Bool) {
        let Some((owner_id, owner_team)) = sim.object(owner).map(|o| (o.id, o.team_id)) else {
            return;
        };
        let upgraded = self.has_required_upgrade(sim, owner_team);

        if giving == FALSE {
            sim.clear_weapon_bonus_condition(obj, WeaponBonusConditionType::Enthusiastic);
            sim.clear_weapon_bonus_condition(obj, WeaponBonusConditionType::Subliminal);
            return;
        }

        let Some((armed, has_body)) = sim.object(obj).map(|o| {
            (
                o.weapon_set.has_any_damage_weapon() != FALSE,
                o.body_slot.is_some(),
            )
        }) else {
            return;
        };
        if armed {
            sim.set_weapon_bonus_condition(obj, WeaponBonusConditionType::Enthusiastic);
            if upgraded {
                sim.set_weapon_bonus_condition(obj, WeaponBonusConditionType::Subliminal);
            }
        }

        if has_body {
            let percent = if upgraded {
                self.data.upgraded_auto_heal_percent_per_second
            } else {
                self.data.auto_heal_percent_per_second
            };
            let amount = percent / LOGICFRAMES_PER_SECOND_REAL * sim.object_max_health(obj);
            // Healing from several towers does not stack.
            sim.attempt_healing_from_sole_benefactor(
                obj,
                amount,
                owner_id,
                self.data.scan_delay_in_frames,
            );
        }
    }

    /// `removeAllInfluence`.
    fn remove_all_influence(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        for id in std::mem::take(&mut self.inside_list) {
            if let Some(obj) = sim.find_object(id) {
                self.effect_logic(sim, owner, obj, FALSE);
            }
        }
    }

    /// Whether the tower is hidden from the local player, who then gets no pulse effect.
    fn hidden_from_local(sim: &dyn SimView, owner_team: TeamId, obj: ObjectSlot) -> bool {
        sim.player_index(owner_team) != sim.local_player_index()
            && sim.object(obj).is_some_and(|o| {
                o.status.test(ObjectStatusType::Stealthed as Int) != FALSE
                    && o.status.test(ObjectStatusType::Detected as Int) == FALSE
            })
    }

    /// `doScan`.
    fn do_scan(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let Some((team, container, center)) = sim
            .object(owner)
            .map(|o| (o.team_id, o.contained_by, *o.thing.get_position()))
        else {
            return;
        };

        let upgrade_present = match &self.upgrade_required {
            Some((mask, UpgradeType::Player)) => sim.player_has_upgrade(team, mask) != FALSE,
            Some((mask, UpgradeType::Object)) => sim.object_has_upgrade(owner, mask) != FALSE,
            Some((_, UpgradeType::Count)) | None => false,
        };

        let mut do_fx = true;
        if let Some(overlord) = container {
            if Self::hidden_from_local(sim, team, overlord) {
                do_fx = false;
            }
            // A vehicle riding inside something cannot scan.
            if sim.object_is_kind_of(owner, KindOfType::Vehicle) != FALSE
                && sim.object_is_kind_of(owner, KindOfType::PortableStructure) == FALSE
            {
                return;
            }
            // Neither can a tower riding on something that is itself inside something.
            if sim
                .object(overlord)
                .is_some_and(|o| o.contained_by.is_some())
            {
                return;
            }
        }
        if Self::hidden_from_local(sim, team, owner) {
            do_fx = false;
        }
        if do_fx {
            let fx = if upgrade_present {
                &self.data.upgraded_pulse_fx
            } else {
                &self.data.pulse_fx
            };
            if let Some(fx) = fx {
                sim.trigger_fx_list(fx, &center);
            }
        }

        // Allied, alive, non-structure objects in range on the same side of the map edge.
        // Entries are pushed at the head.
        let owner_off_map = sim
            .object(owner)
            .map_or(0, |o| o.private_status & PRIVATE_STATUS_OFF_MAP);
        let mut new_list = Vec::new();
        for obj in sim.find_objects_in_range(&center, self.data.scan_radius) {
            if sim.player_relationship(owner, obj) != Relationship::Allies
                || sim.object_is_kind_of(obj, KindOfType::Structure) != FALSE
            {
                continue;
            }
            let Some((id, dead, off_map)) = sim.object(obj).map(|o| {
                (
                    o.id,
                    o.is_effectively_dead() != FALSE,
                    o.private_status & PRIVATE_STATUS_OFF_MAP,
                )
            }) else {
                continue;
            };
            if dead
                || off_map != owner_off_map
                || (obj == owner && self.data.affects_self == FALSE)
            {
                continue;
            }
            new_list.insert(0, id);
        }

        for &id in &self.inside_list {
            if !new_list.contains(&id) {
                if let Some(obj) = sim.find_object(id) {
                    self.effect_logic(sim, owner, obj, FALSE);
                }
            }
        }
        self.inside_list = new_list;
    }

    /// `getEnclosingContainedBy` (retail: contained twice over).
    fn is_shut_away(sim: &dyn SimView, owner: ObjectSlot) -> bool {
        #[cfg(feature = "retail-crc")]
        {
            sim.object(owner)
                .and_then(|o| o.contained_by)
                .and_then(|c| sim.object(c))
                .is_some_and(|c| c.contained_by.is_some())
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            let mut child = owner;
            while let Some(container) = sim.object(child).and_then(|o| o.contained_by) {
                if sim.contain_is_enclosing_container_for(container, child) != FALSE {
                    return true;
                }
                child = container;
            }
            false
        }
    }
}

impl UpdateFacet for PropagandaTowerBehavior {
    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }

    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let Some((under_construction, sold, dead, mut disabled)) = sim.object(owner).map(|o| {
            (
                o.status.test(ObjectStatusType::UnderConstruction as Int) != FALSE,
                o.status.test(ObjectStatusType::Sold as Int) != FALSE,
                o.is_effectively_dead() != FALSE,
                o.disabled_mask.clone(),
            )
        }) else {
            return UpdateSleepTime::NONE;
        };

        if under_construction {
            return UpdateSleepTime::NONE;
        }
        if sold {
            self.remove_all_influence(sim, owner);
            return UpdateSleepTime::FOREVER;
        }
        if dead {
            return UpdateSleepTime::FOREVER;
        }
        // Being held does not stop the tower; any other disable does.
        disabled.set(DisabledType::Held as Int, Int::from(FALSE));
        if disabled.any() != FALSE || Self::is_shut_away(sim, owner) {
            self.remove_all_influence(sim, owner);
            return UpdateSleepTime::NONE;
        }

        let now = sim.frame();
        if now.wrapping_sub(self.last_scan_frame) >= self.data.scan_delay_in_frames {
            self.do_scan(sim, owner);
            self.last_scan_frame = now;
        }

        let mut kept = Vec::with_capacity(self.inside_list.len());
        for id in std::mem::take(&mut self.inside_list) {
            let obj = sim.find_object(id).filter(|&obj| {
                [
                    KindOfType::Score,
                    KindOfType::ScoreCreate,
                    KindOfType::ScoreDestroy,
                    KindOfType::MpCountForVictory,
                ]
                .into_iter()
                .any(|kind| sim.object_is_kind_of(obj, kind) != FALSE)
            });
            if let Some(obj) = obj {
                self.effect_logic(sim, owner, obj, TRUE);
                kept.push(id);
            }
        }
        self.inside_list = kept;
        UpdateSleepTime::NONE
    }
}

impl DieFacet for PropagandaTowerBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        self.remove_all_influence(sim, ctx.owner_slot);
    }
}

impl BehaviorModule for PropagandaTowerBehavior {
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
            .map(|u| (u.upgrade_mask().clone(), u.upgrade_type()));
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.remove_all_influence(sim, ctx.owner_slot);
    }

    fn on_capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _old_team: TeamId,
        new_team: TeamId,
    ) {
        // The tower does not work for the neutral player.
        if sim.player_index(new_team) == sim.player_index(sim.neutral_team_id()) {
            self.remove_all_influence(sim, ctx.owner_slot);
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
        } else {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.last_scan_frame)?;

        let mut count = self.inside_list.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Load {
            if !self.inside_list.is_empty() {
                return Err(XferError::ListNotEmpty);
            }
            for _ in 0..count {
                let mut id = ObjectId::INVALID;
                xfer.xfer_object_id(&mut id)?;
                self.inside_list.insert(0, id);
            }
        } else {
            for id in &mut self.inside_list {
                xfer.xfer_object_id(id)?;
            }
        }
        Ok(())
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

pub fn parse_propaganda_tower_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = PropagandaTowerBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("Radius") {
            r.parse_real(&mut data.scan_radius)
        } else if field.eq_ignore_ascii_case("DelayBetweenUpdates") {
            r.parse_duration_unsigned_int(&mut data.scan_delay_in_frames)
        } else if field.eq_ignore_ascii_case("HealPercentEachSecond") {
            r.parse_percent_to_real(&mut data.auto_heal_percent_per_second)
        } else if field.eq_ignore_ascii_case("UpgradedHealPercentEachSecond") {
            r.parse_percent_to_real(&mut data.upgraded_auto_heal_percent_per_second)
        } else if field.eq_ignore_ascii_case("PulseFX") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.pulse_fx = resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if field.eq_ignore_ascii_case("UpgradeRequired") {
            r.parse_ascii_string(&mut data.upgrade_required)
        } else if field.eq_ignore_ascii_case("UpgradedPulseFX") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.upgraded_pulse_fx = resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if field.eq_ignore_ascii_case("AffectsSelf") {
            r.parse_bool(&mut data.affects_self)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_propaganda_tower_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<PropagandaTowerBehaviorModuleData>()
        .expect("PropagandaTowerBehavior data type mismatch")
        .clone();
    Box::new(PropagandaTowerBehavior::new(typed))
}
