//! `PKT-409`: `Squad` (`Squad.cpp`): the object ids of a selection or a hotkey
//! group. The type is defined beside the command dispatcher that fills it
//! (`zh_common`); the parts that need the world are here.
//!
//! Golden vectors: `docs/golden_vectors/PKT-423.json`, shared with the groups
//! (`crates/zh_logic/tests/golden_pkt423_ai_group.rs`).
//!
//! `m_objectsCached` is not kept: the queries return their list. So `xfer` cannot
//! make the retail load check that the cache is empty. `Squad::crc` is empty.

use crate::ai::ai_group::{self, GroupId};
use crate::sim::{SimContext, SimView};
use zh_common::types::{ObjectId, TeamId, UnsignedShort, XferVersion};
use zh_common::xfer::{Xfer, XferError, XferMode};

pub use zh_common::runtime::game_logic_dispatch::{PlayerSquads, Squad};

/// `Object::isSelectable`. `Object::setSelectable` is not ported, so the
/// object's own flag is its template's `SELECTABLE`.
pub fn is_selectable(sim: &SimContext, id: ObjectId) -> bool {
    use crate::rules::kind_of::KindOfType;
    use crate::rules::object_status_types::ObjectStatusType;
    let Some(slot) = sim.find_object(id) else {
        return false;
    };
    let kind = |k| sim.object_is_kind_of(slot, k) != 0;
    if kind(KindOfType::AlwaysSelectable) {
        return true;
    }
    let Some(object) = sim.object(slot) else {
        return false;
    };
    kind(KindOfType::Selectable)
        && object.status.test(ObjectStatusType::Unselectable.as_i32()) == 0
        && object.is_effectively_dead() == 0
        && (cfg!(feature = "retail-crc") || !kind(KindOfType::NoSelect))
}

/// `getAllObjects`: forgets the ids whose object is gone, and returns the rest.
pub fn all_objects(squad: &mut Squad, sim: &SimContext) -> Vec<ObjectId> {
    squad
        .ids
        .retain(|&id| sim.find_object(ObjectId(id as _)).is_some());
    squad.ids.iter().map(|&id| ObjectId(id as _)).collect()
}

/// `getLiveObjects`: [`all_objects`] without those that cannot be selected.
pub fn live_objects(squad: &mut Squad, sim: &SimContext) -> Vec<ObjectId> {
    let mut objects = all_objects(squad, sim);
    objects.retain(|&id| is_selectable(sim, id));
    objects
}

/// `squadFromTeam`: the members of the team, in team list order.
pub fn squad_from_team(squad: &mut Squad, sim: &SimContext, team: TeamId, clear_first: bool) {
    if clear_first {
        squad.ids.clear();
    }
    if let Some(members) = sim.world.team_members.get(&team) {
        squad.ids.extend(
            members
                .iter()
                .filter_map(|&slot| sim.object(slot))
                .map(|o| o.id.0 as u32),
        );
    }
}

/// `squadFromAIGroup`. The ids are assigned over the old ones, so whether the
/// caller asked to clear first makes no difference.
pub fn squad_from_ai_group(squad: &mut Squad, sim: &SimContext, group: GroupId) {
    if sim.world.ai_groups.find(group).is_none() {
        return;
    }
    squad.ids = ai_group::member_ids(sim, group)
        .into_iter()
        .map(|id| id.0 as u32)
        .collect();
}

/// `aiGroupFromSquad`.
pub fn ai_group_from_squad(squad: &mut Squad, sim: &mut SimContext, group: GroupId) {
    for id in live_objects(squad, sim) {
        ai_group::add(sim, group, id);
    }
}

/// `isOnSquad`.
pub fn is_on_squad(squad: &Squad, id: ObjectId) -> bool {
    squad.ids.contains(&(id.0 as u32))
}

/// `Squad::xfer`. Loading appends to what the squad holds.
pub fn xfer_squad(squad: &mut Squad, xfer: &mut dyn Xfer) -> Result<(), XferError> {
    let current: XferVersion = 1;
    let mut version = current;
    xfer.xfer_version(&mut version, current)?;
    let mut count = squad.ids.len() as UnsignedShort;
    xfer.xfer_unsigned_short(&mut count)?;
    if xfer.mode() == XferMode::Load {
        for _ in 0..count {
            let mut id = ObjectId::INVALID;
            xfer.xfer_object_id(&mut id)?;
            squad.ids.push(id.0 as u32);
        }
    } else {
        for &id in &squad.ids {
            let mut id = ObjectId(id as _);
            xfer.xfer_object_id(&mut id)?;
        }
    }
    Ok(())
}
