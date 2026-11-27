//! Gate V2 Golden Vector test suite for `PKT-145` (`TurretAI`, `TurretAIData`, `TurretStateMachine`).
//!
//! Verifies all 1,000 C++-generated vectors in `docs/golden_vectors/PKT-145.json`:
//! - Exact 19-entry `TURRET_AI_DATA_FIELD_PARSE` INI table and IEEE-754 float/int field parity
//! - Exact PRNG sequence parity (including `lo >= hi` no-advance guard in idle scan interval/angle)
//! - Exact frame-by-frame state transitions (`IDLE`, `IDLESCAN`, `AIM`, `FIRE`, `RECENTER`, `HOLD`)
//! - Exact turret angle & pitch `f32` bit parity across all 20 simulated ticks per seed
//! - Exact `Snapshot` `XferCRC` and `XferSave` / `XferLoad` round-trip parity

#[cfg(all(feature = "retail-xfer-save", not(feature = "retail-crc")))]
use zh_common::crc::Crc;
#[cfg(all(feature = "retail-xfer-save", not(feature = "retail-crc")))]
use zh_common::golden_json::encode_hex_bytes;
#[cfg(not(feature = "retail-crc"))]
use zh_common::golden_json::{decode_hex_bytes, get_i32_array, get_u32_array};
use zh_common::golden_json::{extract_vector_lines, get_i32, get_str, get_u32};
use zh_common::ini::IniReader;
#[cfg(not(feature = "retail-crc"))]
use zh_common::ini::convert_duration_from_msecs_to_frames;
#[cfg(not(feature = "retail-crc"))]
use zh_common::math::geometry::{GeometryInfo, GeometryType};
#[cfg(not(feature = "retail-crc"))]
use zh_common::math::{gm_acosf, gm_ceilf, gm_cosf, gm_sinf, gm_sqrtf, Coord3D};
#[cfg(not(feature = "retail-crc"))]
use zh_common::random_value::PrngState;
use zh_common::types::{TRUE};
#[cfg(not(feature = "retail-crc"))]
use zh_common::types::{Bool, Int, ObjectId, Real, TeamId, UnsignedInt, FALSE};
#[cfg(not(feature = "retail-crc"))]
use zh_common::xfer::{Xfer, XferCrc, XferLoad, XferSave};
#[cfg(not(feature = "retail-crc"))]
use zh_common::core::state_machine::StateReturnType;
use zh_logic::combat::turret_ai::{TurretAIData, TURRET_AI_DATA_FIELD_PARSE};
#[cfg(not(feature = "retail-crc"))]
use zh_logic::combat::turret_ai::{FireStateStep, TurretAI, TurretWorldAccess, WhichTurretType};
#[cfg(not(feature = "retail-crc"))]
use zh_logic::gamelogic::global_data::WeaponBonus;
#[cfg(not(feature = "retail-crc"))]
use zh_logic::object::weapon::{Weapon, WeaponReloadType, WeaponTemplate, WEAPON_UNDERSIZE};
#[cfg(not(feature = "retail-crc"))]
use zh_logic::object::weapon_set::{CanAttackResult, WeaponSlotType, WEAPONSLOT_COUNT};
#[cfg(not(feature = "retail-crc"))]
use zh_logic::rules::combat_types::WeaponStatus;
#[cfg(not(feature = "retail-crc"))]
use zh_logic::rules::kind_of::KindOfType;

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-145.json");
#[cfg(not(feature = "retail-crc"))]
const ATTACKER_ID: ObjectId = ObjectId(1);
#[cfg(not(feature = "retail-crc"))]
const VICTIM_ID: ObjectId = ObjectId(2);
#[cfg(not(feature = "retail-crc"))]
const TEAM_A: TeamId = TeamId(1);
#[cfg(not(feature = "retail-crc"))]
const TEAM_B: TeamId = TeamId(2);

#[cfg(not(feature = "retail-crc"))]
struct MockTurretWorld {
    frame: UnsignedInt,
    rng: PrngState,
    owner_pos: Coord3D,
    owner_angle: Real,
    owner_geom: GeometryInfo,
    owner_under_construction: Bool,
    owner_turret_rotate: Bool,
    owner_team: TeamId,
    next_mood_check_frame: UnsignedInt,
    #[allow(dead_code)]
    mood_check_rate_frames: UnsignedInt,
    turret_sync: WhichTurretType,
    turrets_linked: Bool,
    weapons: [Weapon; WEAPONSLOT_COUNT],

    victim_alive: bool,
    victim_pos: Coord3D,
    victim_geom: GeometryInfo,
    victim_immobile: Bool,
    victim_team: TeamId,
}

#[cfg(not(feature = "retail-crc"))]
impl MockTurretWorld {
    fn new(
        frame0: UnsignedInt,
        rand_seed: UnsignedInt,
        owner_pos: Coord3D,
        owner_angle: Real,
        victim_pos: Coord3D,
        victim_immobile: Bool,
        atk_range: Real,
        min_range: Real,
        wpn_params: [i32; 4],
        mood_ms: UnsignedInt,
        turrets_linked: Bool,
    ) -> Self {
        let mut rng = PrngState::new();
        rng.init_random_with_seed(rand_seed);

        let mut tmpl = WeaponTemplate::default();
        tmpl.primary_damage = 15.0_f32;
        tmpl.attack_range = atk_range;
        tmpl.minimum_attack_range = min_range;
        tmpl.weapon_speed = 999_999.0_f32;
        tmpl.clip_size = wpn_params[0];
        tmpl.min_delay_between_shots =
            gm_ceilf(convert_duration_from_msecs_to_frames(wpn_params[1] as Real)) as Int;
        tmpl.max_delay_between_shots = tmpl.min_delay_between_shots;
        tmpl.clip_reload_time =
            gm_ceilf(convert_duration_from_msecs_to_frames(wpn_params[2] as Real)) as Int;
        tmpl.pre_attack_delay =
            gm_ceilf(convert_duration_from_msecs_to_frames(wpn_params[3] as Real)) as Int;
        tmpl.reload_type = WeaponReloadType::AutoReload;

        let make_wpn = |slot: WeaponSlotType| {
            let mut w = Weapon::new(tmpl.clone(), slot, frame0);
            w.ammo_in_clip = tmpl.clip_size as UnsignedInt;
            w.status = if w.ammo_in_clip > 0 {
                WeaponStatus::ReadyToFire
            } else {
                WeaponStatus::OutOfAmmo
            };
            w
        };

        let mood_frames = gm_ceilf(convert_duration_from_msecs_to_frames(mood_ms as Real)) as u32;

        Self {
            frame: frame0,
            rng,
            owner_pos,
            owner_angle,
            owner_geom: GeometryInfo::new(GeometryType::Box, FALSE, 20.0_f32, 15.0_f32, 10.0_f32),
            owner_under_construction: FALSE,
            owner_turret_rotate: FALSE,
            owner_team: TEAM_A,
            next_mood_check_frame: frame0,
            mood_check_rate_frames: mood_frames,
            turret_sync: WhichTurretType::Invalid,
            turrets_linked,
            weapons: [
                make_wpn(WeaponSlotType::Primary),
                make_wpn(WeaponSlotType::Secondary),
                make_wpn(WeaponSlotType::Tertiary),
            ],
            victim_alive: true,
            victim_pos,
            victim_geom: GeometryInfo::new(
                GeometryType::Cylinder,
                FALSE,
                14.0_f32,
                12.0_f32,
                12.0_f32,
            ),
            victim_immobile,
            victim_team: TEAM_B,
        }
    }

    fn dist_sqr_boundingspheres_2d(&self, pos_b: &Coord3D, geom_b: Option<&GeometryInfo>) -> Real {
        let dx = pos_b.x - self.owner_pos.x;
        let dy = pos_b.y - self.owner_pos.y;
        let actual_dist_sqr = dx * dx + dy * dy;
        let total_rad = self.owner_geom.get_bounding_circle_radius()
            + geom_b.map_or(0.0_f32, |g| g.get_bounding_circle_radius());
        if total_rad > 0.0_f32 {
            let actual_dist = gm_sqrtf(actual_dist_sqr);
            let shrunken_dist = actual_dist - total_rad;
            if shrunken_dist <= 0.0_f32 {
                0.0_f32
            } else {
                shrunken_dist * shrunken_dist
            }
        } else {
            actual_dist_sqr
        }
    }

    fn weapon_in_range(
        &self,
        slot: WeaponSlotType,
        pos_b: &Coord3D,
        geom_b: Option<&GeometryInfo>,
    ) -> Bool {
        let w = &self.weapons[slot.as_i32() as usize];
        let bonus = WeaponBonus::new();
        let atk = w.template.get_attack_range(&bonus);
        let atk_sqr = atk * atk;
        let mut min_r = w.template.minimum_attack_range - WEAPON_UNDERSIZE;
        if min_r < 0.0_f32 {
            min_r = 0.0_f32;
        }
        let min_sqr = min_r * min_r;
        let dist_sqr = self.dist_sqr_boundingspheres_2d(pos_b, geom_b);
        if dist_sqr < min_sqr {
            return FALSE;
        }
        if dist_sqr <= atk_sqr {
            TRUE
        } else {
            FALSE
        }
    }
}

#[cfg(not(feature = "retail-crc"))]
impl TurretWorldAccess for MockTurretWorld {
    fn current_frame(&self) -> UnsignedInt {
        self.frame
    }

    fn random_int(&mut self, lo: Int, hi: Int) -> Int {
        self.rng.get_game_logic_random_value(lo, hi)
    }
    fn random_real(&mut self, lo: Real, hi: Real) -> Real {
        self.rng.get_game_logic_random_value_real(lo, hi)
    }

    fn object_exists(&self, id: ObjectId) -> bool {
        id == ATTACKER_ID || (id == VICTIM_ID && self.victim_alive)
    }

    fn is_object_effectively_dead(&self, id: ObjectId) -> bool {
        !(id == ATTACKER_ID || (id == VICTIM_ID && self.victim_alive))
    }

    fn get_object_team(&self, id: ObjectId) -> Option<TeamId> {
        if id == ATTACKER_ID {
            Some(self.owner_team)
        } else if id == VICTIM_ID && self.victim_alive {
            Some(self.victim_team)
        } else {
            None
        }
    }

    fn get_object_position(&self, id: ObjectId) -> Option<Coord3D> {
        if id == ATTACKER_ID {
            Some(self.owner_pos)
        } else if id == VICTIM_ID && self.victim_alive {
            Some(self.victim_pos)
        } else {
            None
        }
    }

    fn is_object_kind_of(&self, id: ObjectId, kind: KindOfType) -> Bool {
        if id == VICTIM_ID {
            match kind {
                KindOfType::Immobile => self.victim_immobile,
                _ => FALSE,
            }
        } else {
            FALSE
        }
    }

    fn is_owner_under_construction(&self, _owner: ObjectId) -> Bool {
        self.owner_under_construction
    }

    fn set_model_condition_turret_rotate(&mut self, _owner: ObjectId, set: Bool) {
        self.owner_turret_rotate = set;
    }

    fn react_to_turret_change(
        &mut self,
        _owner: ObjectId,
        _turret: WhichTurretType,
        _old_angle: Real,
        _new_pitch: Real,
    ) {
    }

    fn get_owner_current_weapon_slot(&self, _owner: ObjectId) -> Option<WeaponSlotType> {
        Some(WeaponSlotType::Primary)
    }

    fn get_weapon_attack_range(&self, _owner: ObjectId, slot: WeaponSlotType) -> Real {
        let w = &self.weapons[slot.as_i32() as usize];
        w.template.get_attack_range(&WeaponBonus::new())
    }

    fn is_weapon_in_range_of_object(
        &self,
        _owner: ObjectId,
        slot: WeaponSlotType,
        target: ObjectId,
    ) -> Bool {
        if target != VICTIM_ID || !self.victim_alive {
            return FALSE;
        }
        self.weapon_in_range(slot, &self.victim_pos, Some(&self.victim_geom))
    }

    fn is_weapon_in_range_of_pos(
        &self,
        _owner: ObjectId,
        slot: WeaponSlotType,
        pos: &Coord3D,
    ) -> Bool {
        self.weapon_in_range(slot, pos, None)
    }

    fn are_turrets_linked(&self, _owner: ObjectId) -> Bool {
        self.turrets_linked
    }

    fn get_owner_ai_goal_object(&mut self, _owner: ObjectId) -> ObjectId {
        ObjectId::INVALID
    }

    fn is_owner_able_to_attack(&mut self, _owner: ObjectId) -> Bool {
        TRUE
    }

    fn can_owner_attack_specific_object(
        &mut self,
        _owner: ObjectId,
        target: ObjectId,
        force_attacking: Bool,
    ) -> CanAttackResult {
        if target != VICTIM_ID || !self.victim_alive {
            return CanAttackResult::NotPossible;
        }
        if self.owner_team != self.victim_team || force_attacking == TRUE {
            CanAttackResult::Possible
        } else {
            CanAttackResult::NotPossible
        }
    }

    fn add_targeter(&mut self, _target: ObjectId, _targeter: ObjectId, _add: Bool) {}

    fn is_temporarily_preventing_aim_success(&self, _target: ObjectId) -> Bool {
        FALSE
    }

    fn get_bridge_closest_attack_point(&self, _owner: ObjectId, _bridge: ObjectId) -> Coord3D {
        Coord3D::default()
    }

    fn get_relative_angle_2d(&self, _owner: ObjectId, pos: &Coord3D) -> Real {
        let mut vx = pos.x - self.owner_pos.x;
        let mut vy = pos.y - self.owner_pos.y;
        let dist = gm_sqrtf(vx * vx + vy * vy);
        if dist == 0.0_f32 {
            return 0.0_f32;
        }
        let dir_x = gm_cosf(self.owner_angle);
        let dir_y = gm_sinf(self.owner_angle);
        let dist_inv = 1.0_f32 / dist;
        vx *= dist_inv;
        vy *= dist_inv;

        let mut c = dir_x * vx + dir_y * vy;
        if c < -1.0_f32 {
            c = -1.0_f32;
        } else if c > 1.0_f32 {
            c = 1.0_f32;
        }
        let mut value = gm_acosf(c);
        let perp_z = dir_x * vy - dir_y * vx;
        if perp_z < 0.0_f32 {
            value = -value;
        }
        value
    }

    fn get_vector_to_center_3d(&self, _owner: ObjectId, pos: &Coord3D) -> Coord3D {
        Coord3D::new(
            pos.x - self.owner_pos.x,
            pos.y - self.owner_pos.y,
            pos.z - self.owner_pos.z,
        )
    }

    fn get_owner_max_height_above_position(&self, _owner: ObjectId) -> Real {
        self.owner_geom.get_max_height_above_position()
    }

    fn get_next_idle_mood_target_frame(&mut self, _owner: ObjectId) -> UnsignedInt {
        self.next_mood_check_frame
    }

    fn reset_next_mood_check_time(&mut self, _owner: ObjectId) {
        self.next_mood_check_frame = self.frame.wrapping_add(1);
    }

    fn get_turret_sync(&mut self, _owner: ObjectId) -> WhichTurretType {
        self.turret_sync
    }

    fn set_turret_sync(&mut self, _owner: ObjectId, tur: WhichTurretType) {
        self.turret_sync = tur;
    }

    fn is_idle_mood_ignored(&mut self, _owner: ObjectId) -> Bool {
        FALSE
    }

    fn acquire_idle_mood_target(&mut self, _owner: ObjectId) -> ObjectId {
        ObjectId::INVALID
    }

    fn choose_best_weapon_for_target(&mut self, _owner: ObjectId, _target: ObjectId) {}

    fn out_of_weapon_range_object(&mut self, _owner: ObjectId, victim: ObjectId) -> Bool {
        if victim == VICTIM_ID && self.victim_alive {
            if self.weapon_in_range(WeaponSlotType::Primary, &self.victim_pos, Some(&self.victim_geom))
                == FALSE
            {
                return TRUE;
            }
        }
        FALSE
    }

    fn on_enter_fire_state(&mut self, _owner: ObjectId, _victim: ObjectId) -> StateReturnType {
        let now = self.frame;
        let w = &mut self.weapons[0];
        if now.wrapping_add(1) >= w.when_we_can_fire_again {
            let delay = w.template.get_pre_attack_delay(&WeaponBonus::new());
            if delay > 0 {
                w.status = WeaponStatus::PreAttack;
                w.when_pre_attack_finished = now.wrapping_add(delay as UnsignedInt);
            }
        }
        StateReturnType::Continue
    }

    fn on_update_fire_state(
        &mut self,
        _owner: ObjectId,
        _turret: WhichTurretType,
        is_attacking_object: Bool,
        victim: ObjectId,
        _goal_pos: &Coord3D,
        is_slot_ok_to_fire: Bool,
    ) -> FireStateStep {
        let now = self.frame;
        if is_attacking_object == TRUE
            && (!self.object_exists(victim) || self.is_object_effectively_dead(victim))
        {
            return FireStateStep {
                status: StateReturnType::Failure,
                did_fire: FALSE,
                transfer_victim: None,
            };
        }

        let status = self.weapons[0].get_status(now);
        if status == WeaponStatus::PreAttack {
            return FireStateStep {
                status: StateReturnType::Continue,
                did_fire: FALSE,
                transfer_victim: None,
            };
        } else if status != WeaponStatus::ReadyToFire {
            return FireStateStep {
                status: StateReturnType::Failure,
                did_fire: FALSE,
                transfer_victim: None,
            };
        }

        if is_slot_ok_to_fire == FALSE {
            return FireStateStep {
                status: StateReturnType::Failure,
                did_fire: FALSE,
                transfer_victim: None,
            };
        }

        let bonus = WeaponBonus::new();
        let w = &mut self.weapons[0];
        w.last_fire_frame = now;
        if w.ammo_in_clip > 0 {
            w.ammo_in_clip -= 1;
        }
        if w.ammo_in_clip == 0 {
            w.status = WeaponStatus::ReloadingClip;
            w.when_last_reload_started = now;
            let reload = w.template.get_clip_reload_time(&bonus);
            w.when_we_can_fire_again = now.wrapping_add(reload as UnsignedInt);
            w.ammo_in_clip = w.template.clip_size as UnsignedInt;
        } else {
            w.status = WeaponStatus::BetweenFiringShots;
            let delay = w.template.get_delay_between_shots(&bonus, &mut self.rng);
            w.when_last_reload_started = now;
            w.when_we_can_fire_again = now.wrapping_add(delay as UnsignedInt);
        }

        let transfer_victim = if is_attacking_object == TRUE && victim != ObjectId::INVALID {
            Some(ObjectId::INVALID)
        } else {
            None
        };

        FireStateStep {
            status: StateReturnType::Success,
            did_fire: TRUE,
            transfer_victim,
        }
    }

    fn on_exit_fire_state(&mut self, _owner: ObjectId) {
        let now = self.frame;
        if self.weapons[0].get_status(now) == WeaponStatus::PreAttack {
            self.weapons[0].when_pre_attack_finished = 0;
        }
    }
}

#[test]
fn test_pkt145_oracle_integrity() {
    assert!(VECTORS_JSON.contains("\"packet_id\": \"PKT-145\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"seed_0_diagnostics\""));
    assert!(VECTORS_JSON.contains("TurretAI.h"));
    assert!(VECTORS_JSON.contains("TurretAI.cpp"));
    assert_eq!(TURRET_AI_DATA_FIELD_PARSE.len(), 19);

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000, "PKT-145 oracle must contain 1,000 seeds");
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(get_u32(line, "seed"), Some(idx as u32));
        assert!(get_u32(line, "rcrc_init").is_some());
        assert!(get_u32(line, "xsave_crc").is_some());
        assert!(get_i32(line, "xlen").is_some());
    }

    let s0 = lines[0];
    let xlen = get_i32(s0, "xlen").expect("Seed 0 xlen") as usize;
    assert_eq!(get_str(s0, "xhex").expect("Seed 0 xhex").len(), xlen * 2);
}

#[test]
fn test_pkt145_ini_all_19_fields() {
    let ini_block = "\
      TurretTurnRate = 90\n\
      TurretPitchRate = 45\n\
      NaturalTurretAngle = 15\n\
      NaturalTurretPitch = -5\n\
      FirePitch = 20\n\
      MinPhysicalPitch = -10\n\
      GroundUnitPitch = 12\n\
      TurretFireAngleSweep = SECONDARY 18\n\
      TurretSweepSpeedModifier = TERTIARY 0.75\n\
      ControlledWeaponSlots = PRIMARY TERTIARY\n\
      AllowsPitch = Yes\n\
      InterTurretDelay = 500\n\
      MinIdleScanAngle = 5\n\
      MaxIdleScanAngle = 25\n\
      MinIdleScanInterval = 100\n\
      MaxIdleScanInterval = 400\n\
      RecenterTime = 1500\n\
      InitiallyDisabled = Yes\n\
      FiresWhileTurning = Yes\n\
    End\n";
    let mut td = TurretAIData::new();
    let mut reader = IniReader::new("test.ini", ini_block.as_bytes(), zh_common::ini::IniLoadType::Overwrite);
    td.parse_ini_reader(&mut reader).expect("parse all 19 TurretAIData fields");
    assert_eq!(td.turret_weapon_slots, 0b101);
    assert_eq!(td.inter_turret_delay, 15);
    assert_eq!(td.recenter_time, 45);
    assert_eq!(td.is_allows_pitch, TRUE);
    assert_eq!(td.initially_disabled, TRUE);
    assert_eq!(td.fires_while_turning, TRUE);
    assert_eq!(td.turret_sweep_speed_modifier[2], 0.75_f32);
}

/// Full 1,000-seed oracle parity test against `GeneralsX` (`RETAIL_COMPATIBLE_CRC == 0`).
#[cfg(not(feature = "retail-crc"))]
#[test]
fn test_pkt145_turret_ai_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for (seed, line) in lines.iter().enumerate() {
        let ini_text = get_str(line, "ini").expect("ini").replace("\\n", "\n");
        let mut td = TurretAIData::new();
        td.parse_ini_block(&ini_text).expect("parse TurretAIData");

        let td_bits = get_u32_array::<15>(line, "td_bits").unwrap();
        let got_bits = [
            td.turn_rate.to_bits(),
            td.pitch_rate.to_bits(),
            td.natural_turret_angle.to_bits(),
            td.natural_turret_pitch.to_bits(),
            td.fire_pitch.to_bits(),
            td.min_pitch.to_bits(),
            td.ground_unit_pitch.to_bits(),
            td.turret_fire_angle_sweep[0].to_bits(),
            td.turret_fire_angle_sweep[1].to_bits(),
            td.turret_fire_angle_sweep[2].to_bits(),
            td.turret_sweep_speed_modifier[0].to_bits(),
            td.turret_sweep_speed_modifier[1].to_bits(),
            td.turret_sweep_speed_modifier[2].to_bits(),
            td.min_idle_scan_angle.to_bits(),
            td.max_idle_scan_angle.to_bits(),
        ];
        assert_eq!(got_bits, td_bits, "seed {seed} td_bits mismatch");

        let td_ints = get_u32_array::<7>(line, "td_ints").unwrap();
        let got_ints = [
            td.turret_weapon_slots,
            td.min_idle_scan_interval,
            td.max_idle_scan_interval,
            td.recenter_time,
            td.initially_disabled as u32,
            td.fires_while_turning as u32,
            td.is_allows_pitch as u32,
        ];
        assert_eq!(got_ints, td_ints, "seed {seed} td_ints mismatch");

        let tur = WhichTurretType::from_i32(get_i32(line, "tur").unwrap());
        let mode = get_i32(line, "mode").unwrap();
        let linked = get_u32(line, "linked").unwrap() as Bool;
        let force_atk = get_u32(line, "force").unwrap() as Bool;
        let v_immob = get_u32(line, "v_immob").unwrap() as Bool;
        let atk_rng = f32::from_bits(get_u32(line, "atk_rng").unwrap());
        let min_rng = f32::from_bits(get_u32(line, "min_rng").unwrap());
        let wpn_p = get_i32_array::<4>(line, "wpn_p").unwrap();
        let mood_ms = get_u32(line, "mood_ms").unwrap();
        let rand_seed = get_u32(line, "rand_seed").unwrap();
        let frame0 = get_u32(line, "frame0").unwrap();
        let o_pos_raw = get_u32_array::<4>(line, "o_pos").unwrap();
        let t_pos_raw = get_u32_array::<3>(line, "t_pos").unwrap();
        let v_pos_raw = get_u32_array::<3>(line, "v_pos").unwrap();

        let owner_pos = Coord3D::new(
            f32::from_bits(o_pos_raw[0]),
            f32::from_bits(o_pos_raw[1]),
            f32::from_bits(o_pos_raw[2]),
        );
        let owner_angle = f32::from_bits(o_pos_raw[3]);
        let target_pos = Coord3D::new(
            f32::from_bits(t_pos_raw[0]),
            f32::from_bits(t_pos_raw[1]),
            f32::from_bits(t_pos_raw[2]),
        );
        let victim_pos = Coord3D::new(
            f32::from_bits(v_pos_raw[0]),
            f32::from_bits(v_pos_raw[1]),
            f32::from_bits(v_pos_raw[2]),
        );

        let mut world = MockTurretWorld::new(
            frame0,
            rand_seed,
            owner_pos,
            owner_angle,
            victim_pos,
            v_immob,
            atk_rng,
            min_rng,
            wpn_p,
            mood_ms,
            linked,
        );

        let init_disabled = td.initially_disabled == TRUE;
        let mut turret = TurretAI::new(ATTACKER_ID, td.clone(), tur, &mut world)
            .expect("TurretAI::new");

        assert_eq!(
            world.rng.get_game_logic_random_seed_crc(),
            get_u32(line, "rcrc_init").unwrap(),
            "seed {seed} rcrc_init"
        );
        assert_eq!(
            turret.sm.get_current_state_id() as i32,
            get_i32(line, "init_state").unwrap(),
            "seed {seed} init_state"
        );
        assert_eq!(
            turret.get_turret_angle().to_bits(),
            get_u32(line, "init_angle").unwrap(),
            "seed {seed} init_angle"
        );
        assert_eq!(
            turret.get_turret_pitch().to_bits(),
            get_u32(line, "init_pitch").unwrap(),
            "seed {seed} init_pitch"
        );
        assert_eq!(
            turret.sleep_until,
            get_u32(line, "init_sleep_until").unwrap(),
            "seed {seed} init_sleep_until"
        );

        let mut slot_ok_mask = 0u32;
        for s in 0..WEAPONSLOT_COUNT {
            if turret.is_weapon_slot_on_turret(WeaponSlotType::from_i32(s as Int).unwrap()) == TRUE {
                slot_ok_mask |= 1u32 << s;
            }
        }
        assert_eq!(slot_ok_mask, get_u32(line, "slot_ok_mask").unwrap(), "seed {seed} slot_ok_mask");
        assert_eq!(
            turret.is_turret_in_natural_position(&world) as i32,
            get_i32(line, "init_natural").unwrap(),
            "seed {seed} init_natural"
        );

        let want_states = get_i32_array::<20>(line, "tick_state").unwrap();
        let want_angles = get_u32_array::<20>(line, "tick_angle").unwrap();
        let want_pitches = get_u32_array::<20>(line, "tick_pitch").unwrap();
        let want_sleeps = get_u32_array::<20>(line, "tick_sleep").unwrap();
        let want_rcrcs = get_u32_array::<20>(line, "tick_rcrc").unwrap();
        let want_xhex = get_str(line, "xhex").unwrap();
        let want_xbytes = decode_hex_bytes(want_xhex).unwrap();

        for t in 0..20usize {
            world.frame = frame0 + 1 + (t as UnsignedInt);

            if init_disabled && t == 2 {
                turret.set_turret_enabled(TRUE, &world);
            }
            if mode == 0 {
                if t == 6 {
                    turret.set_turret_target_position(Some(&target_pos), &mut world);
                }
                if t == 13 {
                    turret.set_turret_target_position(None, &mut world);
                }
            } else if mode == 1 {
                if t == 2 {
                    turret.set_turret_target_object(VICTIM_ID, force_atk, &mut world);
                }
                if t == 14 {
                    if seed % 8 == 1 {
                        world.victim_team = TEAM_A;
                    } else {
                        turret.set_turret_target_object(ObjectId::INVALID, FALSE, &mut world);
                    }
                }
            } else if mode == 2 {
                if t == 0 {
                    world.owner_under_construction = TRUE;
                }
                if t == 3 {
                    world.owner_under_construction = FALSE;
                }
                if t == 5 {
                    turret.set_turret_target_position(Some(&target_pos), &mut world);
                }
                if t == 9 {
                    turret.recenter_turret(&mut world);
                }
                if t == 12 {
                    turret.set_turret_enabled(FALSE, &world);
                }
                if t == 15 {
                    turret.set_turret_enabled(TRUE, &world);
                }
            } else if t == 13 {
                turret.set_turret_target_object(VICTIM_ID, FALSE, &mut world);
            }

            if t == 8 && (seed % 3 == 0) {
                turret.notify_fired();
            }

            let slp = turret.update_turret_ai(&mut world);
            assert_eq!(
                turret.sm.get_current_state_id() as i32,
                want_states[t],
                "seed {seed} tick {t} state"
            );
            assert_eq!(
                turret.get_turret_angle().to_bits(),
                want_angles[t],
                "seed {seed} tick {t} angle"
            );
            assert_eq!(
                turret.get_turret_pitch().to_bits(),
                want_pitches[t],
                "seed {seed} tick {t} pitch"
            );
            assert_eq!(slp.0, want_sleeps[t], "seed {seed} tick {t} sleep");
            assert_eq!(
                world.rng.get_game_logic_random_seed_crc(),
                want_rcrcs[t],
                "seed {seed} tick {t} rcrc"
            );

            if t == 10 {
                let mut xcrc = XferCrc::new();
                xcrc.xfer_snapshot(&mut turret).unwrap();
                assert_eq!(xcrc.get_crc(), get_u32(line, "xcrc").unwrap(), "seed {seed} xcrc");

                let mut xsave = XferSave::new();
                xsave.xfer_snapshot(&mut turret).unwrap();
                let sbytes = xsave.as_bytes();

                #[cfg(feature = "retail-xfer-save")]
                {
                    assert_eq!(encode_hex_bytes(sbytes), want_xhex, "seed {seed} xhex");
                    assert_eq!(sbytes.len(), get_i32(line, "xlen").unwrap() as usize, "seed {seed} xlen");
                    let mut c = Crc::new();
                    c.compute_crc(sbytes, sbytes.len() as Int);
                    assert_eq!(c.get(), get_u32(line, "xsave_crc").unwrap(), "seed {seed} xsave_crc");
                }

                #[cfg(not(feature = "retail-xfer-save"))]
                {
                    // Under default (v2 StateMachine) features, verify round-trip AND suffix parity
                    // after TurretAI v2 header + TurretStateMachine v1/v2 block.
                    assert_eq!(sbytes[0], 2);
                    assert_eq!(sbytes[1], 2);
                    assert_eq!(want_xbytes[0], 2);
                    assert_eq!(want_xbytes[1], 1);
                    let suffix_start = sbytes.len() - (want_xbytes.len() - 2);
                    assert_eq!(
                        &sbytes[suffix_start..],
                        &want_xbytes[2..],
                        "seed {seed} TurretAI payload suffix parity"
                    );

                    let mut reloaded_v2 = turret.clone();
                    let mut xload_v2 = XferLoad::new(sbytes);
                    xload_v2.xfer_snapshot(&mut reloaded_v2).unwrap();
                    assert_eq!(reloaded_v2, turret, "seed {seed} v2 save/load round-trip");
                }
            }
        }

        // Verify loading the C++ v1 retail save snapshot into a fresh TurretAI matches post_* fields.
        let mut dummy_rng = PrngState::new();
        dummy_rng.init_random_with_seed(1);
        world.rng = dummy_rng;
        let mut loaded = TurretAI::new(ATTACKER_ID, td, tur, &mut world).unwrap();
        let mut xload_cpp = XferLoad::new(&want_xbytes);
        xload_cpp.xfer_snapshot(&mut loaded).unwrap();
        assert_eq!(
            loaded.sm.get_current_state_id() as i32,
            get_i32(line, "post_state").unwrap(),
            "seed {seed} post_state"
        );
        assert_eq!(
            loaded.get_turret_angle().to_bits(),
            get_u32(line, "post_angle").unwrap(),
            "seed {seed} post_angle"
        );
        assert_eq!(
            loaded.get_turret_pitch().to_bits(),
            get_u32(line, "post_pitch").unwrap(),
            "seed {seed} post_pitch"
        );
    }
}
