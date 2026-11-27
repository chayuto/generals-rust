//! `PKT-427` (partial): `AISkirmishPlayer` as far as a game start reaches it:
//! `newMap` with `adjustBuildList`, which moves the faction's build list to the
//! player's start spot and turns it toward the middle of the map.
//!
//! Not ported: `update` and what it drives.
//!
//! Golden tests: `crates/zh_logic/tests/golden_ai_new_map.rs`
//! (`docs/golden_vectors/AiNewMap.json`, recorded from the running reference engine
//! by `tests/cpp_harness/gen_ai_new_map.py`) and `golden_game_start.rs`.

use zh_common::math::{gm_cosf, gm_sinf, Coord3D, Region3D};
use std::collections::BTreeMap;

use zh_common::types::{Bool, Int, Real, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

use crate::ai::ai_player::{
    build_initial_structures, compute_center_and_radius_of_base, AiPlayer,
};
use crate::player::build_assistant::{BuildContext, PI};
use crate::player::seams::PlayerIndex;
use crate::rules::kind_of::KindOfType;
use crate::sim::SimView;
use crate::spatial::sides_list::BuildListInfo;

/// The members `AISkirmishPlayer::xfer` adds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AiSkirmishPlayer {
    pub cur_front_base_defense: Int,
    pub cur_flank_base_defense: Int,
    pub cur_front_left_defense_angle: Real,
    pub cur_front_right_defense_angle: Real,
    pub cur_left_flank_left_defense_angle: Real,
    pub cur_left_flank_right_defense_angle: Real,
    pub cur_right_flank_left_defense_angle: Real,
    pub cur_right_flank_right_defense_angle: Real,
    /// Not saved: when to look for an enemy again.
    pub frame_to_check_enemy: UnsignedInt,
    /// Not saved.
    pub current_enemy: Option<PlayerIndex>,
}

/// What `acquireEnemy` reads of the players.
pub trait EnemyView {
    fn player_count(&self) -> Int;
    /// `player`'s relationship to `other`'s default team is `ENEMIES`.
    fn is_enemy(&self, player: PlayerIndex, other: PlayerIndex) -> bool;
    fn has_any_objects(&self, player: PlayerIndex) -> bool;
    fn has_any_units(&self, player: PlayerIndex) -> bool;
    fn has_any_build_facility(&self, player: PlayerIndex) -> bool;
    /// Where the player's structures stand, in `iterateObjects` order.
    fn structure_positions(&self, player: PlayerIndex) -> Vec<Coord3D>;
}

/// `getAiEnemy`: looks again every five seconds. A player whose AI is not a
/// skirmish one, or who has none, has no enemy.
pub fn get_ai_enemy(
    ai_players: &mut BTreeMap<Int, AiPlayer>,
    view: &dyn EnemyView,
    frame: UnsignedInt,
    player: PlayerIndex,
) -> Option<PlayerIndex> {
    let skirmish = ai_players.get_mut(&player)?.skirmish.as_mut()?;
    if frame >= skirmish.frame_to_check_enemy {
        skirmish.frame_to_check_enemy = frame.wrapping_add(5 * 30);
        acquire_enemy(ai_players, view, frame, player);
    }
    ai_players.get(&player)?.skirmish.as_ref()?.current_enemy
}

/// `PartitionManager`'s `HUGE_DIST`.
const HUGE_DIST: Real = 1_000_000.0;

/// `acquireEnemy`: the enemy kept while it has units and build facilities, else
/// the nearest one, counting those other skirmish AIs already fight as farther.
fn acquire_enemy(
    ai_players: &mut BTreeMap<Int, AiPlayer>,
    view: &dyn EnemyView,
    frame: UnsignedInt,
    player: PlayerIndex,
) {
    let Some(ai) = ai_players.get(&player) else {
        return;
    };
    let base = ai.base_center;
    let current = ai.skirmish.as_ref().and_then(|s| s.current_enemy);
    let in_bad_shape =
        |p: PlayerIndex| !view.has_any_units(p) || !view.has_any_build_facility(p);
    if current.is_some_and(|enemy| !in_bad_shape(enemy)) {
        return;
    }
    let mut best = None;
    let mut best_distance = HUGE_DIST * HUGE_DIST;
    for i in 0..view.player_count() {
        if !view.is_enemy(player, i) || !view.has_any_objects(i) {
            continue;
        }
        let bad = in_bad_shape(i);
        // `getPlayerStructureBounds`, all zero without structures.
        let positions = view.structure_positions(i);
        let first = positions.first().copied().unwrap_or_default();
        let (mut lo_x, mut lo_y, mut hi_x, mut hi_y) = (first.x, first.y, first.x, first.y);
        for p in &positions {
            if p.x < lo_x {
                lo_x = p.x;
            }
            if p.y < lo_y {
                lo_y = p.y;
            }
            if p.x > hi_x {
                hi_x = p.x;
            }
            if p.y > hi_y {
                hi_y = p.y;
            }
        }
        let x = lo_x + (hi_x - lo_x) / 2.0;
        let y = lo_y + (hi_y - lo_y) / 2.0;
        let mut distance = (x - base.x) * (x - base.x) + (y - base.y) * (y - base.y);
        if bad {
            distance = HUGE_DIST * HUGE_DIST * 0.5;
        }
        for k in 0..view.player_count() {
            if k == i {
                continue;
            }
            // `getCurrentEnemy`, which may make that AI look again.
            if get_ai_enemy(ai_players, view, frame, k) == Some(i) {
                distance += (500 * 500) as Real;
            }
            if get_ai_enemy(ai_players, view, frame, k) == Some(player) {
                distance -= (25 * 25) as Real;
                if distance < 0.0 {
                    distance = 0.0;
                }
            }
        }
        if distance < best_distance {
            best = Some(i);
            best_distance = distance;
        }
    }
    if let (Some(best), Some(skirmish)) = (
        best,
        ai_players.get_mut(&player).and_then(|a| a.skirmish.as_mut()),
    ) {
        skirmish.current_enemy = Some(best);
    }
}

impl AiSkirmishPlayer {
    pub(super) fn xfer_members(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        xfer.xfer_int(&mut self.cur_front_base_defense)?;
        xfer.xfer_int(&mut self.cur_flank_base_defense)?;
        xfer.xfer_real(&mut self.cur_front_left_defense_angle)?;
        xfer.xfer_real(&mut self.cur_front_right_defense_angle)?;
        xfer.xfer_real(&mut self.cur_left_flank_left_defense_angle)?;
        xfer.xfer_real(&mut self.cur_left_flank_right_defense_angle)?;
        xfer.xfer_real(&mut self.cur_right_flank_left_defense_angle)?;
        xfer.xfer_real(&mut self.cur_right_flank_right_defense_angle)
    }
}

/// The angle a base at `start` is turned by: its ninth of the map picks the
/// direction.
fn base_angle(start: &Coord3D, bounds: &Region3D, rotate: bool) -> Real {
    let width = bounds.hi.x - bounds.lo.x;
    let height = bounds.hi.y - bounds.lo.y;
    let mut grid = 0;
    if start.x > bounds.lo.x + width / 3.0 {
        grid += 1;
    }
    if start.x > bounds.lo.x + 2.0 * width / 3.0 {
        grid += 1;
    }
    if start.y > bounds.lo.y + height / 3.0 {
        grid += 3;
    }
    if start.y > bounds.lo.y + 2.0 * height / 3.0 {
        grid += 3;
    }
    let mut angle: Real = 0.0;
    if rotate {
        angle = match grid {
            1 => PI / 4.0,
            2 => PI / 2.0,
            3 => -PI / 4.0,
            5 => 3.0 * PI / 4.0,
            6 => -PI / 2.0,
            7 => -3.0 * PI / 4.0,
            8 => PI,
            _ => 0.0,
        };
    }
    angle += 3.0 * PI / 4.0;
    angle
}

/// The part of `adjustBuildList` after the starting command center is gone. Command
/// center entries are marked as built from the start. If the first entry of the list
/// is a command center, every entry is turned about the last command center entry
/// and moved to `start`; the reference tests the first entry only.
pub fn move_build_list(
    list: &mut [BuildListInfo],
    is_command_center: impl Fn(&str) -> bool,
    start: &Coord3D,
    bounds: &Region3D,
    rotate: bool,
) {
    let mut center = None;
    for info in list.iter_mut() {
        if is_command_center(&info.template_name) {
            center = Some(info.location);
            info.is_initially_built = TRUE;
        }
    }
    let angle = base_angle(start, bounds, rotate);
    let s = gm_sinf(angle);
    let c = gm_cosf(angle);
    let Some(center) = center else {
        return;
    };
    if !list
        .first()
        .is_some_and(|head| is_command_center(&head.template_name))
    {
        return;
    }
    for info in list {
        let mut at = info.location;
        at.x -= center.x;
        at.y -= center.y;
        let x = at.x * c - at.y * s;
        let y = at.y * c + at.x * s;
        at.x = x + start.x;
        at.y = y + start.y;
        info.location = at;
    }
}

/// `adjustBuildList`: removes the player's starting command center, whose place the
/// list's own command center takes.
fn adjust_build_list(cx: &mut BuildContext<'_>, list: &mut [BuildListInfo], player: PlayerIndex) {
    let found = cx.sim.world.objects.world_iter().find(|&slot| {
        cx.sim
            .world
            .objects
            .get(slot)
            .is_some_and(|o| cx.sim.controlling_player(o.team_id) == Some(player))
            && cx.sim.object_is_kind_of(slot, KindOfType::CommandCenter) != FALSE
    });
    let Some((slot, start)) = found.and_then(|slot| {
        let obj = cx.sim.world.objects.get(slot)?;
        Some((slot, *obj.thing.get_position()))
    }) else {
        return;
    };
    cx.sim.record_structure_undone(slot);
    cx.sim.remove_object_from_pathfind_map(slot);
    cx.sim.destroy_object_now(slot);
    let rotate: Bool = cx.sim.rules.ai_data.rotate_skirmish_bases;
    let factory = &cx.sim.rules.thing_factory;
    move_build_list(
        list,
        |name| {
            factory
                .find_template_final(name)
                .is_some_and(|t| t.is_kind_of(KindOfType::CommandCenter) != FALSE)
        },
        &start,
        &cx.pathfind_extent,
        rotate != FALSE,
    );
}

/// `AISkirmishPlayer::newMap`.
pub fn new_map(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    let Some(side) = cx.players.player(player).map(|p| p.side.clone()) else {
        return;
    };
    let faction = cx
        .sim
        .rules
        .ai_data
        .side_build_lists
        .iter()
        .find(|l| l.side == side)
        .map(|l| l.build_list.clone());
    if let Some(mut list) = faction {
        adjust_build_list(cx, &mut list, player);
        if let Some(p) = cx.players.player_mut(player) {
            p.build_list = list;
        }
        compute_center_and_radius_of_base(cx, player);
    }
    build_initial_structures(cx, player, 0);
}
