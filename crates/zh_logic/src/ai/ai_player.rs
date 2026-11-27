//! `PKT-426` (partial): `AIPlayer` as far as a game start reaches it: the
//! constructor's state, `newMap`, `buildStructureNow` and its `xfer`.
//!
//! Not ported: `update` and everything it drives (base and team building, repairs,
//! upgrades, superweapon targeting), so the build and ready queues stay empty.
//!
//! Golden tests: `crates/zh_logic/tests/golden_ai_new_map.rs`
//! (`docs/golden_vectors/AiNewMap.json`, recorded from the running reference engine
//! by `tests/cpp_harness/gen_ai_new_map.py`) and `golden_game_start.rs`.

use zh_common::math::{gm_sqrtf, Coord2D, Coord3D};
use zh_common::types::{Bool, Int, ObjectId, Real, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError, XferMode};

use crate::ai::ai_skirmish_player::{self, AiSkirmishPlayer};
use crate::arena::ObjectSlot;
use crate::object::object_script_status_bits::OBJECT_STATUS_SCRIPT_UNSELLABLE;
use crate::player::build_assistant::{build_object_now, BuildContext};
use crate::player::seams::{AiKind, PlayerIndex};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use crate::spatial::sides_list::BuildListInfo;

/// `MAX_STRUCTURES_TO_REPAIR`.
pub const MAX_STRUCTURES_TO_REPAIR: usize = 2;
/// `INVALID_SKILLSET_SELECTION`.
pub const INVALID_SKILLSET_SELECTION: Int = -1;
/// `DIFFICULTY_EASY`, `DIFFICULTY_NORMAL`, `DIFFICULTY_HARD`.
pub const DIFFICULTY_EASY: Int = 0;
pub const DIFFICULTY_NORMAL: Int = 1;
pub const DIFFICULTY_HARD: Int = 2;

/// The state of one computer player (`Player::m_ai`). The constructor leaves
/// some members to the zeroed memory pool block.
#[derive(Debug, Clone, PartialEq)]
pub struct AiPlayer {
    pub player: PlayerIndex,
    pub ready_to_build_team: Bool,
    pub ready_to_build_structure: Bool,
    pub team_timer: Int,
    pub structure_timer: Int,
    pub build_delay: Int,
    pub team_delay: Int,
    pub team_seconds: Int,
    pub cur_warehouse_id: ObjectId,
    pub frame_last_building_built: Int,
    pub difficulty: Int,
    pub skillset_selector: Int,
    pub base_center: Coord3D,
    pub base_center_set: Bool,
    pub base_radius: Real,
    pub structures_to_repair: [ObjectId; MAX_STRUCTURES_TO_REPAIR],
    pub repair_dozer: ObjectId,
    pub structures_in_queue: Int,
    pub dozer_queued_for_repair: Bool,
    pub dozer_is_repairing: Bool,
    pub bridge_timer: Int,
    /// The members `AISkirmishPlayer` adds; `None` for a plain `AIPlayer`.
    pub skirmish: Option<AiSkirmishPlayer>,
}

impl AiPlayer {
    /// The constructor. `difficulty` is the script engine's global difficulty and
    /// `team_seconds` is `AIData::m_teamSeconds`.
    pub fn new(
        kind: AiKind,
        player: PlayerIndex,
        frame: Int,
        difficulty: Int,
        team_seconds: Int,
    ) -> Self {
        Self {
            player,
            ready_to_build_team: FALSE,
            ready_to_build_structure: FALSE,
            team_timer: 2,
            structure_timer: 2,
            build_delay: 0,
            team_delay: 0,
            team_seconds,
            cur_warehouse_id: ObjectId::INVALID,
            frame_last_building_built: frame,
            difficulty,
            skillset_selector: INVALID_SKILLSET_SELECTION,
            base_center: Coord3D::default(),
            base_center_set: FALSE,
            base_radius: 0.0,
            structures_to_repair: [ObjectId::INVALID; MAX_STRUCTURES_TO_REPAIR],
            repair_dozer: ObjectId::INVALID,
            structures_in_queue: 0,
            dozer_queued_for_repair: FALSE,
            dozer_is_repairing: FALSE,
            bridge_timer: 0,
            skirmish: (kind == AiKind::Skirmish).then(AiSkirmishPlayer::default),
        }
    }

    /// `AIPlayer::xfer` or `AISkirmishPlayer::xfer`, with empty team queues.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        match self.skirmish.take() {
            None => self.xfer_base(xfer),
            Some(mut skirmish) => {
                let mut version: XferVersion = 1;
                let result = xfer
                    .xfer_version(&mut version, 1)
                    .and_then(|()| self.xfer_base(xfer))
                    .and_then(|()| skirmish.xfer_members(xfer));
                self.skirmish = Some(skirmish);
                result
            }
        }
    }

    fn xfer_base(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        // The build queue and the ready queue.
        for _ in 0..2 {
            let mut count = 0;
            xfer.xfer_unsigned_short(&mut count)?;
            if count != 0 && xfer.mode() == XferMode::Load {
                return Err(XferError::InvalidData);
            }
        }
        let mut player = self.player;
        xfer.xfer_int(&mut player)?;
        if player != self.player {
            return Err(XferError::InvalidData);
        }
        xfer_flag(xfer, &mut self.ready_to_build_team)?;
        xfer_flag(xfer, &mut self.ready_to_build_structure)?;
        xfer.xfer_int(&mut self.team_timer)?;
        xfer.xfer_int(&mut self.structure_timer)?;
        xfer.xfer_int(&mut self.build_delay)?;
        xfer.xfer_int(&mut self.team_delay)?;
        xfer.xfer_int(&mut self.team_seconds)?;
        xfer.xfer_object_id(&mut self.cur_warehouse_id)?;
        xfer.xfer_int(&mut self.frame_last_building_built)?;
        xfer.xfer_int(&mut self.difficulty)?;
        xfer.xfer_int(&mut self.skillset_selector)?;
        xfer.xfer_coord3d(&mut self.base_center)?;
        xfer_flag(xfer, &mut self.base_center_set)?;
        xfer.xfer_real(&mut self.base_radius)?;
        for id in &mut self.structures_to_repair {
            xfer.xfer_object_id(id)?;
        }
        xfer.xfer_object_id(&mut self.repair_dozer)?;
        xfer.xfer_int(&mut self.structures_in_queue)?;
        xfer_flag(xfer, &mut self.dozer_queued_for_repair)?;
        xfer_flag(xfer, &mut self.dozer_is_repairing)?;
        xfer.xfer_int(&mut self.bridge_timer)
    }
}

fn xfer_flag(xfer: &mut dyn Xfer, value: &mut Bool) -> Result<(), XferError> {
    let mut byte = i8::from(*value != FALSE);
    xfer.xfer_byte(&mut byte)?;
    *value = if byte != 0 { TRUE } else { FALSE };
    Ok(())
}

/// `PlayerList::newMap`: every computer player's `newMap`, in player order.
pub fn players_new_map(cx: &mut BuildContext<'_>) {
    for player in 0..cx.players.player_count() {
        let Some(kind) = cx.players.player(player).and_then(|p| p.ai) else {
            continue;
        };
        match kind {
            AiKind::Standard => new_map(cx, player),
            AiKind::Skirmish => ai_skirmish_player::new_map(cx, player),
        }
    }
}

/// `AIPlayer::newMap`: lists the player's factories, then builds what the map's
/// build list says is there from the start.
pub fn new_map(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    // The reference takes the head of the list before adding to its front, so the
    // added entries are not built.
    let owned: Vec<ObjectSlot> = cx
        .sim
        .world
        .objects
        .world_iter()
        .filter(|&slot| {
            let Some(obj) = cx.sim.world.objects.get(slot) else {
                return false;
            };
            cx.sim.controlling_player(obj.team_id) == Some(player)
                && cx.sim.production_module(slot).is_some()
        })
        .collect();
    let added = owned.len();
    for slot in owned {
        add_to_build_list(cx, player, slot);
    }
    compute_center_and_radius_of_base(cx, player);
    build_initial_structures(cx, player, added);
}

/// `Player::addToBuildList`.
fn add_to_build_list(cx: &mut BuildContext<'_>, player: PlayerIndex, slot: ObjectSlot) {
    let Some(obj) = cx.sim.world.objects.get(slot) else {
        return;
    };
    let mut info = BuildListInfo::new();
    info.object_id = obj.id;
    info.location = *obj.thing.get_position();
    info.angle = obj.thing.get_orientation();
    info.num_rebuilds = 0;
    info.template_name = obj
        .thing
        .template_override()
        .get_final(&cx.sim.rules.thing_factory)
        .map_or_else(String::new, |t| t.get_name().to_owned());
    if let Some(p) = cx.players.player_mut(player) {
        p.build_list.insert(0, info);
    }
}

/// The loop both `newMap`s end in, from entry `first` of the build list.
pub(super) fn build_initial_structures(
    cx: &mut BuildContext<'_>,
    player: PlayerIndex,
    first: usize,
) {
    let mut at = first;
    while let Some(info) = cx.players.player(player).and_then(|p| p.build_list.get(at)) {
        let initially_built = info.is_initially_built != FALSE;
        let known = !info.template_name.is_empty()
            && cx
                .sim
                .rules
                .thing_factory
                .find_template_id(&info.template_name)
                .is_some();
        if known {
            if initially_built {
                build_structure_now(cx, player, at);
            } else if let Some(p) = cx.players.player_mut(player) {
                p.build_list[at].increment_num_rebuilds();
            }
        }
        at += 1;
    }
}

/// `computeCenterAndRadiusOfBase` over the `(location, bounding circle radius)` of
/// every build list entry with a known template: the mean location, the distance to
/// the farthest edge, and whether there was any entry.
pub fn base_center_and_radius(buildings: &[(Coord3D, Real)]) -> (Coord2D, Real, Bool) {
    let mut total = Coord2D { x: 0.0, y: 0.0 };
    for (at, _) in buildings {
        total.x += at.x;
        total.y += at.y;
    }
    let count = buildings.len() as Int;
    if count > 0 {
        total.x /= count as Real;
        total.y /= count as Real;
    }
    let mut max_squared: Real = 0.0;
    for (at, circle) in buildings {
        let mut dx = at.x - total.x;
        let mut dy = at.y - total.y;
        if dx < 0.0 {
            dx = -dx;
        }
        if dy < 0.0 {
            dy = -dy;
        }
        let radius = circle * 0.4; // determinism-ok: `getBoundingCircleRadius()*0.4f`
        dx += radius;
        dy += radius;
        let squared = dx * dx + dy * dy;
        if squared > max_squared {
            max_squared = squared;
        }
    }
    let set = if count > 0 { TRUE } else { FALSE };
    (total, gm_sqrtf(max_squared), set)
}

/// `computeCenterAndRadiusOfBase( &m_baseCenter, &m_baseRadius )`.
pub(super) fn compute_center_and_radius_of_base(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    let buildings: Vec<(Coord3D, Real)> = cx
        .players
        .player(player)
        .map_or(&[][..], |p| p.build_list.as_slice())
        .iter()
        .filter(|info| !info.template_name.is_empty())
        .filter_map(|info| {
            let template = cx
                .sim
                .rules
                .thing_factory
                .find_template_final(&info.template_name)?;
            Some((
                info.location,
                template
                    .get_template_geometry_info()
                    .get_bounding_circle_radius(),
            ))
        })
        .collect();
    let (center, radius, set) = base_center_and_radius(&buildings);
    if let Some(ai) = cx.sim.world.ai_players.get_mut(&player) {
        ai.base_center_set = set;
        ai.base_center.x = center.x;
        ai.base_center.y = center.y;
        ai.base_radius = radius;
    }
}

/// `buildStructureNow` of build list entry `at`. The rally point offset is read by
/// the reference and never applied (`gotOffset;` instead of an assignment).
fn build_structure_now(cx: &mut BuildContext<'_>, player: PlayerIndex, at: usize) {
    let Some(info) = cx
        .players
        .player(player)
        .and_then(|p| p.build_list.get(at))
        .cloned()
    else {
        return;
    };
    let Some(template) = cx
        .sim
        .rules
        .thing_factory
        .find_template_id(&info.template_name)
    else {
        return;
    };
    let Some(building) = build_object_now(cx, template, &info.location, info.angle, player) else {
        return;
    };
    let Some(id) = cx.sim.world.objects.id_of(building) else {
        return;
    };
    // `updateObjValuesFromMapProperties` with the entry's name, health and
    // sellability.
    if !info.building_name.is_empty() {
        cx.named.push((info.building_name.clone(), id));
    }
    #[cfg(feature = "retail-crc")]
    cx.sim.set_initial_health(building, info.health);
    if let Some(obj) = cx.sim.world.objects.get_mut(building) {
        if info.unsellable != FALSE {
            obj.script_status |= OBJECT_STATUS_SCRIPT_UNSELLABLE;
        } else {
            obj.script_status &= !OBJECT_STATUS_SCRIPT_UNSELLABLE;
        }
    }
    let frame = cx.sim.world.frame;
    if let Some(p) = cx.players.player_mut(player) {
        p.build_list[at].object_id = id;
        p.build_list[at].object_timestamp = frame.wrapping_add(1);
    }
    cx.sim
        .clear_status(building, ObjectStatusType::UnderConstruction);
    cx.sim
        .clear_status(building, ObjectStatusType::Reconstructing);
    cx.sim.update_upgrade_modules(building);
    if !info.script.is_empty() {
        cx.unported("ScriptEngine::runObjectScript");
    }
    check_for_supply_center(cx, player, at, building);
}

/// `checkForSupplyCenter`.
fn check_for_supply_center(
    cx: &mut BuildContext<'_>,
    player: PlayerIndex,
    at: usize,
    building: ObjectSlot,
) {
    let rules = &cx.sim.rules;
    let docks_supplies = cx
        .sim
        .world
        .objects
        .get(building)
        .and_then(|o| o.thing.template_override().get_final(&rules.thing_factory))
        .is_some_and(|t| {
            t.get_behavior_module_info()
                .nuggets()
                .iter()
                .any(|n| n.name == "SupplyCenterDockUpdate")
        });
    if !docks_supplies {
        return;
    }
    let difficulty = cx
        .sim
        .world
        .ai_players
        .get(&player)
        .map_or(DIFFICULTY_NORMAL, |ai| ai.difficulty);
    let Some(p) = cx.players.player_mut(player) else {
        return;
    };
    let mut desired = 0;
    for side in &cx.sim.rules.ai_data.side_info {
        if side.side == p.side {
            match difficulty {
                DIFFICULTY_EASY => desired = side.easy,
                DIFFICULTY_NORMAL => desired = side.normal,
                DIFFICULTY_HARD => desired = side.hard,
                _ => {}
            }
        }
    }
    let info = &mut p.build_list[at];
    info.is_supply_building = TRUE;
    info.current_gatherers = -1;
    info.desired_gatherers = desired.wrapping_add(1);
}

/// `AIPlayer::onStructureProduced`.
pub fn on_structure_produced(cx: &mut BuildContext<'_>, player: PlayerIndex, building: ObjectSlot) {
    if let Some(ai) = cx.sim.world.ai_players.get_mut(&player) {
        ai.team_delay = 0;
        ai.build_delay = 0;
    }
    let Some(id) = cx.sim.world.objects.id_of(building) else {
        return;
    };
    let list = cx
        .players
        .player(player)
        .map_or(&[][..], |p| p.build_list.as_slice());
    if list.iter().any(|info| info.object_id == id) {
        // Finishing a listed structure: its map properties and script.
        cx.unported("AIPlayer::onStructureProduced of a listed structure");
        return;
    }
    // A listed structure may be a rebuild hole which made this one.
    let holes = list.iter().any(|info| {
        cx.sim
            .world
            .objects
            .slot_by_id(info.object_id)
            .is_some_and(|s| cx.sim.object_is_kind_of(s, KindOfType::RebuildHole) != FALSE)
    });
    if holes {
        cx.unported("AIPlayer::onStructureProduced with a rebuild hole");
    }
}
