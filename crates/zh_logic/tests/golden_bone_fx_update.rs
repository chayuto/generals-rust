//! `PKT-310 BoneFXUpdate` (with `BoneFXDamage`) against the unmodified
//! reference: `docs/golden_vectors/BoneFXUpdate.json`
//! (`tests/cpp_harness/gen_bone_fx_update.cpp`).

use zh_common::crc::Crc;
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{ObjectId, TeamId, FALSE, TRUE};
use zh_common::xfer::{XferError, XferLoad, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::module::BehaviorModule;
use zh_logic::modules::update::bone_fx_update::{BoneEffect, BoneFXUpdate};
use zh_logic::rules::combat_types::DamageType;
use zh_logic::sim::{SimApi, SimContext, SimWorld};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/BoneFXUpdate.json");

fn lines() -> Vec<String> {
    GOLDEN
        .lines()
        .filter_map(|l| {
            let l = l.trim().trim_end_matches(',').strip_prefix('"')?;
            Some(l.strip_suffix('"')?.replace("\\n", "\n"))
        })
        .collect()
}

fn header(lines: &[String], name: &str) -> String {
    lines
        .iter()
        .find_map(|l| l.strip_prefix(&format!("{name} ")))
        .expect(name)
        .to_owned()
}

fn new_sim(lines: &[String], seed: u32, frame: u32) -> SimContext {
    let mut sim = SimContext::new();
    sim.rules.fx_list_names.insert("FX_B".to_owned());
    sim.rules.ocl_names.insert("OCL_B".to_owned());
    let objects = header(lines, "objects");
    let mut reader = IniReader::new("bone.ini", objects.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("objects");
    sim.rules.thing_factory.post_process_load();
    sim.world = SimWorld::new();
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    sim.world.frame = frame;
    sim.world.next_object_id = 1;
    sim.world.player_default_teams.insert(1, TeamId(1));
    sim.add_player_team(TeamId(1), 1);
    sim.world.rng.init_random_with_seed(seed);
    sim
}

fn module_of(sim: &SimContext, slot: ObjectSlot) -> ModuleSlot {
    let object = sim.world.objects.get(slot).expect("object");
    object
        .behaviors
        .iter()
        .map(|&(m, _)| m)
        .find(|&m| {
            sim.world
                .modules
                .payload(m)
                .and_then(|p| p.as_any())
                .is_some_and(|a| a.is::<BoneFXUpdate>())
        })
        .expect("BoneFXUpdate")
}

fn save(sim: &mut SimContext, module: ModuleSlot) -> Vec<u8> {
    let mut xfer = XferSave::new();
    sim.dispatch_module(module, |m, sim, _| m.xfer(&mut xfer, sim).expect("save"))
        .expect("module");
    xfer.as_bytes().to_vec()
}

fn crc_of(bytes: &[u8]) -> u32 {
    let mut crc = Crc::new();
    crc.compute_crc(bytes, bytes.len() as i32);
    crc.get()
}

/// `body active frames crc len`, as the harness writes it.
fn state_text(sim: &mut SimContext, module: ModuleSlot) -> String {
    let bytes = save(sim, module);
    let fx = sim
        .world
        .modules
        .payload(module)
        .and_then(|p| p.as_any())
        .and_then(|a| a.downcast_ref::<BoneFXUpdate>())
        .expect("BoneFXUpdate");
    let body = fx.cur_body_state as usize;
    let frames: String = fx
        .next_frames
        .iter()
        .flat_map(|kind| kind[body].iter())
        .map(|f| format!("{f},"))
        .collect();
    format!(
        "{body} {} {frames} {} {}",
        u8::from(fx.active != FALSE),
        crc_of(&bytes),
        bytes.len()
    )
}

/// A saved module loads into one that saves the same bytes.
fn check_load(sim: &mut SimContext, module: ModuleSlot) {
    let bytes = save(sim, module);
    let mut copy = sim
        .world
        .modules
        .payload(module)
        .and_then(|p| p.as_any())
        .and_then(|a| a.downcast_ref::<BoneFXUpdate>())
        .expect("BoneFXUpdate")
        .clone();
    copy.next_frames = [[[7; 8]; 4]; 3];
    copy.bone_positions = [[[Coord3D::new(1.0, 2.0, 3.0); 8]; 4]; 3];
    copy.cur_body_state = 3;
    copy.bones_resolved = [TRUE; 4];
    copy.active = TRUE;
    let mut load = XferLoad::new(&bytes);
    zh_logic::modules::module::BehaviorModule::xfer(&mut copy, &mut load, sim).expect("load");
    let mut again = XferSave::new();
    zh_logic::modules::module::BehaviorModule::xfer(&mut copy, &mut again, sim).expect("save");
    assert_eq!(again.as_bytes(), bytes);
}

#[test]
fn bone_fx_update_matches_reference() {
    let lines = lines();
    let mut sim = new_sim(&lines, 1, 1);
    let mut object = None;
    let mut module = None;
    let (mut trials, mut ops, mut marks_start) = (0, 0, 0);
    for line in &lines {
        if let Some(rest) = line.strip_prefix("trial ") {
            let mut words = rest.splitn(4, ' ');
            let mut next = || -> u32 { words.next().expect("word").parse().expect("number") };
            let (variant, seed, frame) = (next(), next(), next());
            sim = new_sim(&lines, seed, frame);
            marks_start = sim.world.spawned_ocls.len();
            let name = format!("G_Bone{variant}");
            let template = sim.rules.thing_factory.find_template_id(&name).expect("t");
            let slot = sim.factory_new_object(template, TeamId(1));
            assert_eq!(sim.world.objects.id_of(slot), Some(ObjectId(1)));
            object = Some(slot);
            module = Some(module_of(&sim, slot));
            assert_eq!(
                state_text(&mut sim, module.expect("module")),
                words.next().expect("state"),
                "{line}"
            );
            trials += 1;
        } else if let Some(rest) = line.strip_prefix("op ") {
            let words: Vec<&str> = rest.split(' ').collect();
            let number = |i: usize| -> u32 { words[i].parse().expect("number") };
            let (code, kind, amount, frame) = (number(0), number(1), number(2), number(3));
            let (slot, module) = (object.expect("object"), module.expect("module"));
            sim.world.frame = frame;
            match code {
                0 => {
                    sim.dispatch_module(module, |m, sim, ctx| {
                        m.as_update_mut().expect("update").update(sim, ctx);
                    })
                    .expect("module");
                }
                1 | 2 => {
                    let mut info = DamageInfo::default();
                    info.input.damage_type = if code == 1 {
                        DamageType::from_i32(kind as i32).expect("type")
                    } else {
                        DamageType::Healing
                    };
                    info.input.amount = amount as f32;
                    if code == 1 {
                        sim.attempt_damage(slot, &mut info);
                    } else {
                        sim.attempt_healing(slot, &mut info);
                    }
                }
                _ => sim.stop_all_bone_fx(slot),
            }
            let got = format!(
                "{} {} {} {}",
                state_text(&mut sim, module),
                sim.world.spawned_ocls.len() - marks_start,
                sim.world.rng.get_game_logic_random_seed_crc(),
                sim.world.rng.get_game_client_random_value(0, 1_000_000),
            );
            assert_eq!(got, words[4..].join(" "), "{line}");
            if ops % 50 == 0 {
                check_load(&mut sim, module);
            }
            ops += 1;
        }
    }
    assert_eq!((trials, ops), (60, 1440));
}

#[test]
fn bone_fx_update_covers_bones_rubble_and_particle_ids() {
    let default_effect = BoneEffect::default();
    assert_eq!(default_effect.delay_low, 0.0);
    assert_eq!(default_effect.delay_high, 0.0);

    let lines = lines();
    let mut sim = new_sim(&lines, 1, 100);
    let custom = "\
Object G_FxCustom
  KindOf = STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = BoneFXDamage ModuleTag_Damage
  End
  Behavior = BoneFXUpdate ModuleTag_Fx
    PristineFXList1 = bone:B1 onlyonce:Yes 0 0 fxlist:FX_B
    RubbleOCL1 = bone:B1 onlyonce:Yes 0 0 ocl:OCL_B
  End
End
";
    let mut reader = IniReader::new("custom.ini", custom.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("custom object");
    sim.rules.thing_factory.post_process_load();

    let mut bone = Matrix3D::identity();
    bone.rows[0][3] = 12.345;
    bone.rows[1][3] = -67.89;
    bone.rows[2][3] = 3.125;
    sim.world
        .pristine_bones
        .insert(("G_FxCustom".to_owned(), "B1".to_owned()), vec![bone]);

    let tmpl = sim
        .rules
        .thing_factory
        .find_template_id("G_FxCustom")
        .expect("G_FxCustom");
    let slot = sim.factory_new_object(tmpl, TeamId(1));
    sim.set_object_position(slot, &Coord3D::new(100.0, 200.0, 0.0));
    let module = module_of(&sim, slot);

    sim.dispatch_module(module, |m, sim, ctx| {
        m.as_update_mut().expect("update").update(sim, ctx);
    })
    .expect("module");
    assert_eq!(
        sim.world.triggered_fx_lists,
        vec![("FX_B".to_owned(), Coord3D::new(112.345, 132.11, 3.125))]
    );
    check_load(&mut sim, module);

    // Transition all the way to Rubble (`cur_body_state == 3`) and fire `RubbleOCL1`.
    sim.dispatch_module(module, |m, sim, ctx| {
        m.as_update_mut()
            .expect("update")
            .change_body_damage_state(sim, ctx, BodyDamageType::Pristine, BodyDamageType::Rubble);
        m.as_update_mut().expect("update").update(sim, ctx);
    })
    .expect("module");
    assert_eq!(sim.world.spawned_ocls.len(), 1);
    assert_eq!(sim.world.spawned_ocls[0].0, "OCL_B");

    // Non-empty `particle_system_ids` roundtrip and non-empty load guard.
    let mut fresh = sim
        .world
        .modules
        .payload(module)
        .and_then(|p| p.as_any())
        .and_then(|a| a.downcast_ref::<BoneFXUpdate>())
        .expect("BoneFXUpdate")
        .clone();
    fresh.particle_system_ids = vec![11, 22];
    let mut saver = XferSave::new();
    fresh.xfer(&mut saver, &sim).expect("save with psys ids");
    let saved = saver.as_bytes().to_vec();

    let mut non_empty = fresh.clone();
    let mut bad_load = XferLoad::new(&saved);
    assert!(matches!(
        non_empty.xfer(&mut bad_load, &sim),
        Err(XferError::InvalidData)
    ));

    let mut empty = fresh;
    empty.particle_system_ids.clear();
    let mut load = XferLoad::new(&saved);
    empty.xfer(&mut load, &sim).expect("load psys ids");
    assert_eq!(empty.particle_system_ids, vec![11, 22]);
    let mut resaved = XferSave::new();
    empty.xfer(&mut resaved, &sim).expect("resave psys ids");
    assert_eq!(resaved.as_bytes(), saved);
}
