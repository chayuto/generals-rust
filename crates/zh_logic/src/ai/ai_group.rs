//! `PKT-423`: `AIGroup` (`AIGroup.cpp`), with the group list `AI` holds
//! (`createGroup`, `destroyGroup`, `crc`).
//!
//! Golden vectors: `docs/golden_vectors/PKT-423.json`
//! (`tests/cpp_harness/gen_pkt423_ai_group.cpp`),
//! `crates/zh_logic/tests/golden_pkt423_ai_group.rs`.
//!
//! The groups live in [`AiGroups`] (`SimWorld::ai_groups`); an object names its
//! group in `ObjectRecord::ai_group` (`Object::m_group`). Orders reach a member
//! through its AI module's command entry. What the groups ask of code that is not
//! ported is behind [`crate::ai::seams::AiSeams`].
//!
//! The retail build (`RETAIL_COMPATIBLE_AIGROUP`) is followed: a group that loses
//! its last member through `remove` destroys itself.
//!
//! Not ported: `groupDoCommandButton*`, `groupGoProne` (no such AI command here),
//! `groupSetEmoticon` (client), `getCommandButtonSourceObject` (command sets), and
//! the surrender orders (not compiled in the reference).

use crate::ai::seams::ai_seams;
use crate::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use crate::modules::module::{AttitudeType, CanMakeType, CommandSourceType, SpecialPowerFacet};
use crate::modules::update::ai_update::{self, AiCommandParms, AiCommandType, AiUpdate};
use crate::object::weapon_set::{CanAttackResult, WeaponLockType, WeaponSlotType};
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{WeaponSetFlags, WeaponSetType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::science::SCIENCE_INVALID;
use crate::rules::upgrade::UpgradeType;
use crate::sim::{SimApi, SimContext, SimView};
use crate::spatial::partition_query::iterator_merge_sort;
use zh_common::math::{gm_cosf, gm_sinf, reference_div_f32, Coord2D, Coord3D, Region3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// `AIGroup::m_id`, unique while a game runs.
pub type GroupId = UnsignedInt;

/// `NO_FORMATION_ID`.
pub const NO_FORMATION_ID: UnsignedInt = 0;
/// `NO_MAX_SHOTS_LIMIT`.
pub const NO_MAX_SHOTS_LIMIT: Int = 0x7fff_ffff;

const PATHFIND_CELL_SIZE_F: Real = 10.0;
const PATH_DIAMETER_IN_CELLS: Int = 6;
/// `PI`.
const PI: Real = 3.141_592_7;
const LOGICFRAMES_PER_SECOND: Int = 30;
const ATTACK_NEW_TARGET: UnsignedInt = 0;
const ATTACK_NEW_TARGET_FORCED: UnsignedInt = 1;

/// The members of `TAiData` that group movement reads, with the constructor's
/// defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupAiData {
    pub min_infantry_for_group: Int,
    pub min_vehicles_for_group: Int,
    pub min_distance_for_group: Real,
    pub distance_requires_group: Real,
}

impl Default for GroupAiData {
    fn default() -> Self {
        Self {
            min_infantry_for_group: 3,
            min_vehicles_for_group: 4,
            min_distance_for_group: 100.0,
            distance_requires_group: 0.0,
        }
    }
}

/// `AIGroup`.
#[derive(Debug, Clone)]
pub struct AiGroup {
    pub id: GroupId,
    /// `m_memberList`, in list order.
    pub members: Vec<ObjectId>,
    pub member_list_size: UnsignedInt,
    pub speed: Real,
    pub dirty: Bool,
    /// `m_groundPath`: the optimized nodes, first to last.
    pub ground_path: Option<Vec<Coord3D>>,
}

impl AiGroup {
    fn new(id: GroupId) -> Self {
        Self {
            id,
            members: Vec::new(),
            member_list_size: 0,
            speed: 0.0,
            dirty: FALSE,
            ground_path: None,
        }
    }

    /// `crc`.
    pub fn crc(&self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        for member in &self.members {
            let mut bytes = member.0.to_le_bytes();
            xfer.xfer_user(&mut bytes)?;
        }
        let mut size = self.member_list_size;
        xfer.xfer_unsigned_int(&mut size)?;
        let mut leader = ObjectId::INVALID;
        xfer.xfer_object_id(&mut leader)?;
        let mut speed = self.speed;
        xfer.xfer_real(&mut speed)?;
        let mut dirty = self.dirty;
        xfer.xfer_bool(&mut dirty)?;
        let mut id = self.id;
        xfer.xfer_unsigned_int(&mut id)
    }

    /// `xfer`: only the version; the members are not saved.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let current: XferVersion = 1;
        let mut version = current;
        xfer.xfer_version(&mut version, current)
    }
}

/// The group side of `AI`.
#[derive(Debug, Clone)]
pub struct AiGroups {
    /// `m_groupList`, in creation order.
    pub list: Vec<AiGroup>,
    /// `m_nextGroupID`: the last id handed out.
    pub next_group_id: UnsignedInt,
    /// `m_nextFormationID`.
    pub next_formation_id: UnsignedInt,
    pub data: GroupAiData,
}

impl Default for AiGroups {
    fn default() -> Self {
        Self {
            list: Vec::new(),
            next_group_id: 0,
            next_formation_id: NO_FORMATION_ID + 1,
            data: GroupAiData::default(),
        }
    }
}

impl AiGroups {
    /// `findGroup`.
    pub fn find(&self, id: GroupId) -> Option<&AiGroup> {
        self.list.iter().find(|g| g.id == id)
    }

    fn find_mut(&mut self, id: GroupId) -> Option<&mut AiGroup> {
        self.list.iter_mut().find(|g| g.id == id)
    }

    /// `getNextFormationID`.
    pub fn take_formation_id(&mut self) -> UnsignedInt {
        let id = self.next_formation_id;
        self.next_formation_id = id.wrapping_add(1);
        id
    }

    /// The part of `AI::crc` after the pathfinder and the AI data.
    pub fn crc_groups(&self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        for group in &self.list {
            let mut marker = String::from("MARKER:AIGroup");
            xfer.xfer_ascii_string(&mut marker)?;
            group.crc(xfer)?;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Membership
// -----------------------------------------------------------------------------

/// `AI::createGroup`.
pub fn create_group(sim: &mut SimContext) -> GroupId {
    let groups = &mut sim.world.ai_groups;
    groups.next_group_id = groups.next_group_id.wrapping_add(1);
    let id = groups.next_group_id;
    groups.list.push(AiGroup::new(id));
    id
}

/// `AI::destroyGroup`: every member leaves.
pub fn destroy_group(sim: &mut SimContext, group: GroupId) {
    let Some(at) = sim.world.ai_groups.list.iter().position(|g| g.id == group) else {
        return;
    };
    let removed = sim.world.ai_groups.list.remove(at);
    for member in removed.members {
        set_object_group(sim, member, None);
    }
}

fn set_object_group(sim: &mut SimContext, member: ObjectId, group: Option<GroupId>) {
    if let Some(slot) = sim.find_object(member) {
        if let Some(object) = sim.object_mut(slot) {
            object.ai_group = group;
        }
    }
}

/// `Object::leaveGroup`.
pub fn leave_group(sim: &mut SimContext, member: ObjectId) {
    let group = sim
        .find_object(member)
        .and_then(|slot| sim.object_mut(slot))
        .and_then(|o| o.ai_group.take());
    if let Some(group) = group {
        remove(sim, group, member);
    }
}

/// Whether [`add`] keeps the object.
pub fn can_join(sim: &SimContext, member: ObjectId) -> bool {
    sim.find_object(member).is_some_and(|slot| {
        has_ai(sim, slot)
            || is_kind(sim, slot, KindOfType::Structure)
            || is_kind(sim, slot, KindOfType::AlwaysSelectable)
    })
}

/// `add`: units with an AI, structures and always selectable objects join, at
/// the back, leaving whatever group they were in.
pub fn add(sim: &mut SimContext, group: GroupId, member: ObjectId) {
    if !can_join(sim, member) {
        return;
    }
    let Some(entry) = sim.world.ai_groups.find_mut(group) else {
        return;
    };
    entry.members.push(member);
    entry.member_list_size = entry.member_list_size.wrapping_add(1);
    // `Object::enterGroup`. Joining twice removes the earlier entry.
    leave_group(sim, member);
    set_object_group(sim, member, Some(group));
    if let Some(entry) = sim.world.ai_groups.find_mut(group) {
        entry.dirty = TRUE;
    }
}

/// `remove`. `true` when that emptied and so destroyed the group.
pub fn remove(sim: &mut SimContext, group: GroupId, member: ObjectId) -> bool {
    let Some(entry) = sim.world.ai_groups.find_mut(group) else {
        return false;
    };
    let Some(at) = entry.members.iter().position(|&m| m == member) else {
        return false;
    };
    entry.members.remove(at);
    entry.member_list_size = entry.member_list_size.wrapping_sub(1);
    entry.dirty = TRUE;
    let empty = entry.members.is_empty();
    set_object_group(sim, member, None);
    if empty {
        destroy_group(sim, group);
    }
    empty
}

/// `getAllIDs`.
pub fn member_ids(sim: &SimContext, group: GroupId) -> Vec<ObjectId> {
    sim.world
        .ai_groups
        .find(group)
        .map_or_else(Vec::new, |g| g.members.clone())
}

/// `isMember`.
pub fn is_member(sim: &SimContext, group: GroupId, member: ObjectId) -> bool {
    sim.world
        .ai_groups
        .find(group)
        .is_some_and(|g| g.members.contains(&member))
}

/// `containsAnyObjectsNotOwnedByPlayer`.
pub fn contains_any_not_owned_by(sim: &SimContext, group: GroupId, player: Int) -> bool {
    member_ids(sim, group)
        .into_iter()
        .any(|id| owner_player(sim, id).is_some_and(|p| p != Some(player)))
}

/// `removeAnyObjectsNotOwnedByPlayer`. `true` when that destroyed the group.
pub fn remove_any_not_owned_by(sim: &mut SimContext, group: GroupId, player: Int) -> bool {
    for member in member_ids(sim, group) {
        if owner_player(sim, member).is_some_and(|p| p != Some(player))
            && remove(sim, group, member)
        {
            return true;
        }
    }
    false
}

/// The controlling player of a live object.
fn owner_player(sim: &SimContext, id: ObjectId) -> Option<Option<Int>> {
    let team = sim.object(sim.find_object(id)?)?.team_id;
    Some(sim.controlling_player(team))
}

/// Visits the members as a `std::list` walk does: members that join during the
/// walk are visited, members that leave before their turn are not.
fn for_each_member(
    sim: &mut SimContext,
    group: GroupId,
    mut f: impl FnMut(&mut SimContext, ObjectId, ObjectSlot),
) {
    let mut next = 0;
    while let Some(&member) = sim
        .world
        .ai_groups
        .find(group)
        .and_then(|g| g.members.get(next))
    {
        if let Some(slot) = sim.find_object(member) {
            f(sim, member, slot);
        }
        let now = sim
            .world
            .ai_groups
            .find(group)
            .and_then(|g| g.members.iter().position(|&m| m == member));
        if let Some(at) = now {
            next = at + 1;
        }
    }
}

fn members_with_slots(sim: &SimContext, group: GroupId) -> Vec<(ObjectId, ObjectSlot)> {
    member_ids(sim, group)
        .into_iter()
        .filter_map(|id| sim.find_object(id).map(|slot| (id, slot)))
        .collect()
}

// -----------------------------------------------------------------------------
// What a member is
// -----------------------------------------------------------------------------

/// `Pathfinder::removeGoal` of a member: the sim's map, else the seam.
fn remove_goal(sim: &mut SimContext, member: ObjectId, slot: ObjectSlot) {
    if sim.pathfinder().is_some() {
        sim.pathfinder_remove_goal(slot);
    } else {
        ai_seams(sim).remove_goal(member);
    }
}

fn has_ai(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot).is_some_and(|o| o.ai_slot.is_some())
}

fn is_kind(sim: &SimContext, slot: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(slot, kind) != FALSE
}

fn is_held(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.is_disabled_by_type(DisabledType::Held) != FALSE)
}

fn has_status(sim: &SimContext, slot: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
}

fn position(sim: &SimContext, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

fn layer_of(sim: &SimContext, slot: ObjectSlot) -> PathfindLayerEnum {
    sim.object_layer(slot).unwrap_or(PathfindLayerEnum::Ground)
}

fn circle_radius(sim: &SimContext, slot: ObjectSlot) -> Real {
    sim.object(slot)
        .map_or(0.0, |o| o.geometry.get_bounding_circle_radius())
}

fn major_radius(sim: &SimContext, slot: ObjectSlot) -> Real {
    sim.object(slot)
        .map_or(0.0, |o| o.geometry.get_major_radius())
}

fn formation_id(sim: &SimContext, slot: ObjectSlot) -> UnsignedInt {
    sim.object(slot).map_or(NO_FORMATION_ID, |o| o.formation_id)
}

fn formation_offset(sim: &SimContext, slot: ObjectSlot) -> Coord2D {
    sim.object(slot)
        .map_or_else(Coord2D::default, |o| o.formation_offset)
}

fn ai_module(sim: &SimContext, slot: ObjectSlot) -> Option<ModuleSlot> {
    sim.object(slot)?.ai_slot
}

fn context(sim: &SimContext, module: ModuleSlot) -> ModuleContext {
    sim.world.modules.meta(module).context(module)
}

/// The resident AI module; `None` for a module stand-in.
fn resident_ai(sim: &mut SimContext, slot: ObjectSlot) -> Option<&mut AiUpdate> {
    let module = ai_module(sim, slot)?;
    sim.resident_state_mut(module)?.downcast_mut::<AiUpdate>()
}

fn doing_ground_movement(sim: &mut SimContext, slot: ObjectSlot) -> bool {
    let Some(module) = ai_module(sim, slot) else {
        return false;
    };
    if resident_ai(sim, slot).is_none() {
        return sim.ai_is_doing_ground_movement(slot) != FALSE;
    }
    let ctx = context(sim, module);
    ai_update::is_doing_ground_movement(sim, ctx) != FALSE
}

/// `getCurLocomotorSpeed`.
fn locomotor_speed(sim: &mut SimContext, slot: ObjectSlot) -> Real {
    let condition = sim.object_body_damage_state(slot);
    let penalty = sim.rules.global_data.movement_penalty_damage_state;
    resident_ai(sim, slot)
        .and_then(|ai| ai.cur_locomotor())
        .map_or(0.0, |l| l.get_max_speed_for_condition(condition, penalty))
}

/// The movement priority of the current locomotor, if there is one.
fn move_priority(sim: &mut SimContext, slot: ObjectSlot) -> Option<Int> {
    resident_ai(sim, slot)
        .and_then(|ai| ai.cur_locomotor())
        .map(|l| l.template.move_priority as Int)
}

const LOCO_MOVES_BACK: Int = 0;
const LOCO_MOVES_MIDDLE: Int = 1;
const LOCO_MOVES_FRONT: Int = 2;

// -----------------------------------------------------------------------------
// Orders to one object
// -----------------------------------------------------------------------------

/// `aiDoCommand` on the AI of `slot`; nothing for an object without one.
pub fn send_order(sim: &mut SimContext, slot: ObjectSlot, parms: AiCommandParms) {
    if !has_ai(sim, slot) {
        return;
    }
    let Some(owner) = sim.object(slot).map(|o| o.id) else {
        return;
    };
    if ai_seams(sim).group_order(owner, &parms) != FALSE {
        return;
    }
    sim.deliver_ai_command(slot, parms);
}

fn parms(cmd: AiCommandType, source: CommandSourceType) -> AiCommandParms {
    AiCommandParms::new(cmd, source)
}

fn at(cmd: AiCommandType, source: CommandSourceType, pos: &Coord3D) -> AiCommandParms {
    let mut p = parms(cmd, source);
    p.pos = *pos;
    p
}

fn on(cmd: AiCommandType, source: CommandSourceType, obj: Option<ObjectId>) -> AiCommandParms {
    let mut p = parms(cmd, source);
    p.obj = obj;
    p
}

/// The orders that give every member with an AI the same command.
fn order_all(sim: &mut SimContext, group: GroupId, command: &AiCommandParms) {
    for_each_member(sim, group, |sim, _, slot| {
        send_order(sim, slot, command.clone());
    });
}

// -----------------------------------------------------------------------------
// Group measurements
// -----------------------------------------------------------------------------

/// `getCenter`: the mean position of the free members with an AI, else of all
/// free members. `false` with a NaN center when there are none.
pub fn center(sim: &SimContext, group: GroupId) -> (bool, Coord3D) {
    let mut center = Coord3D::new(0.0, 0.0, 0.0);
    let members = members_with_slots(sim, group);
    let mut count: Int = 0;
    for pass in 0..2 {
        if pass == 1 && (count != 0 || members.is_empty()) {
            break;
        }
        for &(_, slot) in &members {
            if is_held(sim, slot) || (pass == 0 && !has_ai(sim, slot)) {
                continue;
            }
            let p = position(sim, slot);
            center.x += p.x;
            center.y += p.y;
            center.z += p.z;
            count += 1;
        }
    }
    center.x = reference_div_f32(center.x, count as Real);
    center.y = reference_div_f32(center.y, count as Real);
    center.z = reference_div_f32(center.z, count as Real);
    (count > 0, center)
}

/// `Team::getTeamAsAIGroup` into a new group, and then `getCenter` and
/// `getCount`. With `RETAIL_COMPATIBLE_AIGROUP` nobody destroys the group.
pub fn team_group_center(sim: &mut SimContext, members: &[ObjectId]) -> (Coord3D, Int) {
    let group = create_group(sim);
    for &member in members {
        add(sim, group, member);
    }
    let (_, middle) = center(sim, group);
    let count = sim
        .world
        .ai_groups
        .find(group)
        .map_or(0, |g| g.member_list_size as Int);
    (middle, count)
}

/// `getMinMaxAndCenter`. The flag says whether the group is a formation: two or
/// more free members with an AI, the first of which has a formation id.
pub fn min_max_and_center(sim: &SimContext, group: GroupId) -> (bool, Coord2D, Coord2D, Coord3D) {
    let mut count: Int = 0;
    let mut min = Coord2D { x: 1e10, y: 1e10 };
    let mut max = Coord2D { x: -1e10, y: -1e10 };
    let mut center = Coord3D::new(0.0, 0.0, 0.0);
    let mut id = NO_FORMATION_ID;
    for (_, slot) in members_with_slots(sim, group) {
        if is_held(sim, slot) || !has_ai(sim, slot) {
            continue;
        }
        let p = position(sim, slot);
        center.x += p.x;
        center.y += p.y;
        center.z += p.z;
        min.x = if min.x > p.x { p.x } else { min.x };
        max.x = if max.x < p.x { p.x } else { max.x };
        min.y = if min.y > p.y { p.y } else { min.y };
        max.y = if max.y < p.y { p.y } else { max.y };
        if count == 0 {
            // Later members never change the id.
            id = formation_id(sim, slot);
        }
        count += 1;
    }
    center.x = reference_div_f32(center.x, count as Real);
    center.y = reference_div_f32(center.y, count as Real);
    center.z = reference_div_f32(center.z, count as Real);
    (id != NO_FORMATION_ID && count >= 2, min, max, center)
}

/// `recompute`: the speed of the slowest healthy enough member.
fn recompute(sim: &mut SimContext, group: GroupId) {
    let penalty = sim.rules.global_data.movement_penalty_damage_state;
    let Some(entry) = sim.world.ai_groups.find_mut(group) else {
        return;
    };
    entry.ground_path = None;
    // `9999999999.9f`.
    let mut speed: Real = 1.0e10;
    for (_, slot) in members_with_slots(sim, group) {
        if is_kind(sim, slot, KindOfType::Immobile) || is_held(sim, slot) || !has_ai(sim, slot) {
            continue;
        }
        let max = locomotor_speed(sim, slot);
        if speed > max && (sim.object_body_damage_state(slot) as Int) < penalty as Int {
            speed = max;
        }
    }
    if let Some(entry) = sim.world.ai_groups.find_mut(group) {
        entry.speed = speed;
        entry.dirty = FALSE;
    }
}

fn recompute_if_dirty(sim: &mut SimContext, group: GroupId) {
    if sim
        .world
        .ai_groups
        .find(group)
        .is_some_and(|g| g.dirty != FALSE)
    {
        recompute(sim, group);
    }
}

/// `getSpeed`.
pub fn speed(sim: &mut SimContext, group: GroupId) -> Real {
    recompute_if_dirty(sim, group);
    sim.world.ai_groups.find(group).map_or(0.0, |g| g.speed)
}

// -----------------------------------------------------------------------------
// `SimpleObjectIterator`
// -----------------------------------------------------------------------------

/// The list of a `SimpleObjectIterator`: `insert` puts at the head.
#[derive(Default)]
struct ObjectList(Vec<(ObjectId, ObjectSlot, Real)>);

impl ObjectList {
    fn insert(&mut self, id: ObjectId, slot: ObjectSlot, numeric: Real) {
        self.0.insert(0, (id, slot, numeric));
    }

    fn sort_near_to_far(&mut self) {
        iterator_merge_sort(&mut self.0, |a, b| a.2 - b.2);
    }

    fn sort_far_to_near(&mut self) {
        iterator_merge_sort(&mut self.0, |a, b| b.2 - a.2);
    }

    fn objects(&self) -> Vec<(ObjectId, ObjectSlot)> {
        self.0.iter().map(|&(id, slot, _)| (id, slot)).collect()
    }
}

// -----------------------------------------------------------------------------
// Group movement
// -----------------------------------------------------------------------------

fn ground_height(sim: &SimContext, x: Real, y: Real) -> Real {
    sim.terrain.get_ground_height(x, y, None)
}

fn map_extent(sim: &SimContext, maximum_pathfind: bool) -> Region3D {
    let fallback = sim.terrain_extent();
    ai_seams(sim).map_extent(fallback, maximum_pathfind)
}

/// `clampToMap`: one cell inside the extent a player of this kind may use.
fn clamp_to_map(sim: &SimContext, dest: &mut Coord3D, human: bool) {
    let mut extent = map_extent(sim, !human);
    extent.hi.x -= PATHFIND_CELL_SIZE_F;
    extent.hi.y -= PATHFIND_CELL_SIZE_F;
    extent.lo.x += PATHFIND_CELL_SIZE_F;
    extent.lo.y += PATHFIND_CELL_SIZE_F;
    if dest.x < extent.lo.x {
        dest.x = extent.lo.x;
    }
    if dest.y < extent.lo.y {
        dest.y = extent.lo.y;
    }
    if dest.x > extent.hi.x {
        dest.x = extent.hi.x;
    }
    if dest.y > extent.hi.y {
        dest.y = extent.hi.y;
    }
}

/// `clampWaypointPosition`: the height is taken from the ground only when the
/// point was not strictly inside.
fn clamp_waypoint(sim: &SimContext, pos: &mut Coord3D, margin: Int) {
    let mut extent = map_extent(sim, false);
    let margin = margin as Real;
    extent.hi.x -= margin;
    extent.hi.y -= margin;
    extent.lo.x += margin;
    extent.lo.y += margin;
    if extent.lo.x < pos.x && pos.x < extent.hi.x && extent.lo.y < pos.y && pos.y < extent.hi.y {
        return;
    }
    if pos.x > extent.hi.x {
        pos.x = extent.hi.x;
    } else if pos.x < extent.lo.x {
        pos.x = extent.lo.x;
    }
    if pos.y > extent.hi.y {
        pos.y = extent.hi.y;
    } else if pos.y < extent.lo.y {
        pos.y = extent.lo.y;
    }
    pos.z = ground_height(sim, pos.x, pos.y);
}

fn adjust_and_claim(
    sim: &mut SimContext,
    owner: ObjectId,
    dest: &mut Coord3D,
    group_dest: Option<&Coord3D>,
) {
    ai_seams(sim).adjust_destination(owner, dest, group_dest);
    ai_seams(sim).update_goal(owner, dest, PathfindLayerEnum::Ground);
}

/// `computeIndividualDestination`: the member keeps its offset from the center,
/// pulled in to six radii.
fn individual_destination(
    sim: &mut SimContext,
    group_dest: &Coord3D,
    member: ObjectId,
    slot: ObjectSlot,
    center: &Coord3D,
    formation: bool,
) -> Coord3D {
    let pos = position(sim, slot);
    let mut v = if formation {
        formation_offset(sim, slot)
    } else {
        Coord2D {
            x: pos.x - center.x,
            y: pos.y - center.y,
        }
    };
    let mut length = v.length();
    let limit = 6.0 * circle_radius(sim, slot);
    if length > limit {
        length = limit;
    }
    v.normalize();
    v.x *= length;
    v.y *= length;
    let layer = sim.get_layer_for_destination(group_dest);
    let mut dest = Coord3D::new(group_dest.x + v.x, group_dest.y + v.y, 0.0);
    dest.z = sim.get_layer_height(dest.x, dest.y, layer);
    if has_ai(sim, slot) && doing_ground_movement(sim, slot) {
        let toward = (!formation).then_some(group_dest);
        adjust_and_claim(sim, member, &mut dest, toward);
    }
    dest
}

fn dist_sqr_2d(a: &Coord3D, b: &Coord3D) -> Real {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

/// `friend_computeGroundPath`: one path for the whole group when it is worth it.
fn compute_ground_path(sim: &mut SimContext, group: GroupId, pos: &Coord3D) -> bool {
    recompute_if_dirty(sim, group);
    let (_, min, max, mut center) = min_max_and_center(sim, group);
    let data = sim.world.ai_groups.data.clone();
    let required = data.distance_requires_group * data.distance_requires_group;
    let mut dist_sqr = 4.0 * required;
    let mut close_enough = false;
    let mut infantry: Int = 0;
    let mut vehicles: Int = 0;
    let mut center_vehicle: Option<ObjectSlot> = None;
    let mut center_vehicle_dist = dist_sqr * 10.0;
    for (member, slot) in members_with_slots(sim, group) {
        remove_goal(sim, member, slot);
        if is_held(sim, slot) || !has_ai(sim, slot) {
            continue;
        }
        if is_kind(sim, slot, KindOfType::Infantry) {
            infantry += 1;
        } else if is_kind(sim, slot, KindOfType::Vehicle) {
            if is_kind(sim, slot, KindOfType::Aircraft) {
                continue;
            }
            vehicles += 1;
        } else {
            continue;
        }
        let unit = position(sim, slot);
        let to_goal = dist_sqr_2d(&unit, pos);
        if to_goal < dist_sqr {
            dist_sqr = to_goal;
        }
        let to_center = dist_sqr_2d(&unit, &center);
        if center_vehicle.is_none() || to_center < center_vehicle_dist {
            center_vehicle = Some(slot);
            center_vehicle_dist = to_center;
        }
    }
    let Some(center_vehicle) = center_vehicle else {
        return false;
    };
    center = position(sim, center_vehicle);

    let dx = max.x - min.x;
    let dy = max.y - min.y;
    if dx * dx + dy * dy > required {
        dist_sqr = dx * dx + dy * dy;
    }
    if dist_sqr < data.min_distance_for_group * data.min_distance_for_group {
        return false;
    }
    if dist_sqr > required {
        close_enough = true;
    }
    if infantry > 6 {
        close_enough = true;
    }
    if vehicles > 4 {
        close_enough = true;
    }
    if !close_enough {
        let mut passable = true;
        for (member, slot) in members_with_slots(sim, group) {
            if !is_kind(sim, slot, KindOfType::Infantry) {
                continue;
            }
            let Some(surfaces) =
                resident_ai(sim, slot).map(|ai| ai.locomotor_set.get_valid_surfaces())
            else {
                continue;
            };
            let layer = layer_of(sim, slot);
            let from = position(sim, slot);
            if ai_seams(sim).is_line_passable(member, surfaces, layer, &from, &center) == FALSE {
                passable = false;
            }
        }
        if passable {
            close_enough = true;
        }
    }
    if !close_enough {
        return false;
    }
    let path = ai_seams(sim).find_ground_path(&center, pos, PATH_DIAMETER_IN_CELLS);
    let found = path.is_some();
    if let Some(entry) = sim.world.ai_groups.find_mut(group) {
        entry.ground_path = path.filter(|p| !p.is_empty());
    }
    found
}

fn ground_path(sim: &SimContext, group: GroupId) -> Option<Vec<Coord3D>> {
    sim.world.ai_groups.find(group)?.ground_path.clone()
}

fn drop_ground_path(sim: &mut SimContext, group: GroupId) {
    if let Some(entry) = sim.world.ai_groups.find_mut(group) {
        entry.ground_path = None;
    }
}

/// `farEnoughSqr`.
const FAR_ENOUGH_SQR: Real = (PATH_DIAMETER_IN_CELLS as Real * PATHFIND_CELL_SIZE_F)
    * (PATH_DIAMETER_IN_CELLS as Real * PATHFIND_CELL_SIZE_F);

/// The first node far from the start and the last node far from the end.
fn path_ends(path: &[Coord3D]) -> (Option<usize>, Option<usize>) {
    let first = path[0];
    let last = path[path.len() - 1];
    let start = path
        .iter()
        .position(|n| dist_sqr_2d(n, &first) > FAR_ENOUGH_SQR);
    let end = path
        .iter()
        .rposition(|n| dist_sqr_2d(n, &last) > FAR_ENOUGH_SQR);
    (start, end)
}

fn unit_vector(x: Real, y: Real) -> Coord2D {
    let mut v = Coord2D { x, y };
    v.normalize();
    v
}

/// The two kinds of group that march along the ground path.
#[derive(Clone, Copy, PartialEq, Eq)]
enum March {
    Infantry,
    Vehicles,
}

/// What both marches work out about the ground path.
struct Route {
    path: Vec<Coord3D>,
    /// `startNode`.
    start: usize,
    start_point: Coord3D,
    end_point: Coord3D,
    start_vector: Coord2D,
    end_vector: Coord2D,
    start_normal: Coord2D,
    end_normal: Coord2D,
}

fn route(path: Vec<Coord3D>, start: usize, end: usize) -> Route {
    let start_point = path[0];
    let end_point = path[path.len() - 1];
    let start_vector = unit_vector(path[start].x - start_point.x, path[start].y - start_point.y);
    let end_vector = unit_vector(end_point.x - path[end].x, end_point.y - path[end].y);
    Route {
        start_normal: unit_vector(-start_vector.y, start_vector.x),
        end_normal: unit_vector(-end_vector.y, end_vector.x),
        path,
        start,
        start_point,
        end_point,
        start_vector,
        end_vector,
    }
}

/// `(columnDelta, wideColumnDelta)` as `setTmpValue` packs them.
fn pack(wide: Int, column: Int) -> Int {
    wide.wrapping_shl(16) | (column & 0x0000_ffff)
}

fn unpack(tmp: Int) -> (Int, Int) {
    (tmp >> 16, Int::from((tmp & 0xffff) as i16))
}

/// The least filled of the columns `step` apart that is closest to `wanted`.
fn pick_column(columns: &mut [Int], step: usize, wanted: Int) -> Option<Int> {
    let mut min = 10000;
    for i in (0..columns.len()).step_by(step) {
        if columns[i] < min {
            min = columns[i];
        }
    }
    let mut delta = 10000;
    let mut best = None;
    for i in (0..columns.len()).step_by(step) {
        if columns[i] == min {
            let dx = wanted.wrapping_sub(i as Int).wrapping_abs();
            if dx < delta {
                delta = dx;
                best = Some(i);
            }
        }
    }
    let best = best?;
    columns[best] += 1;
    Some(best as Int)
}

/// `friend_moveInfantryToPos` and `friend_moveVehicleToPos`: members fan into
/// columns along the ground path. `false` when the group does not march.
fn march(
    sim: &mut SimContext,
    group: GroupId,
    pos: &Coord3D,
    source: CommandSourceType,
    kind: March,
) -> bool {
    let Some(path) = ground_path(sim, group) else {
        return false;
    };
    let (has_center, center) = center(sim, group);
    if !has_center {
        return false;
    }
    let (start, mut end) = path_ends(&path);
    if kind == March::Vehicles && end == Some(0) {
        end = None;
    }
    let (Some(start), Some(end)) = (start, end) else {
        drop_ground_path(sim, group);
        return false;
    };
    let mut r = route(path, start, end);

    let mut by_side = ObjectList::default();
    let mut human = false;
    let mut units: Int = 0;
    let mut use_end_vector = false;
    for (member, slot) in members_with_slots(sim, group) {
        if is_held(sim, slot) {
            continue;
        }
        let wanted = match kind {
            March::Infantry => KindOfType::Infantry,
            March::Vehicles => KindOfType::Vehicle,
        };
        if !is_kind(sim, slot, wanted) || !has_ai(sim, slot) {
            continue;
        }
        match kind {
            March::Infantry => {
                if is_kind(sim, slot, KindOfType::MobNexus) {
                    return false;
                }
            }
            March::Vehicles => {
                if !doing_ground_movement(sim, slot) {
                    continue;
                }
            }
        }
        if let Some(team) = sim.object(slot).map(|o| o.team_id) {
            if sim.controlling_player(team).is_some() {
                human = sim.player_is_human(team) != FALSE;
            }
        }
        let unit = position(sim, slot);
        remove_goal(sim, member, slot);
        let dx = unit.x - center.x;
        let dy = unit.y - center.y;
        by_side.insert(member, slot, dx * r.start_normal.x + dy * r.start_normal.y);
        units += 1;
        if dist_sqr_2d(&unit, &r.start_point) > dist_sqr_2d(&unit, &r.end_point) {
            use_end_vector = true;
        }
    }
    let minimum = match kind {
        March::Infantry => sim.world.ai_groups.data.min_infantry_for_group,
        March::Vehicles => sim.world.ai_groups.data.min_vehicles_for_group,
    };
    if units < minimum {
        return false;
    }

    if use_end_vector {
        r.start_vector = r.end_vector;
        r.start_normal = r.end_normal;
        let mut again = ObjectList::default();
        for (member, slot) in by_side.objects() {
            again.insert(member, slot, 0.0);
        }
        by_side = ObjectList::default();
        for (member, slot) in again.objects() {
            let unit = position(sim, slot);
            let dx = unit.x - center.x;
            let dy = unit.y - center.y;
            by_side.insert(member, slot, dx * r.start_normal.x + dy * r.start_normal.y);
        }
    }

    by_side.sort_far_to_near();
    let (narrow_columns, few) = match kind {
        March::Infantry => (3, 16),
        March::Vehicles => (2, 5),
    };
    // `setTmpValue` of each unit.
    let mut tmp: Vec<(ObjectId, Int)> = Vec::new();
    let mut by_depth = ObjectList::default();
    for (index, (member, slot)) in by_side.objects().into_iter().enumerate() {
        let index = index as Int;
        let divisor = ((units + 1) / narrow_columns).max(1);
        let mut column = 1 - (index / divisor);
        let mut wide = match kind {
            March::Infantry => {
                if column < -(narrow_columns / 2) {
                    column = -(narrow_columns / 2);
                }
                let divisor = ((units + 3) / 5).max(1);
                (2 - (index / divisor)).max(-2)
            }
            March::Vehicles => {
                if column == 0 {
                    column = -1;
                }
                let divisor = ((units + 1) / 3).max(1);
                (1 - (index / divisor)).max(-1)
            }
        };
        if units < few {
            wide = column;
        }
        tmp.push((member, pack(wide, column)));

        let unit = position(sim, slot);
        let dx = unit.x - center.x;
        let dy = unit.y - center.y;
        let mut adjust: Int = 0;
        if kind == March::Infantry {
            match move_priority(sim, slot).unwrap_or(LOCO_MOVES_FRONT) {
                LOCO_MOVES_MIDDLE => adjust = -1000,
                LOCO_MOVES_BACK => adjust = -2000,
                _ => {}
            }
        }
        by_depth.insert(
            member,
            slot,
            adjust as Real + dx * r.start_vector.x + dy * r.start_vector.y,
        );
    }
    by_depth.sort_far_to_near();

    let mut narrow = [0; 3];
    let mut wide_columns = [0; 5];
    for priority in (LOCO_MOVES_BACK..=LOCO_MOVES_FRONT).rev() {
        for (member, slot) in by_depth.objects() {
            let Some(entry) = tmp.iter().position(|e| e.0 == member) else {
                continue;
            };
            // Vehicles ignore the priority, so each is placed three times over.
            if kind == March::Infantry
                && priority != move_priority(sim, slot).unwrap_or(LOCO_MOVES_MIDDLE)
            {
                continue;
            }
            let (mut wide, mut column) = unpack(tmp[entry].1);
            match kind {
                March::Infantry => {
                    if let Some(best) = pick_column(&mut narrow, 1, 1 + column) {
                        column = best - 1;
                    }
                    if let Some(best) = pick_column(&mut wide_columns, 1, 2 + wide) {
                        wide = best - 2;
                    }
                }
                March::Vehicles => {
                    if let Some(best) = pick_column(&mut narrow, 2, 1 + column) {
                        column = best - 1;
                    }
                    if let Some(best) = pick_column(&mut wide_columns[..3], 1, 1 + wide) {
                        wide = best - 1;
                    }
                }
            }
            if units < few {
                wide = column;
            }
            tmp[entry].1 = pack(wide, column);
        }
    }

    let mut filled: [Int; 5] = [0; 5];
    let layer = sim.get_layer_for_destination(pos);
    for (member, slot) in by_depth.objects() {
        let packed = tmp.iter().find(|e| e.0 == member).map_or(0, |e| e.1);
        let (mut wide, column) = unpack(packed);
        let slot_index = (wide + 2).clamp(0, 4) as usize;
        let factor = filled[slot_index];
        filled[slot_index] = factor + 1;

        let mut points = side_points(sim, &r, slot, kind, column, factor, human);

        let mut dest = *pos;
        let limit = match kind {
            March::Infantry => 2,
            March::Vehicles => 3,
        };
        wide = wide.clamp(-limit, limit);
        let offset = match kind {
            March::Infantry => PATHFIND_CELL_SIZE_F * 2.2,
            March::Vehicles if units < 5 => PATHFIND_CELL_SIZE_F * 1.5,
            March::Vehicles => PATHFIND_CELL_SIZE_F * 3.2,
        };
        dest.x += offset * wide as Real * r.end_normal.x;
        dest.y += offset * wide as Real * r.end_normal.y;
        if factor & 1 != 0 {
            dest.x += PATHFIND_CELL_SIZE_F * r.end_normal.x;
            dest.y += PATHFIND_CELL_SIZE_F * r.end_normal.y;
        }
        if kind == March::Infantry {
            let delta = move_priority(sim, slot).unwrap_or(LOCO_MOVES_MIDDLE) - LOCO_MOVES_FRONT;
            dest.x += delta as Real * PATHFIND_CELL_SIZE_F * r.end_vector.x;
            dest.y += delta as Real * PATHFIND_CELL_SIZE_F * r.end_vector.y;
        }
        dest.x -= factor as Real * offset * r.end_vector.x;
        dest.y -= factor as Real * offset * r.end_vector.y;
        dest.z = sim.get_layer_height(dest.x, dest.y, layer);

        while let Some(&back) = points.last() {
            let along = r.end_vector.x * (dest.x - back.x) + r.end_vector.y * (dest.y - back.y);
            if along <= 0.0 {
                points.pop();
            } else {
                break;
            }
        }
        clamp_to_map(sim, &mut dest, human);
        adjust_and_claim(sim, member, &mut dest, None);
        points.push(dest);
        let mut command = parms(AiCommandType::FollowPath, source);
        command.coords = points;
        send_order(sim, slot, command);
    }
    true
}

/// The points of one unit's path, offset to its side of the corners.
fn side_points(
    sim: &SimContext,
    r: &Route,
    slot: ObjectSlot,
    kind: March,
    column: Int,
    factor: Int,
    human: bool,
) -> Vec<Coord3D> {
    let nodes = &r.path;
    let mut points: Vec<Coord3D> = Vec::new();
    let mut node = r.start;
    let mut previous = 0;
    let mut prev_pos = position(sim, slot);
    while node < nodes.len() {
        let mut dest = nodes[node];
        let Some(next) =
            (node + 1..nodes.len()).find(|&i| dist_sqr_2d(&nodes[i], &dest) > FAR_ENOUGH_SQR)
        else {
            break;
        };
        let along = Coord2D {
            x: nodes[next].x - nodes[previous].x,
            y: nodes[next].y - nodes[previous].y,
        };
        let normal = unit_vector(-along.y, along.x);
        let offset = match kind {
            March::Infantry => PATHFIND_CELL_SIZE_F * 2.1,
            March::Vehicles => PATHFIND_CELL_SIZE_F * 1.5,
        };
        dest.x += offset * column as Real * normal.x;
        dest.y += offset * column as Real * normal.y;
        let half = 0.5 * PATHFIND_CELL_SIZE_F;
        if factor & 1 != 0 {
            dest.x += half * normal.x;
            dest.y += half * normal.y;
        } else {
            dest.x -= half * normal.x;
            dest.y -= half * normal.y;
        }
        let current = Coord2D {
            x: dest.x - prev_pos.x,
            y: dest.y - prev_pos.y,
        };
        clamp_to_map(sim, &mut dest, human);
        if along.x * current.x + along.y * current.y > 0.0 {
            points.push(dest);
            prev_pos = dest;
        }
        node += 1;
        let mut i = previous + 1;
        while i < nodes.len() && i != node {
            if node < nodes.len() && dist_sqr_2d(&nodes[i], &nodes[node]) > FAR_ENOUGH_SQR {
                previous = i;
            }
            i += 1;
        }
    }
    points
}

/// `friend_moveFormationToPos`.
fn move_formation(sim: &mut SimContext, group: GroupId, pos: &Coord3D, source: CommandSourceType) {
    if !center(sim, group).0 {
        return;
    }
    let mut end_point = *pos;
    let mut segment: Option<Vec<Coord3D>> = None;
    if let Some(path) = ground_path(sim, group) {
        end_point = path[path.len() - 1];
        let (start, mut end) = path_ends(&path);
        // The end must come strictly before the start is not allowed to reach it.
        if let (Some(s), Some(e)) = (start, end) {
            if s >= e {
                end = None;
            }
        }
        match (start, end) {
            (Some(s), Some(e)) => segment = Some(path[s..=e].to_vec()),
            _ => drop_ground_path(sim, group),
        }
    }
    for (member, slot) in members_with_slots(sim, group) {
        if is_held(sim, slot) || !has_ai(sim, slot) {
            continue;
        }
        let offset = formation_offset(sim, slot);
        let mut dest = end_point;
        dest.x += offset.x;
        dest.y += offset.y;
        match &segment {
            Some(nodes) => {
                let mut points: Vec<Coord3D> = nodes
                    .iter()
                    .map(|n| Coord3D::new(n.x + offset.x, n.y + offset.y, n.z))
                    .collect();
                adjust_and_claim(sim, member, &mut dest, None);
                points.push(dest);
                let mut command = parms(AiCommandType::FollowPath, source);
                command.coords = points;
                send_order(sim, slot, command);
            }
            None => send_order(sim, slot, at(AiCommandType::MoveToPosition, source, &dest)),
        }
    }
}

/// The stealth draw of a player's order to a unit that fights while idle but
/// must not break its cover at once.
fn delay_mood_check_for_stealth(sim: &mut SimContext, slot: ObjectSlot, source: CommandSourceType) {
    if source != CommandSourceType::FromPlayer
        || !has_status(sim, slot, ObjectStatusType::CanStealth)
    {
        return;
    }
    let auto_acquire = match resident_ai(sim, slot) {
        Some(ai) => ai.data.auto_acquire_enemies_when_idle,
        None => return,
    };
    if auto_acquire == 0
        || has_status(sim, slot, ObjectStatusType::Stealthed)
        || has_status(sim, slot, ObjectStatusType::Detected)
    {
        return;
    }
    let stealth = sim
        .object(slot)
        .and_then(|o| o.stealth_slot)
        .and_then(|ms| sim.world.modules.payload(ms))
        .and_then(|m| m.as_stealth())
        .and_then(|s| s.stealth_params());
    // `canAutoAcquireWhileStealthed`.
    const AAS_IDLE_STEALTHED: UnsignedInt = 0x02;
    if stealth.is_some_and(|s| s.granted_by_special_power != FALSE)
        || auto_acquire & AAS_IDLE_STEALTHED != 0
    {
        return;
    }
    let Some(stealth) = stealth else {
        return;
    };
    let random = sim.rng_logic_int(0, LOGICFRAMES_PER_SECOND) as UnsignedInt;
    let when = sim
        .frame()
        .wrapping_add(stealth.stealth_delay)
        .wrapping_add(random);
    if let Some(ai) = resident_ai(sim, slot) {
        ai.next_mood_check_time = when;
        ai.randomly_offset_mood_check = FALSE;
    }
}

/// `groupMoveToPosition`.
pub fn group_move_to_position(
    sim: &mut SimContext,
    group: GroupId,
    pos_in: &Coord3D,
    add_waypoint: bool,
    source: CommandSourceType,
) {
    let mut pos = *pos_in;
    let mut infantry_marched = false;
    let mut vehicles_marched = false;
    let (mut formation, mut min, mut max, mut center) = min_max_and_center(sim, group);
    if add_waypoint {
        formation = false;
    }
    if !add_waypoint && !formation {
        compute_ground_path(sim, group, &pos);
        infantry_marched = march(sim, group, &pos, source, March::Infantry);
        vehicles_marched = march(sim, group, &pos, source, March::Vehicles);
    }
    recompute_if_dirty(sim, group);

    let mut tighten = false;
    let gather = sim.rules.global_data.group_move_click_to_gather_factor;
    if !formation && source == CommandSourceType::FromPlayer && gather > 0.0 {
        // `ScaleRect2D`.
        let scale = gather - 1.0;
        let dw = (max.x - min.x) * scale * 0.5;
        let dh = (max.y - min.y) * scale * 0.5;
        min.x -= dw;
        min.y -= dh;
        max.x += dw;
        max.y += dh;
        if pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y {
            tighten = true;
        }
    }

    let mut extra_margin: Real = 0.0;
    let wider = |margin: Real, other: Real| if margin > other { margin } else { other };
    for (_, slot) in members_with_slots(sim, group) {
        if is_kind(sim, slot, KindOfType::ProducedAtHelipad) {
            formation = false;
            extra_margin = wider(extra_margin, major_radius(sim, slot));
        } else if is_kind(sim, slot, KindOfType::Aircraft) {
            if has_ai(sim, slot) && !doing_ground_movement(sim, slot) {
                tighten = false;
                formation = false;
            }
            // `STD_AIRCRAFT_EXTRA_MARGIN`.
            extra_margin = wider(extra_margin, 100.0);
        }
    }
    // `STD_WAYPOINT_CLAMP_MARGIN`.
    let margin = (40.0 + extra_margin) as Int;
    clamp_waypoint(sim, &mut pos, margin);

    if tighten {
        formation = false;
        if !add_waypoint {
            let dx = ((max.x - min.x) / PATHFIND_CELL_SIZE_F) as Int;
            // The width again, not the height.
            let dy = ((max.x - min.x) / PATHFIND_CELL_SIZE_F) as Int;
            if dx.wrapping_mul(dy) < 2000 {
                group_tighten_to_position(sim, group, &pos, false, source);
                return;
            }
        }
    }
    if formation {
        compute_ground_path(sim, group, &pos);
        move_formation(sim, group, &pos, source);
        return;
    }

    let mut iter = ObjectList::default();
    for (member, slot) in members_with_slots(sim, group) {
        if is_held(sim, slot) || is_kind(sim, slot, KindOfType::Immobile) || !has_ai(sim, slot) {
            continue;
        }
        if is_kind(sim, slot, KindOfType::Infantry) && infantry_marched {
            continue;
        }
        if is_kind(sim, slot, KindOfType::Vehicle)
            && vehicles_marched
            && doing_ground_movement(sim, slot)
            && !is_kind(sim, slot, KindOfType::CliffJumper)
        {
            continue;
        }
        let unit = position(sim, slot);
        remove_goal(sim, member, slot);
        let dx = unit.x - pos.x;
        let dy = unit.y - pos.y;
        iter.insert(member, slot, 0.0 + dx * dx + dy * dy);
    }
    iter.sort_near_to_far();
    let mut first = true;
    for (member, slot) in iter.objects() {
        if let Some(object) = sim.object_mut(slot) {
            object.formation_id = NO_FORMATION_ID;
        }
        if first {
            center = position(sim, slot);
            first = false;
        }
        let dest = individual_destination(sim, &pos, member, slot, &center, false);
        delay_mood_check_for_stealth(sim, slot, source);
        let cmd = if add_waypoint {
            AiCommandType::FollowPathAppend
        } else {
            AiCommandType::MoveToPosition
        };
        send_order(sim, slot, at(cmd, source, &dest));
    }
}

/// `groupScatter`: away from a center that creeps west with each unit.
pub fn group_scatter(sim: &mut SimContext, group: GroupId, source: CommandSourceType) {
    recompute_if_dirty(sim, group);
    let (_, _, _, mut center) = min_max_and_center(sim, group);
    let mut iter = ObjectList::default();
    for (member, slot) in members_with_slots(sim, group) {
        if is_held(sim, slot) || is_kind(sim, slot, KindOfType::Immobile) || !has_ai(sim, slot) {
            continue;
        }
        let unit = position(sim, slot);
        remove_goal(sim, member, slot);
        let dx = unit.x - center.x;
        let dy = unit.y - center.y;
        iter.insert(member, slot, dx * dx + dy * dy);
    }
    iter.sort_far_to_near();
    for (_, slot) in iter.objects() {
        center.x -= 0.01;
        let unit = position(sim, slot);
        let mut dest = unit;
        let delta = unit_vector(unit.x - center.x, unit.y - center.y);
        let radius = circle_radius(sim, slot);
        dest.x += delta.x * 4.0 * radius;
        dest.y += delta.y * 4.0 * radius;
        send_order(sim, slot, at(AiCommandType::MoveToPosition, source, &dest));
    }
}

/// `getHelicopterOffset`: rings around the goal, 70 apart.
fn helicopter_offset(pos: &mut Coord3D, index: Int) {
    if index == 0 {
        return;
    }
    let circle: Real = 2.0 * PI;
    let diameter: Real = 70.0;
    let mut radius = diameter;
    let mut circumference = radius * circle;
    let mut angle: Real = 0.0;
    let mut step = diameter / circumference * circle;
    for _ in 1..index {
        angle += step;
        if angle > circle {
            radius += diameter;
            circumference = radius * circle;
            step = diameter / circumference * circle;
            angle -= circle;
        }
    }
    let center = *pos;
    pos.x = center.x + (gm_sinf(angle) * radius);
    pos.y = center.y + (gm_cosf(angle) * radius);
}

/// `groupTightenToPosition`.
pub fn group_tighten_to_position(
    sim: &mut SimContext,
    group: GroupId,
    pos: &Coord3D,
    add_waypoint: bool,
    source: CommandSourceType,
) {
    let mut iter = ObjectList::default();
    for (member, slot) in members_with_slots(sim, group) {
        if is_held(sim, slot) || is_kind(sim, slot, KindOfType::Immobile) || !has_ai(sim, slot) {
            continue;
        }
        let unit = position(sim, slot);
        iter.insert(member, slot, dist_sqr_2d(&unit, pos));
    }
    iter.sort_near_to_far();
    let mut helicopters: Int = 0;
    for (_, slot) in iter.objects() {
        if add_waypoint {
            send_order(sim, slot, at(AiCommandType::FollowPathAppend, source, pos));
        } else if is_kind(sim, slot, KindOfType::ProducedAtHelipad) {
            let mut spot = *pos;
            helicopter_offset(&mut spot, helicopters);
            helicopters = helicopters.wrapping_add(1);
            let command = at(
                AiCommandType::TightenToPosition,
                CommandSourceType::FromAi,
                &spot,
            );
            send_order(sim, slot, command);
        } else {
            send_order(sim, slot, at(AiCommandType::TightenToPosition, source, pos));
        }
    }
    for (member, slot) in iter.objects() {
        let unit = position(sim, slot);
        ai_seams(sim).update_pos(member, &unit);
    }
}

/// `groupCreateFormation`: toggles. A formation id is used up either way.
pub fn group_create_formation(sim: &mut SimContext, group: GroupId) {
    let (mut formation, _, _, center) = min_max_and_center(sim, group);
    let mut id = sim.world.ai_groups.take_formation_id();
    let members = members_with_slots(sim, group);
    if members.len() == 1 && formation_id(sim, members[0].1) != NO_FORMATION_ID {
        formation = true;
    }
    if formation {
        id = NO_FORMATION_ID;
    }
    for (_, slot) in members {
        if !has_ai(sim, slot) {
            continue;
        }
        let pos = position(sim, slot);
        if let Some(object) = sim.object_mut(slot) {
            object.formation_id = id;
            object.formation_offset = Coord2D {
                x: pos.x - center.x,
                y: pos.y - center.y,
            };
        }
    }
}

// -----------------------------------------------------------------------------
// The other orders
// -----------------------------------------------------------------------------

/// `groupIdle`.
pub fn group_idle(sim: &mut SimContext, group: GroupId, source: CommandSourceType) {
    for_each_member(sim, group, |sim, _, slot| {
        if has_ai(sim, slot) {
            send_order(sim, slot, parms(AiCommandType::Idle, source));
            delay_mood_check_for_stealth(sim, slot, source);
        } else if sim.object(slot).is_some_and(|o| o.contain_slot.is_some()) {
            for rider in sim.contain_items(slot) {
                if let Some(rider) = sim.find_object(rider) {
                    send_order(sim, rider, parms(AiCommandType::Idle, source));
                }
            }
        }
        with_spawn(sim, slot, |spawn, sim, ctx| {
            spawn.order_slaves_to_go_idle(sim, ctx, source);
        });
    });
}

fn with_spawn(
    sim: &mut SimContext,
    slot: ObjectSlot,
    f: impl FnOnce(&mut dyn crate::modules::module::SpawnBehaviorFacet, &mut dyn SimApi, ModuleContext),
) {
    let Some(module) = sim.object(slot).and_then(|o| o.spawn_slot) else {
        return;
    };
    if sim.world.modules.is_executing(module) {
        return;
    }
    let _ = sim.dispatch_module(module, |m, sim, ctx| {
        if let Some(spawn) = m.as_spawn_behavior_mut() {
            f(spawn, sim, ctx);
        }
    });
}

fn slaves_have_freedom(sim: &SimContext, slot: ObjectSlot) -> Option<bool> {
    let module = sim.object(slot)?.spawn_slot?;
    let spawn = sim.world.modules.payload(module)?.as_spawn_behavior()?;
    Some(spawn.do_slaves_have_freedom() != FALSE)
}

/// The passengers that may fire out of `slot`.
fn firing_passengers(sim: &SimContext, slot: ObjectSlot) -> Vec<ObjectSlot> {
    if sim.object(slot).is_none_or(|o| o.contain_slot.is_none())
        || sim.contain_is_passenger_allowed_to_fire(slot, ObjectId::INVALID) == FALSE
    {
        return Vec::new();
    }
    sim.contain_items(slot)
        .into_iter()
        .filter_map(|id| sim.find_object(id))
        .collect()
}

fn can_attack(result: CanAttackResult) -> bool {
    matches!(
        result,
        CanAttackResult::Possible | CanAttackResult::PossibleAfterMoving
    )
}

/// `groupAttackObjectPrivate`.
pub fn group_attack_object(
    sim: &mut SimContext,
    group: GroupId,
    forced: bool,
    victim: ObjectId,
    max_shots: Int,
    source: CommandSourceType,
) {
    let Some(victim_slot) = sim.find_object(victim) else {
        return;
    };
    let victim_pos = position(sim, victim_slot);
    let mut iter = ObjectList::default();
    for (member, slot) in members_with_slots(sim, group) {
        // Retail leaves the passengers of the group out here.
        if cfg!(feature = "retail-crc") && is_held(sim, slot) {
            continue;
        }
        let unit = position(sim, slot);
        iter.insert(member, slot, dist_sqr_2d(&unit, &victim_pos));
    }
    iter.sort_near_to_far();
    let (cmd, attack_type) = if forced {
        (AiCommandType::ForceAttackObject, ATTACK_NEW_TARGET_FORCED)
    } else {
        (AiCommandType::AttackObject, ATTACK_NEW_TARGET)
    };
    let mut command = on(cmd, source, Some(victim));
    command.int_value = max_shots;
    for (member, slot) in iter.objects() {
        for rider in firing_passengers(sim, slot) {
            let Some(target) = sim.find_object(victim) else {
                break;
            };
            let result = sim.able_to_attack_specific_object(rider, attack_type, target, source);
            if can_attack(result) {
                send_order(sim, rider, command.clone());
            }
        }
        if slaves_have_freedom(sim, slot) == Some(false) {
            if let Some(target) = sim.find_object(victim) {
                with_spawn(sim, slot, |spawn, sim, ctx| {
                    spawn.order_slaves_to_attack_target(sim, ctx, target, max_shots, source);
                });
            }
        }
        if member != victim {
            send_order(sim, slot, command.clone());
        }
    }
}

/// `groupAttackPosition`. Without a position each member shoots at its own feet.
pub fn group_attack_position(
    sim: &mut SimContext,
    group: GroupId,
    pos: Option<&Coord3D>,
    max_shots: Int,
    source: CommandSourceType,
) {
    let pos = pos.copied();
    for_each_member(sim, group, |sim, _, slot| {
        let spot = pos.unwrap_or_else(|| position(sim, slot));
        let mut command = at(AiCommandType::AttackPosition, source, &spot);
        command.int_value = max_shots;
        for rider in firing_passengers(sim, slot) {
            let result = sim.able_to_use_weapon_against_target(
                rider,
                ATTACK_NEW_TARGET,
                None,
                &spot,
                source,
            );
            if can_attack(result) {
                send_order(sim, rider, command.clone());
            }
        }
        if slaves_have_freedom(sim, slot) == Some(false) {
            with_spawn(sim, slot, |spawn, sim, ctx| {
                spawn.order_slaves_to_attack_position(sim, ctx, &spot, max_shots, source);
            });
        }
        send_order(sim, slot, command);
    });
}

/// `groupAttackMoveToPosition`.
pub fn group_attack_move_to_position(
    sim: &mut SimContext,
    group: GroupId,
    pos: &Coord3D,
    max_shots: Int,
    source: CommandSourceType,
) {
    for_each_member(sim, group, |sim, _, slot| {
        let command = if sim.object_is_able_to_attack(slot) != FALSE {
            let mut command = at(AiCommandType::AttackMoveToPosition, source, pos);
            command.int_value = max_shots;
            command
        } else {
            at(AiCommandType::MoveToPosition, source, pos)
        };
        send_order(sim, slot, command);
    });
}

/// `groupAttackTeam`. The AI order does not carry the team yet.
pub fn group_attack_team(
    sim: &mut SimContext,
    group: GroupId,
    max_shots: Int,
    source: CommandSourceType,
) {
    let mut command = parms(AiCommandType::AttackTeam, source);
    command.int_value = max_shots;
    order_all(sim, group, &command);
}

/// `groupHunt`.
pub fn group_hunt(sim: &mut SimContext, group: GroupId, source: CommandSourceType) {
    order_all(sim, group, &parms(AiCommandType::Hunt, source));
}

/// `groupExecuteRailedTransport`.
pub fn group_execute_railed_transport(
    sim: &mut SimContext,
    group: GroupId,
    source: CommandSourceType,
) {
    order_all(
        sim,
        group,
        &parms(AiCommandType::ExecuteRailedTransport, source),
    );
}

/// `groupHackInternet`.
pub fn group_hack_internet(sim: &mut SimContext, group: GroupId, source: CommandSourceType) {
    order_all(sim, group, &parms(AiCommandType::HackInternet, source));
}

/// The orders about one other object: `groupRepair`, `groupResumeConstruction`,
/// `groupGetHealed`, `groupGetRepaired`, `groupEnter`, `groupDock`, `groupExit`.
pub fn group_order_on_object(
    sim: &mut SimContext,
    group: GroupId,
    cmd: AiCommandType,
    target: ObjectId,
    source: CommandSourceType,
) {
    order_all(sim, group, &on(cmd, source, Some(target)));
}

/// `groupMoveToAndEvacuate` and `groupMoveToAndEvacuateAndExit`.
pub fn group_move_to_and_evacuate(
    sim: &mut SimContext,
    group: GroupId,
    pos: &Coord3D,
    and_exit: bool,
    source: CommandSourceType,
) {
    let cmd = if and_exit {
        AiCommandType::MoveToPositionAndEvacuateAndExit
    } else {
        AiCommandType::MoveToPositionAndEvacuate
    };
    order_all(sim, group, &at(cmd, source, pos));
}

/// The four `groupFollowWaypointPath*` orders, by their command.
pub fn group_follow_waypoint_path(
    sim: &mut SimContext,
    group: GroupId,
    cmd: AiCommandType,
    waypoint: (UnsignedInt, String),
    source: CommandSourceType,
) {
    let mut command = parms(cmd, source);
    command.waypoint = Some(waypoint);
    order_all(sim, group, &command);
}

/// `groupCombatDrop`.
pub fn group_combat_drop(
    sim: &mut SimContext,
    group: GroupId,
    target: Option<ObjectId>,
    pos: &Coord3D,
    source: CommandSourceType,
) {
    let mut command = on(AiCommandType::CombatDrop, source, target);
    command.pos = *pos;
    order_all(sim, group, &command);
}

/// `groupEvacuate`: airborne aircraft land first, structures put riders out.
pub fn group_evacuate(sim: &mut SimContext, group: GroupId, source: CommandSourceType) {
    for_each_member(sim, group, |sim, _, slot| {
        if has_ai(sim, slot) {
            if is_kind(sim, slot, KindOfType::Aircraft)
                && has_status(sim, slot, ObjectStatusType::AirborneTarget)
            {
                let mut pos = position(sim, slot);
                let layer = sim.get_highest_layer_for_destination(&pos, FALSE);
                pos.z = sim.get_layer_height(pos.x, pos.y, layer);
                send_order(
                    sim,
                    slot,
                    at(AiCommandType::MoveToPositionAndEvacuate, source, &pos),
                );
            } else {
                send_order(sim, slot, parms(AiCommandType::Evacuate, source));
            }
        } else if is_kind(sim, slot, KindOfType::Structure)
            && sim.object(slot).is_some_and(|o| o.contain_slot.is_some())
        {
            sim.contain_order_all_to_exit(slot, source, FALSE);
        }
    });
}

/// `groupGuardPosition`.
pub fn group_guard_position(
    sim: &mut SimContext,
    group: GroupId,
    pos: &Coord3D,
    guard_mode: Int,
    source: CommandSourceType,
) {
    let mut command = at(AiCommandType::GuardPosition, source, pos);
    command.int_value = guard_mode;
    order_all(sim, group, &command);
}

/// `groupGuardObject`.
pub fn group_guard_object(
    sim: &mut SimContext,
    group: GroupId,
    target: ObjectId,
    guard_mode: Int,
    source: CommandSourceType,
) {
    let mut command = on(AiCommandType::GuardObject, source, Some(target));
    command.int_value = guard_mode;
    order_all(sim, group, &command);
}

/// `groupGuardArea` and `groupAttackArea`: the area by name and center.
pub fn group_order_on_area(
    sim: &mut SimContext,
    group: GroupId,
    cmd: AiCommandType,
    area: (String, Coord3D),
    guard_mode: Int,
    source: CommandSourceType,
) {
    let mut command = parms(cmd, source);
    command.area = Some(area);
    command.int_value = guard_mode;
    order_all(sim, group, &command);
}

/// `groupCheer`.
pub fn group_cheer(sim: &mut SimContext, group: GroupId) {
    for_each_member(sim, group, |sim, _, slot| {
        sim.set_special_model_condition_state(
            slot,
            ModelConditionFlagType::SpecialCheering,
            (LOGICFRAMES_PER_SECOND * 3) as UnsignedInt,
        );
    });
}

/// `groupSell`: over a copy, since selling deselects.
pub fn group_sell(sim: &mut SimContext, group: GroupId) {
    for member in member_ids(sim, group) {
        ai_seams(sim).sell_object(member);
    }
}

/// `groupToggleOvercharge`.
pub fn group_toggle_overcharge(sim: &mut SimContext, group: GroupId) {
    for_each_member(sim, group, |sim, _, slot| {
        let modules: Vec<ModuleSlot> = sim.object(slot).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, mask)| mask.contains(FacetMask::OVERCHARGE))
                .map(|&(module, _)| module)
                .collect()
        });
        for module in modules {
            let _ = sim.dispatch_module(module, |m, sim, ctx| {
                if let Some(overcharge) = m.as_overcharge_mut() {
                    overcharge.toggle(sim, ctx);
                }
            });
        }
    });
}

/// `setAttitude`.
pub fn set_attitude(sim: &mut SimContext, group: GroupId, attitude: AttitudeType) {
    for_each_member(sim, group, |sim, _, slot| {
        if let Some(module) = ai_module(sim, slot) {
            if let Some(ai) = resident_ai(sim, slot) {
                ai.attitude = attitude;
            } else {
                let _ = sim.dispatch_module(module, |m, _, _| {
                    if let Some(ai) = m.as_ai_mut() {
                        ai.set_attitude(attitude);
                    }
                });
            }
        }
    });
}

/// `setMineClearingDetail`.
pub fn set_mine_clearing_detail(sim: &mut SimContext, group: GroupId, set: bool) {
    for_each_member(sim, group, |sim, _, slot| {
        sim.set_weapon_set_flag(slot, WeaponSetType::MineClearingDetail, Bool::from(set));
    });
}

/// `setWeaponLockForGroup`: whether any member could lock.
pub fn set_weapon_lock(
    sim: &mut SimContext,
    group: GroupId,
    weapon: WeaponSlotType,
    lock: WeaponLockType,
) -> bool {
    let mut any = false;
    for_each_member(sim, group, |sim, _, slot| {
        if sim.lock_weapon_slot(slot, weapon, lock) != FALSE {
            any = true;
        }
    });
    any
}

/// `releaseWeaponLockForGroup`.
pub fn release_weapon_lock(sim: &mut SimContext, group: GroupId, lock: WeaponLockType) {
    for_each_member(sim, group, |sim, _, slot| {
        if let Some(object) = sim.object_mut(slot) {
            object.weapon_set.release_weapon_lock(lock);
        }
    });
}

/// `setWeaponSetFlag`: only members whose template has that exact set.
pub fn set_weapon_set_flag(sim: &mut SimContext, group: GroupId, flag: WeaponSetType) {
    for_each_member(sim, group, |sim, _, slot| {
        let mut flags = WeaponSetFlags::new();
        flags.set(flag as Int, 1);
        let found = sim.object(slot).is_some_and(|o| {
            o.thing
                .template_override()
                .get_final(&sim.rules.thing_factory)
                .is_some_and(|t| t.find_weapon_template_set(&flags).is_some())
        });
        if found {
            sim.set_weapon_set_flag(slot, flag, TRUE);
        }
    });
}

/// `queueUpgrade`: each member that can afford and produce it queues it.
pub fn queue_upgrade(sim: &mut SimContext, group: GroupId, upgrade: NameKeyType) {
    let Some((kind, cost, mask)) = sim
        .rules
        .upgrade_center
        .find_upgrade_by_key(upgrade)
        .map(|u| {
            (
                u.upgrade_type(),
                u.calc_cost_to_build(None),
                u.upgrade_mask().clone(),
            )
        })
    else {
        return;
    };
    for_each_member(sim, group, |sim, _, slot| {
        let Some(team) = sim.object(slot).map(|o| o.team_id) else {
            return;
        };
        // `canAffordUpgrade`: the cost is compared as an unsigned amount.
        if sim.controlling_player(team).is_none() || sim.player_money(team) < cost as UnsignedInt {
            return;
        }
        if kind == UpgradeType::Object
            && (sim.object_has_upgrade(slot, &mask) != FALSE
                || sim.affected_by_upgrade(slot, upgrade) == FALSE)
        {
            return;
        }
        if sim.can_produce_upgrade(slot, upgrade) == FALSE {
            return;
        }
        let _ = sim.with_production(slot, |production, sim, ctx| {
            if production.can_queue_upgrade() != CanMakeType::QueueFull {
                production.queue_upgrade(sim, ctx, upgrade);
            }
        });
    });
}

// -----------------------------------------------------------------------------
// Special powers
// -----------------------------------------------------------------------------

/// What a special power order aims at.
#[derive(Debug, Clone, Copy)]
pub enum PowerTarget {
    None,
    Location {
        location: Coord3D,
        angle: Real,
        object_in_way: Option<ObjectId>,
    },
    Object(ObjectId),
}

/// `groupDoSpecialPower`, `...AtLocation` and `...AtObject`. Permission is the
/// action manager's, behind the seam.
pub fn group_do_special_power(
    sim: &mut SimContext,
    group: GroupId,
    power: UnsignedInt,
    target: PowerTarget,
    options: UnsignedInt,
) {
    for_each_member(sim, group, |sim, member, slot| {
        let Some(science) = sim
            .rules
            .special_power_store
            .find_special_power_template_by_id(power)
            .map(|t| t.required_science())
        else {
            return;
        };
        if science != SCIENCE_INVALID {
            let team = sim.object(slot).map(|o| o.team_id);
            if team.is_none_or(|team| sim.player_has_science(team, science) == FALSE) {
                return;
            }
        }
        let Some(module) = sim.find_special_power_module(slot, power) else {
            return;
        };
        if ai_seams(sim).can_do_special_power(member, power, target_of(&target), options) == FALSE {
            return;
        }
        let victim = match target {
            PowerTarget::Object(id) => match sim.find_object(id) {
                Some(slot) => Some(slot),
                None => return,
            },
            _ => None,
        };
        sim.with_special_power_module(
            module,
            &mut |sp: &mut dyn SpecialPowerFacet, sim, ctx| match (target, victim) {
                (PowerTarget::None, _) => sp.do_special_power(sim, ctx, options),
                (
                    PowerTarget::Location {
                        location, angle, ..
                    },
                    _,
                ) => {
                    sp.do_special_power_at_location(sim, ctx, &location, angle, options);
                }
                (PowerTarget::Object(_), Some(victim)) => {
                    sp.do_special_power_at_object(sim, ctx, victim, options);
                }
                (PowerTarget::Object(_), None) => {}
            },
        );
        if let Some(object) = sim.object_mut(slot) {
            object.set_undetected_defector(FALSE);
        }
    });
}

fn target_of(target: &PowerTarget) -> (Option<ObjectId>, Option<Coord3D>) {
    match *target {
        PowerTarget::None => (None, None),
        PowerTarget::Location {
            location,
            object_in_way,
            ..
        } => (object_in_way, Some(location)),
        PowerTarget::Object(id) => (Some(id), None),
    }
}

/// `groupOverrideSpecialPowerDestination`.
pub fn group_override_special_power_destination(
    sim: &mut SimContext,
    group: GroupId,
    power_type: Int,
    location: &Coord3D,
) {
    for member in member_ids(sim, group) {
        ai_seams(sim).override_special_power_destination(member, power_type, location);
    }
}

/// `getSpecialPowerSourceObject`.
pub fn special_power_source(
    sim: &SimContext,
    group: GroupId,
    power: UnsignedInt,
) -> Option<ObjectId> {
    sim.rules
        .special_power_store
        .find_special_power_template_by_id(power)?;
    members_with_slots(sim, group)
        .into_iter()
        .find(|&(_, slot)| sim.find_special_power_module(slot, power).is_some())
        .map(|(id, _)| id)
}

// -----------------------------------------------------------------------------
// Questions
// -----------------------------------------------------------------------------

fn facet_flag(
    sim: &mut SimContext,
    slot: ObjectSlot,
    resident: impl Fn(&AiUpdate) -> bool,
    facet: impl Fn(&dyn crate::modules::module::AiUpdateFacet) -> Bool,
) -> bool {
    if let Some(ai) = resident_ai(sim, slot) {
        return resident(ai);
    }
    ai_module(sim, slot)
        .and_then(|ms| sim.world.modules.payload(ms))
        .and_then(|m| m.as_ai())
        .is_some_and(|ai| facet(ai) != FALSE)
}

fn effectively_dead(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

/// `isIdle`.
pub fn is_idle(sim: &mut SimContext, group: GroupId) -> bool {
    let mut idle = true;
    for (_, slot) in members_with_slots(sim, group) {
        if !has_ai(sim, slot) {
            continue;
        }
        idle = facet_flag(sim, slot, |ai| ai.is_idle() != FALSE, |ai| ai.is_idle())
            || effectively_dead(sim, slot);
        if !idle {
            return false;
        }
    }
    idle
}

/// `isBusy`.
pub fn is_busy(sim: &mut SimContext, group: GroupId) -> bool {
    let mut busy = true;
    for (_, slot) in members_with_slots(sim, group) {
        if !has_ai(sim, slot) {
            continue;
        }
        busy = facet_flag(
            sim,
            slot,
            |ai| ai.machine.is_in_busy_state() != FALSE,
            |_| FALSE,
        ) && !effectively_dead(sim, slot);
        if !busy {
            return false;
        }
    }
    busy
}

/// `isGroupAiDead`.
pub fn is_ai_dead(sim: &SimContext, group: GroupId) -> bool {
    members_with_slots(sim, group)
        .into_iter()
        .all(|(_, slot)| effectively_dead(sim, slot))
}
