//! `PKT-334`: `HelicopterSlowDeathBehavior` (`HelicopterSlowDeathUpdate.h`, `.cpp`), a
//! `SlowDeathBehavior` that spirals, sheds its blades, hits the ground and blows up.
//!
//! Packet: `packets/phase5/PKT-334_HelicopterSlowDeathUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt334_267_382_346_misc.rs`.
//!
//! Client only and not modelled: the ambient and death sounds, the ejected pilot's voice
//! lines and the attached particle system (`AttachParticle*` is parsed and unused).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    damage_info_from_die_op, resolve_optional_asset_name, xfer_module_version,
};
use crate::modules::behavior::slow_death_behavior::{SlowDeathBehaviorModuleData, SlowDeathCore};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, SlowDeathFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType, DisabledType};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, LOGICFRAMES_PER_SECOND, SEPS_DEFAULT};
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, Matrix3D};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, VeterancyLevel, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

const ORBIT_DIRECTION_LEFT: Int = 1;

/// INI template data matching `HelicopterSlowDeathBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct HelicopterSlowDeathBehaviorModuleData {
    pub slow_death: SlowDeathBehaviorModuleData,
    pub spiral_orbit_turn_rate: Real,
    pub spiral_orbit_forward_speed: Real,
    pub spiral_orbit_forward_speed_damping: Real,
    pub min_self_spin: Real,
    pub max_self_spin: Real,
    pub self_spin_update_delay: Real,
    pub self_spin_update_amount: Real,
    pub fall_how_fast: Real,
    pub min_blade_fly_off_delay: Real,
    pub max_blade_fly_off_delay: Real,
    pub attach_particle_system: Option<String>,
    pub attach_particle_bone: String,
    pub attach_particle_loc: Coord3D,
    pub blade_object_name: String,
    pub blade_bone: String,
    pub ocl_eject_pilot: Option<String>,
    pub fx_blade: Option<String>,
    pub ocl_blade: Option<String>,
    pub fx_hit_ground: Option<String>,
    pub ocl_hit_ground: Option<String>,
    pub fx_final_blow_up: Option<String>,
    pub ocl_final_blow_up: Option<String>,
    pub delay_from_ground_to_final_death: Real,
    pub final_rubble_object: String,
    pub death_sound: String,
    pub max_braking: Real,
}

impl HelicopterSlowDeathBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            slow_death: SlowDeathBehaviorModuleData::new(tag_key, module_name_key),
            spiral_orbit_turn_rate: 0.0,
            spiral_orbit_forward_speed: 0.0,
            spiral_orbit_forward_speed_damping: 1.0,
            min_self_spin: 0.0,
            max_self_spin: 0.0,
            self_spin_update_delay: 0.0,
            self_spin_update_amount: 0.0,
            fall_how_fast: 0.0,
            min_blade_fly_off_delay: 0.0,
            max_blade_fly_off_delay: 0.0,
            attach_particle_system: None,
            attach_particle_bone: String::new(),
            attach_particle_loc: Coord3D::default(),
            blade_object_name: String::new(),
            blade_bone: String::new(),
            ocl_eject_pilot: None,
            fx_blade: None,
            ocl_blade: None,
            fx_hit_ground: None,
            ocl_hit_ground: None,
            fx_final_blow_up: None,
            ocl_final_blow_up: None,
            delay_from_ground_to_final_death: 0.0,
            final_rubble_object: String::new(),
            death_sound: String::new(),
            max_braking: 99999.0,
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let fx = parse_ctx.fx_list_names;
        let ocl = parse_ctx.ocl_names;
        let asset_field = [
            (
                "AttachParticle",
                parse_ctx.particle_system_names,
                &mut self.attach_particle_system,
            ),
            ("OCLEjectPilot", ocl, &mut self.ocl_eject_pilot),
            ("FXBlade", fx, &mut self.fx_blade),
            ("OCLBlade", ocl, &mut self.ocl_blade),
            ("FXHitGround", fx, &mut self.fx_hit_ground),
            ("OCLHitGround", ocl, &mut self.ocl_hit_ground),
            ("FXFinalBlowUp", fx, &mut self.fx_final_blow_up),
            ("OCLFinalBlowUp", ocl, &mut self.ocl_final_blow_up),
        ]
        .into_iter()
        .find(|(name, _, _)| field.eq_ignore_ascii_case(name));
        if let Some((_, known, out)) = asset_field {
            return Some(reader.next_token(SEPS_DEFAULT).map(|tok| {
                *out = resolve_optional_asset_name(tok, known);
            }));
        }
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        Some(if is("SpiralOrbitTurnRate") {
            reader.parse_angular_velocity_real(&mut self.spiral_orbit_turn_rate)
        } else if is("SpiralOrbitForwardSpeed") {
            reader.parse_velocity_real(&mut self.spiral_orbit_forward_speed)
        } else if is("SpiralOrbitForwardSpeedDamping") {
            reader.parse_real(&mut self.spiral_orbit_forward_speed_damping)
        } else if is("MinSelfSpin") {
            reader.parse_angular_velocity_real(&mut self.min_self_spin)
        } else if is("MaxSelfSpin") {
            reader.parse_angular_velocity_real(&mut self.max_self_spin)
        } else if is("SelfSpinUpdateDelay") {
            reader.parse_duration_real(&mut self.self_spin_update_delay)
        } else if is("SelfSpinUpdateAmount") {
            reader.parse_angle_real(&mut self.self_spin_update_amount)
        } else if is("FallHowFast") {
            reader.parse_percent_to_real(&mut self.fall_how_fast)
        } else if is("MinBladeFlyOffDelay") {
            reader.parse_duration_real(&mut self.min_blade_fly_off_delay)
        } else if is("MaxBladeFlyOffDelay") {
            reader.parse_duration_real(&mut self.max_blade_fly_off_delay)
        } else if is("AttachParticleBone") {
            reader.parse_ascii_string(&mut self.attach_particle_bone)
        } else if is("AttachParticleLoc") {
            reader.parse_coord3d(&mut self.attach_particle_loc)
        } else if is("BladeObjectName") {
            reader.parse_ascii_string(&mut self.blade_object_name)
        } else if is("BladeBoneName") {
            reader.parse_ascii_string(&mut self.blade_bone)
        } else if is("DelayFromGroundToFinalDeath") {
            reader.parse_duration_real(&mut self.delay_from_ground_to_final_death)
        } else if is("FinalRubbleObject") {
            reader.parse_ascii_string(&mut self.final_rubble_object)
        } else if is("SoundDeathLoop") {
            reader.next_token(SEPS_DEFAULT).map(|tok| {
                self.death_sound = tok.to_owned();
            })
        } else if is("MaxBraking") {
            reader.parse_acceleration_real(&mut self.max_braking)
        } else {
            return self.slow_death.parse_field(field, reader, parse_ctx);
        })
    }
}

impl ModuleData for HelicopterSlowDeathBehaviorModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.slow_death.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.slow_death.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x003 | FacetMask::SLOW_DEATH.0)
    }
}

/// Runtime instance of `HelicopterSlowDeathBehavior` (`PKT-334`).
pub struct HelicopterSlowDeathBehavior {
    pub data: HelicopterSlowDeathBehaviorModuleData,
    pub core: SlowDeathCore,
    /// Spin to the left or right (+1 or -1).
    pub orbit_direction: Int,
    pub forward_angle: Real,
    pub forward_speed: Real,
    pub self_spin: Real,
    pub self_spin_towards_max: Bool,
    pub last_self_spin_update_frame: UnsignedInt,
    /// Frames left until the blade flies off; counts down.
    pub blade_fly_off_frame: UnsignedInt,
    pub hit_ground_frame: UnsignedInt,
}

fn position_of(sim: &dyn SimView, slot: crate::arena::ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map(|o| *o.thing.get_position())
        .unwrap_or_default()
}

fn transform_of(sim: &dyn SimView, slot: crate::arena::ObjectSlot) -> Option<Matrix3D> {
    sim.object(slot).map(|o| *o.thing.get_transform_matrix())
}

impl HelicopterSlowDeathBehavior {
    pub fn new(data: HelicopterSlowDeathBehaviorModuleData) -> Self {
        Self {
            data,
            core: SlowDeathCore::new(),
            orbit_direction: ORBIT_DIRECTION_LEFT,
            forward_angle: 0.0,
            forward_speed: 0.0,
            self_spin: 0.0,
            self_spin_towards_max: FALSE,
            last_self_spin_update_frame: 0,
            blade_fly_off_frame: 0,
            hit_ground_frame: 0,
        }
    }

    /// `SlowDeathBehavior::onDie`.
    fn die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.core.on_die(&self.data.slow_death, sim, ctx, dmg) != FALSE {
            self.begin(sim, ctx, dmg);
        }
    }

    /// `HelicopterSlowDeathBehavior::beginSlowDeath`.
    fn begin(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.core
            .begin_slow_death(&self.data.slow_death, sim, ctx, dmg);
        let us = ctx.owner_slot;

        // A real value stored in an unsigned frame count.
        self.blade_fly_off_frame = sim.rng_logic_real(
            self.data.min_blade_fly_off_delay,
            self.data.max_blade_fly_off_delay,
        ) as UnsignedInt;

        self.orbit_direction = ORBIT_DIRECTION_LEFT;
        self.forward_angle = sim.object(us).map_or(0.0, |o| o.thing.get_orientation());
        self.forward_speed = self.data.spiral_orbit_forward_speed;
        self.self_spin = self.data.min_self_spin;
        self.self_spin_towards_max = TRUE;

        if sim.has_ai(us) == FALSE {
            return;
        }
        let gravity = sim.rules().global_data.gravity;
        sim.set_locomotor_max_lift(us, -gravity * (1.0 - self.data.fall_how_fast));
        sim.set_locomotor_max_braking(us, self.data.max_braking);
    }

    fn update_self_spin(&mut self, sim: &mut dyn SimApi) {
        if self.data.self_spin_update_delay == 0.0
            || (sim.frame().wrapping_sub(self.last_self_spin_update_frame) as Real)
                <= self.data.self_spin_update_delay
        {
            return;
        }
        let step = self.data.self_spin_update_amount / (LOGICFRAMES_PER_SECOND as Real);
        if self.self_spin_towards_max == TRUE {
            self.self_spin += step;
            if self.self_spin > self.data.max_self_spin {
                self.self_spin = self.data.max_self_spin;
                self.self_spin_towards_max = FALSE;
            }
        } else {
            self.self_spin -= step;
            if self.self_spin < self.data.min_self_spin {
                self.self_spin = self.data.min_self_spin;
                self.self_spin_towards_max = TRUE;
            }
        }
        self.last_self_spin_update_frame = sim.frame();
    }

    fn blade_fly_off(&self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let us = ctx.owner_slot;
        let mut blade_pos = position_of(sim, us);
        if sim.object_has_drawable(us) != FALSE {
            if let Some(bone) = sim
                .pristine_bone_transforms(us, &self.data.blade_bone)
                .first()
            {
                blade_pos = Coord3D::new(bone.rows[0][3], bone.rows[1][3], bone.rows[2][3]);
            }
            // Transformed even when the bone was not found, so a missing bone sends the
            // object's own world position through its transform a second time.
            if let Some(mtx) = transform_of(sim, us) {
                blade_pos = Matrix3D::transform_vector(&mtx, &blade_pos);
            }
        }
        if let Some(fx) = &self.data.fx_blade {
            sim.trigger_fx_list(fx, &blade_pos);
        }
        if let Some(ocl) = &self.data.ocl_blade {
            sim.spawn_ocl(ocl, us, &blade_pos, &blade_pos);
        }
        if let Some(ocl) = &self.data.ocl_eject_pilot {
            if sim.object_veterancy_level(us) > VeterancyLevel::Regular {
                let pos = position_of(sim, us);
                sim.spawn_ocl(ocl, us, &pos, &pos);
            }
        }
    }

    fn fx_and_ocl(
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        fx: Option<&String>,
        ocl: Option<&String>,
    ) {
        let pos = position_of(sim, ctx.owner_slot);
        if let Some(fx) = fx {
            sim.trigger_fx_list(fx, &pos);
        }
        if let Some(ocl) = ocl {
            sim.spawn_ocl(ocl, ctx.owner_slot, &pos, &pos);
        }
    }

    /// `HelicopterSlowDeathBehavior::update`.
    fn tick(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let _ = self.core.update_core(&self.data.slow_death, sim, ctx);
        if self.core.is_slow_death_activated() == FALSE {
            return UpdateSleepTime::NONE;
        }
        let us = ctx.owner_slot;

        if self.hit_ground_frame == 0 {
            if let Some(mut xfrm) = transform_of(sim, us) {
                xfrm.in_place_pre_rotate_z(self.self_spin * (self.orbit_direction as Real));
                sim.set_object_transform_matrix(us, &xfrm);
            }

            self.update_self_spin(sim);

            let force = Coord3D::new(
                gm_cosf(self.forward_angle) * self.forward_speed,
                gm_sinf(self.forward_angle) * self.forward_speed,
                0.0,
            );
            sim.physics_apply_motive_force(us, &force);

            self.forward_angle += self.data.spiral_orbit_turn_rate * (self.orbit_direction as Real);
            self.forward_speed *= self.data.spiral_orbit_forward_speed_damping;

            if self.blade_fly_off_frame > 0 {
                self.blade_fly_off_frame -= 1;
                if self.blade_fly_off_frame == 0 {
                    self.blade_fly_off(sim, ctx);
                }
            }
        }

        let mut hit_a_tree = false;
        if self.hit_ground_frame == 0 && sim.has_physics(us) != FALSE {
            if let Some(tree) = sim.find_object(sim.physics_last_collidee(us)) {
                hit_a_tree = sim.object_is_kind_of(tree, KindOfType::Shrubbery) != FALSE;
            }
        }

        if self.hit_ground_frame == 0 {
            let pos = position_of(sim, us);
            let mut top = pos;
            top.z = 99999.0;
            let layer = sim.get_highest_layer_for_destination(&top, FALSE);
            sim.set_object_layer(us, layer);
            let ground = sim.get_layer_height(top.x, top.y, layer);
            let z = position_of(sim, us).z;
            if z <= ground + 1.0 || hit_a_tree {
                self.hit_ground_frame = sim.frame();
                Self::fx_and_ocl(
                    sim,
                    ctx,
                    self.data.fx_hit_ground.as_ref(),
                    self.data.ocl_hit_ground.as_ref(),
                );
                sim.set_disabled(us, DisabledType::Held);
                if sim.object_has_drawable(us) != FALSE {
                    sim.set_model_condition_state(us, ModelConditionFlagType::SpecialDamaged);
                }
            }
        }

        if self.hit_ground_frame != 0
            && (sim.frame().wrapping_sub(self.hit_ground_frame) as Real)
                > self.data.delay_from_ground_to_final_death
        {
            Self::fx_and_ocl(
                sim,
                ctx,
                self.data.fx_final_blow_up.as_ref(),
                self.data.ocl_final_blow_up.as_ref(),
            );
            // C++ throws for an unknown template; no rubble is made here.
            let template = sim
                .rules()
                .thing_factory
                .find_template_id(&self.data.final_rubble_object);
            if let (Some(template), Some(team)) = (template, sim.object(us).map(|o| o.team_id)) {
                let rubble = sim.spawn_object(template, team, Coord3D::default(), 0.0);
                if let Some(mtx) = transform_of(sim, us) {
                    sim.set_object_transform_matrix(rubble, &mtx);
                }
            }
            sim.destroy_object(us);
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for HelicopterSlowDeathBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.slow_death.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.on_object_created(sim, ctx);
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some(dmg) = damage_info_from_die_op(&op) {
            self.die(sim, ctx, &dmg);
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.core.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)?;
        xfer.xfer_int(&mut self.orbit_direction)?;
        xfer.xfer_real(&mut self.forward_angle)?;
        xfer.xfer_real(&mut self.forward_speed)?;
        xfer.xfer_real(&mut self.self_spin)?;
        xfer.xfer_bool(&mut self.self_spin_towards_max)?;
        xfer.xfer_unsigned_int(&mut self.last_self_spin_update_frame)?;
        xfer.xfer_unsigned_int(&mut self.blade_fly_off_frame)?;
        xfer.xfer_unsigned_int(&mut self.hit_ground_frame)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.load_post_process(sim, ctx);
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

    fn as_slow_death(&self) -> Option<&dyn SlowDeathFacet> {
        Some(self)
    }

    fn as_slow_death_mut(&mut self) -> Option<&mut dyn SlowDeathFacet> {
        Some(self)
    }
}

impl DieFacet for HelicopterSlowDeathBehavior {
    fn is_die_applicable(&self, sim: &dyn SimView, ctx: ModuleContext, dmg: &DamageInfo) -> Bool {
        self.core
            .is_die_applicable(&self.data.slow_death, sim, ctx.owner_slot, dmg)
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.die(sim, ctx, dmg);
    }
}

impl SlowDeathFacet for HelicopterSlowDeathBehavior {
    fn is_die_applicable(&self, sim: &dyn SimView, ctx: ModuleContext, dmg: &DamageInfo) -> Bool {
        self.core
            .is_die_applicable(&self.data.slow_death, sim, ctx.owner_slot, dmg)
    }

    fn get_probability_modifier(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Int {
        self.core
            .get_probability_modifier(&self.data.slow_death, sim, ctx.owner_slot, dmg)
    }

    fn begin_slow_death(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.begin(sim, ctx, dmg);
    }

    fn is_slow_death_in_progress(&self) -> Bool {
        self.core.is_slow_death_activated()
    }
}

impl UpdateFacet for HelicopterSlowDeathBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.tick(sim, ctx)
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

pub fn parse_helicopter_slow_death_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = HelicopterSlowDeathBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    if data.slow_death.probability_modifier < 1 {
        return Err(IniError::InvalidData);
    }
    Ok(Box::new(data))
}

pub fn create_helicopter_slow_death_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<HelicopterSlowDeathBehaviorModuleData>()
        .expect("HelicopterSlowDeathBehavior data type mismatch")
        .clone();
    Box::new(HelicopterSlowDeathBehavior::new(typed))
}
