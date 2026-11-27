//! `PKT-286`: `ParkingPlaceBehaviorModuleData` and `ParkingPlaceBehavior`
//! (`ParkingPlaceBehavior.h`, `ParkingPlaceBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-286_ParkingPlaceBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt286_parking_place.rs`.
//!
//! `JetAIUpdate` is unported: its two answers come from [`AiUpdateFacet::jet_ai`] and the
//! move orders go to the AI seams. `ProductionUpdate` holds at most `DOOR_COUNT_MAX` doors
//! (the C++ indexes past the array beyond that), so a space numbered higher reports
//! `DoorNoneAvailable`.

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ObjectSlot, ThingTemplateId};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{
    BehaviorModule, CommandSourceType, DieFacet, ExitDoorType, ExitFacet, ModuleData,
    ParkingPlaceFacet, ParkingPlaceInfo, RunwayReservationType, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::scheduler::UPDATE_SLEEP_FOREVER;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, Matrix3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, TeamId, UnsignedByte, UnsignedInt, UnsignedShort,
    UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// `LOGICFRAMES_PER_SECOND / 5`.
const HEAL_RATE_FRAMES: UnsignedInt = 30 / 5;
/// `SECONDS_PER_LOGICFRAME_REAL`.
const SECONDS_PER_LOGICFRAME_REAL: Real = 1.0 / 30.0;
/// `ParkingPlaceBehaviorInterface::InvalidRunway`.
const INVALID_RUNWAY: Int = -1;

/// INI template data matching `ParkingPlaceBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParkingPlaceBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub heal_amount: Real,
    pub num_rows: Int,
    pub num_cols: Int,
    pub approach_height: Real,
    pub landing_deck_height_offset: Real,
    pub has_runways: Bool,
    pub park_in_hangars: Bool,
}

impl ParkingPlaceBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            heal_amount: 0.0,
            num_rows: 0,
            num_cols: 0,
            approach_height: 0.0,
            landing_deck_height_offset: 0.0,
            has_runways: FALSE,
            park_in_hangars: FALSE,
        }
    }
}

impl ModuleData for ParkingPlaceBehaviorModuleData {
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
        FacetMask(0x003 | FacetMask::PARKING_PLACE.0 | FacetMask::EXIT_INTERFACE.0)
    }
}

/// `ParkingPlaceInfo` of one space.
#[derive(Debug, Clone, Copy, Default)]
pub struct ParkingSpace {
    pub hangar_start: Coord3D,
    pub hangar_start_orient: Real,
    pub location: Coord3D,
    pub prep: Coord3D,
    pub orientation: Real,
    pub runway: Int,
    pub door: Int,
    pub object_in_space: ObjectId,
    pub reserved_for_exit: Bool,
    pub postponed_runway_reservation_for_takeoff: Bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Runway {
    pub start: Coord3D,
    pub end: Coord3D,
    pub in_use_by: ObjectId,
    pub next_in_line_for_takeoff: ObjectId,
    pub was_in_line: Bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Healing {
    pub getting_healed_id: ObjectId,
    pub heal_start_frame: UnsignedInt,
}

/// Runtime instance of `ParkingPlaceBehavior`.
pub struct ParkingPlaceBehavior {
    pub data: ParkingPlaceBehaviorModuleData,
    pub schedule: UpdateScheduleTracker,
    pub spaces: Vec<ParkingSpace>,
    pub runways: Vec<Runway>,
    pub healing: Vec<Healing>,
    pub next_heal_frame: UnsignedInt,
    pub got_info: Bool,
    pub heli_rally_point: Coord3D,
    pub heli_rally_point_exists: Bool,
    /// The owner, for `buildInfo` during a load.
    owner: Option<ObjectSlot>,
}

fn door_of(door: Int) -> ExitDoorType {
    ExitDoorType::from_i32(door).unwrap_or(ExitDoorType::DoorNoneAvailable)
}

/// `Object::getSingleLogicalBonePosition`: whether the bone exists, its world position and
/// world transform; without it the object's own.
fn single_logical_bone(
    sim: &dyn SimView,
    owner: ObjectSlot,
    name: &str,
) -> (Bool, Coord3D, Matrix3D) {
    let Some(obj) = sim.object(owner) else {
        return (FALSE, Coord3D::default(), Matrix3D::identity());
    };
    if sim.object_has_drawable(owner) != FALSE {
        if let Some(bone) = sim.pristine_bone_transforms(owner, name).first() {
            let mut position = Coord3D::default();
            let mut transform = Matrix3D::identity();
            let local = bone.get_translation();
            obj.thing.convert_bone_pos_to_world_pos(
                Some(&local),
                Some(bone),
                Some(&mut position),
                Some(&mut transform),
            );
            return (TRUE, position, transform);
        }
    }
    (
        FALSE,
        *obj.thing.get_position(),
        *obj.thing.get_transform_matrix(),
    )
}

/// `findObjectByID` that also drops the effectively dead.
fn living(sim: &dyn SimView, id: ObjectId) -> Option<ObjectSlot> {
    let slot = sim.find_object(id)?;
    (sim.object(slot)?.is_effectively_dead() == FALSE).then_some(slot)
}

fn takeoff_or_landing(sim: &dyn SimView, slot: ObjectSlot) -> bool {
    sim.object_jet_ai(slot)
        .is_some_and(|j| j.takeoff_or_landing_in_progress != FALSE)
}

fn is_kind(sim: &dyn SimView, tmpl: ThingTemplateId, kind: KindOfType) -> bool {
    sim.rules()
        .thing_factory
        .find_by_template_id_final(tmpl.0 as UnsignedShort)
        .is_some_and(|t| t.is_kind_of(kind) != FALSE)
}

impl ParkingPlaceBehavior {
    pub fn new(data: ParkingPlaceBehaviorModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            spaces: Vec::new(),
            runways: Vec::new(),
            healing: Vec::new(),
            next_heal_frame: UPDATE_SLEEP_FOREVER,
            got_info: FALSE,
            heli_rally_point: Coord3D::default(),
            heli_rally_point_exists: FALSE,
            owner: None,
        }
    }

    /// The reading half of `buildInfo`. `true` when the information was built now, in
    /// which case every door it numbered still has to be released.
    fn read_info(&mut self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        if self.got_info != FALSE {
            return false;
        }
        let Some(obj) = sim.object(owner) else {
            return false;
        };
        if obj
            .status
            .test(ObjectStatusType::UnderConstruction.as_i32())
            != FALSE
            || obj.status.test(ObjectStatusType::Sold.as_i32()) != FALSE
        {
            return false;
        }
        let mut door = 0;
        for row in 0..self.data.num_rows {
            for col in 0..self.data.num_cols {
                let name = |suffix: &str| format!("Runway{}{suffix}{}", col + 1, row + 1);
                let (_, hangar_start, mtx) =
                    single_logical_bone(sim, owner, &format!("{}Han", name("Park")));
                let hangar_start_orient = mtx.get_z_rotation();
                let (_, location, mtx) = single_logical_bone(sim, owner, &name("Parking"));
                let orientation = mtx.get_z_rotation();
                let (_, prep, _) = single_logical_bone(sim, owner, &name("Prep"));
                self.spaces.push(ParkingSpace {
                    hangar_start,
                    hangar_start_orient,
                    location,
                    prep,
                    orientation,
                    runway: col,
                    door,
                    object_in_space: ObjectId::INVALID,
                    reserved_for_exit: FALSE,
                    postponed_runway_reservation_for_takeoff: FALSE,
                });
                door += 1;
            }
        }
        if self.data.has_runways != FALSE {
            for col in 0..self.data.num_cols {
                let (_, start, _) =
                    single_logical_bone(sim, owner, &format!("RunwayStart{}", col + 1));
                let (_, end, _) = single_logical_bone(sim, owner, &format!("RunwayEnd{}", col + 1));
                self.runways.push(Runway {
                    start,
                    end,
                    in_use_by: ObjectId::INVALID,
                    next_in_line_for_takeoff: ObjectId::INVALID,
                    was_in_line: FALSE,
                });
            }
        }
        self.got_info = TRUE;
        true
    }

    fn build_info(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if self.read_info(&*sim, owner) {
            for space in &self.spaces {
                sim.production_set_hold_door_open(owner, door_of(space.door), FALSE);
            }
        }
    }

    fn purge_dead(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        self.build_info(sim, owner);
        for space in &mut self.spaces {
            if space.object_in_space != ObjectId::INVALID
                && living(&*sim, space.object_in_space).is_none()
            {
                space.object_in_space = ObjectId::INVALID;
                space.reserved_for_exit = FALSE;
                space.postponed_runway_reservation_for_takeoff = FALSE;
                sim.production_set_hold_door_open(owner, door_of(space.door), FALSE);
            }
        }
        for runway in &mut self.runways {
            if runway.in_use_by != ObjectId::INVALID && living(&*sim, runway.in_use_by).is_none() {
                runway.in_use_by = ObjectId::INVALID;
                runway.was_in_line = FALSE;
            }
            if runway.next_in_line_for_takeoff != ObjectId::INVALID
                && living(&*sim, runway.next_in_line_for_takeoff).is_none()
            {
                runway.next_in_line_for_takeoff = ObjectId::INVALID;
            }
        }
        // The C++ loop never advances past an invalid id (only a load could make one).
        let before = self.healing.len();
        self.healing.retain(|h| {
            h.getting_healed_id == ObjectId::INVALID || living(&*sim, h.getting_healed_id).is_some()
        });
        if self.healing.len() != before {
            self.reset_wake_frame(&*sim);
        }
    }

    fn reset_wake_frame(&mut self, sim: &dyn SimView) {
        self.next_heal_frame = if self.healing.is_empty() {
            UPDATE_SLEEP_FOREVER
        } else {
            sim.frame().wrapping_add(HEAL_RATE_FRAMES)
        };
    }

    /// `findPPI`.
    fn find_ppi(&self, id: ObjectId) -> Option<usize> {
        if self.got_info == FALSE || id == ObjectId::INVALID {
            return None;
        }
        self.spaces.iter().position(|s| s.object_in_space == id)
    }

    /// `findEmptyPPI`.
    fn find_empty_ppi(&self) -> Option<usize> {
        if self.got_info == FALSE {
            return None;
        }
        self.spaces
            .iter()
            .position(|s| s.object_in_space == ObjectId::INVALID && s.reserved_for_exit == FALSE)
    }

    /// `calcPPInfo`.
    fn fill_pp_info(&self, id: ObjectId, info: &mut ParkingPlaceInfo) {
        let Some(i) = self.find_ppi(id) else {
            return;
        };
        let space = &self.spaces[i];
        // The C++ reads past an empty runway list here.
        let runway = usize::try_from(space.runway)
            .ok()
            .and_then(|r| self.runways.get(r))
            .copied()
            .unwrap_or_default();
        let d = &self.data;
        let hangar = d.park_in_hangars != FALSE;
        info.parking_space = if hangar {
            space.hangar_start
        } else {
            space.location
        };
        info.runway_prep = space.prep;
        info.parking_orientation = if hangar {
            space.hangar_start_orient
        } else {
            space.orientation
        };
        info.runway_start = runway.start;
        info.runway_end = runway.end;
        info.runway_approach = runway.end;
        const APPROACH_DIST: Real = 0.75;
        info.runway_approach.x += (runway.end.x - runway.start.x) * APPROACH_DIST;
        info.runway_approach.y += (runway.end.y - runway.start.y) * APPROACH_DIST;
        info.runway_approach.z = runway.end.z + d.approach_height + d.landing_deck_height_offset;
        info.runway_exit = info.runway_approach;
        info.hangar_internal = space.hangar_start;
        info.hangar_internal_orient = space.hangar_start_orient;
        let mut vector = info.runway_start;
        vector.x -= info.runway_end.x;
        vector.y -= info.runway_end.y;
        vector.z -= info.runway_end.z;
        info.runway_takeoff_dist = vector.length();
        if let Some(r) = self.runways.iter().find(|r| r.in_use_by == id) {
            if r.was_in_line != FALSE {
                info.runway_start = info.runway_prep;
            }
        }
    }

    fn postpone_runway_reservation(&mut self, space: usize, for_landing: Bool) -> bool {
        if self.spaces.len() > self.runways.len() && space >= self.runways.len() {
            let postponed = &mut self.spaces[space].postponed_runway_reservation_for_takeoff;
            if for_landing != FALSE {
                *postponed = FALSE;
            } else if *postponed == FALSE {
                *postponed = TRUE;
                return true;
            }
        }
        false
    }

    fn release_space_of(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, id: ObjectId) {
        self.purge_dead(sim, owner);
        if let Some(space) = self.spaces.iter_mut().find(|s| s.object_in_space == id) {
            space.object_in_space = ObjectId::INVALID;
            space.reserved_for_exit = FALSE;
            space.postponed_runway_reservation_for_takeoff = FALSE;
            sim.production_set_hold_door_open(owner, door_of(space.door), FALSE);
        }
        if let Some(obj) = sim.find_object(id) {
            sim.clear_status(obj, ObjectStatusType::DeckHeightOffset);
        }
    }

    fn reserve_space_of(
        &mut self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        id: ObjectId,
        parking_offset: Real,
        info: Option<&mut ParkingPlaceInfo>,
    ) -> Bool {
        self.purge_dead(sim, owner);
        let Some(i) = self.find_ppi(id).or_else(|| self.find_empty_ppi()) else {
            return FALSE;
        };
        self.spaces[i].object_in_space = id;
        self.spaces[i].reserved_for_exit = FALSE;
        if self.data.landing_deck_height_offset != 0.0 {
            if let Some(obj) = sim.find_object(id) {
                sim.set_status(obj, ObjectStatusType::DeckHeightOffset);
            }
        }
        if let Some(info) = info {
            self.fill_pp_info(id, info);
            if parking_offset != 0.0 {
                let angle = self.spaces[i].orientation;
                info.parking_space.x += parking_offset * gm_cosf(angle);
                info.parking_space.y += parking_offset * gm_sinf(angle);
            }
        }
        sim.production_set_hold_door_open(owner, door_of(self.spaces[i].door), TRUE);
        TRUE
    }

    /// The parked units a death or a defection acts on, in space order.
    fn occupant(&self, sim: &dyn SimView, i: usize) -> Option<ObjectSlot> {
        let id = self.spaces[i].object_in_space;
        if id == ObjectId::INVALID {
            return None;
        }
        living(sim, id)
    }

    fn kill_parked(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        self.purge_dead(sim, owner);
        for i in 0..self.spaces.len() {
            let Some(unit) = self.occupant(&*sim, i) else {
                continue;
            };
            if sim.object_is_above_terrain(unit) != FALSE && !takeoff_or_landing(&*sim, unit) {
                continue;
            }
            kill_object(sim, unit, DamageType::Unresistable, DeathType::Normal);
        }
        self.purge_dead(sim, owner);
    }
}

impl ParkingPlaceFacet for ParkingPlaceBehavior {
    fn should_reserve_door_when_queued(&self, sim: &dyn SimView, tmpl: ThingTemplateId) -> Bool {
        Bool::from(!is_kind(sim, tmpl, KindOfType::ProducedAtHelipad))
    }

    fn has_available_space_for(&self, sim: &dyn SimView, tmpl: ThingTemplateId) -> Bool {
        if self.got_info == FALSE {
            return FALSE;
        }
        if is_kind(sim, tmpl, KindOfType::ProducedAtHelipad) {
            return TRUE;
        }
        Bool::from(self.spaces.iter().any(|s| {
            (s.object_in_space == ObjectId::INVALID || living(sim, s.object_in_space).is_none())
                && s.reserved_for_exit == FALSE
        }))
    }

    fn has_reserved_space(&self, id: ObjectId) -> Bool {
        Bool::from(
            self.got_info != FALSE
                && id != ObjectId::INVALID
                && self.spaces.iter().any(|s| s.object_in_space == id),
        )
    }

    fn get_space_index(&self, id: ObjectId) -> Int {
        if id == ObjectId::INVALID {
            return -1;
        }
        self.spaces
            .iter()
            .position(|s| s.object_in_space == id)
            .map_or(-1, |i| i as Int)
    }

    fn reserve_space(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        id: ObjectId,
        parking_offset: Real,
        info: Option<&mut ParkingPlaceInfo>,
    ) -> Bool {
        self.reserve_space_of(sim, ctx.owner_slot, id, parking_offset, info)
    }

    fn release_space(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, id: ObjectId) {
        self.release_space_of(sim, ctx.owner_slot, id);
    }

    fn reserve_runway(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        id: ObjectId,
        for_landing: Bool,
    ) -> Bool {
        self.purge_dead(sim, ctx.owner_slot);
        let mut runway = INVALID_RUNWAY;
        if let Some(i) = self.spaces.iter().position(|s| s.object_in_space == id) {
            if !cfg!(feature = "retail-crc") && self.postpone_runway_reservation(i, for_landing) {
                return FALSE;
            }
            runway = self.spaces[i].runway;
        }
        // A space of a field without runways makes the C++ index past the list.
        let Some(info) = usize::try_from(runway)
            .ok()
            .and_then(|r| self.runways.get_mut(r))
        else {
            return FALSE;
        };
        if info.in_use_by == id {
            TRUE
        } else if info.in_use_by == ObjectId::INVALID {
            info.in_use_by = id;
            if info.next_in_line_for_takeoff == id {
                info.next_in_line_for_takeoff = ObjectId::INVALID;
                info.was_in_line = TRUE;
            } else {
                info.was_in_line = FALSE;
            }
            TRUE
        } else {
            if for_landing == FALSE && info.next_in_line_for_takeoff == ObjectId::INVALID {
                info.next_in_line_for_takeoff = id;
            }
            FALSE
        }
    }

    fn release_runway(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, id: ObjectId) {
        self.purge_dead(sim, ctx.owner_slot);
        for runway in &mut self.runways {
            if runway.in_use_by == id {
                runway.in_use_by = ObjectId::INVALID;
                runway.was_in_line = FALSE;
            }
            if runway.next_in_line_for_takeoff == id {
                runway.next_in_line_for_takeoff = ObjectId::INVALID;
            }
        }
    }

    fn calc_pp_info(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        id: ObjectId,
        info: &mut ParkingPlaceInfo,
    ) {
        self.fill_pp_info(id, info);
    }

    fn get_runway_index(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, id: ObjectId) -> Int {
        self.purge_dead(sim, ctx.owner_slot);
        self.spaces
            .iter()
            .find(|s| s.object_in_space == id)
            .map_or(INVALID_RUNWAY, |s| s.runway)
    }

    fn get_runway_count(&self) -> Int {
        self.runways.len() as Int
    }

    fn get_runway_reservation(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        runway: Int,
        _rtype: RunwayReservationType,
    ) -> ObjectId {
        self.purge_dead(sim, ctx.owner_slot);
        usize::try_from(runway)
            .ok()
            .and_then(|r| self.runways.get(r))
            .map_or(ObjectId::INVALID, |r| r.in_use_by)
    }

    fn transfer_runway_reservation_to_next_in_line_for_takeoff(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        id: ObjectId,
    ) {
        self.purge_dead(sim, ctx.owner_slot);
        if let Some(runway) = self.runways.iter_mut().find(|r| r.in_use_by == id) {
            if runway.next_in_line_for_takeoff != ObjectId::INVALID {
                runway.in_use_by = runway.next_in_line_for_takeoff;
                runway.was_in_line = TRUE;
                runway.next_in_line_for_takeoff = ObjectId::INVALID;
            }
        }
    }

    fn get_approach_height(&self) -> Real {
        self.data.approach_height
    }

    fn get_landing_deck_height_offset(&self) -> Real {
        self.data.landing_deck_height_offset
    }

    fn set_healee(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        healee: ObjectSlot,
        add: Bool,
    ) {
        let Some(id) = sim.object(healee).map(|o| o.id) else {
            return;
        };
        if add != FALSE {
            if self.healing.iter().any(|h| h.getting_healed_id == id) {
                return;
            }
            self.healing.push(Healing {
                getting_healed_id: id,
                heal_start_frame: sim.frame(),
            });
            self.reset_wake_frame(&*sim);
        } else {
            let before = self.healing.len();
            self.healing.retain(|h| h.getting_healed_id != id);
            if self.healing.len() != before {
                self.reset_wake_frame(&*sim);
            }
        }
    }

    fn kill_all_parked_units(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.kill_parked(sim, ctx.owner_slot);
    }

    fn defect_all_parked_units(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_team: TeamId,
        detection_time: UnsignedInt,
    ) {
        let owner = ctx.owner_slot;
        self.purge_dead(sim, owner);
        for i in 0..self.spaces.len() {
            let Some(unit) = self.occupant(&*sim, i) else {
                continue;
            };
            if sim.object_is_above_terrain(unit) != FALSE && !takeoff_or_landing(&*sim, unit) {
                let Some((id, team, producer)) =
                    sim.object(unit).map(|o| (o.id, o.team_id, o.producer_id))
                else {
                    continue;
                };
                if sim.player_index(new_team) != sim.player_index(team) {
                    self.release_space_of(sim, owner, id);
                    if producer == ctx.owner_id {
                        if let Some(o) = sim.object_mut(unit) {
                            o.producer_id = ObjectId::INVALID;
                        }
                    }
                }
            } else {
                sim.defect_object(unit, new_team, detection_time, TRUE);
            }
        }
        self.purge_dead(sim, owner);
    }
}

impl ExitFacet for ParkingPlaceBehavior {
    fn reserve_door_for_exit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        tmpl_id: ThingTemplateId,
        _specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        self.purge_dead(sim, ctx.owner_slot);
        if is_kind(&*sim, tmpl_id, KindOfType::ProducedAtHelipad) {
            return ExitDoorType::DoorNoneNeeded;
        }
        match self.find_empty_ppi() {
            Some(i) => {
                self.spaces[i].object_in_space = ObjectId::INVALID;
                self.spaces[i].reserved_for_exit = TRUE;
                door_of(self.spaces[i].door)
            }
            None => ExitDoorType::DoorNoneAvailable,
        }
    }

    fn exit_object_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
        exit_door: ExitDoorType,
    ) {
        let owner = ctx.owner_slot;
        let Some((new_id, helipad)) = sim.object(new_obj).map(|o| {
            let helipad = o
                .thing
                .is_kind_of(&sim.rules().thing_factory, KindOfType::ProducedAtHelipad);
            (o.id, helipad != FALSE)
        }) else {
            return;
        };
        if exit_door != ExitDoorType::DoorNoneNeeded {
            let door = exit_door.as_i32();
            let Some(space) = self.spaces.iter_mut().find(|s| {
                s.object_in_space == ObjectId::INVALID
                    && s.reserved_for_exit == TRUE
                    && s.door == door
            }) else {
                return;
            };
            space.object_in_space = new_id;
            space.reserved_for_exit = FALSE;
        }
        let parking_offset = sim.object_jet_ai(new_obj).map_or(0.0, |j| j.parking_offset);
        let mut info = ParkingPlaceInfo::default();
        if helipad {
            let (_, position, mtx) = single_logical_bone(&*sim, owner, "HeliPark01");
            info.hangar_internal = position;
            info.hangar_internal_orient = mtx.get_z_rotation();
            info.parking_space = info.hangar_internal;
            info.parking_orientation = info.hangar_internal_orient;
        } else if self.reserve_space_of(sim, owner, new_id, parking_offset, Some(&mut info))
            == FALSE
        {
            // The C++ also leaves `hangarInternal` uninitialized on this path.
            if let Some(o) = sim.object(owner) {
                info.parking_space = *o.thing.get_position();
                info.parking_orientation = o.thing.get_orientation();
            }
        }
        sim.set_object_position(new_obj, &info.hangar_internal);
        sim.set_object_orientation(new_obj, info.hangar_internal_orient);
        sim.pathfinder_add_object(new_obj);
        if sim.object_has_ai(new_obj) == FALSE {
            return;
        }
        if helipad && self.heli_rally_point_exists != FALSE {
            let rally = self.heli_rally_point;
            sim.ai_move_to_position(new_obj, &rally, CommandSourceType::FromAi);
        } else if !helipad {
            sim.ai_follow_exit_production_path(
                new_obj,
                &[info.parking_space],
                Some(owner),
                CommandSourceType::FromAi,
            );
        } else {
            sim.ai_move_to_position(new_obj, &info.parking_space, CommandSourceType::FromAi);
        }
    }

    fn unreserve_door_for_exit(&mut self, exit_door: ExitDoorType) {
        if exit_door != ExitDoorType::DoorNoneNeeded {
            let door = exit_door.as_i32();
            if let Some(space) = self.spaces.iter_mut().find(|s| s.door == door) {
                space.object_in_space = ObjectId::INVALID;
                space.reserved_for_exit = FALSE;
            }
        }
    }

    fn set_rally_point(&mut self, pos: &Coord3D) {
        self.heli_rally_point_exists = TRUE;
        self.heli_rally_point = *pos;
    }

    fn get_rally_point(&self) -> Option<&Coord3D> {
        (self.heli_rally_point_exists != FALSE).then_some(&self.heli_rally_point)
    }

    fn get_natural_rally_point(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rally_point: &mut Coord3D,
        _offset: Bool,
    ) -> Bool {
        let (found, position, _) = single_logical_bone(sim, ctx.owner_slot, "HeliPark01");
        *rally_point = position;
        found
    }

    fn get_exit_position(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        exit_pos: &mut Coord3D,
    ) -> Bool {
        let (found, position, _) = single_logical_bone(sim, ctx.owner_slot, "HeliPark01");
        *exit_pos = position;
        found
    }
}

impl UpdateFacet for ParkingPlaceBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        self.purge_dead(sim, owner);
        let now = sim.frame();
        if now >= self.next_heal_frame {
            self.next_heal_frame = now.wrapping_add(HEAL_RATE_FRAMES);
            let mut i = 0;
            while i < self.healing.len() {
                let id = self.healing[i].getting_healed_id;
                if id == ObjectId::INVALID {
                    i += 1;
                    continue;
                }
                let Some(unit) = living(&*sim, id) else {
                    self.healing.remove(i);
                    continue;
                };
                let mut heal = DamageInfo::default();
                heal.input.damage_type = DamageType::Healing;
                heal.input.death_type = DeathType::NoneType;
                heal.input.source_id = ctx.owner_id;
                heal.input.amount =
                    HEAL_RATE_FRAMES as Real * self.data.heal_amount * SECONDS_PER_LOGICFRAME_REAL;
                sim.attempt_healing(unit, &mut heal);
                i += 1;
            }
        }
        UpdateSleepTime::NONE
    }
}

impl DieFacet for ParkingPlaceBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        self.kill_parked(sim, ctx.owner_slot);
    }
}

impl BehaviorModule for ParkingPlaceBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        self.owner = Some(ctx.owner_slot);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    /// A release asked for while this module was running (a parked aircraft dying inside
    /// `killAllParkedUnits`).
    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let DeferredSelfOp::ParkingRelease(id) = op {
            self.release_space_of(sim, ctx.owner_slot, id);
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    /// The doors `buildInfo` releases during a load are left to the production module's own
    /// loaded state.
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 3)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        let loading = xfer.mode() == XferMode::Load;
        if loading {
            if let Some(owner) = self.owner {
                self.read_info(sim, owner);
            }
        }

        let mut count = self.spaces.len() as UnsignedByte;
        xfer.xfer_unsigned_byte(&mut count)?;
        if loading {
            for i in 0..usize::from(count) {
                let mut id = ObjectId::INVALID;
                let mut reserved = FALSE;
                xfer.xfer_object_id(&mut id)?;
                xfer.xfer_bool(&mut reserved)?;
                if let Some(space) = self.spaces.get_mut(i) {
                    space.object_in_space = id;
                    space.reserved_for_exit = reserved;
                }
            }
        } else {
            for space in &mut self.spaces {
                xfer.xfer_object_id(&mut space.object_in_space)?;
                xfer.xfer_bool(&mut space.reserved_for_exit)?;
            }
        }

        let mut count = self.runways.len() as UnsignedByte;
        xfer.xfer_unsigned_byte(&mut count)?;
        if loading {
            for i in 0..usize::from(count) {
                let mut in_use = ObjectId::INVALID;
                let mut next = ObjectId::INVALID;
                let mut was_in_line = FALSE;
                xfer.xfer_object_id(&mut in_use)?;
                xfer.xfer_object_id(&mut next)?;
                xfer.xfer_bool(&mut was_in_line)?;
                if let Some(runway) = self.runways.get_mut(i) {
                    runway.in_use_by = in_use;
                    runway.next_in_line_for_takeoff = next;
                    runway.was_in_line = was_in_line;
                }
            }
        } else {
            for runway in &mut self.runways {
                xfer.xfer_object_id(&mut runway.in_use_by)?;
                xfer.xfer_object_id(&mut runway.next_in_line_for_takeoff)?;
                xfer.xfer_bool(&mut runway.was_in_line)?;
            }
        }

        let mut count = self.healing.len() as UnsignedByte;
        xfer.xfer_unsigned_byte(&mut count)?;
        if loading {
            self.healing = vec![Healing::default(); usize::from(count)];
        }
        // A list longer than the count byte is still written whole.
        for healing in &mut self.healing {
            xfer.xfer_object_id(&mut healing.getting_healed_id)?;
            xfer.xfer_unsigned_int(&mut healing.heal_start_frame)?;
        }

        if version >= 2 {
            xfer.xfer_coord3d(&mut self.heli_rally_point)?;
            xfer.xfer_bool(&mut self.heli_rally_point_exists)?;
        }
        if version >= 3 {
            xfer.xfer_unsigned_int(&mut self.next_heal_frame)?;
        } else if loading {
            self.next_heal_frame = 0;
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

    fn as_parking_place(&self) -> Option<&dyn ParkingPlaceFacet> {
        Some(self)
    }

    fn as_parking_place_mut(&mut self) -> Option<&mut dyn ParkingPlaceFacet> {
        Some(self)
    }

    fn as_exit(&self) -> Option<&dyn ExitFacet> {
        Some(self)
    }

    fn as_exit_mut(&mut self) -> Option<&mut dyn ExitFacet> {
        Some(self)
    }
}

pub fn parse_parking_place_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = ParkingPlaceBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("NumRows") {
            r.parse_int(&mut data.num_rows)
        } else if is("NumCols") {
            r.parse_int(&mut data.num_cols)
        } else if is("ApproachHeight") {
            r.parse_real(&mut data.approach_height)
        } else if is("LandingDeckHeightOffset") {
            r.parse_real(&mut data.landing_deck_height_offset)
        } else if is("HasRunways") {
            r.parse_bool(&mut data.has_runways)
        } else if is("ParkInHangars") {
            r.parse_bool(&mut data.park_in_hangars)
        } else if is("HealAmountPerSecond") {
            r.parse_real(&mut data.heal_amount)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_parking_place_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<ParkingPlaceBehaviorModuleData>()
        .expect("ParkingPlaceBehavior data type mismatch")
        .clone();
    Box::new(ParkingPlaceBehavior::new(typed))
}
