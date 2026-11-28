//! `PKT-410` `Team`, `TeamPrototype`, `TeamFactory` and the relation maps (`Team.cpp`).
//!
//! Golden test: `tests/golden_players_teams.rs`.
//!
//! Instance and member lists are intrusive lists in the original: new entries go
//! to the front. Script, AI and object behaviour is reached through
//! [`PlayerWorld`](crate::player::seams::PlayerWorld).

use std::collections::BTreeMap;

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UnsignedShort,
    XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

use crate::arena::{ObjectSlot, ThingTemplateId};
use crate::gamelogic::global_data::PlayerType;
use crate::modules::update::ai_update::AiUpdate;
use crate::object::thing_template::ThingTemplate;
use crate::player::player_list::PlayerList;
use crate::player::seams::{AiKind, PlayerEnv, PlayerIndex, PlayerWorld};
use crate::rules::disabled_types::DisabledType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::scripts::scripts::Script;
use crate::sim::{SimContext, SimView, SimWorld};
use crate::spatial::sides_list::SidesList;

pub type TeamPrototypeId = UnsignedInt;

pub const MAX_UNIT_TYPES: usize = 7;
pub const MAX_GENERIC_SCRIPTS: usize = 16;
/// `ATTITUDE_NORMAL`.
pub const ATTITUDE_NORMAL: Int = 0;

/// `xferUser` of a four byte id.
fn xfer_u32(xfer: &mut dyn Xfer, value: &mut u32) -> Result<(), XferError> {
    let mut bytes = value.to_le_bytes();
    xfer.xfer_user(&mut bytes)?;
    *value = u32::from_le_bytes(bytes);
    Ok(())
}

pub(crate) fn xfer_team_id(xfer: &mut dyn Xfer, id: &mut TeamId) -> Result<(), XferError> {
    let mut raw = id.0 as u32;
    xfer_u32(xfer, &mut raw)?;
    id.0 = raw as i32;
    Ok(())
}

pub(crate) fn flag(xfer: &mut dyn Xfer, value: &mut bool) -> Result<(), XferError> {
    let mut b: Bool = if *value { TRUE } else { FALSE };
    xfer.xfer_bool(&mut b)?;
    *value = b != FALSE;
    Ok(())
}

/// A `std::hash_map<unsigned, Relationship>` of the reference build (libc++
/// `unordered_map`), kept in its iteration order so saves match byte for byte.
/// `TeamRelationMap` keys are team ids, `PlayerRelationMap` keys player indices.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RelationMap {
    entries: Vec<(u32, Relationship)>,
    buckets: usize,
}

fn next_prime(n: usize) -> usize {
    let mut p = n.max(2);
    while (2..).take_while(|d| d * d <= p).any(|d| p % d == 0) {
        p += 1;
    }
    p
}

fn bucket_of(key: u32, buckets: usize) -> usize {
    let hash = key as usize;
    if buckets.is_power_of_two() {
        hash & (buckets - 1)
    } else {
        hash % buckets
    }
}

impl RelationMap {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, key: u32) -> Option<Relationship> {
        self.entries.iter().find(|e| e.0 == key).map(|e| e.1)
    }

    /// Entries in iteration order.
    pub fn entries(&self) -> &[(u32, Relationship)] {
        &self.entries
    }

    /// `clear` keeps the bucket count, which later insertions depend on.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn remove(&mut self, key: u32) -> bool {
        match self.entries.iter().position(|e| e.0 == key) {
            Some(at) => {
                self.entries.remove(at);
                true
            }
            None => false,
        }
    }

    /// `m_map[key] = r`.
    pub fn set(&mut self, key: u32, r: Relationship) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.0 == key) {
            e.1 = r;
            return;
        }
        if self.entries.len() + 1 > self.buckets {
            let power_of_two = self.buckets > 2 && self.buckets.is_power_of_two();
            let grown = 2 * self.buckets + usize::from(!power_of_two);
            self.rehash(grown.max(self.entries.len() + 1));
        }
        let bucket = bucket_of(key, self.buckets);
        let at = self
            .entries
            .iter()
            .position(|e| bucket_of(e.0, self.buckets) == bucket)
            .unwrap_or(0);
        self.entries.insert(at, (key, r));
    }

    fn rehash(&mut self, wanted: usize) {
        let n = if wanted == 1 {
            2
        } else if wanted & (wanted - 1) != 0 {
            next_prime(wanted)
        } else {
            wanted
        };
        if n <= self.buckets {
            return;
        }
        self.buckets = n;
        // libc++ `__do_rehash`: nodes of a bucket already passed move to its front.
        let old = std::mem::take(&mut self.entries);
        let mut tail_bucket = None;
        for (key, r) in old {
            let bucket = bucket_of(key, n);
            let first = self
                .entries
                .iter()
                .position(|e| bucket_of(e.0, n) == bucket);
            match first {
                Some(at) if tail_bucket != Some(bucket) => self.entries.insert(at, (key, r)),
                _ => {
                    self.entries.push((key, r));
                    tail_bucket = Some(bucket);
                }
            }
        }
    }

    /// `TeamRelationMap::xfer` / `PlayerRelationMap::xfer`. Both keys are four
    /// bytes; loading assigns over what is already there.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut count = self.entries.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Save {
            for &(mut key, r) in &self.entries {
                xfer_u32(xfer, &mut key)?;
                let mut raw = r as u32;
                xfer_u32(xfer, &mut raw)?;
            }
        } else {
            for _ in 0..count {
                let mut key = 0;
                xfer_u32(xfer, &mut key)?;
                let mut raw = 0;
                xfer_u32(xfer, &mut raw)?;
                self.set(key, Relationship::from_i32(raw as i32));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateUnitsInfo {
    pub min_units: Int,
    pub max_units: Int,
    pub unit_thing_name: String,
}

/// `TeamTemplateInfo`, read from a team's dictionary.
#[derive(Debug, Clone, Default)]
pub struct TeamTemplateInfo {
    pub units: Vec<CreateUnitsInfo>,
    pub home_location: Coord3D,
    pub has_home_location: bool,
    pub script_on_create: String,
    pub script_on_idle: String,
    pub initial_idle_frames: Int,
    pub script_on_enemy_sighted: String,
    pub script_on_all_clear: String,
    pub script_on_unit_destroyed: String,
    pub script_on_destroyed: String,
    pub destroyed_threshold: Real,
    pub is_ai_recruitable: bool,
    pub is_base_defense: bool,
    pub is_perimeter_defense: bool,
    pub automatically_reinforce: bool,
    pub transports_return: bool,
    pub avoid_threats: bool,
    pub attack_common_target: bool,
    pub max_instances: Int,
    /// The only saved field.
    pub production_priority: Int,
    pub production_priority_success_increase: Int,
    pub production_priority_failure_decrease: Int,
    pub initial_team_attitude: Int,
    pub transport_unit_type: String,
    pub start_reinforce_waypoint: String,
    pub team_starts_full: bool,
    pub transports_exit: bool,
    pub veterancy: Int,
    pub production_condition: String,
    pub execute_actions: bool,
    pub generic_scripts: [String; MAX_GENERIC_SCRIPTS],
}

/// Dictionary reads that intern the key on use, as `TheKey_*` does.
struct Reader<'a> {
    dict: &'a Dict,
    name_keys: &'a mut NameKeyGenerator,
}

impl Reader<'_> {
    fn key(&mut self, name: &str) -> NameKeyType {
        self.name_keys.name_to_key(name)
    }
    fn string(&mut self, name: &str) -> String {
        let key = self.key(name);
        self.dict.get_ascii_string(key, None)
    }
    fn int(&mut self, name: &str) -> Int {
        let key = self.key(name);
        self.dict.get_int(key, None)
    }
    fn real(&mut self, name: &str) -> Real {
        let key = self.key(name);
        self.dict.get_real(key, None)
    }
    fn bool(&mut self, name: &str) -> bool {
        let key = self.key(name);
        self.dict.get_bool(key, None) != FALSE
    }
}

impl TeamTemplateInfo {
    pub fn from_dict(
        dict: &Dict,
        name_keys: &mut NameKeyGenerator,
        world: &dyn PlayerWorld,
    ) -> Self {
        let mut d = Reader { dict, name_keys };
        let mut out = Self::default();
        for i in 1..=MAX_UNIT_TYPES {
            let min = d.int(&format!("teamUnitMinCount{i}"));
            let max = d.int(&format!("teamUnitMaxCount{i}"));
            let key = d.key(&format!("teamUnitType{i}"));
            let mut exists: Bool = FALSE;
            let name = dict.get_ascii_string(key, Some(&mut exists));
            if max > 0 && exists != FALSE {
                out.units.push(CreateUnitsInfo {
                    min_units: min,
                    max_units: max,
                    unit_thing_name: name,
                });
            }
        }

        let home_key = d.key("teamHome");
        let mut exists: Bool = FALSE;
        let home = dict.get_ascii_string(home_key, Some(&mut exists));
        if exists != FALSE {
            if let Some(location) = world.waypoint_location(&home) {
                out.home_location = location;
                out.has_home_location = true;
            }
        }

        out.script_on_create = d.string("teamOnCreateScript");
        out.is_ai_recruitable = d.bool("teamIsAIRecruitable");
        out.is_base_defense = d.bool("teamIsBaseDefense");
        out.is_perimeter_defense = d.bool("teamIsPerimeterDefense");
        out.automatically_reinforce = d.bool("teamAutoReinforce");

        let attitude_key = d.key("teamAggressiveness");
        let mut exists: Bool = FALSE;
        let attitude = dict.get_int(attitude_key, Some(&mut exists));
        out.initial_team_attitude = if exists != FALSE {
            attitude
        } else {
            ATTITUDE_NORMAL
        };

        out.transports_return = d.bool("teamTransportsReturn");
        out.avoid_threats = d.bool("teamAvoidThreats");
        out.attack_common_target = d.bool("teamAttackCommonTarget");
        out.max_instances = d.int("teamMaxInstances");
        out.script_on_idle = d.string("teamOnIdleScript");
        out.initial_idle_frames = d.int("teamInitialIdleFrames");
        out.script_on_enemy_sighted = d.string("teamEnemySightedScript");
        out.script_on_all_clear = d.string("teamAllClearScript");
        out.script_on_destroyed = d.string("teamOnDestroyedScript");
        out.destroyed_threshold = d.real("teamDestroyedThreshold");
        out.script_on_unit_destroyed = d.string("teamOnUnitDestroyedScript");
        out.production_priority = d.int("teamProductionPriority");
        out.production_priority_success_increase = d.int("teamProductionPrioritySuccessIncrease");
        out.production_priority_failure_decrease = d.int("teamProductionPriorityFailureDecrease");
        out.production_condition = d.string("teamProductionCondition");
        out.execute_actions = d.bool("teamExecutesActionsOnCreate");

        let hook = d.key("teamGenericScriptHook");
        let hook = d.name_keys.key_to_name(hook).to_owned();
        for (i, script) in out.generic_scripts.iter_mut().enumerate() {
            *script = d.string(&format!("{hook}{i}"));
        }

        out.transport_unit_type = d.string("teamTransport");
        out.transports_exit = d.bool("teamTransportsExit");
        out.team_starts_full = d.bool("teamStartsFull");
        out.start_reinforce_waypoint = d.string("teamReinforcementOrigin");
        out.veterancy = d.int("teamVeterancy");
        out
    }

    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_int(&mut self.production_priority)
    }
}

/// `Team`.
#[derive(Debug, Clone, PartialEq)]
pub struct Team {
    pub id: TeamId,
    pub prototype: TeamPrototypeId,
    /// `m_dlinkhead_TeamMemberList`, newest first.
    pub members: Vec<ObjectId>,
    pub state: String,
    pub entered_or_exited: bool,
    pub active: bool,
    pub created: bool,
    pub check_enemy_sighted: bool,
    pub see_enemy: bool,
    pub prev_see_enemy: bool,
    pub was_idle: bool,
    pub destroy_threshold: Int,
    pub cur_units: Int,
    /// Id of `m_currentWaypoint`, 0 for none.
    pub current_waypoint: UnsignedInt,
    pub should_attempt_generic_script: [bool; MAX_GENERIC_SCRIPTS],
    pub is_recruitability_set: bool,
    pub is_recruitable: bool,
    pub common_attack_target: ObjectId,
    pub team_relations: RelationMap,
    pub player_relations: RelationMap,
    /// Member ids read from a save, checked by `load_post_process`.
    pub xfer_member_ids: Vec<ObjectId>,
}

impl Team {
    fn new(prototype: TeamPrototypeId, id: TeamId, info: &TeamTemplateInfo) -> Self {
        Self {
            id,
            prototype,
            members: Vec::new(),
            state: String::new(),
            entered_or_exited: false,
            active: false,
            created: false,
            check_enemy_sighted: !info.script_on_all_clear.is_empty()
                || !info.script_on_enemy_sighted.is_empty(),
            see_enemy: false,
            prev_see_enemy: false,
            was_idle: false,
            destroy_threshold: 0,
            cur_units: 0,
            current_waypoint: 0,
            should_attempt_generic_script: [true; MAX_GENERIC_SCRIPTS],
            is_recruitability_set: false,
            is_recruitable: false,
            common_attack_target: ObjectId::INVALID,
            team_relations: RelationMap::default(),
            player_relations: RelationMap::default(),
            xfer_member_ids: Vec::new(),
        }
    }

    pub fn set_active(&mut self) {
        if !self.active {
            self.created = true;
            self.active = true;
        }
    }

    pub fn set_recruitable(&mut self, recruitable: bool) {
        self.is_recruitability_set = true;
        self.is_recruitable = recruitable;
    }

    /// `prependTo_TeamMemberList`.
    pub fn add_member(&mut self, object: ObjectId) {
        if !self.members.contains(&object) {
            self.members.insert(0, object);
        }
    }

    pub fn remove_member(&mut self, object: ObjectId) {
        self.members.retain(|&m| m != object);
    }

    pub fn set_override_team_relationship(&mut self, team: TeamId, r: Relationship) {
        if team != TeamId::INVALID {
            self.team_relations.set(team.0 as u32, r);
        }
    }

    /// `TeamId::INVALID` removes every override.
    pub fn remove_override_team_relationship(&mut self, team: TeamId) -> bool {
        if self.team_relations.is_empty() {
            return false;
        }
        if team == TeamId::INVALID {
            self.team_relations.clear();
            return true;
        }
        self.team_relations.remove(team.0 as u32)
    }

    pub fn set_override_player_relationship(&mut self, player: PlayerIndex, r: Relationship) {
        // `PLAYER_INDEX_INVALID`.
        if player != -1 {
            self.player_relations.set(player as u32, r);
        }
    }

    /// `-1` removes every override.
    pub fn remove_override_player_relationship(&mut self, player: PlayerIndex) -> bool {
        if self.player_relations.is_empty() {
            return false;
        }
        if player == -1 {
            self.player_relations.clear();
            return true;
        }
        self.player_relations.remove(player as u32)
    }

    pub fn xfer(&mut self, xfer: &mut dyn Xfer, world: &dyn PlayerWorld) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut id = self.id;
        xfer_team_id(xfer, &mut id)?;
        if id != self.id {
            return Err(XferError::InvalidData);
        }
        let mut count = self.members.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Save {
            for &member in &self.members {
                let mut member = member;
                xfer.xfer_object_id(&mut member)?;
            }
        } else {
            for _ in 0..count {
                let mut member = ObjectId::INVALID;
                xfer.xfer_object_id(&mut member)?;
                self.xfer_member_ids.push(member);
            }
        }
        xfer.xfer_ascii_string(&mut self.state)?;
        flag(xfer, &mut self.entered_or_exited)?;
        flag(xfer, &mut self.active)?;
        flag(xfer, &mut self.created)?;
        flag(xfer, &mut self.check_enemy_sighted)?;
        flag(xfer, &mut self.see_enemy)?;
        flag(xfer, &mut self.prev_see_enemy)?;
        flag(xfer, &mut self.was_idle)?;
        xfer.xfer_int(&mut self.destroy_threshold)?;
        xfer.xfer_int(&mut self.cur_units)?;
        xfer.xfer_unsigned_int(&mut self.current_waypoint)?;
        if xfer.mode() == XferMode::Load && !world.waypoint_exists(self.current_waypoint) {
            self.current_waypoint = 0;
        }
        let mut scripts = MAX_GENERIC_SCRIPTS as UnsignedShort;
        xfer.xfer_unsigned_short(&mut scripts)?;
        if usize::from(scripts) != MAX_GENERIC_SCRIPTS {
            return Err(XferError::InvalidData);
        }
        for attempt in &mut self.should_attempt_generic_script {
            flag(xfer, attempt)?;
        }
        flag(xfer, &mut self.is_recruitability_set)?;
        flag(xfer, &mut self.is_recruitable)?;
        xfer.xfer_object_id(&mut self.common_attack_target)?;
        self.team_relations.xfer(xfer)?;
        self.player_relations.xfer(xfer)
    }

    /// Every saved member must already be in the list.
    pub fn load_post_process(&mut self) -> Result<(), XferError> {
        if !self
            .xfer_member_ids
            .iter()
            .all(|id| self.members.contains(id))
        {
            return Err(XferError::InvalidData);
        }
        self.xfer_member_ids.clear();
        Ok(())
    }
}

/// `TeamPrototype`.
#[derive(Debug, Clone)]
pub struct TeamPrototype {
    pub id: TeamPrototypeId,
    pub name: String,
    /// `m_owningPlayer`.
    pub owner: PlayerIndex,
    pub singleton: bool,
    pub production_condition_always_false: bool,
    pub production_condition_script: Option<Script>,
    retrieved_generic_scripts: bool,
    generic_scripts: [Option<UnsignedInt>; MAX_GENERIC_SCRIPTS],
    pub template: TeamTemplateInfo,
    pub attack_priority_name: String,
    /// `m_dlinkhead_TeamInstanceList`, newest first.
    pub instances: Vec<Team>,
}

impl TeamPrototype {
    pub fn count_team_instances(&self) -> Int {
        self.instances.len() as Int
    }

    pub fn increase_ai_priority_for_success(&mut self) {
        let t = &mut self.template;
        t.production_priority = t
            .production_priority
            .wrapping_add(t.production_priority_success_increase);
    }

    pub fn decrease_ai_priority_for_failure(&mut self) {
        let t = &mut self.template;
        t.production_priority = t
            .production_priority
            .wrapping_sub(t.production_priority_failure_decrease);
    }
}

/// `TeamFactory`. A prototype whose name is already taken is not in the name map
/// but stays in its owner's list, as in the original.
#[derive(Debug, Clone, Default)]
pub struct TeamFactory {
    prototypes: BTreeMap<TeamPrototypeId, TeamPrototype>,
    /// `m_prototypes`: iteration is by name key.
    by_name: BTreeMap<NameKeyType, TeamPrototypeId>,
    pub unique_prototype_id: TeamPrototypeId,
    pub unique_team_id: UnsignedInt,
}

impl TeamFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prototype(&self, id: TeamPrototypeId) -> Option<&TeamPrototype> {
        self.prototypes.get(&id)
    }

    pub fn prototype_mut(&mut self, id: TeamPrototypeId) -> Option<&mut TeamPrototype> {
        self.prototypes.get_mut(&id)
    }

    /// Prototype ids of `m_prototypes`, in its order.
    pub fn listed_prototypes(&self) -> impl Iterator<Item = TeamPrototypeId> + '_ {
        self.by_name.values().copied()
    }

    /// `findTeamPrototype`; interns the name.
    pub fn find_team_prototype(
        &self,
        name: &str,
        name_keys: &mut NameKeyGenerator,
    ) -> Option<TeamPrototypeId> {
        self.by_name.get(&name_keys.name_to_key(name)).copied()
    }

    /// `findTeamPrototypeByID` (listed prototypes only).
    pub fn find_team_prototype_by_id(&self, id: TeamPrototypeId) -> Option<TeamPrototypeId> {
        self.listed_prototypes().find(|&p| p == id)
    }

    /// `findTeamByID`.
    pub fn team(&self, id: TeamId) -> Option<&Team> {
        if id == TeamId::INVALID {
            return None;
        }
        self.listed_prototypes()
            .find_map(|p| self.prototypes[&p].instances.iter().find(|t| t.id == id))
    }

    pub fn team_mut(&mut self, id: TeamId) -> Option<&mut Team> {
        let prototype = self.team(id)?.prototype;
        self.prototypes
            .get_mut(&prototype)?
            .instances
            .iter_mut()
            .find(|t| t.id == id)
    }

    /// The prototype of a team.
    pub fn prototype_of(&self, team: TeamId) -> Option<&TeamPrototype> {
        self.prototypes.get(&self.team(team)?.prototype)
    }

    /// `Team::getControllingPlayer`.
    pub fn controlling_player(&self, team: TeamId) -> Option<PlayerIndex> {
        Some(self.prototype_of(team)?.owner)
    }

    /// `Team::getRelationship`.
    pub fn team_relationship(
        &self,
        this: TeamId,
        that: Option<TeamId>,
        players: &PlayerList,
    ) -> Relationship {
        let Some(team) = self.team(this) else {
            return Relationship::Neutral;
        };
        let that_player = that.and_then(|t| self.controlling_player(t));
        if let Some(that) = that {
            if let Some(r) = team.team_relations.get(that.0 as u32) {
                return r;
            }
        }
        if let Some(player) = that_player {
            if let Some(r) = team.player_relations.get(player as u32) {
                return r;
            }
        }
        match self
            .controlling_player(this)
            .and_then(|p| players.player(p))
        {
            Some(owner) => owner.relationship(that, that_player),
            None => Relationship::Neutral,
        }
    }

    /// `clear`: prototypes leave the name map, then are destroyed in its order.
    pub fn clear(&mut self, players: &mut PlayerList, env: &mut PlayerEnv<'_>) {
        let doomed: Vec<TeamPrototypeId> = self.by_name.values().copied().collect();
        self.by_name.clear();
        for id in doomed {
            // `deleteTeamCallback`: each instance leaves the list first.
            while let Some(team) = self
                .prototypes
                .get(&id)
                .and_then(|p| p.instances.first())
                .map(|t| t.id)
            {
                let proto = self.prototypes.get_mut(&id).expect("listed prototype");
                let instance = proto.instances.remove(0);
                self.team_about_to_be_deleted(team, players);
                destroy_team(instance, players, env);
            }
            if let Some(proto) = self.prototypes.remove(&id) {
                if let Some(owner) = players.player_mut(proto.owner) {
                    owner.remove_team_from_list(id);
                }
            }
        }
        // Prototypes outside the map are never destroyed; their owners still
        // hold them.
    }

    /// `TeamFactory::reset`: the id counters restart.
    pub fn reset(&mut self, players: &mut PlayerList, env: &mut PlayerEnv<'_>) {
        self.unique_prototype_id = 0;
        self.unique_team_id = 0;
        self.clear(players, env);
    }

    /// `initFromSides`.
    pub fn init_from_sides(
        &mut self,
        sides: &SidesList,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) {
        self.clear(players, env);
        for info in &sides.team_rec.teams {
            let dict = &info.dict;
            let mut d = Reader {
                dict,
                name_keys: &mut *env.name_keys,
            };
            let name = d.string("teamName");
            let owner = d.string("teamOwner");
            let singleton = d.bool("teamIsSingleton");
            self.init_team(&name, &owner, singleton, dict, players, env);
        }
    }

    /// `initTeam`: a new prototype, and its team if it is a singleton.
    pub fn init_team(
        &mut self,
        name: &str,
        owner: &str,
        singleton: bool,
        dict: &Dict,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) -> TeamPrototypeId {
        let key = env.name_keys.name_to_key(owner);
        let mut player = players.find_player_with_name_key(key);
        if player.is_none() && !owner.is_empty() {
            player = players.find_player_with_name(owner);
        }
        let player = player.unwrap_or(0);

        self.unique_prototype_id = self.unique_prototype_id.wrapping_add(1);
        let id = self.unique_prototype_id;
        let template = TeamTemplateInfo::from_dict(dict, env.name_keys, &*env.world);
        let name_key = env.name_keys.name_to_key(name);
        self.by_name.entry(name_key).or_insert(id);
        self.prototypes.insert(
            id,
            TeamPrototype {
                id,
                name: name.to_owned(),
                owner: player,
                singleton,
                production_condition_always_false: false,
                production_condition_script: None,
                retrieved_generic_scripts: false,
                generic_scripts: [None; MAX_GENERIC_SCRIPTS],
                template,
                attack_priority_name: String::new(),
                instances: Vec::new(),
            },
        );
        if let Some(p) = players.player_mut(player) {
            p.add_team_to_list(id);
        }
        if singleton {
            self.create_inactive_team(name, players, env);
        }
        id
    }

    /// `new Team`: at the front of the prototype's list.
    fn new_instance(&mut self, prototype: TeamPrototypeId) -> Option<TeamId> {
        let proto = self.prototypes.get_mut(&prototype)?;
        self.unique_team_id = self.unique_team_id.wrapping_add(1);
        let id = TeamId(self.unique_team_id as i32);
        let team = Team::new(prototype, id, &proto.template);
        proto.instances.insert(0, team);
        Some(id)
    }

    /// `createInactiveTeam`. The create actions run even when a singleton's team
    /// already exists.
    pub fn create_inactive_team(
        &mut self,
        name: &str,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) -> Option<TeamId> {
        let id = self.find_team_prototype(name, env.name_keys)?;
        let proto = &self.prototypes[&id];
        let actions = proto
            .template
            .execute_actions
            .then(|| proto.template.production_condition.clone());
        let existing = proto
            .instances
            .first()
            .filter(|_| proto.singleton)
            .map(|t| t.id);
        let team = match existing {
            Some(team) => team,
            None => {
                let made = self.new_instance(id)?;
                env.world.notify_team_created(players, self, made);
                made
            }
        };
        if let Some(condition) = actions {
            env.world
                .execute_team_create_actions(players, self, &condition);
        }
        Some(team)
    }

    /// `createTeam`.
    pub fn create_team(
        &mut self,
        name: &str,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) -> Option<TeamId> {
        let team = self.create_inactive_team(name, players, env)?;
        if let Some(t) = self.team_mut(team) {
            t.set_active();
        }
        Some(team)
    }

    /// `createTeamOnPrototype`.
    pub fn create_team_on_prototype(&mut self, prototype: TeamPrototypeId) -> Option<TeamId> {
        let proto = self.prototypes.get(&prototype)?;
        if proto.singleton {
            if let Some(team) = proto.instances.first() {
                return Some(team.id);
            }
        }
        let team = self.new_instance(prototype)?;
        if let Some(t) = self.team_mut(team) {
            t.set_active();
        }
        Some(team)
    }

    /// `findTeam`: the newest instance; a non-singleton without one gets an
    /// inactive team.
    pub fn find_team(
        &mut self,
        name: &str,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) -> Option<TeamId> {
        let id = self.find_team_prototype(name, env.name_keys)?;
        let proto = &self.prototypes[&id];
        match proto.instances.first() {
            Some(team) => Some(team.id),
            None if !proto.singleton => self.create_inactive_team(name, players, env),
            None => None,
        }
    }

    /// `teamAboutToBeDeleted`: every team and player forgets its override.
    pub fn team_about_to_be_deleted(&mut self, team: TeamId, players: &mut PlayerList) {
        for id in self.by_name.values() {
            if let Some(proto) = self.prototypes.get_mut(id) {
                for instance in &mut proto.instances {
                    instance.remove_override_team_relationship(team);
                }
            }
        }
        players.team_about_to_be_deleted(team);
    }

    /// `teamAboutToBeDeleted` followed by `deleteInstance`.
    pub fn delete_team_instance(
        &mut self,
        team: TeamId,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) {
        let Some(prototype) = self.team(team).map(|t| t.prototype) else {
            return;
        };
        self.team_about_to_be_deleted(team, players);
        let Some(proto) = self.prototypes.get_mut(&prototype) else {
            return;
        };
        if let Some(at) = proto.instances.iter().position(|t| t.id == team) {
            let instance = proto.instances.remove(at);
            destroy_team(instance, players, env);
        }
    }

    /// `Team::setControllingPlayer` / `TeamPrototype::setControllingPlayer`.
    pub fn set_controlling_player(
        &mut self,
        team: TeamId,
        new_player: PlayerIndex,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) {
        let Some((prototype, members)) = self.team(team).map(|t| (t.prototype, t.members.clone()))
        else {
            return;
        };
        let Some(proto) = self.prototypes.get_mut(&prototype) else {
            return;
        };
        let old = proto.owner;
        if players.player(new_player).is_none() {
            return;
        }
        if let Some(p) = players.player_mut(old) {
            p.remove_team_from_list(prototype);
        }
        proto.owner = new_player;
        if let Some(p) = players.player_mut(new_player) {
            p.add_team_to_list(prototype);
        }
        for member in members {
            env.world.member_changed_player(member, old, new_player);
        }
    }

    /// Instance ids of a prototype, newest first.
    pub fn instance_ids(&self, prototype: TeamPrototypeId) -> Vec<TeamId> {
        self.prototypes
            .get(&prototype)
            .map(|p| p.instances.iter().map(|t| t.id).collect())
            .unwrap_or_default()
    }

    /// `TeamPrototype::updateState`: update each instance, then delete the empty
    /// active ones that are neither singletons nor a default team.
    pub fn update_prototype_state(
        &mut self,
        prototype: TeamPrototypeId,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) {
        for team in self.instance_ids(prototype) {
            self.update_team_state(team, players, env);
        }
        loop {
            let Some(proto) = self.prototypes.get(&prototype) else {
                return;
            };
            if proto.singleton {
                return;
            }
            let default = players.player(proto.owner).and_then(|p| p.default_team);
            let empty = proto
                .instances
                .iter()
                .find(|t| t.members.is_empty() && default != Some(t.id) && t.active)
                .map(|t| t.id);
            let Some(team) = empty else {
                return;
            };
            self.team_about_to_be_deleted(team, players);
            if let Some(proto) = self.prototypes.get_mut(&prototype) {
                if let Some(at) = proto.instances.iter().position(|t| t.id == team) {
                    let instance = proto.instances.remove(at);
                    destroy_team(instance, players, env);
                }
            }
        }
    }

    /// `Team::updateState`.
    pub fn update_team_state(
        &mut self,
        id: TeamId,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) {
        let Some(team) = self.team_mut(id) else {
            return;
        };
        team.entered_or_exited = false;
        if !team.active {
            return;
        }
        let prototype = team.prototype;
        let created = std::mem::take(&mut team.created);
        let info = self.prototypes[&prototype].template.clone();

        if created {
            if !info.script_on_create.is_empty() {
                env.world
                    .run_script(players, self, &info.script_on_create, id);
            }
            if !info.script_on_destroyed.is_empty() {
                if let Some(team) = self.team_mut(id) {
                    for _ in 0..team.members.len() {
                        team.cur_units = team.cur_units.wrapping_add(1);
                    }
                    let units = team.cur_units as Real;
                    team.destroy_threshold = (units - units * info.destroyed_threshold) as Int;
                    if team.destroy_threshold > team.cur_units.wrapping_sub(1) {
                        team.destroy_threshold = team.cur_units.wrapping_sub(1);
                    }
                    if team.destroy_threshold < 0 {
                        team.destroy_threshold = 0;
                    }
                }
            }
        }

        let Some(team) = self.team_mut(id) else {
            return;
        };
        if team.check_enemy_sighted {
            team.prev_see_enemy = team.see_enemy;
            team.see_enemy = false;
            let mut any_alive = false;
            for &member in &team.members {
                if env.world.member_is_effectively_dead(member) {
                    continue;
                }
                any_alive = true;
                if env.world.member_sees_enemy(member) {
                    team.see_enemy = true;
                    break;
                }
            }
            if any_alive && team.prev_see_enemy != team.see_enemy {
                let script = if team.see_enemy {
                    &info.script_on_enemy_sighted
                } else {
                    &info.script_on_all_clear
                };
                env.world.run_script(players, self, script, id);
            }
        }

        if !info.script_on_destroyed.is_empty() {
            let Some(team) = self.team_mut(id) else {
                return;
            };
            let previous = team.cur_units;
            team.cur_units = team
                .members
                .iter()
                .filter(|&&m| !env.world.member_is_effectively_dead(m))
                .count() as Int;
            if team.cur_units != previous && team.cur_units <= team.destroy_threshold {
                env.world
                    .run_script(players, self, &info.script_on_destroyed, id);
                if let Some(team) = self.team_mut(id) {
                    team.destroy_threshold = -1;
                }
            }
        }

        if !info.script_on_idle.is_empty() {
            let Some(team) = self.team_mut(id) else {
                return;
            };
            let mut idle = true;
            let mut any_alive = false;
            for &member in &team.members {
                if env.world.member_is_effectively_dead(member) {
                    continue;
                }
                let Some(member_idle) = env.world.member_ai_idle(member) else {
                    continue;
                };
                any_alive = true;
                if !member_idle {
                    idle = false;
                }
            }
            if any_alive && idle && team.was_idle {
                env.world
                    .run_script(players, self, &info.script_on_idle, id);
            }
            if let Some(team) = self.team_mut(id) {
                team.was_idle = idle;
            }
        }
    }

    /// `TeamPrototype::getGenericScript`: the scripts are looked up once.
    fn generic_script(
        &mut self,
        prototype: TeamPrototypeId,
        index: usize,
        world: &mut dyn PlayerWorld,
    ) -> Option<UnsignedInt> {
        let proto = self.prototypes.get_mut(&prototype)?;
        if !proto.retrieved_generic_scripts {
            proto.retrieved_generic_scripts = true;
            for i in 0..MAX_GENERIC_SCRIPTS {
                let name = &proto.template.generic_scripts[i];
                proto.generic_scripts[i] = if name.is_empty() {
                    None
                } else {
                    world.duplicate_script(name)
                };
            }
        }
        proto.generic_scripts[index]
    }

    /// `Team::updateGenericScripts`.
    pub fn update_generic_scripts(
        &mut self,
        id: TeamId,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) {
        for i in 0..MAX_GENERIC_SCRIPTS {
            let Some(team) = self.team_mut(id) else {
                return;
            };
            if !team.should_attempt_generic_script[i] {
                continue;
            }
            let prototype = team.prototype;
            let script = self.generic_script(prototype, i, env.world);
            let mut keep = script.is_some();
            if let Some(script) = script {
                if env.world.evaluate_script(players, self, script, id) {
                    if env.world.script_is_one_shot(script) {
                        keep = false;
                    }
                    if let Some(team) = self.team_mut(id) {
                        team.should_attempt_generic_script[i] = keep;
                    }
                    env.world.execute_script_action(players, self, script, id);
                    continue;
                }
            }
            if !keep {
                if let Some(team) = self.team_mut(id) {
                    team.should_attempt_generic_script[i] = false;
                }
            }
        }
    }

    /// `Team::notifyTeamOfObjectDeath`.
    pub fn notify_team_of_object_death(
        &mut self,
        id: TeamId,
        players: &mut PlayerList,
        env: &mut PlayerEnv<'_>,
    ) {
        let Some(proto) = self.prototype_of(id) else {
            return;
        };
        let script = proto.template.script_on_unit_destroyed.clone();
        if !script.is_empty() {
            env.world.run_script(players, self, &script, id);
        }
    }

    /// `TeamFactory::xfer`, with each `TeamPrototype` and `Team` under it.
    /// `TeamFactory::crc` hashes nothing.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer, world: &dyn PlayerWorld) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer_u32(xfer, &mut self.unique_team_id)?;
        let mut count = self.by_name.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if usize::from(count) != self.by_name.len() {
            return Err(XferError::InvalidData);
        }
        if xfer.mode() == XferMode::Save {
            let listed: Vec<TeamPrototypeId> = self.listed_prototypes().collect();
            for mut id in listed {
                xfer_u32(xfer, &mut id)?;
                self.xfer_prototype(id, xfer, world)?;
            }
        } else {
            for _ in 0..count {
                let mut id = 0;
                xfer_u32(xfer, &mut id)?;
                let id = self
                    .find_team_prototype_by_id(id)
                    .ok_or(XferError::InvalidData)?;
                self.xfer_prototype(id, xfer, world)?;
            }
        }
        Ok(())
    }

    fn xfer_prototype(
        &mut self,
        id: TeamPrototypeId,
        xfer: &mut dyn Xfer,
        world: &dyn PlayerWorld,
    ) -> Result<(), XferError> {
        let proto = self.prototypes.get_mut(&id).ok_or(XferError::InvalidData)?;
        let mut version: XferVersion = 2;
        xfer.xfer_version(&mut version, 2)?;
        xfer.xfer_int(&mut proto.owner)?;
        if version >= 2 {
            xfer.xfer_ascii_string(&mut proto.attack_priority_name)?;
        }
        flag(xfer, &mut proto.production_condition_always_false)?;
        proto.template.xfer(xfer)?;
        let mut count = proto.instances.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Save {
            for team in &mut proto.instances {
                let mut team_id = team.id;
                xfer_team_id(xfer, &mut team_id)?;
                team.xfer(xfer, world)?;
            }
        } else {
            for _ in 0..count {
                let mut team_id = TeamId::INVALID;
                xfer_team_id(xfer, &mut team_id)?;
                if self.team(team_id).is_none() {
                    let made = self
                        .create_team_on_prototype(id)
                        .ok_or(XferError::InvalidData)?;
                    if let Some(team) = self.team_mut(made) {
                        team.id = team_id;
                    }
                }
                self.team_mut(team_id)
                    .ok_or(XferError::InvalidData)?
                    .xfer(xfer, world)?;
            }
        }
        Ok(())
    }

    /// `TeamFactory::loadPostProcess`: the counters move past the ids in use.
    pub fn load_post_process(&mut self) {
        self.unique_team_id = 0;
        self.unique_prototype_id = 0;
        for id in self.by_name.values() {
            let proto = &self.prototypes[id];
            if proto.id >= self.unique_prototype_id {
                self.unique_prototype_id = proto.id.wrapping_add(1);
            }
            for team in &proto.instances {
                if team.id.0 as u32 >= self.unique_team_id {
                    self.unique_team_id = (team.id.0 as u32).wrapping_add(1);
                }
            }
        }
    }

    /// `Team::loadPostProcess` of every team.
    pub fn load_post_process_teams(&mut self) -> Result<(), XferError> {
        for proto in self.prototypes.values_mut() {
            for team in &mut proto.instances {
                team.load_post_process()?;
            }
        }
        Ok(())
    }
}

/// `~Team`, for an instance already out of its prototype's list.
fn destroy_team(team: Team, players: &mut PlayerList, env: &mut PlayerEnv<'_>) {
    env.world.notify_team_destroyed(team.id);
    for i in 0..players.player_count() {
        if let Some(player) = players.player_mut(i) {
            player.pre_team_destroy(team.id, env.world);
        }
    }
    for member in team.members {
        env.world.detach_member(member);
    }
}

/// Immediately updates `SimWorld`'s per-team maps after `create_inactive_team` allocates
/// or ensures a team instance.
pub fn sync_team_to_sim_world(
    world: &mut SimWorld,
    players: &PlayerList,
    teams: &TeamFactory,
    team: TeamId,
) {
    let Some(t) = teams.team(team) else {
        return;
    };
    let Some(proto) = teams.prototype(t.prototype) else {
        return;
    };
    let owner = proto.owner;
    let Some(player) = players.player(owner) else {
        return;
    };
    if proto.template.attack_common_target {
        world
            .team_common_targets
            .entry(team)
            .or_insert(ObjectId::INVALID);
    }
    world.team_players.insert(team, owner);
    world.team_is_neutral.insert(team, Bool::from(owner == 0));
    world
        .player_is_human
        .insert(team, Bool::from(player.player_type == PlayerType::Human));
    if player.ai == Some(AiKind::Skirmish) {
        world.skirmish_ai_teams.insert(team);
    }
    let lists: Vec<Vec<TeamId>> = player
        .team_prototypes
        .iter()
        .map(|&p| teams.instance_ids(p))
        .filter(|inst| !inst.is_empty())
        .collect();
    world.player_team_prototypes.insert(owner, lists);
}

/// Removes a deleted team instance from `SimWorld`'s team maps.
pub fn remove_team_from_sim_world(world: &mut SimWorld, team: TeamId) {
    world.team_players.remove(&team);
    world.team_members.remove(&team);
    world.team_is_neutral.remove(&team);
    world.player_is_human.remove(&team);
    world.skirmish_ai_teams.remove(&team);
    world.team_common_targets.remove(&team);
    for lists in world.player_team_prototypes.values_mut() {
        for list in lists.iter_mut() {
            list.retain(|&t| t != team);
        }
        lists.retain(|list| !list.is_empty());
    }
}

/// Applies queued same-player `SimContext::set_object_team_same_player` moves
/// to `TeamFactory`'s cached member lists.
pub fn flush_pending_team_member_moves(sim: &mut SimContext, teams: &mut TeamFactory) {
    if sim.world.pending_team_member_moves.is_empty() {
        return;
    }
    for (obj_id, old_team, new_team) in std::mem::take(&mut sim.world.pending_team_member_moves) {
        if old_team != TeamId::INVALID {
            if let Some(t) = teams.team_mut(old_team) {
                t.remove_member(obj_id);
            }
        }
        if new_team != TeamId::INVALID {
            if let Some(t) = teams.team_mut(new_team) {
                t.add_member(obj_id);
            }
        }
    }
}

/// `Team::isIdle` (`Team.cpp:1761-1779`). Non-AI and effectively dead members count as idle.
pub fn team_is_idle(sim: &SimContext, team: TeamId) -> bool {
    let Some(members) = sim.world.team_members.get(&team) else {
        return true;
    };
    for &slot in members {
        let Some(obj) = sim.world.objects.get(slot) else {
            continue;
        };
        if obj.ai_slot.is_none() || obj.is_effectively_dead() != FALSE {
            continue;
        }
        if sim.ai_is_idle(slot) == FALSE {
            return false;
        }
    }
    true
}

/// `Team::getEstimateTeamPosition` (`Team.cpp:2237-2251`).
pub fn estimate_team_position(sim: &SimContext, team: TeamId) -> Option<Coord3D> {
    let &first = sim.world.team_members.get(&team)?.first()?;
    sim.world
        .objects
        .get(first)
        .map(|o| *o.thing.get_position())
}

/// `Team::countObjectsByThingTemplate` (`Team.cpp:1624-1648`).
pub fn count_objects_by_thing_template(
    sim: &SimContext,
    team: TeamId,
    templates: &[ThingTemplateId],
    ignore_dead: bool,
    ignore_under_construction: bool,
) -> Vec<Int> {
    let mut counts = vec![0; templates.len()];
    let Some(members) = sim.world.team_members.get(&team) else {
        return counts;
    };
    for &slot in members {
        let Some(obj) = sim.world.objects.get(slot) else {
            continue;
        };
        let Some(objtmpl) = obj
            .thing
            .template_override()
            .get_final(&sim.rules.thing_factory)
        else {
            continue;
        };
        for (i, &tmpl_id) in templates.iter().enumerate() {
            let Some(target) = sim
                .rules
                .thing_factory
                .find_by_template_id_final(tmpl_id.0 as u16)
            else {
                continue;
            };
            if objtmpl.is_equivalent_to(target) == FALSE {
                continue;
            }
            if ignore_dead && obj.is_effectively_dead() != FALSE {
                continue;
            }
            if ignore_under_construction
                && obj.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE
            {
                continue;
            }
            counts[i] += 1;
            break;
        }
    }
    counts
}

/// `Team::transferUnitsTo` (`Team.cpp:2315-2324`), assuming both teams belong to
/// the same controlling player.
pub fn transfer_units_to(
    sim: &mut SimContext,
    teams: &mut TeamFactory,
    from_team: TeamId,
    to_team: TeamId,
) {
    if from_team == to_team || from_team == TeamId::INVALID || to_team == TeamId::INVALID {
        return;
    }
    loop {
        let first = sim
            .world
            .team_members
            .get(&from_team)
            .and_then(|m| m.first().copied());
        let Some(slot) = first else {
            break;
        };
        sim.set_object_team_same_player(slot, to_team);
    }
    flush_pending_team_member_moves(sim, teams);
}

/// `isInBuildVariations` (`Team.cpp:2327-2339`).
pub fn is_in_build_variations(tt_with_variations: &ThingTemplate, candidate: &ThingTemplate) -> bool {
    let name = candidate.get_name();
    tt_with_variations
        .get_build_variations()
        .iter()
        .any(|bv| bv == name)
}

/// `Team::tryToRecruit` (`Team.cpp:2343-2407`).
pub fn try_to_recruit(
    sim: &SimContext,
    teams: &TeamFactory,
    recruiting_team: TeamId,
    tmpl_id: ThingTemplateId,
    team_home: &Coord3D,
    max_dist: Real,
) -> Option<ObjectSlot> {
    let my_player = teams.controlling_player(recruiting_team)?;
    let my_priority = teams
        .prototype_of(recruiting_team)?
        .template
        .production_priority;
    let default_team = sim.world.players.player(my_player)?.default_team;
    let t_template = sim
        .rules
        .thing_factory
        .find_by_template_id_final(tmpl_id.0 as u16)?;
    let mut dist_sqr = max_dist * max_dist;
    let mut recruit: Option<ObjectSlot> = None;

    for slot in sim.world.objects.world_iter() {
        let Some(obj) = sim.world.objects.get(slot) else {
            continue;
        };
        let Some(obj_tmpl) = obj
            .thing
            .template_override()
            .get_final(&sim.rules.thing_factory)
        else {
            continue;
        };
        if obj_tmpl.is_equivalent_to(t_template) == FALSE
            && !is_in_build_variations(t_template, obj_tmpl)
        {
            continue;
        }
        if sim.controlling_player(obj.team_id) != Some(my_player) {
            continue;
        }
        let is_default_team = default_team == Some(obj.team_id);
        let Some(team) = teams.team(obj.team_id) else {
            continue;
        };
        let Some(proto) = teams.prototype(team.prototype) else {
            continue;
        };
        if !team.active {
            continue;
        }
        if proto.template.production_priority >= my_priority {
            continue;
        }
        let mut team_is_recruitable = is_default_team;
        if proto.template.is_ai_recruitable {
            team_is_recruitable = true;
        }
        if team.is_recruitability_set {
            team_is_recruitable = team.is_recruitable;
        }
        if !team_is_recruitable {
            continue;
        }
        let recruitable = obj
            .ai_slot
            .and_then(|ms| sim.world.modules.payload(ms))
            .and_then(|m| m.as_any())
            .and_then(|a| a.downcast_ref::<AiUpdate>())
            .is_none_or(|ai| ai.is_recruitable != FALSE);
        if !recruitable {
            continue;
        }
        if obj.is_disabled_by_type(DisabledType::Held) != FALSE {
            continue;
        }
        let dx = team_home.x - obj.thing.get_position().x;
        let dy = team_home.y - obj.thing.get_position().y;
        let d2 = dx * dx + dy * dy;
        // Exact C++ quirk (`Team.cpp:2392-2395`): picks first default-team candidate
        // even beyond `maxDist`, updating `distSqr` to its distance.
        if is_default_team && recruit.is_none() {
            recruit = Some(slot);
            dist_sqr = d2;
        }
        if d2 > dist_sqr {
            continue;
        }
        dist_sqr = d2;
        recruit = Some(slot);
    }
    recruit
}
