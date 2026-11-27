//! `PKT-401`: `ActionManager` (`Common/RTS/ActionManager.cpp`), in part. Packet
//! doc: `packets/phase6/PKT-401_ActionManager.md`. Golden test:
//! `tests/golden_s16_dozer_worker_frames.rs`.
//!
//! Only the two questions the dozer and worker AI ask (AI plan slice S16):
//! `canRepairObject` and `canResumeConstructionOf`.
//!
//! `isObjectShroudedForAction` is not answered: the port has no per-object shroud
//! status, so every target counts as seen. The reference refuses a human
//! player's order on a target that is fogged or shrouded for him, which his own
//! structures never are.

use crate::arena::ObjectSlot;
use crate::modules::module::CommandSourceType;
use crate::modules::update::dozer_ai_update::{dozer_task_of, DOZER_TASK_BUILD};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::SimApi;
use zh_common::types::{Bool, Relationship, FALSE, TRUE};

fn under_construction(sim: &dyn SimApi, object: ObjectSlot) -> bool {
    sim.object(object)
        .is_some_and(|o| o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE)
}

fn effectively_dead(sim: &dyn SimApi, object: ObjectSlot) -> bool {
    sim.object(object)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

fn kind(sim: &dyn SimApi, object: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(object, kind) != FALSE
}

/// `canRepairObject`.
pub fn can_repair_object(
    sim: &dyn SimApi,
    obj: ObjectSlot,
    target: Option<ObjectSlot>,
    _source: CommandSourceType,
) -> Bool {
    let Some(target) = target else {
        return FALSE;
    };
    if sim.player_relationship(obj, target) == Relationship::Enemies {
        return FALSE;
    }
    if effectively_dead(sim, target) {
        return FALSE;
    }
    if kind(sim, target, KindOfType::Bridge) || kind(sim, target, KindOfType::BridgeTower) {
        return FALSE;
    }
    if under_construction(sim, obj) || under_construction(sim, target) {
        return FALSE;
    }
    if kind(sim, target, KindOfType::RebuildHole)
        || !kind(sim, obj, KindOfType::Dozer)
        || !kind(sim, target, KindOfType::Structure)
    {
        return FALSE;
    }
    if sim.object_health(target) == sim.object_max_health(target) {
        return FALSE;
    }
    if sim.object(obj).is_some_and(|o| o.contained_by.is_some()) {
        return FALSE;
    }
    TRUE
}

/// `canResumeConstructionOf`.
pub fn can_resume_construction_of(
    sim: &mut dyn SimApi,
    obj: ObjectSlot,
    target: Option<ObjectSlot>,
    _source: CommandSourceType,
) -> Bool {
    let Some(target) = target else {
        return FALSE;
    };
    if !kind(sim, obj, KindOfType::Dozer) {
        return FALSE;
    }
    if cfg!(feature = "retail-crc") {
        if sim.player_relationship(obj, target) != Relationship::Allies {
            return FALSE;
        }
    } else {
        let player = |s: ObjectSlot| sim.object(s).map(|o| sim.player_index(o.team_id));
        if player(obj) != player(target) {
            return FALSE;
        }
    }
    if !under_construction(sim, target) || effectively_dead(sim, obj) {
        return FALSE;
    }
    let Some((builder_id, target_id)) = sim.object(target).map(|o| (o.builder_id, o.id)) else {
        return FALSE;
    };
    if let Some(builder) = sim.find_object(builder_id) {
        if (cfg!(feature = "retail-crc") || !effectively_dead(sim, builder))
            && dozer_task_of(sim, builder) == Some((DOZER_TASK_BUILD, target_id))
        {
            return FALSE;
        }
    }
    TRUE
}
