//! Gate V2-Golden test suite for `WP-5.1-B` (`BodyModule` hierarchy:
//! `ActiveBody`, `BodyModule`, `HighlanderBody`, `HiveStructureBody`, `ImmortalBody`,
//! `InactiveBody`, `StructureBody`, `UndeadBody`, 1,000 C++ reference vectors).

use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32,
};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, TeamId, VeterancyLevel, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::body::BodyModule;
use zh_logic::modules::module::{
    BehaviorModule, MaxHealthChangeType, ModuleData, ModuleSnapshotHeader, ModuleType,
    SlowDeathFacet,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::rules::combat_types::{ArmorSetType, DamageType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{SimApi, SimContext, SimView};

const WP51_BODY_JSON: &str = include_str!("../../../docs/golden_vectors/WP51_Body.json");

#[derive(Debug, Clone)]
struct TestSlowDeathData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    prob_mod: Int,
}

impl ModuleData for TestSlowDeathData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::SLOW_DEATH
    }
}

struct TestSlowDeath {
    module_name_key: NameKeyType,
    prob_mod: Int,
}

impl TestSlowDeath {
    fn parse(
        ini: &mut IniReader<'_>,
        tag_key: NameKeyType,
        module_name_key: NameKeyType,
        _ctx: &mut ModuleParseContext<'_>,
    ) -> Result<Box<dyn ModuleData>, IniError> {
        let mut prob_mod = 10;
        ini.read_block_fields(|field, reader| {
            if field.eq_ignore_ascii_case("ProbabilityModifier") {
                reader.parse_int(&mut prob_mod)
            } else if field.eq_ignore_ascii_case("DestructionDelay") {
                let mut tmp = 0u32;
                reader.parse_duration_unsigned_int(&mut tmp)
            } else {
                Err(IniError::UnknownToken)
            }
        })?;
        Ok(Box::new(TestSlowDeathData {
            tag_key,
            module_name_key,
            prob_mod,
        }))
    }

    fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let sdd = data.as_any().downcast_ref::<TestSlowDeathData>().unwrap();
        Box::new(Self {
            module_name_key: data.module_name_key(),
            prob_mod: sdd.prob_mod,
        })
    }
}

impl BehaviorModule for TestSlowDeath {
    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let mut v: XferVersion = 1;
        xfer.xfer_version(&mut v, 1)?;
        ModuleSnapshotHeader::xfer_behavior_module_header(xfer)
    }

    fn as_slow_death(&self) -> Option<&dyn SlowDeathFacet> {
        Some(self)
    }

    fn as_slow_death_mut(&mut self) -> Option<&mut dyn SlowDeathFacet> {
        Some(self)
    }
}

impl SlowDeathFacet for TestSlowDeath {
    fn is_die_applicable(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _dmg: &DamageInfo,
    ) -> Bool {
        TRUE
    }

    fn get_probability_modifier(&self, _dmg: &DamageInfo) -> Int {
        self.prob_mod
    }

    fn begin_slow_death(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &DamageInfo) {}
}

fn setup_shared_rules(sim: &mut SimContext) {
    sim.rules.fx_list_names.insert("FX_BodyHit".to_owned());
    sim.rules.damage_fx_names.insert("BodyHitFX".to_owned());

    let mut dfx_ini = IniReader::new(
        "dfx.ini",
        b"DamageFX BodyHitFX\n  ThrottleTime = Default 200\n  AmountForMajorFX = Default 50\n  MajorFX = Default FX_BodyHit\n  MinorFX = Default FX_BodyHit\nEnd\n",
        IniLoadType::Overwrite,
    );
    sim.rules
        .damage_fx_store
        .load_from_ini(&mut dfx_ini, &mut sim.rules.name_keys, |_| Some(NameKeyType(1)))
        .expect("load DamageFX");

    let mut armor_ini = IniReader::new(
        "armor.ini",
        b"Armor BodyBaseArmor\n  Armor = Default 100%\n  Armor = SMALL_ARMS 50%\n  Armor = EXPLOSION 80%\n  Armor = FLAME 125%\n  Armor = SUBDUAL_MISSILE 100%\nEnd\nArmor BodyVetArmor\n  Armor = Default 50%\n  Armor = SMALL_ARMS 25%\nEnd\n",
        IniLoadType::Overwrite,
    );
    sim.rules
        .armor_store
        .load_from_ini(&mut armor_ini, &mut sim.rules.name_keys)
        .expect("load Armor");

    sim.rules.module_registry.register_module(
        &mut sim.rules.name_keys,
        "SlowDeathBehavior",
        ModuleType::Behavior,
        FacetMask::SLOW_DEATH,
        SleepyUpdatePhase::Normal,
        false,
        TestSlowDeath::parse,
        TestSlowDeath::create,
    );
}

fn body_xfer_bytes(sim: &mut SimContext, owner: ObjectSlot) -> Vec<u8> {
    let bs = sim.object(owner).unwrap().body_slot.unwrap();
    sim.dispatch_module(bs, |m, sim_v, _| {
        let mut xs = XferSave::new();
        m.xfer(&mut xs, sim_v).expect("body xfer save");
        xs.into_bytes()
    })
    .expect("dispatch body xfer")
}

#[test]
fn test_wp51_body_oracle_integrity() {
    assert!(WP51_BODY_JSON.contains("\"packet\": \"WP-5.1-B\""));
    assert!(WP51_BODY_JSON.contains("\"seed_count\": 1000"));
    assert!(WP51_BODY_JSON.contains("\"seed0_diagnostics\":"));
    assert_eq!(extract_vector_lines(WP51_BODY_JSON).len(), 1000);
}

#[test]
fn test_wp51_body_1000_golden_vectors() {
    let lines = extract_vector_lines(WP51_BODY_JSON);
    assert_eq!(lines.len(), 1000);

    let mut sim = SimContext::new();
    setup_shared_rules(&mut sim);

    // Abstract BodyModule contract verification.
    {
        let mut bm = BodyModule::new(NameKeyType(1));
        assert_eq!(
            zh_logic::modules::BodyFacet::get_health(&bm).to_bits(),
            0.0_f32.to_bits()
        );
        zh_logic::modules::BodyFacet::apply_damage_scalar(&mut bm, 0.5);
        assert_eq!(
            zh_logic::modules::BodyFacet::get_damage_scalar(&bm).to_bits(),
            0.5_f32.to_bits()
        );
        let mut xs = XferSave::new();
        bm.xfer(&mut xs, &sim).expect("BodyModule xfer save");
        assert_eq!(xs.as_bytes(), &[1, 1, 1, 1, 0, 0, 0, 63]);
        let mut bm2 = BodyModule::new(NameKeyType(1));
        let mut xl = XferLoad::new(xs.as_bytes());
        bm2.xfer(&mut xl, &sim).expect("BodyModule xfer load");
        assert_eq!(
            zh_logic::modules::BodyFacet::get_damage_scalar(&bm2).to_bits(),
            0.5_f32.to_bits()
        );
    }

    let dmg_types = [
        DamageType::SmallArms,
        DamageType::Explosion,
        DamageType::Flame,
        DamageType::Sniper,
        DamageType::Unresistable,
    ];
    let change_types = [
        MaxHealthChangeType::SameCurrentHealth,
        MaxHealthChangeType::PreserveRatio,
        MaxHealthChangeType::AddCurrentHealthToo,
        MaxHealthChangeType::FullyHeal,
    ];
    let body_states = [
        BodyDamageType::Pristine,
        BodyDamageType::Damaged,
        BodyDamageType::ReallyDamaged,
        BodyDamageType::Rubble,
    ];

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed") as usize;
        assert_eq!(seed, idx);

        let frame0 = get_u32(line, "frame").unwrap();
        let rng_seed = get_u32(line, "rng_seed").unwrap();
        let max_hp = f32::from_bits(get_u32(line, "max_hp").unwrap());
        let init_hp = f32::from_bits(get_u32(line, "init_hp").unwrap());
        let sub_cap = f32::from_bits(get_u32(line, "sub_cap").unwrap());
        let sub_rate_ms = get_u32(line, "sub_rate_ms").unwrap();
        let sub_heal = f32::from_bits(get_u32(line, "sub_heal").unwrap());
        let scalar1 = f32::from_bits(get_u32(line, "scalar1").unwrap());
        let dmg1 = f32::from_bits(get_u32(line, "dmg1").unwrap());
        let dt_idx = get_i32(line, "dt_idx").unwrap() as usize;
        let kill1 = get_i32(line, "kill1").unwrap() != 0;
        let heal1 = f32::from_bits(get_u32(line, "heal1").unwrap());
        let ct_idx = get_i32(line, "ct_idx").unwrap() as usize;
        let new_max_hp = f32::from_bits(get_u32(line, "new_max_hp").unwrap());
        let initial_pct = get_i32(line, "initial_pct").unwrap();
        let ds_idx = get_i32(line, "ds_idx").unwrap() as usize;
        let hero_bonus = f32::from_bits(get_u32(line, "hero_bonus").unwrap());
        let rubble_h = f32::from_bits(get_u32(line, "rubble_h").unwrap());
        let sec_max_hp = f32::from_bits(get_u32(line, "sec_max_hp").unwrap());
        let front_crushed = get_i32(line, "front_crushed").unwrap() != 0;
        let back_crushed = get_i32(line, "back_crushed").unwrap() != 0;
        let start_uc = get_i32(line, "start_uc").unwrap() != 0;

        let want_act0 = get_i32_vec(line, "act0").unwrap();
        let want_act_x0 = decode_hex_bytes(get_str(line, "act_x0").unwrap()).unwrap();
        let want_act1 = get_i32_vec(line, "act1").unwrap();
        let want_act_x1 = decode_hex_bytes(get_str(line, "act_x1").unwrap()).unwrap();
        let want_act2 = get_i32_vec(line, "act2").unwrap();
        let want_act2_x1 = decode_hex_bytes(get_str(line, "act2_x1").unwrap()).unwrap();
        let want_high = get_i32_vec(line, "high").unwrap();
        let want_high_x1 = decode_hex_bytes(get_str(line, "high_x1").unwrap()).unwrap();
        let want_imm = get_i32_vec(line, "imm").unwrap();
        let want_imm_x1 = decode_hex_bytes(get_str(line, "imm_x1").unwrap()).unwrap();
        let want_inac = get_i32_vec(line, "inac").unwrap();
        let want_inac_x1 = decode_hex_bytes(get_str(line, "inac_x1").unwrap()).unwrap();
        let want_str = get_i32_vec(line, "str").unwrap();
        let want_str_x1 = decode_hex_bytes(get_str(line, "str_x1").unwrap()).unwrap();
        let want_hive = get_i32_vec(line, "hive").unwrap();
        let want_hive_x1 = decode_hex_bytes(get_str(line, "hive_x1").unwrap()).unwrap();
        let want_und = get_i32_vec(line, "und").unwrap();
        let want_und_x1 = decode_hex_bytes(get_str(line, "und_x1").unwrap()).unwrap();

        sim.world.frame = frame0;
        sim.world.rng.init_random_with_seed(rng_seed);
        sim.rules.global_data.health_bonus[3] = hero_bonus;

        let t_atk = format!("B51_Atk_{seed}");
        let t_act = format!("B51_Act_{seed}");
        let t_high = format!("B51_High_{seed}");
        let t_imm = format!("B51_Imm_{seed}");
        let t_inac = format!("B51_Inac_{seed}");
        let t_str = format!("B51_Str_{seed}");
        let t_hive = format!("B51_Hive_{seed}");
        let t_und = format!("B51_Und_{seed}");

        let ini_text = format!(
            "Object {t_atk}\n  KindOf = VEHICLE SELECTABLE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\nEnd\n\
Object {t_act}\n  KindOf = SELECTABLE STRUCTURE\n  StructureRubbleHeight = {rubble_i}\n\
  ArmorSet\n    Conditions = None\n    Armor = BodyBaseArmor\n    DamageFX = BodyHitFX\n  End\n\
  ArmorSet\n    Conditions = VETERAN\n    Armor = BodyVetArmor\n    DamageFX = BodyHitFX\n  End\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {max_hp}\n    InitialHealth = {init_hp}\n    SubdualDamageCap = {sub_cap}\n    SubdualDamageHealRate = {sub_rate_ms}\n    SubdualDamageHealAmount = {sub_heal}\n  End\nEnd\n\
Object {t_high}\n  KindOf = SELECTABLE\n  ArmorSet\n    Conditions = None\n    Armor = BodyBaseArmor\n  End\n  Body = HighlanderBody ModuleTag_Body\n    MaxHealth = {max_hp}\n    InitialHealth = {max_hp}\n  End\nEnd\n\
Object {t_imm}\n  KindOf = SELECTABLE\n  Body = ImmortalBody ModuleTag_Body\n    MaxHealth = {max_hp}\n    InitialHealth = {max_hp}\n  End\nEnd\n\
Object {t_inac}\n  KindOf = SELECTABLE\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n\
Object {t_str}\n  KindOf = STRUCTURE SELECTABLE\n  StructureRubbleHeight = {rubble_i}\n  Body = StructureBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\nEnd\n\
Object {t_hive}\n  KindOf = STRUCTURE SELECTABLE\n  Body = HiveStructureBody ModuleTag_Body\n    MaxHealth = 180\n    InitialHealth = 180\n    PropagateDamageTypesToSlavesWhenExisting = NONE +SMALL_ARMS\n    SwallowDamageTypesIfSlavesNotExisting = ALL -UNRESISTABLE\n  End\nEnd\n\
Object {t_und}\n  KindOf = VEHICLE SELECTABLE\n  ArmorSet\n    Conditions = None\n    Armor = BodyBaseArmor\n  End\n  ArmorSet\n    Conditions = SECOND_LIFE\n    Armor = BodyVetArmor\n  End\n  Body = UndeadBody ModuleTag_Body\n    MaxHealth = 120\n    InitialHealth = 120\n    SecondLifeMaxHealth = {sec_max_hp}\n  End\n  Behavior = SlowDeathBehavior ModuleTag_SD\n    ProbabilityModifier = 10\n    DestructionDelay = 1000\n  End\nEnd\n",
            rubble_i = rubble_h as i32
        );
        let mut reader = IniReader::new("body.ini", ini_text.as_bytes(), IniLoadType::Overwrite);
        sim.rules
            .load_thing_templates_from_ini(&mut reader)
            .expect("load body templates");

        let id_of = |name: &str, s: &SimContext| s.rules.thing_factory.find_template_id(name).unwrap();
        let atk = sim.spawn_object(id_of(&t_atk, &sim), TeamId(2), Coord3D::default(), 0.0);
        let atk_id = sim.object(atk).unwrap().id;
        let act = sim.spawn_object(id_of(&t_act, &sim), TeamId(1), Coord3D::default(), 0.0);
        if start_uc {
            sim.set_status(act, ObjectStatusType::UnderConstruction);
        }

        let bs_act = sim.object(act).unwrap().body_slot.unwrap();
        let act_x0 = body_xfer_bytes(&mut sim, act);
        assert_eq!(act_x0, want_act_x0, "seed {seed} act_x0");

        let (hp0, ds0, rate_f, est_snip, est_sub, est_kg) = sim
            .dispatch_module(bs_act, |m, s_api, ctx| {
                let b = m.as_body().unwrap();
                let mut e1 = DamageInfo::default();
                e1.input.damage_type = DamageType::Sniper;
                e1.input.amount = 50.0;
                let mut e2 = DamageInfo::default();
                e2.input.damage_type = DamageType::SubdualMissile;
                e2.input.amount = 30.0;
                let mut e3 = DamageInfo::default();
                e3.input.damage_type = DamageType::KillGarrisoned;
                e3.input.amount = 2.0;
                let r = (
                    b.get_health().to_bits(),
                    b.get_damage_state().as_i32(),
                    b.get_subdual_damage_heal_rate(),
                    b.estimate_damage(s_api, ctx, &e1).to_bits(),
                    b.estimate_damage(s_api, ctx, &e2).to_bits(),
                    b.estimate_damage(s_api, ctx, &e3).to_bits(),
                );
                m.as_body_mut().unwrap().apply_damage_scalar(scalar1);
                r
            })
            .unwrap();
        assert_eq!(hp0, want_act0[0] as u32, "seed {seed} act_hp0");
        assert_eq!(ds0, want_act0[1], "seed {seed} act_ds0");
        assert_eq!(rate_f, want_act0[2] as u32, "seed {seed} act_rate_f");
        assert_eq!(est_snip, want_act0[3] as u32, "seed {seed} est_snip");
        assert_eq!(est_sub, want_act0[4] as u32, "seed {seed} est_sub");
        assert_eq!(est_kg, want_act0[5] as u32, "seed {seed} est_kg");

        let mut hit1 = DamageInfo::default();
        hit1.input.source_id = atk_id;
        hit1.input.damage_type = dmg_types[dt_idx];
        hit1.input.amount = dmg1;
        hit1.input.kill = Bool::from(kill1);
        sim.attempt_damage(act, &mut hit1);

        let rng_crc1 = sim.world.rng.get_game_logic_random_seed_crc();
        let (hp1, prev1, ds1, dmg_ts1, clr1, clr2) = sim
            .dispatch_module(bs_act, |m, _s, _ctx| {
                let b = m.as_body_mut().unwrap();
                let tup = (
                    b.get_health().to_bits(),
                    b.get_previous_health().to_bits(),
                    b.get_damage_state().as_i32(),
                    b.get_last_damage_timestamp().unwrap_or(u32::MAX),
                    b.get_clearable_last_attacker().0,
                );
                b.clear_last_attacker();
                let c2 = b.get_clearable_last_attacker().0;
                b.set_front_crushed(Bool::from(front_crushed));
                b.set_back_crushed(Bool::from(back_crushed));
                b.set_armor_set_flag(ArmorSetType::CrateUpgradeOne);
                (tup.0, tup.1, tup.2, tup.3, tup.4, c2)
            })
            .unwrap();
        let act_obj = sim.object(act).unwrap();
        let dead1 = i32::from(act_obj.is_effectively_dead() != FALSE);
        let gz1 = act_obj.geometry.get_max_height_above_position().to_bits();
        let nocol1 = i32::from(act_obj.status.test(ObjectStatusType::NoCollisions.as_i32()) != FALSE);
        assert_eq!(hit1.output.actual_damage_dealt.to_bits(), want_act1[0] as u32, "seed {seed} act_dealt1");
        assert_eq!(hit1.output.actual_damage_clipped.to_bits(), want_act1[1] as u32, "seed {seed} act_clip1");
        assert_eq!(hp1, want_act1[2] as u32, "seed {seed} act_hp1");
        assert_eq!(prev1, want_act1[3] as u32, "seed {seed} act_prev1");
        assert_eq!(ds1, want_act1[4], "seed {seed} act_ds1");
        assert_eq!(dead1, want_act1[5], "seed {seed} act_dead1");
        assert_eq!(dmg_ts1, want_act1[6] as u32, "seed {seed} act_dmg_ts1");
        assert_eq!(clr1, want_act1[7], "seed {seed} act_clr_atk1");
        assert_eq!(clr2, want_act1[8], "seed {seed} act_clr_atk2");
        assert_eq!(gz1, want_act1[9] as u32, "seed {seed} act_gz1");
        assert_eq!(nocol1, want_act1[10], "seed {seed} act_nocol1");
        assert_eq!(rng_crc1, want_act1[11] as u32, "seed {seed} act_rng_crc1");

        let act_x1 = body_xfer_bytes(&mut sim, act);
        assert_eq!(act_x1, want_act_x1, "seed {seed} act_x1");
        // XferLoad roundtrip verification on first seed cleanly without allocating per-seed dummy
        if seed == 0 {
            sim.dispatch_module(bs_act, |m, sim_v, _| {
                let mut xl = XferLoad::new(&act_x1);
                m.xfer(&mut xl, sim_v).expect("act_x1 load");
                let mut xs = XferSave::new();
                m.xfer(&mut xs, sim_v).expect("act_x1 re-save");
                assert_eq!(xs.as_bytes(), act_x1.as_slice(), "seed 0 act_x1 roundtrip");
            })
            .unwrap();
        }

        // Second fresh ActiveBody for healing / subdual / setMaxHealth / setInitialHealth / setDamageState / veterancy
        let act2 = sim.spawn_object(id_of(&t_act, &sim), TeamId(1), Coord3D::default(), 0.0);
        let bs2 = sim.object(act2).unwrap().body_slot.unwrap();
        let mut pre_dmg = DamageInfo::default();
        pre_dmg.input.source_id = atk_id;
        pre_dmg.input.damage_type = DamageType::SmallArms;
        pre_dmg.input.amount = 80.0;
        sim.attempt_damage(act2, &mut pre_dmg);

        let mut heal_info = DamageInfo::default();
        heal_info.input.damage_type = DamageType::Healing;
        heal_info.input.amount = heal1;
        sim.attempt_healing(act2, &mut heal_info);
        let (heal_hp, heal_ts) = sim
            .dispatch_module(bs2, |m, _, _| {
                let b = m.as_body().unwrap();
                (
                    b.get_health().to_bits(),
                    b.get_last_healing_timestamp().unwrap_or(u32::MAX),
                )
            })
            .unwrap();

        let mut sub_hit = DamageInfo::default();
        sub_hit.input.damage_type = DamageType::SubdualMissile;
        sub_hit.input.amount = sub_cap;
        sim.attempt_damage(act2, &mut sub_hit);
        let sub_is_sub1 = sim
            .dispatch_module(bs2, |m, sim_v, ctx| {
                i32::from(m.as_body().unwrap().is_subdued(sim_v, ctx) != FALSE)
            })
            .unwrap();
        let sub_dis1 = i32::from(
            sim.object(act2).unwrap().is_disabled_by_type(DisabledType::Subdued) != FALSE,
        );

        let mut sub_rec = DamageInfo::default();
        sub_rec.input.damage_type = DamageType::SubdualUnresistable;
        sub_rec.input.amount = -(sub_heal * 20.0);
        sim.attempt_damage(act2, &mut sub_rec);
        let (sub_is_sub2, sub_after) = sim
            .dispatch_module(bs2, |m, sim_v, ctx| {
                let b = m.as_body().unwrap();
                (
                    i32::from(b.is_subdued(sim_v, ctx) != FALSE),
                    b.get_current_subdual_damage().to_bits(),
                )
            })
            .unwrap();

        let (maxhp_h, maxhp_m, init_h, setds_h, setds_s, vet_h, vet_m, vet_hero, ind_unch) = sim
            .dispatch_module(bs2, |m, sim_a, ctx| {
                let b = m.as_body_mut().unwrap();
                b.set_max_health(sim_a, ctx, new_max_hp, change_types[ct_idx]);
                let mh_h = b.get_health().to_bits();
                let mh_m = b.get_max_health().to_bits();
                b.set_initial_health(sim_a, ctx, initial_pct);
                let in_h = b.get_health().to_bits();
                b.set_damage_state(sim_a, ctx, body_states[ds_idx]);
                let sd_h = b.get_health().to_bits();
                let sd_s = b.get_damage_state().as_i32();
                b.internal_change_health(sim_a, ctx, new_max_hp * 0.5);
                b.on_veterancy_level_changed(
                    sim_a,
                    ctx,
                    VeterancyLevel::Regular,
                    VeterancyLevel::Heroic,
                    FALSE,
                );
                let vh = b.get_health().to_bits();
                let vm = b.get_max_health().to_bits();
                let v_hero = i32::from(b.test_armor_set_flag(ArmorSetType::Hero) != FALSE);
                b.set_indestructible(sim_a, ctx, TRUE);
                let before = b.get_health();
                let mut ih = DamageInfo::default();
                ih.input.damage_type = DamageType::Unresistable;
                ih.input.amount = 9999.0;
                b.attempt_damage(sim_a, ctx, &mut ih);
                let iu = i32::from(b.get_health() == before);
                (mh_h, mh_m, in_h, sd_h, sd_s, vh, vm, v_hero, iu)
            })
            .unwrap();

        assert_eq!(heal_info.output.actual_damage_dealt.to_bits(), want_act2[0] as u32, "seed {seed} heal_dealt");
        assert_eq!(heal_info.output.actual_damage_clipped.to_bits(), want_act2[1] as u32, "seed {seed} heal_clip");
        assert_eq!(heal_hp, want_act2[2] as u32, "seed {seed} heal_hp");
        assert_eq!(heal_ts, want_act2[3] as u32, "seed {seed} heal_ts");
        assert_eq!(sub_is_sub1, want_act2[4], "seed {seed} sub1");
        assert_eq!(sub_dis1, want_act2[5], "seed {seed} dis1");
        assert_eq!(sub_is_sub2, want_act2[6], "seed {seed} sub2");
        assert_eq!(sub_after, want_act2[7] as u32, "seed {seed} sub_after");
        assert_eq!(maxhp_h, want_act2[8] as u32, "seed {seed} maxhp_h");
        assert_eq!(maxhp_m, want_act2[9] as u32, "seed {seed} maxhp_m");
        assert_eq!(init_h, want_act2[10] as u32, "seed {seed} init_h");
        assert_eq!(setds_h, want_act2[11] as u32, "seed {seed} setds_h");
        assert_eq!(setds_s, want_act2[12], "seed {seed} setds_s");
        assert_eq!(vet_h, want_act2[13] as u32, "seed {seed} vet_h");
        assert_eq!(vet_m, want_act2[14] as u32, "seed {seed} vet_m");
        assert_eq!(vet_hero, want_act2[15], "seed {seed} vet_hero");
        assert_eq!(ind_unch, want_act2[16], "seed {seed} ind_unch");
        assert_eq!(body_xfer_bytes(&mut sim, act2), want_act2_x1, "seed {seed} act2_x1");

        // HighlanderBody
        let o_high = sim.spawn_object(id_of(&t_high, &sim), TeamId(1), Coord3D::default(), 0.0);
        let bs_high = sim.object(o_high).unwrap().body_slot.unwrap();
        let mut hd1 = DamageInfo::default();
        hd1.input.damage_type = DamageType::Explosion;
        hd1.input.amount = max_hp * 3.0;
        sim.attempt_damage(o_high, &mut hd1);
        let high_hp1 = sim.dispatch_module(bs_high, |m, _, _| m.as_body().unwrap().get_health().to_bits()).unwrap();
        let high_dead1 = i32::from(sim.object(o_high).unwrap().is_effectively_dead() != FALSE);
        assert_eq!(body_xfer_bytes(&mut sim, o_high), want_high_x1, "seed {seed} high_x1");
        let mut hd2 = DamageInfo::default();
        hd2.input.damage_type = DamageType::Unresistable;
        hd2.input.amount = 50.0;
        sim.attempt_damage(o_high, &mut hd2);
        let high_hp2 = sim.dispatch_module(bs_high, |m, _, _| m.as_body().unwrap().get_health().to_bits()).unwrap();
        let high_dead2 = i32::from(sim.object(o_high).unwrap().is_effectively_dead() != FALSE);
        assert_eq!([high_hp1 as i32, high_dead1, high_hp2 as i32, high_dead2], want_high[..], "seed {seed} high");

        // ImmortalBody
        let o_imm = sim.spawn_object(id_of(&t_imm, &sim), TeamId(1), Coord3D::default(), 0.0);
        let bs_imm = sim.object(o_imm).unwrap().body_slot.unwrap();
        sim.dispatch_module(bs_imm, |m, sim_a, ctx| {
            m.as_body_mut().unwrap().internal_change_health(sim_a, ctx, -9999.0);
        }).unwrap();
        let mut im_hit = DamageInfo::default();
        im_hit.input.damage_type = DamageType::Unresistable;
        im_hit.input.amount = 9999.0;
        sim.attempt_damage(o_imm, &mut im_hit);
        let imm_hp = sim.dispatch_module(bs_imm, |m, _, _| m.as_body().unwrap().get_health().to_bits()).unwrap();
        let imm_dead = i32::from(sim.object(o_imm).unwrap().is_effectively_dead() != FALSE);
        assert_eq!([imm_hp as i32, imm_dead], want_imm[..], "seed {seed} imm");
        assert_eq!(body_xfer_bytes(&mut sim, o_imm), want_imm_x1, "seed {seed} imm_x1");

        // InactiveBody
        let o_inac = sim.spawn_object(id_of(&t_inac, &sim), TeamId(1), Coord3D::default(), 0.0);
        let bs_inac = sim.object(o_inac).unwrap().body_slot.unwrap();
        let inac_dead0 = i32::from(sim.object(o_inac).unwrap().is_effectively_dead() != FALSE);
        let inac_hp0 = sim.dispatch_module(bs_inac, |m, _, _| m.as_body().unwrap().get_health().to_bits()).unwrap();
        let mut in_d1 = DamageInfo::default();
        in_d1.input.damage_type = DamageType::Explosion;
        in_d1.input.amount = 100.0;
        sim.attempt_damage(o_inac, &mut in_d1);
        let mut in_d2 = DamageInfo::default();
        in_d2.input.damage_type = DamageType::Unresistable;
        in_d2.input.amount = 1.0;
        sim.attempt_damage(o_inac, &mut in_d2);
        assert_eq!(
            [
                inac_dead0,
                inac_hp0 as i32,
                i32::from(in_d1.output.no_effect != FALSE),
                i32::from(in_d2.output.no_effect != FALSE),
            ],
            want_inac[..],
            "seed {seed} inac"
        );
        assert_eq!(body_xfer_bytes(&mut sim, o_inac), want_inac_x1, "seed {seed} inac_x1");

        // StructureBody
        let o_str = sim.spawn_object(id_of(&t_str, &sim), TeamId(1), Coord3D::default(), 0.0);
        let bs_str = sim.object(o_str).unwrap().body_slot.unwrap();
        sim.dispatch_module(bs_str, |m, _, _| {
            m.as_body_mut().unwrap().set_constructor_object(atk_id);
        }).unwrap();
        let mut sd_hit = DamageInfo::default();
        sd_hit.input.damage_type = DamageType::Unresistable;
        sd_hit.input.amount = 500.0;
        sim.attempt_damage(o_str, &mut sd_hit);
        let str_ds = sim.dispatch_module(bs_str, |m, _, _| m.as_body().unwrap().get_damage_state().as_i32()).unwrap();
        let str_obj = sim.object(o_str).unwrap();
        let str_gz = str_obj.geometry.get_max_height_above_position().to_bits();
        let str_nocol = i32::from(str_obj.status.test(ObjectStatusType::NoCollisions.as_i32()) != FALSE);
        assert_eq!([str_ds, str_gz as i32, str_nocol], want_str[..], "seed {seed} str");
        assert_eq!(body_xfer_bytes(&mut sim, o_str), want_str_x1, "seed {seed} str_x1");

        // HiveStructureBody
        let o_hive = sim.spawn_object(id_of(&t_hive, &sim), TeamId(1), Coord3D::default(), 0.0);
        let bs_hive = sim.object(o_hive).unwrap().body_slot.unwrap();
        let mut hv_hit = DamageInfo::default();
        hv_hit.input.source_id = atk_id;
        hv_hit.input.damage_type = DamageType::SmallArms;
        hv_hit.input.amount = 20.0;
        sim.attempt_damage(o_hive, &mut hv_hit);
        let hive_hp = sim.dispatch_module(bs_hive, |m, _, _| m.as_body().unwrap().get_health().to_bits()).unwrap();
        assert_eq!(hive_hp, want_hive[0] as u32, "seed {seed} hive_hp");
        assert_eq!(body_xfer_bytes(&mut sim, o_hive), want_hive_x1, "seed {seed} hive_x1");

        // UndeadBody
        let o_und = sim.spawn_object(id_of(&t_und, &sim), TeamId(1), Coord3D::default(), 0.0);
        let bs_und = sim.object(o_und).unwrap().body_slot.unwrap();
        let und_rng_pre = sim.world.rng.get_game_logic_random_seed_crc();
        assert_eq!(und_rng_pre, want_und[0] as u32, "seed {seed} und_rng_pre");
        let mut ud1 = DamageInfo::default();
        ud1.input.damage_type = DamageType::SmallArms;
        ud1.input.amount = 300.0;
        sim.attempt_damage(o_und, &mut ud1);
        let (und_hp1, und_max1, und_arm_sec) = sim
            .dispatch_module(bs_und, |m, _, _| {
                let b = m.as_body_mut().unwrap();
                (
                    b.get_health().to_bits(),
                    b.get_max_health().to_bits(),
                    i32::from(b.test_armor_set_flag(ArmorSetType::SecondLife) != FALSE),
                )
            })
            .unwrap();
        assert_eq!(und_hp1, want_und[1] as u32, "seed {seed} und_hp1");
        assert_eq!(und_max1, want_und[2] as u32, "seed {seed} und_max1");
        assert_eq!(und_arm_sec, want_und[3], "seed {seed} und_arm_sec");
        assert_eq!(body_xfer_bytes(&mut sim, o_und), want_und_x1, "seed {seed} und_x1");
        let mut ud2 = DamageInfo::default();
        ud2.input.damage_type = DamageType::SmallArms;
        ud2.input.amount = sec_max_hp * 4.0 + 10.0;
        sim.attempt_damage(o_und, &mut ud2);
        let und_hp2 = sim.dispatch_module(bs_und, |m, _, _| m.as_body().unwrap().get_health().to_bits()).unwrap();
        let und_dead2 = i32::from(sim.object(o_und).unwrap().is_effectively_dead() != FALSE);
        assert_eq!(und_hp2, want_und[4] as u32, "seed {seed} und_hp2");
        assert_eq!(und_dead2, want_und[5], "seed {seed} und_dead2");

        for s in [atk, act, act2, o_high, o_imm, o_inac, o_str, o_hive, o_und] {
            sim.destroy_object_now(s);
        }
        sim.process_destroy_list();
    }
}
