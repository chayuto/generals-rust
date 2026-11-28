//! `PKT-391`: `ObjectCreationList` runtime (`ObjectCreationList::create` and all
//! five `ObjectCreationNugget::create` implementations from
//! `GameLogic/Object/ObjectCreationList.cpp`).

use crate::arena::{ModuleSlot, ObjectSlot, ThingTemplateId};
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::combat::locomotor_set::LocomotorSetType;
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::CommandSourceType;
use crate::modules::update::float_update::FloatUpdate;
use crate::modules::update::lifetime_update::LifetimeUpdate;
use crate::modules::update::physics_update;
use crate::modules::update::radius_decal_update::RadiusDecalUpdate;
use crate::object::object_creation_list::{
    AttackNugget, DeliverPayloadData, DeliverPayloadNugget, GenericNugget, ObjectCreationNugget,
    RandomForceNugget,
};
use crate::object::object_script_status_bits::OBJECT_STATUS_SCRIPT_TARGETABLE;
use crate::object::weapon_fire::create_and_fire_temp_weapon;
use crate::object::weapon_set::{WeaponLockType, WeaponSlotType};
use crate::pathfinder::pathfind_cell::CellType as PathfindCellType;
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PATHFIND_CELL_SIZE_F};
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::sim::{
    FindPositionOptions, SimApi, SimContext, SimView, FPF_USE_HIGHEST_LAYER,
    PRIVATE_STATUS_OFF_MAP,
};
use crate::terrain::terrain_logic::TerrainShape;
use zh_common::math::{gm_atan2f, gm_cosf, gm_sinf, gm_sqrtf, Coord3D, Matrix3D, PI};
use zh_common::types::{
    Bool, Int, ObjectId, Real, TeamId, UnsignedInt, FALSE, TRUE,
};

/// `INVALID_ANGLE` from `Common/GameType.h`.
pub const INVALID_ANGLE: Real = -100.0;

// `DebrisDisposition` flags matching `DEBRIS_DISPOSITION_NAMES` in `object_creation_list.rs`.
const DISPOSITION_LIKE_EXISTING: UnsignedInt = 1 << 0;
const DISPOSITION_ON_GROUND_ALIGNED: UnsignedInt = 1 << 1;
const DISPOSITION_SEND_IT_FLYING: UnsignedInt = 1 << 2;
const DISPOSITION_SEND_IT_UP: UnsignedInt = 1 << 3;
const DISPOSITION_SEND_IT_OUT: UnsignedInt = 1 << 4;
const DISPOSITION_RANDOM_FORCE: UnsignedInt = 1 << 5;
const DISPOSITION_FLOATING: UnsignedInt = 1 << 6;
const DISPOSITION_INHERIT_VELOCITY: UnsignedInt = 1 << 7;
const DISPOSITION_WHIRLING: UnsignedInt = 1 << 8;

/// Which of the three C++ `ObjectCreationList::create` overloads is being called.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OclCreateMode {
    /// `ObjectCreationList::create(ocl, primary, secondary, lifetimeFrames)`
    Object {
        secondary: Option<ObjectSlot>,
    },
    /// `ObjectCreationList::create(ocl, primaryObj, primary, secondary, angle, lifetimeFrames)`
    PositionAngle {
        primary_pos: Coord3D,
        secondary_pos: Option<Coord3D>,
        angle: Real,
    },
    /// `ObjectCreationList::create(ocl, primaryObj, primary, secondary, createOwner, lifetimeFrames)`
    PositionCreateOwner {
        primary_pos: Coord3D,
        secondary_pos: Option<Coord3D>,
        create_owner: Bool,
    },
}

/// Arguments to [`create_ocl`] / [`SimApi::create_ocl`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OclCreateArgs {
    pub primary: ObjectSlot,
    pub mode: OclCreateMode,
    pub lifetime_frames: UnsignedInt,
}

impl OclCreateArgs {
    /// `ObjectCreationList::create(ocl, primary, secondary, 0)`.
    #[inline]
    pub const fn object(primary: ObjectSlot, secondary: Option<ObjectSlot>) -> Self {
        Self {
            primary,
            mode: OclCreateMode::Object { secondary },
            lifetime_frames: 0,
        }
    }

    /// `ObjectCreationList::create(ocl, primary, secondary, lifetimeFrames)`.
    #[inline]
    pub const fn object_with_lifetime(
        primary: ObjectSlot,
        secondary: Option<ObjectSlot>,
        lifetime_frames: UnsignedInt,
    ) -> Self {
        Self {
            primary,
            mode: OclCreateMode::Object { secondary },
            lifetime_frames,
        }
    }

    /// `ObjectCreationList::create(ocl, primaryObj, primaryPos, secondaryPos, angle, 0)`.
    #[inline]
    pub const fn position_angle(
        primary: ObjectSlot,
        primary_pos: Coord3D,
        secondary_pos: Option<Coord3D>,
        angle: Real,
    ) -> Self {
        Self {
            primary,
            mode: OclCreateMode::PositionAngle {
                primary_pos,
                secondary_pos,
                angle,
            },
            lifetime_frames: 0,
        }
    }

    /// `ObjectCreationList::create(ocl, primaryObj, primaryPos, secondaryPos, createOwner, 0)`.
    #[inline]
    pub const fn position_create_owner(
        primary: ObjectSlot,
        primary_pos: Coord3D,
        secondary_pos: Option<Coord3D>,
        create_owner: Bool,
    ) -> Self {
        Self {
            primary,
            mode: OclCreateMode::PositionCreateOwner {
                primary_pos,
                secondary_pos,
                create_owner,
            },
            lifetime_frames: 0,
        }
    }
}

/// Recorded payload delivery order for `DeliverPayloadAIUpdate` (unported until Slice L8).
#[derive(Debug, Clone, PartialEq)]
pub struct DeliveredPayloadRecord {
    pub transport_id: ObjectId,
    pub move_to_pos: Coord3D,
    pub target_pos: Coord3D,
    pub data: DeliverPayloadData,
}

/// `calcRandomForce` from `ObjectCreationList.cpp`.
pub fn calc_random_force(
    sim: &mut dyn SimApi,
    min_mag: Real,
    max_mag: Real,
    min_pitch: Real,
    max_pitch: Real,
) -> Coord3D {
    let angle = sim.rng_logic_real(0.0, 2.0 * PI);
    let pitch = sim.rng_logic_real(min_pitch, max_pitch);
    let mag = sim.rng_logic_real(min_mag, max_mag);

    let mut mtx = Matrix3D::identity();
    mtx.scale(mag);
    mtx.rotate_z(angle);
    mtx.rotate_y(-pitch);

    mtx.get_x_vector()
}

/// Executes `ObjectCreationList::create`, matching C++ return value (the first
/// object created by any nugget in the list, or `None`).
///
/// Always appends `(ocl_name, creator_id, log_pos, log_target_pos)` to
/// `sim.world.spawned_ocls` for backward compatibility with isolated tests.
pub fn create_ocl(
    sim: &mut SimContext,
    ocl_name: &str,
    args: OclCreateArgs,
) -> Option<ObjectSlot> {
    let primary_info = sim
        .world
        .objects
        .get(args.primary)
        .map(|o| (o.id, *o.thing.get_position()));
    let creator_id = primary_info.map_or(ObjectId::INVALID, |(id, _)| id);
    let fallback_pos = primary_info.map_or_else(Coord3D::default, |(_, p)| p);

    let (log_pos, log_target_pos) = match args.mode {
        OclCreateMode::Object { secondary } => {
            let sec_pos = secondary
                .and_then(|s| sim.world.objects.get(s))
                .map_or(fallback_pos, |o| *o.thing.get_position());
            (fallback_pos, sec_pos)
        }
        OclCreateMode::PositionAngle {
            primary_pos,
            secondary_pos,
            ..
        }
        | OclCreateMode::PositionCreateOwner {
            primary_pos,
            secondary_pos,
            ..
        } => (primary_pos, secondary_pos.unwrap_or(primary_pos)),
    };

    sim.world
        .spawned_ocls
        .push((ocl_name.to_owned(), creator_id, log_pos, log_target_pos));

    let Some(ocl) = sim.rules.ocl_store.find(ocl_name).cloned() else {
        return None;
    };
    if sim.world.objects.get(args.primary).is_none() {
        return None;
    }

    let mut first_object: Option<ObjectSlot> = None;
    for nugget in &ocl.nuggets {
        let cur = match nugget {
            ObjectCreationNugget::Generic(g) => exec_generic(sim, g, args),
            ObjectCreationNugget::ApplyRandomForce(arf) => exec_apply_random_force(sim, arf, args),
            ObjectCreationNugget::DeliverPayload(dp) => exec_deliver_payload(sim, dp, args),
            ObjectCreationNugget::FireWeapon(w) => exec_fire_weapon(sim, w, args),
            ObjectCreationNugget::Attack(atk) => exec_attack(sim, atk, args),
        };
        if first_object.is_none() {
            first_object = cur;
        }
    }
    first_object
}

fn resolve_positions(
    sim: &SimContext,
    args: OclCreateArgs,
) -> (Option<Coord3D>, Option<Coord3D>, Real, Bool) {
    match args.mode {
        OclCreateMode::Object { secondary } => {
            let p_pos = sim
                .world
                .objects
                .get(args.primary)
                .map(|o| *o.thing.get_position());
            let s_pos = secondary
                .and_then(|s| sim.world.objects.get(s))
                .map(|o| *o.thing.get_position());
            (p_pos, s_pos, INVALID_ANGLE, TRUE)
        }
        OclCreateMode::PositionAngle {
            primary_pos,
            secondary_pos,
            angle,
        } => (Some(primary_pos), secondary_pos, angle, TRUE),
        OclCreateMode::PositionCreateOwner {
            primary_pos,
            secondary_pos,
            create_owner,
        } => (Some(primary_pos), secondary_pos, INVALID_ANGLE, create_owner),
    }
}

fn exec_fire_weapon(
    sim: &mut SimContext,
    weapon_name: &str,
    args: OclCreateArgs,
) -> Option<ObjectSlot> {
    let (primary_pos, secondary_pos, _, _) = resolve_positions(sim, args);
    if sim.world.objects.get(args.primary).is_none() || primary_pos.is_none() {
        return None;
    }
    let Some(sec_pos) = secondary_pos else {
        return None;
    };
    if !weapon_name.is_empty() {
        let _ = create_and_fire_temp_weapon(sim, args.primary, weapon_name, &sec_pos);
    }
    None
}

fn exec_attack(
    sim: &mut SimContext,
    nugget: &AttackNugget,
    args: OclCreateArgs,
) -> Option<ObjectSlot> {
    let (primary_pos, secondary_pos, _, _) = resolve_positions(sim, args);
    if sim.world.objects.get(args.primary).is_none() || primary_pos.is_none() {
        return None;
    }
    let Some(sec_pos) = secondary_pos else {
        return None;
    };

    let has_ai = sim
        .world
        .objects
        .get(args.primary)
        .is_some_and(|o| o.ai_slot.is_some());
    if has_ai {
        if let Some(slot) = WeaponSlotType::from_i32(nugget.weapon_slot) {
            sim.lock_weapon_slot(args.primary, slot, WeaponLockType::LockedTemporarily);
        }
        sim.ai_attack_position(
            args.primary,
            &sec_pos,
            nugget.number_of_shots,
            CommandSourceType::FromAi,
        );
    }

    if let Some(rd_slot) = sim.module_by_name(args.primary, "RadiusDecalUpdate") {
        if !sim.world.modules.is_executing(rd_slot) {
            let decal_name = nugget.delivery_decal.name.clone();
            let radius = nugget.delivery_decal_radius;
            let _ = sim.dispatch_module(rd_slot, |m, sim_api, ctx| {
                if let Some(rd) = m
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<RadiusDecalUpdate>())
                {
                    rd.create_radius_decal(sim_api, ctx, &decal_name, radius);
                    rd.set_kill_when_no_longer_attacking(TRUE);
                }
            });
        }
    }
    None
}

fn exec_apply_random_force(
    sim: &mut SimContext,
    nugget: &RandomForceNugget,
    args: OclCreateArgs,
) -> Option<ObjectSlot> {
    // Only the object-based overload overrides `ObjectCreationNugget::create`.
    if !matches!(args.mode, OclCreateMode::Object { .. }) {
        return None;
    }
    if physics_update::physics(sim, args.primary).is_none() {
        return None;
    }

    let force = calc_random_force(
        sim,
        nugget.min_mag,
        nugget.max_mag,
        nugget.min_pitch,
        nugget.max_pitch,
    );
    sim.physics_apply_force(args.primary, &force);

    let yaw = sim.rng_logic_real(-nugget.spin_rate, nugget.spin_rate);
    let roll = sim.rng_logic_real(-nugget.spin_rate, nugget.spin_rate);
    let pitch = sim.rng_logic_real(-nugget.spin_rate, nugget.spin_rate);
    if let Some(p) = physics_update::physics_mut(sim, args.primary) {
        p.set_yaw_rate(yaw);
        p.set_roll_rate(roll);
        p.set_pitch_rate(pitch);
    }
    None
}

fn default_team_for_object(sim: &SimContext, slot: ObjectSlot) -> TeamId {
    if let Some(team_id) = sim.world.objects.get(slot).map(|o| o.team_id) {
        if sim.controlling_player(team_id).is_some() {
            return sim.player_default_team(team_id);
        }
    }
    let neutral = sim.neutral_team_id();
    sim.player_default_team(neutral)
}

fn set_producer(sim: &mut SimContext, obj: ObjectSlot, producer_id: ObjectId) {
    if let Some(rec) = sim.world.objects.get_mut(obj) {
        rec.producer_id = producer_id;
    }
}

fn transport_locomotor_metrics(
    sim: &SimContext,
    transport: ObjectSlot,
) -> Option<(Real, Real)> {
    let penalty = sim.rules.global_data.movement_penalty_damage_state;
    if let Some(loco) = sim.cur_locomotor(transport) {
        let dmg = sim.object_body_damage_state(transport);
        let max_speed = loco.get_max_speed_for_condition(dmg, penalty);
        return Some((max_speed, loco.preferred_height));
    }
    // `DeliverPayloadAIUpdate` behavior is not ported yet (`DATA_ONLY_MODULES`),
    // so read the normal locomotor template directly from the transport's template.
    let obj = sim.world.objects.get(transport)?;
    let tmpl = obj
        .thing
        .template_override()
        .get_final(&sim.rules.thing_factory)?;
    let loco_name = tmpl.get_locomotor_set(LocomotorSetType::Normal).first()?;
    let loco_tmpl = sim.rules.locomotor_store.find_by_name(loco_name)?;
    let dmg = sim.object_body_damage_state(transport);
    let max_speed = if crate::combat::locomotor::is_condition_better(dmg, penalty) {
        loco_tmpl.max_speed
    } else {
        loco_tmpl.max_speed_damaged
    };
    Some((max_speed, loco_tmpl.preferred_height))
}

fn exec_deliver_payload(
    sim: &mut SimContext,
    nugget: &DeliverPayloadNugget,
    args: OclCreateArgs,
) -> Option<ObjectSlot> {
    let (primary_pos_opt, secondary_pos_opt, _, create_owner) = resolve_positions(sim, args);
    let primary_id = sim.world.objects.get(args.primary).map(|o| o.id)?;
    let primary_pos = primary_pos_opt?;
    let secondary_pos = secondary_pos_opt?;
    let owner_team = default_team_for_object(sim, args.primary);

    let mut ccw_x: Real = 0.0;
    let mut ccw_y: Real = 0.0;
    let mut cw_x: Real = 0.0;
    let mut cw_y: Real = 0.0;

    if nugget.formation_size > 1 {
        let mut dx = primary_pos.x - secondary_pos.x;
        let mut dy = primary_pos.y - secondary_pos.y;
        let length = gm_sqrtf(dx * dx + dy * dy);
        dx /= length;
        dy /= length;

        let radians = 90.0 * PI / 180.0;
        let s = gm_sinf(radians);
        let c = gm_cosf(radians);
        ccw_x = dx * c + dy * -s + dx;
        ccw_y = dx * s + dy * c + dy;

        let s_cw = gm_sinf(-radians);
        let c_cw = gm_cosf(-radians);
        cw_x = dx * c_cw + dy * -s_cw + dx;
        cw_y = dx * s_cw + dy * c_cw + dy;
    }

    let mut first_transport: Option<ObjectSlot> = None;

    for formation_index in 0..nugget.formation_size {
        let mut offset = Coord3D::default();
        let offset_multiplier =
            ((((formation_index + 1) / 2) as Real) * nugget.formation_spacing) as Int;
        if formation_index % 2 != 0 {
            offset.x = ccw_x * (offset_multiplier as Real);
            offset.y = ccw_y * (offset_multiplier as Real);
        } else {
            offset.x = cw_x * (offset_multiplier as Real);
            offset.y = cw_y * (offset_multiplier as Real);
        }

        let mut start_pos = primary_pos;
        let mut move_to_pos = secondary_pos;
        start_pos.add(&offset);
        move_to_pos.add(&offset);

        let mut target_pos = secondary_pos;
        target_pos.x += offset.x * (1.0 - nugget.convergence_factor);
        target_pos.y += offset.y * (1.0 - nugget.convergence_factor);

        if nugget.error_radius > 1.0 && formation_index > 0 {
            let random_radius = sim.rng_logic_real(0.0, nugget.error_radius);
            let random_angle = sim.rng_logic_real(0.0, 2.0 * PI);
            target_pos.x += random_radius * gm_cosf(random_angle);
            target_pos.y += random_radius * gm_sinf(random_angle);
        }

        let orient = gm_atan2f(move_to_pos.y - start_pos.y, move_to_pos.x - start_pos.x);
        if nugget.data.dist_to_target > 0.0 {
            const SLOP: Real = 1.5;
            start_pos.x -= gm_cosf(orient) * nugget.data.dist_to_target * SLOP;
            start_pos.y -= gm_sinf(orient) * nugget.data.dist_to_target * SLOP;
        }

        let transport = if create_owner != FALSE {
            let Some(t) = sim.new_object_from_template(&nugget.transport_name, owner_team) else {
                return None;
            };
            if first_transport.is_none() {
                first_transport = Some(t);
            }
            sim.set_object_position(t, &start_pos);
            sim.set_object_orientation(t, orient);
            set_producer(sim, t, primary_id);
            if let Some(rec) = sim.world.objects.get_mut(t) {
                rec.script_status |= OBJECT_STATUS_SCRIPT_TARGETABLE;
            }
            if nugget.delay_delivery_frames_max > 0 {
                let delay = sim.rng_logic_int(0, nugget.delay_delivery_frames_max as Int) as UnsignedInt;
                let till = sim.frame().wrapping_add(delay);
                sim.set_disabled_until(t, DisabledType::Default, till);
            }
            t
        } else {
            args.primary
        };

        let sp_creator = if formation_index == 0 {
            primary_id
        } else {
            ObjectId::INVALID
        };
        sim.set_special_power_creator(transport, sp_creator);

        let has_deliver_payload_ai =
            sim.object_has_template_module(transport, "DeliverPayloadAIUpdate") != FALSE;
        if has_deliver_payload_ai {
            let loco_metrics = transport_locomotor_metrics(sim, transport);

            if nugget.start_at_max_speed != FALSE && create_owner != FALSE {
                if physics_update::physics(sim, transport).is_some() {
                    if let (Some((max_speed, _)), Some(obj)) =
                        (loco_metrics, sim.world.objects.get(transport))
                    {
                        let dir = obj.thing.get_unit_direction_vector_2d();
                        let mass = physics_update::get_mass(sim, transport);
                        let factor = max_speed * mass;
                        let starting_force =
                            Coord3D::new(dir.x * factor, dir.y * factor, dir.z * factor);
                        sim.physics_apply_motive_force(transport, &starting_force);
                    }
                }
            }

            let mut data = nugget.data.clone();
            if formation_index != 0 {
                data.delivery_decal_radius = 0.0;
            }
            let transport_id = sim
                .world
                .objects
                .get(transport)
                .map_or(ObjectId::INVALID, |o| o.id);
            sim.world.delivered_payloads.push(DeliveredPayloadRecord {
                transport_id,
                move_to_pos,
                target_pos,
                data,
            });

            if nugget.start_at_preferred_height != FALSE && create_owner != FALSE {
                let preferred_height = loco_metrics.map_or(0.0, |(_, h)| h);
                start_pos.z =
                    sim.terrain.get_ground_height(start_pos.x, start_pos.y, None) + preferred_height;
                sim.set_object_position(transport, &start_pos);
            }

            let put_in_container_tmpl = if nugget.put_in_container.is_empty() {
                None
            } else {
                sim.rules
                    .thing_factory
                    .find_template_id(&nugget.put_in_container)
            };

            for item in &nugget.payload {
                let Some(payload_tmpl) = sim.rules.thing_factory.find_template_id(&item.name) else {
                    return None;
                };
                for i in 0..item.count {
                    let mut payload = sim.factory_new_object(payload_tmpl, owner_team);
                    sim.set_object_position(payload, &start_pos);
                    set_producer(sim, payload, transport_id);
                    sim.set_special_power_creator(
                        payload,
                        if formation_index == 0 && i == 0 {
                            primary_id
                        } else {
                            ObjectId::INVALID
                        },
                    );

                    if let Some(container_tmpl) = put_in_container_tmpl {
                        let container = sim.factory_new_object(container_tmpl, owner_team);
                        sim.set_object_position(container, &start_pos);
                        set_producer(sim, container, transport_id);
                        sim.set_special_power_creator(
                            container,
                            if formation_index == 0 && i == 0 {
                                primary_id
                            } else {
                                ObjectId::INVALID
                            },
                        );
                        if sim.is_valid_container_for(container, payload, TRUE) != FALSE {
                            sim.contain_add(container, payload);
                            payload = container;
                        }
                    }

                    if sim.is_valid_container_for(transport, payload, TRUE) != FALSE {
                        sim.contain_add(transport, payload);
                    }
                }
            }
        }
    }

    first_transport
}

fn exec_generic(
    sim: &mut SimContext,
    nugget: &GenericNugget,
    args: OclCreateArgs,
) -> Option<ObjectSlot> {
    match args.mode {
        OclCreateMode::Object { .. } => {
            if nugget.skip_if_significantly_airborne != FALSE
                && sim.object_is_significantly_above_terrain(args.primary) != FALSE
            {
                return None;
            }
            let (pos, mtx, orient) = sim
                .world
                .objects
                .get(args.primary)
                .map(|o| {
                    (
                        *o.thing.get_position(),
                        *o.thing.get_transform_matrix(),
                        o.thing.get_orientation(),
                    )
                })?;
            really_create(
                sim,
                nugget,
                &pos,
                Some(&mtx),
                orient,
                Some(args.primary),
                args.lifetime_frames,
            )
        }
        OclCreateMode::PositionAngle {
            primary_pos, angle, ..
        } => {
            let resolved_angle = if angle == INVALID_ANGLE { 0.0 } else { angle };
            really_create(
                sim,
                nugget,
                &primary_pos,
                None,
                resolved_angle,
                Some(args.primary),
                args.lifetime_frames,
            )
        }
        OclCreateMode::PositionCreateOwner { primary_pos, .. } => really_create(
            sim,
            nugget,
            &primary_pos,
            None,
            0.0,
            Some(args.primary),
            args.lifetime_frames,
        ),
    }
}

fn really_create(
    sim: &mut SimContext,
    nugget: &GenericNugget,
    pos: &Coord3D,
    mtx: Option<&Matrix3D>,
    orientation: Real,
    source_obj: Option<ObjectSlot>,
    lifetime_frames: UnsignedInt,
) -> Option<ObjectSlot> {
    if nugget.names.is_empty() {
        return None;
    }

    if nugget.requires_live_player != FALSE {
        let live = source_obj
            .and_then(|s| sim.world.objects.get(s))
            .is_some_and(|o| {
                sim.controlling_player(o.team_id).is_some()
                    && sim.player_is_active(o.team_id) != FALSE
            });
        if !live {
            return None;
        }
    }

    let debris_owner = match source_obj {
        Some(s) => default_team_for_object(sim, s),
        None => {
            let neutral = sim.neutral_team_id();
            sim.player_default_team(neutral)
        }
    };
    let source_id = source_obj
        .and_then(|s| sim.world.objects.get(s))
        .map_or(ObjectId::INVALID, |o| o.id);
    let source_layer = source_obj
        .and_then(|s| sim.world.objects.get(s))
        .map_or(PathfindLayerEnum::Ground, |o| o.layer);

    let debris_template_id: Option<ThingTemplateId> =
        sim.rules.thing_factory.find_template_id("GenericDebris");

    let mut container: Option<ObjectSlot> = None;
    let mut first_object: Option<ObjectSlot> = None;

    if !nugget.put_in_container.is_empty() {
        if let Some(container_tmpl) = sim
            .rules
            .thing_factory
            .find_template_id(&nugget.put_in_container)
        {
            let c = sim.factory_new_object(container_tmpl, debris_owner);
            container = Some(c);
            first_object = Some(c);
            set_producer(sim, c, source_id);
        }
    }

    let name_max_idx = (nugget.names.len() as Int) - 1;
    for _nn in 0..nugget.debris_to_generate {
        let pick = sim.rng_logic_int(0, name_max_idx) as usize;
        let picked_name = &nugget.names[pick];

        let tmpl = if nugget.names_are_objects != FALSE {
            sim.rules.thing_factory.find_template_id(picked_name)
        } else {
            // `TheGameLODManager->isDebrisSkipped()` is always false in headless default LOD.
            debris_template_id
        };
        let Some(tmpl_id) = tmpl else {
            continue;
        };

        let debris = sim.factory_new_object(tmpl_id, debris_owner);
        if first_object.is_none() {
            first_object = Some(debris);
        }
        set_producer(sim, debris, source_id);

        if nugget.preserve_layer != FALSE && source_obj.is_some() && container.is_none() {
            if source_layer != PathfindLayerEnum::Ground {
                sim.set_object_layer(debris, source_layer);
            }
        }

        if let Some(c) = container {
            if sim.is_valid_container_for(c, debris, TRUE) != FALSE {
                sim.contain_add(c, debris);
            }
        }

        let created_pos = if nugget.spread_formation != FALSE {
            let mut fp_options = FindPositionOptions::default();
            fp_options.min_radius = sim.rng_logic_real(
                nugget.min_distance_a_formation,
                nugget.min_distance_b_formation,
            );
            fp_options.max_radius = nugget.max_distance_formation;
            fp_options.flags = FPF_USE_HIGHEST_LAYER;
            sim.find_position_around(pos, &fp_options).unwrap_or(*pos)
        } else {
            *pos
        };

        do_stuff_to_obj(
            sim,
            nugget,
            debris,
            &created_pos,
            mtx,
            orientation,
            source_obj,
            source_id,
            lifetime_frames,
        );
    }

    if let Some(c) = container {
        do_stuff_to_obj(
            sim,
            nugget,
            c,
            pos,
            mtx,
            orientation,
            source_obj,
            source_id,
            lifetime_frames,
        );
    }

    first_object
}

#[allow(clippy::too_many_arguments)]
fn do_stuff_to_obj(
    sim: &mut SimContext,
    nugget: &GenericNugget,
    obj: ObjectSlot,
    pos: &Coord3D,
    mtx: Option<&Matrix3D>,
    mut orientation: Real,
    source_obj: Option<ObjectSlot>,
    source_id: ObjectId,
    lifetime_frames: UnsignedInt,
) {
    set_producer(sim, obj, source_id);

    if lifetime_frames > 0 {
        set_lifetime_range(sim, obj, lifetime_frames, lifetime_frames);
    } else if nugget.max_frames > 0 {
        set_lifetime_range(sim, obj, nugget.min_frames, nugget.max_frames);
    }

    if nugget.names_are_objects == FALSE && sim.object_has_drawable(obj) != FALSE {
        let debris_draw_count = sim
            .world
            .objects
            .get(obj)
            .and_then(|o| {
                o.thing
                    .template_override()
                    .get_final(&sim.rules.thing_factory)
            })
            .map_or(0, |tmpl| {
                tmpl.get_draw_module_info()
                    .nuggets()
                    .iter()
                    .filter(|n| n.name.eq_ignore_ascii_case("W3DDebrisDraw"))
                    .count()
            });
        if debris_draw_count > 0 && !nugget.anim_sets.is_empty() {
            let max_anim_idx = (nugget.anim_sets.len() as Int) - 1;
            for _ in 0..debris_draw_count {
                let _which = sim.rng_logic_int(0, max_anim_idx);
            }
        }
    }

    let mut offset = nugget.offset;
    if let Some(m) = mtx {
        offset = Matrix3D::rotate_vector(m, &offset);
    }
    let mut chunk_pos = Coord3D::new(pos.x + offset.x, pos.y + offset.y, pos.z + offset.z);

    if nugget.ignore_primary_obstacle != FALSE {
        if let Some(p) = physics_update::physics_mut(sim, obj) {
            p.ignore_collisions_with = source_id;
        }
    }

    let health_percent = sim.rng_logic_real(nugget.min_health, nugget.max_health);
    if sim.world.objects.get(obj).is_some_and(|o| o.body_slot.is_some()) {
        sim.set_initial_health(obj, (health_percent * 100.0) as Int);
    }

    if let Some(src) = source_obj {
        sim.slaved_on_enslave(obj, src);
    }

    if nugget.inherits_veterancy != FALSE {
        if let Some(src) = source_obj {
            if sim.object_is_trainable(obj) != FALSE {
                let v = sim.object_veterancy_level(src);
                sim.set_veterancy_level(obj, v, FALSE, FALSE);
                if let Some(obj_id) = sim.world.objects.id_of(obj) {
                    sim.world.transferred_object_names.push((source_id, obj_id));
                }
            }
        }
    }

    if nugget.invulnerable_time > 0 {
        if let Some(rec) = sim.world.objects.get_mut(obj) {
            rec.set_undetected_defector(TRUE);
        }
        sim.trigger_defection(obj, nugget.invulnerable_time, FALSE);
    }

    if (nugget.disposition & DISPOSITION_INHERIT_VELOCITY) != 0 {
        if let Some(src) = source_obj {
            if let Some(src_vel) = physics_update::physics(sim, src).map(|p| p.vel) {
                if physics_update::physics(sim, obj).is_some() {
                    sim.physics_apply_force(obj, &src_vel);
                }
            }
        }
    }

    if (nugget.disposition & DISPOSITION_LIKE_EXISTING) != 0 {
        if let Some(m) = mtx {
            sim.set_object_transform_matrix(obj, m);
        } else {
            sim.set_object_orientation(obj, orientation);
        }
        sim.set_object_position(obj, &chunk_pos);
        if let Some(src) = source_obj {
            if sim.object_is_above_terrain(src) != FALSE {
                sim.set_physics_allow_to_fall(obj, TRUE);
            }
        }
        if sim.object_is_kind_of(obj, KindOfType::Structure) != FALSE {
            flatten_terrain_for_object(sim, obj);
            let mut adjusted_pos = sim
                .world
                .objects
                .get(obj)
                .map_or(chunk_pos, |o| *o.thing.get_position());
            adjusted_pos.z = sim.terrain.get_ground_height(pos.x, pos.y, None);
            sim.set_object_position(obj, &adjusted_pos);
            sim.pathfinder_add_object(obj);
        }
    }

    if (nugget.disposition & DISPOSITION_ON_GROUND_ALIGNED) != 0 {
        chunk_pos.z = 99999.0;
        let layer = sim.get_highest_layer_for_destination(&chunk_pos, FALSE);
        let rand_orient = sim.rng_logic_real(0.0, 2.0 * PI);
        sim.set_object_orientation(obj, rand_orient);
        chunk_pos.z = sim.get_layer_height(chunk_pos.x, chunk_pos.y, layer);
        if layer != PathfindLayerEnum::Ground {
            chunk_pos.z += 1.0;
        }
        sim.set_object_layer(obj, layer);
        sim.set_object_position(obj, &chunk_pos);
    }

    if (nugget.disposition & DISPOSITION_SEND_IT_OUT) != 0 {
        let rand_orient = sim.rng_logic_real(0.0, 2.0 * PI);
        sim.set_object_orientation(obj, rand_orient);
        chunk_pos.z = sim.terrain.get_ground_height(chunk_pos.x, chunk_pos.y, None);
        sim.set_object_position(obj, &chunk_pos);
        if let Some(p) = physics_update::physics_mut(sim, obj) {
            if nugget.names_are_objects == FALSE {
                p.mass = nugget.mass;
            }
            p.extra_friction = nugget.extra_friction;
            let horiz_force = 4.0 * nugget.disposition_intensity;
            let fx = sim.rng_logic_real(-horiz_force, horiz_force);
            let fy = sim.rng_logic_real(-horiz_force, horiz_force);
            let force = Coord3D::new(fx, fy, 0.0);
            sim.physics_apply_force(obj, &force);
            if nugget.orient_in_force_direction != FALSE {
                orientation = gm_atan2f(force.y, force.x);
            }
        }
    }

    if (nugget.disposition
        & (DISPOSITION_SEND_IT_FLYING | DISPOSITION_SEND_IT_UP | DISPOSITION_RANDOM_FORCE))
        != 0
    {
        if let Some(m) = mtx {
            sim.set_object_transform_matrix(obj, m);
        }
        sim.set_object_position(obj, &chunk_pos);
        if let Some(p) = physics_update::physics_mut(sim, obj) {
            if nugget.names_are_objects == FALSE {
                p.mass = nugget.mass;
            }
            p.extra_bounciness = nugget.extra_bounciness;
            p.extra_friction = nugget.extra_friction;
            p.set_flag(physics_update::ALLOW_BOUNCE, TRUE);

            let spin_rate = if nugget.spin_rate >= 0.0 {
                nugget.spin_rate
            } else {
                (PI / 32.0) * nugget.disposition_intensity
            };
            let yaw_rate = if nugget.yaw_rate >= 0.0 {
                nugget.yaw_rate
            } else {
                spin_rate
            };
            let roll_rate = if nugget.roll_rate >= 0.0 {
                nugget.roll_rate
            } else {
                spin_rate
            };
            let pitch_rate = if nugget.pitch_rate >= 0.0 {
                nugget.pitch_rate
            } else {
                spin_rate
            };

            let yaw = sim.rng_logic_real(-yaw_rate, yaw_rate);
            let roll = sim.rng_logic_real(-roll_rate, roll_rate);
            let pitch = sim.rng_logic_real(-pitch_rate, pitch_rate);

            let force = if (nugget.disposition & DISPOSITION_SEND_IT_FLYING) != 0 {
                let horiz_force = 4.0 * nugget.disposition_intensity;
                let vert_force = 3.0 * nugget.disposition_intensity;
                let fx = sim.rng_logic_real(-horiz_force, horiz_force);
                let fy = sim.rng_logic_real(-horiz_force, horiz_force);
                let fz = sim.rng_logic_real(vert_force * 0.33, vert_force);
                Coord3D::new(fx, fy, fz)
            } else if (nugget.disposition & DISPOSITION_SEND_IT_UP) != 0 {
                let horiz_force = 2.0 * nugget.disposition_intensity;
                let vert_force = 4.0 * nugget.disposition_intensity;
                let fx = sim.rng_logic_real(-horiz_force, horiz_force);
                let fy = sim.rng_logic_real(-horiz_force, horiz_force);
                let fz = sim.rng_logic_real(vert_force * 0.75, vert_force);
                Coord3D::new(fx, fy, fz)
            } else {
                calc_random_force(
                    sim,
                    nugget.min_mag,
                    nugget.max_mag,
                    nugget.min_pitch,
                    nugget.max_pitch,
                )
            };

            sim.physics_apply_force(obj, &force);
            if nugget.orient_in_force_direction != FALSE {
                orientation = gm_atan2f(force.y, force.x);
            }
            physics_update::set_angles(sim, obj, orientation, 0.0, 0.0);
            if let Some(p) = physics_update::physics_mut(sim, obj) {
                p.set_yaw_rate(yaw);
                p.set_roll_rate(roll);
                p.set_pitch_rate(pitch);
            }
        }
    }

    if (nugget.disposition & DISPOSITION_WHIRLING) != 0
        && physics_update::physics(sim, obj).is_some()
    {
        let yaw = sim.rng_logic_real(-nugget.disposition_intensity, nugget.disposition_intensity);
        let roll = sim.rng_logic_real(-nugget.disposition_intensity, nugget.disposition_intensity);
        let pitch = sim.rng_logic_real(-nugget.disposition_intensity, nugget.disposition_intensity);
        if let Some(p) = physics_update::physics_mut(sim, obj) {
            p.set_yaw_rate(yaw);
            p.set_roll_rate(roll);
            p.set_pitch_rate(pitch);
        }
    }

    if (nugget.disposition & DISPOSITION_FLOATING) != 0 {
        set_float_enabled(sim, obj, TRUE);
    }

    if nugget.contain_inside_source_object != FALSE {
        let contained = source_obj.is_some_and(|src| {
            sim.world
                .objects
                .get(src)
                .is_some_and(|o| o.contain_slot.is_some())
                && sim.is_valid_container_for(src, obj, TRUE) != FALSE
        });
        if contained {
            if let Some(src) = source_obj {
                sim.contain_add(src, obj);
            }
        } else {
            sim.destroy_object(obj);
        }
    }

    if nugget.dies_on_bad_land != FALSE && sim.world.objects.get(obj).is_some() {
        let (rider_pos, obj_layer) = sim
            .world
            .objects
            .get(obj)
            .map(|o| (*o.thing.get_position(), o.layer))
            .unwrap_or((Coord3D::default(), PathfindLayerEnum::Ground));
        let mut water_z: Real = 0.0;
        let mut terrain_z: Real = 0.0;
        if sim
            .terrain
            .is_underwater(rider_pos.x, rider_pos.y, Some(&mut water_z), Some(&mut terrain_z))
            != FALSE
            && rider_pos.z <= water_z + 10.0
            && obj_layer == PathfindLayerEnum::Ground
        {
            let mut dmg = DamageInfo::default();
            dmg.input.damage_type = DamageType::Water;
            dmg.input.death_type = DeathType::Flooded;
            dmg.input.source_id = ObjectId::INVALID;
            dmg.input.amount = HUGE_DAMAGE_AMOUNT;
            sim.attempt_damage(obj, &mut dmg);
        }

        if let Some(rec) = sim.world.objects.get(obj) {
            let cur_pos = *rec.thing.get_position();
            let cur_layer = rec.layer;
            let off_map = (rec.private_status & PRIVATE_STATUS_OFF_MAP) != 0;
            let cell_x = (cur_pos.x / PATHFIND_CELL_SIZE_F) as Int;
            let cell_y = (cur_pos.y / PATHFIND_CELL_SIZE_F) as Int;
            let cell_type = sim.pathfind_cell_type(cur_layer, cell_x, cell_y);
            if off_map
                || matches!(
                    cell_type,
                    PathfindCellType::Cliff | PathfindCellType::Water | PathfindCellType::Impassable
                )
            {
                kill_object(sim, obj, DamageType::Unresistable, DeathType::Normal);
            }
        }
    }
}

fn find_module_slot_by_name(sim: &SimContext, owner: ObjectSlot, name: &str) -> Option<ModuleSlot> {
    let key = sim.rules.name_keys.lookup_key(name)?;
    sim.world.objects.get(owner).and_then(|obj| {
        obj.behaviors.iter().find_map(|&(ms, _)| {
            let mod_key = sim
                .world
                .modules
                .payload(ms)
                .map(|m| m.module_name_key());
            (mod_key == Some(key)).then_some(ms)
        })
    })
}

fn set_lifetime_range(
    sim: &mut SimContext,
    owner: ObjectSlot,
    min_frames: UnsignedInt,
    max_frames: UnsignedInt,
) {
    let Some(ms) = find_module_slot_by_name(sim, owner, "LifetimeUpdate") else {
        return;
    };
    if !sim.world.modules.is_executing(ms) {
        let _ = sim.dispatch_module(ms, |m, sim_api, ctx| {
            if let Some(lup) = m
                .as_any_mut()
                .and_then(|a| a.downcast_mut::<LifetimeUpdate>())
            {
                lup.set_lifetime_range(sim_api, ctx, min_frames, max_frames);
            }
        });
    }
}

fn set_float_enabled(sim: &mut SimContext, owner: ObjectSlot, enabled: Bool) {
    let Some(ms) = find_module_slot_by_name(sim, owner, "FloatUpdate") else {
        return;
    };
    if !sim.world.modules.is_executing(ms) {
        let _ = sim.dispatch_module(ms, |m, _, _| {
            if let Some(fu) = m.as_any_mut().and_then(|a| a.downcast_mut::<FloatUpdate>()) {
                fu.set_enabled(enabled);
            }
        });
    }
}

fn flatten_terrain_for_object(sim: &mut SimContext, owner: ObjectSlot) {
    let Some(shape) = sim.world.objects.get(owner).map(|o| TerrainShape {
        position: *o.thing.get_position(),
        angle: o.thing.get_orientation(),
        geom_type: o.geometry.get_geom_type(),
        is_small: o.geometry.get_is_small(),
        major_radius: o.geometry.get_major_radius(),
        minor_radius: o.geometry.get_minor_radius(),
    }) else {
        return;
    };
    if let Some(terrain) = sim.terrain.terrain_logic_mut() {
        terrain.flatten_terrain(&shape);
    }
}
