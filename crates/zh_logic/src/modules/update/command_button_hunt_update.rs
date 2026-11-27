//! `PKT-314`: `CommandButtonHuntUpdateModuleData` and `CommandButtonHuntUpdate`
//! (`CommandButtonHuntUpdate.h`, `CommandButtonHuntUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-314_CommandButtonHuntUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt368_314_ability.rs`.
//!
//! A script hands the module a command button; it then finds targets and gives
//! the button's order to its owner until someone else commands the unit.
//! `ActionManager` is unported: its answers come from
//! [`SimView::action_manager_allows`].
//!
//! Kept quirks:
//! - The timed charge / TNT mine check of the scan is unreachable (it is nested
//!   in the capture-building branch), so it is not here.
//! - A capture hunt drops the relationship filter and skips own and allied
//!   buildings by hand.
//! - A priority from attack info replaces the distance based one; zero skips the target.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, CommandButtonHuntFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::CommandSourceType;
use crate::object::weapon_set::WeaponLockType;
use crate::rules::command_button::{
    CommandButtonId, GuiCommandType, COMMAND_FIRED_BY_SCRIPT, MAX_COMMANDS_PER_SET,
};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::special_power_types::SpecialPowerType;
use crate::sim::{ActionQuery, SimApi, SimView, PRIVATE_STATUS_OFF_MAP};
use crate::spatial::partition_query::shrink;
use std::any::Any;
use zh_common::ini::{IniError, IniReader, LOGICFRAMES_PER_SECOND};
use zh_common::math::gm_sqrtf;
use zh_common::types::{
    Bool, Int, NameKeyType, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// `WWMATH_EPSILON`, the floor `WWMath::Div_Safe` puts under a divisor.
const WWMATH_EPSILON: Real = 0.0001;

/// INI template data matching `CommandButtonHuntUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandButtonHuntUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub scan_frames: UnsignedInt,
    pub scan_range: Real,
}

impl CommandButtonHuntUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            scan_frames: LOGICFRAMES_PER_SECOND as UnsignedInt,
            scan_range: 9999.0,
        }
    }
}

impl ModuleData for CommandButtonHuntUpdateModuleData {
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
        FacetMask(0x001)
    }
}

pub fn parse_command_button_hunt_update_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut d = CommandButtonHuntUpdateModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("ScanRate") {
            r.parse_duration_unsigned_int(&mut d.scan_frames)
        } else if field.eq_ignore_ascii_case("ScanRange") {
            r.parse_real(&mut d.scan_range)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(d))
}

pub fn create_command_button_hunt_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<CommandButtonHuntUpdateModuleData>()
        .expect("CommandButtonHuntUpdateModuleData mismatch");
    Box::new(CommandButtonHuntUpdate::new(typed.clone()))
}

/// The button called `name` in the owner's command set.
fn find_in_command_set(
    sim: &dyn SimView,
    owner: ObjectSlot,
    name: &str,
) -> Option<CommandButtonId> {
    let set_name = sim.object_command_set_string(owner);
    sim.rules().command_store.find_command_set(&set_name)?;
    (0..MAX_COMMANDS_PER_SET)
        .filter_map(|i| sim.command_set_button(&set_name, i))
        .find(|&id| {
            sim.rules()
                .command_store
                .button(id)
                .is_some_and(|b| !b.name.is_empty() && b.name == name)
        })
}

/// What the module reads from its command button.
#[derive(Clone, Copy)]
struct Button {
    id: CommandButtonId,
    command: GuiCommandType,
    special_power: Option<(UnsignedInt, SpecialPowerType)>,
}

/// `Object::doCommandButtonAtObject` for the commands a hunt gives.
fn do_command_button_at_object(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    button: Button,
    target: ObjectSlot,
    cmd_source: CommandSourceType,
) {
    if sim.object(owner).map_or(true, |o| o.is_disabled() != FALSE) {
        return;
    }
    match button.command {
        GuiCommandType::SpecialPower => {
            if let Some((template_id, _)) = button.special_power {
                let options = sim
                    .rules()
                    .command_store
                    .button(button.id)
                    .map_or(0, |b| b.options)
                    | COMMAND_FIRED_BY_SCRIPT;
                let forced = Bool::from(cmd_source == CommandSourceType::FromScript);
                sim.do_special_power_at_object(owner, template_id, target, options, forced);
            }
        }
        GuiCommandType::HijackVehicle
        | GuiCommandType::ConvertToCarBomb
        | GuiCommandType::SabotageBuilding => sim.ai_enter(owner, target, cmd_source),
        _ => {}
    }
}

/// Runtime instance of `CommandButtonHuntUpdate`.
pub struct CommandButtonHuntUpdate {
    pub data: CommandButtonHuntUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub command_button_name: String,
    pub command_button: Option<CommandButtonId>,
    /// The owner, for the command set lookup a load does.
    owner: Option<ObjectSlot>,
}

impl CommandButtonHuntUpdate {
    pub fn new(data: CommandButtonHuntUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            command_button_name: String::new(),
            command_button: None,
            owner: None,
        }
    }

    fn button(&self, sim: &dyn SimView) -> Option<Button> {
        let id = self.command_button?;
        let button = sim.rules().command_store.button(id)?;
        let special_power = button.special_power_id.and_then(|sp| {
            sim.rules()
                .special_power_store
                .find_special_power_template_by_id(sp)
                .map(|t| (sp, t.special_power_type()))
        });
        Some(Button {
            id,
            command: button.command,
            special_power,
        })
    }

    fn scan_sleep(&self) -> UpdateSleepTime {
        UpdateSleepTime(self.data.scan_frames)
    }

    fn hunt_weapon(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        id: CommandButtonId,
    ) -> UpdateSleepTime {
        if sim.ai_is_idle(owner) != FALSE {
            sim.ai_hunt(owner, CommandSourceType::FromAi);
        }
        if let Some(slot) = sim.rules().command_store.button(id).map(|b| b.weapon_slot) {
            sim.lock_weapon_slot(owner, slot, WeaponLockType::LockedTemporarily);
        }
        UpdateSleepTime::NONE
    }

    fn hunt_target(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        button: Button,
    ) -> UpdateSleepTime {
        if let Some(victim) = self.scan_closest_target(sim, owner, button) {
            do_command_button_at_object(sim, owner, button, victim, CommandSourceType::FromAi);
        }
        self.scan_sleep()
    }

    fn hunt_special_power(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        button: Button,
    ) -> UpdateSleepTime {
        if sim.ai_is_idle(owner) == FALSE {
            return self.scan_sleep();
        }
        if let Some((_, power)) = button.special_power {
            let Some(ability) = sim.find_special_ability_update(owner, power) else {
                return UpdateSleepTime::FOREVER;
            };
            if sim.special_ability_is_active(ability) != FALSE {
                return self.scan_sleep();
            }
        }
        self.hunt_target(sim, owner, button)
    }

    fn hunt_enter(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        button: Button,
    ) -> UpdateSleepTime {
        if sim.ai_is_idle(owner) == FALSE {
            return self.scan_sleep();
        }
        self.hunt_target(sim, owner, button)
    }

    fn run(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) -> UpdateSleepTime {
        if sim.has_ai(owner) == FALSE {
            return UpdateSleepTime::FOREVER;
        }
        let Some(button) = self.button(sim) else {
            return UpdateSleepTime::FOREVER;
        };
        if sim.ai_last_command_source(owner) != CommandSourceType::FromAi {
            self.command_button = None;
            self.command_button_name.clear();
            return UpdateSleepTime::FOREVER;
        }
        match button.command {
            GuiCommandType::SpecialPower => self.hunt_special_power(sim, owner, button),
            GuiCommandType::SwitchWeapon | GuiCommandType::FireWeapon => {
                self.hunt_weapon(sim, owner, button.id)
            }
            GuiCommandType::HijackVehicle
            | GuiCommandType::ConvertToCarBomb
            | GuiCommandType::SabotageBuilding => self.hunt_enter(sim, owner, button),
            _ => UpdateSleepTime::FOREVER,
        }
    }

    /// The objects in scan range that pass the alive, map status, stealth and
    /// relationship filters, nearest first.
    fn candidates(
        &self,
        sim: &dyn SimView,
        me: ObjectSlot,
        relationship: Option<Relationship>,
    ) -> Vec<ObjectSlot> {
        let Some((center, off_map)) = sim.object(me).map(|o| {
            (
                *o.thing.get_position(),
                o.private_status & PRIVATE_STATUS_OFF_MAP,
            )
        }) else {
            return Vec::new();
        };
        sim.find_objects_near_to_far(&center, self.data.scan_range)
            .into_iter()
            .filter(|&other| {
                sim.object(other).is_some_and(|o| {
                    o.is_effectively_dead() == FALSE
                        && o.private_status & PRIVATE_STATUS_OFF_MAP == off_map
                }) && !is_stealthed_and_undetected(sim, me, other)
                    && relationship.map_or(true, |r| sim.player_relationship(me, other) == r)
            })
            .collect()
    }

    fn scan_closest_target(
        &self,
        sim: &dyn SimView,
        me: ObjectSlot,
        button: Button,
    ) -> Option<ObjectSlot> {
        let command = button.command;
        let is_enter = matches!(
            command,
            GuiCommandType::ConvertToCarBomb
                | GuiCommandType::HijackVehicle
                | GuiCommandType::SabotageBuilding
        );
        let mut relationship = Some(if command == GuiCommandType::ConvertToCarBomb {
            Relationship::Neutral
        } else {
            Relationship::Enemies
        });
        let mut is_vehicle_hack = false;
        let mut is_capture = false;
        if !is_enter {
            let (_, power) = button.special_power?;
            is_vehicle_hack = power == SpecialPowerType::BlackLotusDisableVehicleHack;
            is_capture = power == SpecialPowerType::InfantryCaptureBuilding;
            if is_capture {
                relationship = None;
            }
        }
        let others = self.candidates(sim, me, relationship);
        let module = button
            .special_power
            .filter(|&(id, _)| sim.find_special_power_module(me, id).is_some());
        let Some((template_id, _)) = module else {
            if !is_enter {
                return None;
            }
            let query = match command {
                GuiCommandType::HijackVehicle => ActionQuery::HijackVehicle,
                GuiCommandType::SabotageBuilding => ActionQuery::SabotageBuilding,
                _ => ActionQuery::ConvertToCarBomb,
            };
            return others
                .into_iter()
                .find(|&other| sim.action_manager_allows(me, other, query) != FALSE);
        };

        let (my_pos, my_radius, my_team) = sim.object(me).map(|o| {
            (
                *o.thing.get_position(),
                o.geometry.get_bounding_circle_radius(),
                o.team_id,
            )
        })?;
        let modifier_distance = sim.rules().ai_data.attack_priority_distance_modifier;
        let mut best = None;
        let mut effective_priority: Int = 0;
        let mut actual_priority: Int = 0;
        for other in others {
            let Some((pos, radius, team, disabled)) = sim.object(other).map(|o| {
                (
                    *o.thing.get_position(),
                    o.geometry.get_bounding_circle_radius(),
                    o.team_id,
                    o.is_disabled() != FALSE,
                )
            }) else {
                continue;
            };
            if disabled && is_vehicle_hack {
                continue;
            }
            if is_capture
                && (sim.player_index(my_team) == sim.player_index(team)
                    || sim.player_relationship(me, other) == Relationship::Allies)
            {
                continue;
            }
            let query = ActionQuery::SpecialPowerAtObject(template_id);
            if sim.action_manager_allows(me, other, query) == FALSE {
                continue;
            }
            let dx = pos.x - my_pos.x;
            let dy = pos.y - my_pos.y;
            let dist = gm_sqrtf(shrink(dx * dx + dy * dy, my_radius + radius).0);
            let current = sim
                .ai_attack_priority(me, other)
                .unwrap_or((self.data.scan_range - dist) as Int);
            if current == 0 {
                continue;
            }
            let divisor = if modifier_distance > WWMATH_EPSILON {
                modifier_distance
            } else {
                WWMATH_EPSILON
            };
            let modifier = (dist / divisor) as Int;
            let modified = current.wrapping_sub(modifier).max(1);
            if modified > effective_priority {
                effective_priority = modified;
                actual_priority = current;
                best = Some(other);
            }
            if modified == effective_priority && current > actual_priority {
                effective_priority = modified;
                actual_priority = current;
                best = Some(other);
            }
        }
        best
    }
}

/// `PartitionFilterStealthedAndUndetected(me, FALSE)`, as "reject this object".
fn is_stealthed_and_undetected(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    let Some(o) = sim.object(other) else {
        return false;
    };
    let test = |s: ObjectStatusType| o.status.test(s.as_i32()) != FALSE;
    if test(ObjectStatusType::Stealthed) && !test(ObjectStatusType::Detected) {
        if sim.object_is_kind_of(other, KindOfType::Disguiser) == FALSE {
            return true;
        }
        if test(ObjectStatusType::Disguised) {
            if let Some(params) = sim
                .stealth_params(other)
                .filter(|p| p.disguised_template.is_some())
            {
                // A bomb truck disguised as a friend of ours counts as hidden.
                let my_team = sim.object(me).map(|m| m.team_id);
                if let Some(team) = my_team {
                    return sim.team_relationship_to_player(team, params.disguised_player_index)
                        != Relationship::Enemies;
                }
            }
            return false;
        }
        return false;
    }
    // A building garrisoned by enemies we cannot see.
    let items = sim.contain_items(other);
    let Some(&first) = items.first() else {
        return false;
    };
    if sim.contain_stealth_units_contained(other) != sim.contain_count(other) {
        return false;
    }
    let hidden = sim
        .find_object(first)
        .and_then(|s| sim.object(s))
        .is_some_and(|m| m.status.test(ObjectStatusType::Detected.as_i32()) == FALSE);
    if !hidden {
        return false;
    }
    let Some(team) = sim.object(me).map(|m| m.team_id) else {
        return false;
    };
    // Only a garrison has an apparent controller.
    sim.contain_apparent_controlling_player(other, sim.player_index(team))
        .is_some_and(|p| sim.team_relationship_to_player(team, p) == Relationship::Enemies)
}

impl UpdateFacet for CommandButtonHuntUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.run(sim, ctx.owner_slot)
    }
}

impl CommandButtonHuntFacet for CommandButtonHuntUpdate {
    fn set_command_button(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, button_name: &str) {
        let owner = ctx.owner_slot;
        button_name.clone_into(&mut self.command_button_name);
        self.command_button = find_in_command_set(sim, owner, button_name);
        if self.command_button.is_none() || sim.has_ai(owner) == FALSE {
            return;
        }
        sim.ai_idle(owner, CommandSourceType::FromAi);
        // The first scan runs at once; its sleep time is dropped.
        self.run(sim, owner);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }
}

impl BehaviorModule for CommandButtonHuntUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        self.owner = Some(ctx.owner_slot);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_ascii_string(&mut self.command_button_name)?;
        if xfer.mode() == XferMode::Load {
            self.command_button = None;
            if !self.command_button_name.is_empty() {
                self.command_button = self
                    .owner
                    .and_then(|o| find_in_command_set(sim, o, &self.command_button_name));
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

    fn as_command_button_hunt_mut(&mut self) -> Option<&mut dyn CommandButtonHuntFacet> {
        Some(self)
    }
}
