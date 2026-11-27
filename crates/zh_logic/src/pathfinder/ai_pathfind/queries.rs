//! The movement-surface half of `Pathfinder` (`PKT-411`): whether an object can
//! stand on or pass a cell, walking the cells along a line, the quick zone test
//! and the straight aircraft path.
//!
//! Golden test: `crates/zh_logic/tests/golden_s3_path.rs`.

use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::{Coord2D, Coord3D, ICoord2D};
use zh_common::types::{Bool, Int, ObjectId, Real, UnsignedByte};

use super::{clip_to_cell, Pathfinder};
use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::path::Path;
use crate::pathfinder::pathfind_cell::{CellFlags, CellType, PathfindCell};
use crate::pathfinder::pathfind_cell_info::{CellIdx, PathfindCellInfo};
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PATHFIND_CELL_SIZE_F};
use crate::rules::combat_types::{LocomotorSurfaceType, LocomotorSurfaceTypeMask};

/// What the checks read from the object that wants to move.
#[derive(Debug, Clone, PartialEq)]
pub struct PathAgent {
    pub id: ObjectId,
    /// `getAIUpdateInterface()` is not null.
    pub has_ai: bool,
    /// `AIUpdateInterface::getIgnoredObstacleID`, invalid without an AI.
    pub ignored_obstacle: ObjectId,
    /// `isAircraftThatAdjustsDestination`, false without an AI.
    pub aircraft_adjusts: bool,
    pub crusher_level: UnsignedByte,
    /// `getGeometryInfo().getBoundingCircleRadius()`.
    pub radius: Real,
    pub layer: PathfindLayerEnum,
    /// `KINDOF_INFANTRY`.
    pub infantry: bool,
}

/// What the checks read from a unit standing in a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathUnit {
    pub has_ai: bool,
    pub crushable_level: UnsignedByte,
    /// `KINDOF_INFANTRY`.
    pub infantry: bool,
}

/// The units the checks look up.
pub trait PathUnits {
    /// `findObjectByID`.
    fn unit(&self, id: ObjectId) -> Option<PathUnit>;
    /// `agent->getRelationship(other) == ALLIES`.
    fn is_ally(&self, agent: ObjectId, other: ObjectId) -> bool;
    /// `agent->canCrushOrSquish(other, TEST_CRUSH_OR_SQUISH)`.
    fn can_crush_or_squish(&self, agent: ObjectId, other: ObjectId) -> bool;
}

/// A world without units.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoPathUnits;

impl PathUnits for NoPathUnits {
    fn unit(&self, _id: ObjectId) -> Option<PathUnit> {
        None
    }
    fn is_ally(&self, _agent: ObjectId, _other: ObjectId) -> bool {
        false
    }
    fn can_crush_or_squish(&self, _agent: ObjectId, _other: ObjectId) -> bool {
        false
    }
}

/// `TCheckMovementInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckMovementInfo {
    pub cell: ICoord2D,
    pub layer: PathfindLayerEnum,
    pub radius: Int,
    pub center_in_cell: bool,
    pub consider_transient: bool,
    pub ally_fixed_count: Int,
    pub enemy_fixed: bool,
    pub ally_moving: bool,
    pub ally_goal: bool,
}

impl Default for CheckMovementInfo {
    fn default() -> Self {
        Self {
            cell: ICoord2D::default(),
            layer: PathfindLayerEnum::Ground,
            radius: 0,
            center_in_cell: false,
            consider_transient: false,
            ally_fixed_count: 0,
            enemy_fixed: false,
            ally_moving: false,
            ally_goal: false,
        }
    }
}

/// One end of a line of sight question.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewParty {
    pub id: ObjectId,
    pub position: Coord3D,
    pub layer: PathfindLayerEnum,
    /// `getContainedBy()->getID()`, invalid outside a container.
    pub container: ObjectId,
    /// The first `SlavedUpdateInterface::getSlaverID`, invalid without one.
    pub slaver: ObjectId,
    /// `KINDOF_ATTACK_NEEDS_LINE_OF_SIGHT`.
    pub needs_line_of_sight: bool,
    pub significantly_above_terrain: bool,
}

/// What the attacker's current weapon says about the terrain in the way. The
/// reference ignores the weapon of an immobile attacker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainSight {
    NoWeapon,
    /// `isContactWeapon`: nothing blocks it, except under `retail-crc`, which
    /// asks for the terrain first.
    Contact {
        terrain_blocked: bool,
    },
    /// `!isClearGoalFiringLineOfSightTerrain`.
    Weapon {
        terrain_blocked: bool,
    },
}

/// A building aircraft go around (`KINDOF_AIRCRAFT_PATH_AROUND`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TallBuilding {
    pub id: ObjectId,
    pub position: Coord3D,
    /// Bounding circle radius.
    pub radius: Real,
}

pub trait TallBuildings {
    /// The object `id`, if it is a building aircraft go around.
    fn tall_building(&self, id: ObjectId) -> Option<TallBuilding>;
    /// The closest one within `radius` of `pos`, from bounding spheres in 2D.
    fn closest_tall_building(&self, pos: &Coord3D, radius: Real) -> Option<TallBuilding>;
}

/// `validLocomotorSurfacesForCellType`.
pub fn valid_locomotor_surfaces_for_cell_type(cell_type: CellType) -> LocomotorSurfaceTypeMask {
    let air = LocomotorSurfaceType::Air.as_i32();
    match cell_type {
        CellType::Clear => LocomotorSurfaceType::Ground.as_i32() | air,
        CellType::Water => LocomotorSurfaceType::Water.as_i32() | air,
        CellType::Cliff => LocomotorSurfaceType::Cliff.as_i32() | air,
        CellType::Rubble => LocomotorSurfaceType::Rubble.as_i32() | air,
        CellType::Obstacle | CellType::BridgeImpassable | CellType::Impassable => air,
    }
}

fn is_impassable(cell_type: CellType) -> bool {
    matches!(
        cell_type,
        CellType::Impassable | CellType::Obstacle | CellType::BridgeImpassable
    )
}

/// The cell holding `pos`, without clipping.
fn cell_of(pos: &Coord3D) -> ICoord2D {
    ICoord2D::new(
        real_to_int_floor(pos.x / PATHFIND_CELL_SIZE_F),
        real_to_int_floor(pos.y / PATHFIND_CELL_SIZE_F),
    )
}

/// `getRadiusAndCenter`: the footprint radius in cells and whether the object
/// stands in the middle of a cell.
pub fn radius_and_center(agent: Option<&PathAgent>) -> (Int, bool) {
    const MAX_RADIUS: Int = 2;
    let Some(agent) = agent else {
        return (0, true);
    };
    let mut diameter = 2.0 * agent.radius;
    if diameter > PATHFIND_CELL_SIZE_F && diameter < 2.0 * PATHFIND_CELL_SIZE_F {
        diameter = 2.0 * PATHFIND_CELL_SIZE_F;
    }
    let mut radius = real_to_int_floor(diameter / PATHFIND_CELL_SIZE_F + 0.3);
    let mut center = false;
    if radius == 0 {
        radius += 1;
    }
    if radius & 1 != 0 {
        center = true;
    }
    radius /= 2;
    if radius > MAX_RADIUS {
        radius = MAX_RADIUS;
        center = true;
    }
    (radius, center)
}

/// `computeNormalRadialOffset`: a point `radius` off the building, on the side
/// away from the line `from`-`to`.
fn normal_radial_offset(from: &Coord3D, to: &Coord3D, building: &Coord3D, radius: Real) -> Coord3D {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let obj_dx = building.x - from.x;
    let obj_dy = building.y - from.y;
    let cross = dx * obj_dy - dy * obj_dx;
    let mut normal = if cross > 0.0 {
        Coord3D::new(dy, -dx, 0.0)
    } else {
        Coord3D::new(-dy, dx, 0.0)
    };
    normal.normalize();
    let mut insert = *building;
    insert.x += normal.x * radius;
    insert.y += normal.y * radius;
    insert
}

/// Moves `end` out to `radius` from the building if it lies inside.
fn push_out(end: &mut Coord3D, building: &Coord3D, radius: Real) -> bool {
    let mut delta = Coord2D::new(end.x - building.x, end.y - building.y);
    if f64::from(delta.length()) > f64::from(radius) * 0.98 {
        return false;
    }
    if f64::from(delta.length()) < 0.1 {
        delta.x = 1.0;
    }
    delta.normalize();
    delta.x *= radius;
    delta.y *= radius;
    end.x = building.x + delta.x;
    end.y = building.y + delta.y;
    true
}

impl Pathfinder {
    fn cell_ref(&self, idx: CellIdx) -> &PathfindCell {
        self.map.grid().cell(idx)
    }

    fn cell_info(&self, idx: CellIdx) -> Option<&PathfindCellInfo> {
        let info = self.cell_ref(idx).info_idx?;
        self.map.pool().get(info)
    }

    fn pos_unit(&self, idx: CellIdx) -> ObjectId {
        self.cell_info(idx)
            .map_or(ObjectId::INVALID, |i| i.pos_unit_id)
    }

    fn obstacle_present(&self, idx: CellIdx, id: ObjectId) -> bool {
        self.cell_ref(idx)
            .is_obstacle_present(id, self.map.pool(), self.map.mode())
    }

    /// `getClippedCell`.
    fn clipped_cell(&self, layer: PathfindLayerEnum, pos: &Coord3D) -> Option<CellIdx> {
        let (cell, _) = clip_to_cell(&self.extent, pos);
        self.get_cell(layer, cell.x, cell.y)
    }

    /// `worldToCell`.
    pub fn world_to_cell(&self, pos: &Coord3D) -> (ICoord2D, bool) {
        clip_to_cell(&self.extent, pos)
    }

    /// The surfaces `chooseBestLocomotorForPosition` looks for: the caller
    /// takes the first locomotor of its set that has one of them.
    pub fn locomotor_surfaces_at(
        &self,
        layer: PathfindLayerEnum,
        pos: &Coord3D,
    ) -> LocomotorSurfaceTypeMask {
        let cell = cell_of(pos);
        let cell_type = self.cell_type(layer, cell.x, cell.y);
        valid_locomotor_surfaces_for_cell_type(cell_type.unwrap_or(CellType::Clear))
    }

    /// `validMovementTerrain` for a locomotor with these legal surfaces.
    pub fn valid_movement_terrain(
        &self,
        layer: PathfindLayerEnum,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
    ) -> bool {
        let at = cell_of(pos);
        let Some(idx) = self.get_cell(layer, at.x, at.y) else {
            return false;
        };
        let cell = self.cell_ref(idx);
        if matches!(cell.cell_type, CellType::Obstacle | CellType::Impassable) {
            return true;
        }
        // The reference compares the layer to a cell type: `CELL_CLEAR` is 0,
        // which is `LAYER_INVALID`.
        if cell.layer == PathfindLayerEnum::Invalid {
            return true;
        }
        surfaces & valid_locomotor_surfaces_for_cell_type(cell.cell_type) != 0
    }

    /// `validMovementPosition` of a cell.
    pub fn valid_movement_cell(
        &self,
        crusher: bool,
        surfaces: LocomotorSurfaceTypeMask,
        cell: Option<CellIdx>,
    ) -> bool {
        let Some(idx) = cell else { return false };
        if self.obstacle_present(idx, self.ignore_obstacle_id) {
            return true;
        }
        let cell = self.cell_ref(idx);
        if crusher && cell.is_obstacle_fence(self.map.pool(), self.map.mode()) {
            return true;
        }
        valid_locomotor_surfaces_for_cell_type(cell.cell_type) & surfaces != 0
    }

    /// `validMovementPosition` at a world position. This overload truncates
    /// toward zero, where the others floor.
    pub fn valid_movement_position(
        &self,
        crusher: bool,
        layer: PathfindLayerEnum,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
    ) -> bool {
        let x = (pos.x / PATHFIND_CELL_SIZE_F) as Int;
        let y = (pos.y / PATHFIND_CELL_SIZE_F) as Int;
        self.valid_movement_cell(crusher, surfaces, self.get_cell(layer, x, y))
    }

    /// `checkDestination`.
    #[allow(clippy::too_many_arguments)]
    pub fn check_destination(
        &self,
        units: &dyn PathUnits,
        agent: Option<&PathAgent>,
        cell_x: Int,
        cell_y: Int,
        layer: PathfindLayerEnum,
        radius: Int,
        center_in_cell: bool,
    ) -> bool {
        let above = radius + Int::from(center_in_cell);
        let ai = agent.filter(|a| a.has_ai);
        let ignored = ai.map_or(ObjectId::INVALID, |a| a.ignored_obstacle);
        let aircraft = ai.is_some_and(|a| a.aircraft_adjusts);
        let own_id = ai.map_or(ObjectId::INVALID, |a| a.id);
        for i in cell_x - radius..cell_x + above {
            for j in cell_y - radius..cell_y + above {
                let Some(idx) = self.get_cell(layer, i, j) else {
                    return false;
                };
                let cell = self.cell_ref(idx);
                if aircraft {
                    if !cell.aircraft_goal {
                        continue;
                    }
                    let goal = self
                        .cell_info(idx)
                        .map_or(ObjectId::INVALID, |i| i.goal_aircraft_id);
                    if goal == own_id {
                        continue;
                    }
                    return false;
                }
                if cell.cell_type == CellType::Obstacle {
                    if self.obstacle_present(idx, ignored) {
                        continue;
                    }
                    return false;
                }
                if is_impassable(cell.cell_type) {
                    return false;
                }
                if cell.flags == CellFlags::NoUnits {
                    continue;
                }
                let goal = self
                    .cell_info(idx)
                    .map_or(ObjectId::INVALID, |i| i.goal_unit_id);
                if goal == own_id || goal == ignored || goal == ObjectId::INVALID {
                    continue;
                }
                let Some(agent) = agent else { return false };
                if units.unit(goal).is_none() {
                    continue;
                }
                if units.is_ally(agent.id, goal) {
                    return false;
                }
                if cell.flags == CellFlags::UnitPresentFixed
                    && !units.can_crush_or_squish(agent.id, goal)
                {
                    return false;
                }
            }
        }
        true
    }

    /// `checkForMovement`: fills in `info` and answers false when the footprint
    /// leaves the map or an ally without an AI stands in it.
    pub fn check_for_movement(
        &self,
        units: &dyn PathUnits,
        agent: Option<&PathAgent>,
        info: &mut CheckMovementInfo,
    ) -> bool {
        info.ally_fixed_count = 0;
        info.ally_moving = false;
        info.ally_goal = false;
        info.enemy_fixed = false;
        const MAX_ALLY: usize = 5;
        let mut allies: Vec<ObjectId> = Vec::with_capacity(MAX_ALLY);
        let Some(agent) = agent else { return true };
        let ignored = if agent.has_ai {
            agent.ignored_obstacle
        } else {
            ObjectId::INVALID
        };
        let above = info.radius + Int::from(info.center_in_cell);
        for i in info.cell.x - info.radius..info.cell.x + above {
            for j in info.cell.y - info.radius..info.cell.y + above {
                let Some(idx) = self.get_cell(info.layer, i, j) else {
                    return false;
                };
                let flags = self.cell_ref(idx).flags;
                if matches!(flags, CellFlags::UnitGoal | CellFlags::UnitGoalOtherMoving) {
                    info.ally_goal = true;
                } else if flags == CellFlags::NoUnits {
                    continue;
                }
                let there = self.pos_unit(idx);
                if there == agent.id || there == ignored {
                    continue;
                }
                let mut checked = None;
                if matches!(
                    flags,
                    CellFlags::UnitPresentMoving | CellFlags::UnitGoalOtherMoving
                ) {
                    let unit = units.unit(there);
                    if unit.is_some() && units.is_ally(agent.id, there) {
                        info.ally_moving = true;
                    }
                    if info.consider_transient {
                        checked = unit;
                    }
                }
                if flags == CellFlags::UnitPresentFixed {
                    checked = units.unit(there);
                }
                let Some(unit) = checked else { continue };
                if agent.has_ai && agent.ignored_obstacle == there {
                    continue;
                }
                // `INFANTRY_MOVES_THROUGH_INFANTRY`.
                if agent.infantry && unit.infantry {
                    continue;
                }
                if units.is_ally(agent.id, there) {
                    if !unit.has_ai {
                        return false;
                    }
                    if !allies.contains(&there) {
                        info.ally_fixed_count += 1;
                        if allies.len() < MAX_ALLY {
                            allies.push(there);
                        }
                    }
                } else if !units.can_crush_or_squish(agent.id, there) {
                    info.enemy_fixed = true;
                }
            }
        }
        true
    }

    /// `clearCellForDiameter`: the width, up to `diameter`, that is clear
    /// around the cell.
    pub fn clear_cell_for_diameter(
        &self,
        units: &dyn PathUnits,
        crusher: bool,
        cell_x: Int,
        cell_y: Int,
        layer: PathfindLayerEnum,
        diameter: Int,
    ) -> Int {
        let radius = diameter / 2;
        let above = radius + Int::from(radius == 0);
        let mut clear = true;
        let cut_corners = radius > 1;
        for i in cell_x - radius..cell_x + above {
            let x_edge = i == cell_x - radius || i == cell_x + above - 1;
            for j in cell_y - radius..cell_y + above {
                let y_edge = j == cell_y - radius || j == cell_y + above - 1;
                if x_edge && y_edge && cut_corners {
                    continue;
                }
                let Some(idx) = self.get_cell(layer, i, j) else {
                    return 0;
                };
                let cell = self.cell_ref(idx);
                if cell.cell_type != CellType::Clear
                    && !(cell.cell_type == CellType::Obstacle
                        && crusher
                        && cell.is_obstacle_fence(self.map.pool(), self.map.mode()))
                {
                    clear = false;
                }
                if cell.flags == CellFlags::UnitPresentFixed && diameter >= 2 {
                    if let Some(unit) = units.unit(self.pos_unit(idx)) {
                        let limit = UnsignedByte::from(crusher);
                        if unit.crushable_level > limit {
                            clear = false;
                        }
                    }
                }
                // Only the column is abandoned: later columns are still looked
                // up, and one off the map answers 0.
                if !clear {
                    break;
                }
            }
        }
        if clear {
            if radius == 0 {
                return 1;
            }
            return 2 * radius;
        }
        if diameter < 2 {
            return 0;
        }
        self.clear_cell_for_diameter(units, crusher, cell_x, cell_y, layer, diameter - 2)
    }

    /// `iterateCellsAlongLine` between two cells. `visit` gets the cell stepped
    /// from, the cell and its coordinates; a non-zero answer ends the walk. A
    /// step off the map ends it with 0.
    pub fn iterate_cells_along_line(
        &self,
        start: ICoord2D,
        end: ICoord2D,
        layer: PathfindLayerEnum,
        visit: &mut dyn FnMut(Option<CellIdx>, CellIdx, Int, Int) -> Int,
    ) -> Int {
        let delta_x = end.x.wrapping_sub(start.x).wrapping_abs();
        let delta_y = end.y.wrapping_sub(start.y).wrapping_abs();
        let (mut x, mut y) = (start.x, start.y);
        let x_step = if end.x >= start.x { 1 } else { -1 };
        let y_step = if end.y >= start.y { 1 } else { -1 };
        let (mut xinc1, mut xinc2) = (x_step, x_step);
        let (mut yinc1, mut yinc2) = (y_step, y_step);
        let (den, mut num, add) = if delta_x >= delta_y {
            xinc1 = 0;
            yinc2 = 0;
            (delta_x, delta_x / 2, delta_y)
        } else {
            xinc2 = 0;
            yinc1 = 0;
            (delta_y, delta_y / 2, delta_x)
        };
        let mut from = None;
        for _ in 0..=den {
            let Some(mut to) = self.get_cell(layer, x, y) else {
                return 0;
            };
            let ret = visit(from, to, x, y);
            if ret != 0 {
                return ret;
            }
            num += add;
            if num >= den {
                num -= den;
                x += xinc1;
                y += yinc1;
                let previous = to;
                let Some(next) = self.get_cell(layer, x, y) else {
                    return 0;
                };
                to = next;
                let ret = visit(Some(previous), to, x, y);
                if ret != 0 {
                    return ret;
                }
            }
            x += xinc2;
            y += yinc2;
            from = Some(to);
        }
        0
    }

    fn walk(
        &self,
        start: &Coord3D,
        end: &Coord3D,
        layer: PathfindLayerEnum,
        visit: &mut dyn FnMut(Option<CellIdx>, CellIdx, Int, Int) -> Int,
    ) -> Int {
        let (start, _) = self.world_to_cell(start);
        let (end, _) = self.world_to_cell(end);
        self.iterate_cells_along_line(start, end, layer, visit)
    }

    /// `isLinePassable`.
    #[allow(clippy::too_many_arguments)]
    pub fn is_line_passable(
        &self,
        units: &dyn PathUnits,
        agent: Option<&PathAgent>,
        surfaces: LocomotorSurfaceTypeMask,
        layer: PathfindLayerEnum,
        start: &Coord3D,
        end: &Coord3D,
        blocked: bool,
        allow_pinched: bool,
    ) -> bool {
        let (radius, center_in_cell) = radius_and_center(agent);
        let crusher = agent.is_some_and(|a| a.crusher_level > 0);
        let mut visit = |from: Option<CellIdx>, to: CellIdx, x: Int, y: Int| -> Int {
            let cell = self.cell_ref(to);
            let mut info = CheckMovementInfo {
                cell: ICoord2D::new(x, y),
                layer: cell.layer,
                radius,
                center_in_cell,
                consider_transient: blocked,
                ..CheckMovementInfo::default()
            };
            if !self.check_for_movement(units, agent, &mut info) {
                return 1;
            }
            if info.ally_fixed_count != 0 || info.enemy_fixed {
                return 1;
            }
            if !allow_pinched && cell.pinched {
                return 1;
            }
            let on_deck = from.is_some_and(|from| {
                cell.layer != PathfindLayerEnum::Ground && self.cell_ref(from).layer == cell.layer
            });
            if on_deck && cell.cell_type == CellType::Clear {
                return 0;
            }
            Int::from(!self.valid_movement_cell(crusher, surfaces, Some(to)))
        };
        self.walk(start, end, layer, &mut visit) == 0
    }

    /// `isGroundPathPassable`.
    pub fn is_ground_path_passable(
        &self,
        units: &dyn PathUnits,
        crusher: bool,
        start: &Coord3D,
        start_layer: PathfindLayerEnum,
        end: &Coord3D,
        diameter: Int,
    ) -> bool {
        let mut visit = |from: Option<CellIdx>, to: CellIdx, x: Int, y: Int| -> Int {
            let layer = self.cell_ref(to).layer;
            if self.clear_cell_for_diameter(units, crusher, x, y, layer, diameter) == diameter {
                return 0;
            }
            let on_deck = from.is_some_and(|from| {
                layer != PathfindLayerEnum::Ground && self.cell_ref(from).layer == layer
            });
            Int::from(!on_deck)
        };
        self.walk(start, end, start_layer, &mut visit) == 0
    }

    /// `isAttackViewBlockedByObstacle`. `uses_line_of_sight` is
    /// `AIData::m_attackUsesLineOfSight`.
    #[allow(clippy::too_many_arguments)]
    pub fn is_attack_view_blocked(
        &self,
        uses_line_of_sight: bool,
        attacker: &ViewParty,
        attacker_pos: &Coord3D,
        victim: Option<&ViewParty>,
        victim_pos: &Coord3D,
        sight: TerrainSight,
    ) -> bool {
        if !uses_line_of_sight || !attacker.needs_line_of_sight {
            return false;
        }
        match sight {
            TerrainSight::NoWeapon => {}
            #[cfg(not(feature = "retail-crc"))]
            TerrainSight::Contact { .. } => return false,
            #[cfg(feature = "retail-crc")]
            TerrainSight::Contact {
                terrain_blocked: true,
            } => return true,
            #[cfg(feature = "retail-crc")]
            TerrainSight::Contact { .. } => {}
            TerrainSight::Weapon {
                terrain_blocked: true,
            } => return true,
            TerrainSight::Weapon { .. } => {}
        }
        let mut layer = victim.map_or(PathfindLayerEnum::Ground, |v| v.layer);
        let victim_cell = {
            let (cell, overflow) = clip_to_cell(&self.extent, victim_pos);
            if overflow {
                None
            } else {
                self.get_cell(layer, cell.x, cell.y)
            }
        };
        let mut skip = 0;
        if attacker.layer != PathfindLayerEnum::Ground {
            skip = 3;
            if layer == PathfindLayerEnum::Ground {
                layer = attacker.layer;
            }
        }
        let mut visit = |_from: Option<CellIdx>, to: CellIdx, _x: Int, _y: Int| -> Int {
            if skip > 0 {
                skip -= 1;
                return 0;
            }
            if self.cell_ref(to).cell_type != CellType::Obstacle {
                return 0;
            }
            let present = |id| self.obstacle_present(to, id);
            if present(attacker.id) {
                return 0;
            }
            if victim.is_some_and(|v| present(v.id) || present(v.slaver)) {
                return 0;
            }
            if present(attacker.container) || present(attacker.slaver) {
                return 0;
            }
            let (pool, mode) = (self.map.pool(), self.map.mode());
            if self.cell_ref(to).is_obstacle_transparent(pool, mode) {
                return 0;
            }
            if victim_cell.is_some_and(|v| present(self.cell_ref(v).obstacle_id(pool, mode))) {
                return 0;
            }
            1
        };
        self.walk(attacker_pos, victim_pos, layer, &mut visit) != 0
    }

    /// `isViewBlockedByObstacle`.
    pub fn is_view_blocked(
        &self,
        uses_line_of_sight: bool,
        from: &ViewParty,
        to: &ViewParty,
        sight: TerrainSight,
    ) -> bool {
        if to.significantly_above_terrain {
            return false;
        }
        self.is_attack_view_blocked(
            uses_line_of_sight,
            from,
            &from.position,
            Some(to),
            &to.position,
            sight,
        )
    }

    /// The zones of the two ends seen through the terrain zones.
    fn terrain_pair(&self, surfaces: LocomotorSurfaceTypeMask, zone: u16) -> u16 {
        let zones = &self.zones;
        let zone = zones.get_effective_terrain_zone(zone);
        let zone = zones.get_effective_zone(surfaces, 0 as Bool, zone);
        zones.get_effective_terrain_zone(zone)
    }

    /// `clientSafeQuickDoesPathExist`.
    pub fn quick_path_exists(
        &self,
        terrain: &dyn ThingTerrainAccess,
        surfaces: LocomotorSurfaceTypeMask,
        from: &Coord3D,
        to: &Coord3D,
    ) -> bool {
        let to_layer = terrain.get_layer_for_destination(to);
        if !self.valid_movement_position(false, to_layer, surfaces, to) {
            return false;
        }
        let from_layer = terrain.get_layer_for_destination(from);
        let (Some(start), Some(goal)) = (
            self.clipped_cell(from_layer, from),
            self.clipped_cell(to_layer, to),
        ) else {
            return false;
        };
        let (start, goal) = (self.cell_ref(start), self.cell_ref(goal));
        if goal.cell_type == CellType::Cliff {
            return false;
        }
        let mut terrain_zone = false;
        let mut first = self.zones.get_effective_zone(surfaces, 0, start.zone);
        if start.cell_type == CellType::Obstacle {
            terrain_zone = true;
            if first == 0 {
                return true;
            }
        }
        let mut second = self.zones.get_effective_zone(surfaces, 0, goal.zone);
        if goal.cell_type == CellType::Obstacle {
            terrain_zone = true;
        }
        if terrain_zone {
            first = self.terrain_pair(surfaces, start.zone);
            second = self.terrain_pair(surfaces, goal.zone);
        }
        first == second
    }

    /// `clientSafeQuickDoesPathExistForUI`.
    pub fn quick_path_exists_for_ui(
        &self,
        terrain: &dyn ThingTerrainAccess,
        surfaces: LocomotorSurfaceTypeMask,
        from: &Coord3D,
        to: &Coord3D,
    ) -> bool {
        let to_layer = terrain.get_layer_for_destination(to);
        let from_layer = terrain.get_layer_for_destination(from);
        let (Some(start), Some(goal)) = (
            self.clipped_cell(from_layer, from),
            self.clipped_cell(to_layer, to),
        ) else {
            return false;
        };
        let (start, goal) = (self.cell_ref(start), self.cell_ref(goal));
        if goal.cell_type == CellType::Cliff {
            return false;
        }
        let first = self.zones.get_effective_zone(surfaces, 0, start.zone);
        let second = self.zones.get_effective_zone(surfaces, 0, goal.zone);
        if first == 0 || second == 0 {
            return true;
        }
        let first = self.terrain_pair(surfaces, start.zone);
        let second = self.terrain_pair(surfaces, goal.zone);
        first == 0 || first == second
    }

    /// `segmentIntersectsTallBuilding`: three points leading around a building
    /// in the way. `to` is moved out of a building it lies in.
    pub fn segment_intersects_tall_building(
        &self,
        buildings: &dyn TallBuildings,
        from: &Coord3D,
        to: &mut Coord3D,
        ignore: ObjectId,
    ) -> Option<[Coord3D; 3]> {
        let mut from = *from;
        // The reference keeps a building found on the first pass.
        let mut found = None;
        for _ in 0..2 {
            let mut visit = |_from: Option<CellIdx>, cell: CellIdx, _x: Int, _y: Int| -> Int {
                let c = self.cell_ref(cell);
                if c.cell_type != CellType::Obstacle {
                    return 0;
                }
                let id = c.obstacle_id(self.map.pool(), self.map.mode());
                match buildings.tall_building(id) {
                    Some(building) if building.id != ignore => {
                        found = Some(building);
                        1
                    }
                    _ => 0,
                }
            };
            let ret = self.walk(&from, to, PathfindLayerEnum::Ground, &mut visit);
            let Some(building) = found.filter(|_| ret != 0) else {
                continue;
            };
            let center = building.position;
            let radius = building.radius + 2.0 * PATHFIND_CELL_SIZE_F;
            if push_out(to, &center, radius) {
                continue;
            }
            push_out(&mut from, &center, radius);
            let second = normal_radial_offset(&from, to, &center, radius);
            let first = normal_radial_offset(&from, &second, &center, radius);
            let third = normal_radial_offset(&second, to, &center, radius);
            return Some([first, second, third]);
        }
        None
    }

    /// `circleClipsTallBuilding`: where to aim instead of `to` when a circle of
    /// `circle` around it would hit a building.
    pub fn circle_clips_tall_building(
        buildings: &dyn TallBuildings,
        from: &Coord3D,
        to: &Coord3D,
        circle: Real,
    ) -> Option<Coord3D> {
        let building = buildings.closest_tall_building(to, circle)?;
        let radius = building.radius + 2.0 * PATHFIND_CELL_SIZE_F;
        let mut adjusted = normal_radial_offset(from, to, &building.position, circle + radius);
        if let Some(other) = buildings.closest_tall_building(&adjusted, circle) {
            if other.id != building.id {
                let radius = other.radius + 2.0 * PATHFIND_CELL_SIZE_F;
                let moved = adjusted;
                adjusted = normal_radial_offset(from, &moved, &other.position, circle + radius);
            }
        }
        Some(adjusted)
    }

    /// `getAircraftPath`. `winged` is a current locomotor with `LOCO_WINGS`;
    /// `avoid` is `getBuildingToNotPathAround`.
    pub fn aircraft_path(
        &self,
        buildings: &dyn TallBuildings,
        position: &Coord3D,
        to: &Coord3D,
        winged: bool,
        avoid: ObjectId,
    ) -> Path {
        let mut path = Path::new();
        let mut destination = *to;
        if winged {
            if let Some(adjusted) = Self::circle_clips_tall_building(buildings, position, to, 100.0)
            {
                destination = adjusted;
            }
        }
        path.prepend_node(&destination, PathfindLayerEnum::Ground);
        let start = Coord3D::new(position.x, position.y, to.z);
        path.prepend_node(&start, PathfindLayerEnum::Ground);
        let mut limit = 20;
        let mut cursor = path.first_node();
        while let Some(mut current) = cursor {
            let Some(next) = path.node(current).next() else {
                break;
            };
            let from = *path.node(current).position();
            let mut end = *path.node(next).position();
            let around = self.segment_intersects_tall_building(buildings, &from, &mut end, avoid);
            path.set_position(next, &end);
            if let Some([first, second, third]) = around {
                path.insert_after(current, &third, PathfindLayerEnum::Ground);
                let middle = path.insert_after(current, &second, PathfindLayerEnum::Ground);
                path.insert_after(current, &first, PathfindLayerEnum::Ground);
                current = middle;
            }
            cursor = path.node(current).next();
            limit -= 1;
            if limit < 0 {
                break;
            }
        }
        let mut cursor = path.first_node();
        while let Some(current) = cursor {
            let next = path.node(current).next();
            if next.is_none() {
                break;
            }
            path.set_next_optimized(current, next);
            cursor = next;
        }
        path.mark_optimized();
        path
    }
}
