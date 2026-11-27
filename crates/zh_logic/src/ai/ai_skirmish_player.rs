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
use zh_common::types::{Bool, Int, Real, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

use crate::ai::ai_player::{build_initial_structures, compute_center_and_radius_of_base};
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
