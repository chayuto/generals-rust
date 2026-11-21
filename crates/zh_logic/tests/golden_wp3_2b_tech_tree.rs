//! Gate V2 Golden Vector parity tests for `WP-3.2b`
//! (`PKT-133` `Science`, `PKT-132` `RankInfo`, `PKT-137` `Upgrade`, `PKT-134` `SpecialPower`).

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32,
};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::types::{Bool, Int, UnsignedInt, VeterancyLevel, FALSE};
use zh_common::xfer::{Snapshot, Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::rules::{
    AcademyClassificationType, RankInfoStore, SciencePlayerAccess, ScienceStore, ScienceTextLookup,
    ScienceType, SpecialPowerObjectAccess, SpecialPowerStore, SpecialPowerTemplate, Upgrade,
    UpgradeCenter, UpgradeMaskType, UpgradePlayerAccess, UpgradeStatusType, SCIENCE_INVALID,
    UPGRADE_MAX_COUNT,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/WP3_2b_TechTree.json");

struct MockTextLookup;

impl ScienceTextLookup for MockTextLookup {
    fn fetch_label(&mut self, label: &str) -> String {
        if label.is_empty() || label == "EMPTY_LABEL" {
            String::new()
        } else {
            format!("TXT:{label}")
        }
    }
}

struct MockPlayer {
    money: UnsignedInt,
    owned_sciences: Vec<ScienceType>,
}

impl SciencePlayerAccess for MockPlayer {
    fn has_science(&self, st: ScienceType) -> bool {
        self.owned_sciences.contains(&st)
    }
}

impl UpgradePlayerAccess for MockPlayer {
    fn money_count(&self) -> UnsignedInt {
        self.money
    }
}

struct MockObject<'a> {
    player: &'a MockPlayer,
    supported_sp_ids: Vec<UnsignedInt>,
}

impl SpecialPowerObjectAccess for MockObject<'_> {
    fn has_special_power_module(&self, template: &SpecialPowerTemplate) -> bool {
        self.supported_sp_ids.contains(&template.id())
    }

    fn controlling_player_has_science(&self, science: ScienceType) -> bool {
        self.player.has_science(science)
    }
}

#[test]
fn test_wp3_2b_oracle_structure_and_metadata() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    for required in [
        "Science.h",
        "Science.cpp",
        "RankInfo.h",
        "RankInfo.cpp",
        "Upgrade.h",
        "Upgrade.cpp",
        "SpecialPower.h",
        "SpecialPower.cpp",
    ] {
        assert!(VECTORS_JSON.contains(required), "missing sha256 for {required}");
    }
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
fn test_wp3_2b_error_paths_and_boundaries() {
    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let mut sci = ScienceStore::new();
    sci.init();
    let mut ranks = RankInfoStore::new();
    ranks.init();
    let mut sp = SpecialPowerStore::new();
    sp.init();
    let mut text = MockTextLookup;

    // 1. Duplicate Science in Overwrite throws InvalidData.
    let s1 = b"Science SCI_ONE\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\n  DisplayName = SCI:One\n  Description = SCI:OneDesc\nEnd\n";
    let mut ini = IniReader::new("s1.ini", s1, IniLoadType::Overwrite);
    sci.load_from_ini(&mut ini, &mut name_keys, &mut text).unwrap();

    let s_dup = b"Science SCI_ONE\n  SciencePurchasePointCost = 2\nEnd\n";
    let mut ini_dup = IniReader::new("dup.ini", s_dup, IniLoadType::Overwrite);
    assert_eq!(
        sci.load_from_ini(&mut ini_dup, &mut name_keys, &mut text),
        Err(IniError::InvalidData)
    );

    // 2. Unknown PrerequisiteScience fails & still interns the unknown key.
    let s_bad_pre = b"Science SCI_TWO\n  PrerequisiteSciences = SCI_DOES_NOT_EXIST\nEnd\n";
    let mut ini_bp = IniReader::new("bp.ini", s_bad_pre, IniLoadType::Overwrite);
    assert_eq!(
        sci.load_from_ini(&mut ini_bp, &mut name_keys, &mut text),
        Err(IniError::InvalidData)
    );

    // 3. EMPTY_LABEL in DisplayName fails.
    let s_bad_lbl = b"Science SCI_BAD_TXT\n  DisplayName = EMPTY_LABEL\nEnd\n";
    let mut ini_bl = IniReader::new("bl.ini", s_bad_lbl, IniLoadType::Overwrite);
    assert_eq!(
        sci.load_from_ini(&mut ini_bl, &mut name_keys, &mut text),
        Err(IniError::InvalidData)
    );

    // 4. RankInfo monotonic / out-of-range checks.
    let r_bad = b"Rank 2\n  RankName = RANK:Two\n  SkillPointsNeeded = 100\n  SciencesGranted = None\n  SciencePurchasePointsGranted = 1\nEnd\n";
    let mut ini_rb = IniReader::new("rb.ini", r_bad, IniLoadType::Overwrite);
    assert_eq!(
        ranks.load_from_ini(&mut ini_rb, &sci, &mut name_keys, &mut text),
        Err(IniError::InvalidData)
    );

    let r1 = b"Rank 1\n  RankName = RANK:One\n  SkillPointsNeeded = 0\n  SciencesGranted = SCI_ONE\n  SciencePurchasePointsGranted = 1\nEnd\n";
    let mut ini_r1 = IniReader::new("r1.ini", r1, IniLoadType::Overwrite);
    ranks.load_from_ini(&mut ini_r1, &sci, &mut name_keys, &mut text).unwrap();

    for oob in [b"Rank 0\n  SkillPointsNeeded = 5\nEnd\n".as_slice(), b"Rank 2\n  SkillPointsNeeded = 5\nEnd\n".as_slice()] {
        let mut ini_o = IniReader::new("roob.ini", oob, IniLoadType::CreateOverrides);
        assert_eq!(
            ranks.load_from_ini(&mut ini_o, &sci, &mut name_keys, &mut text),
            Err(IniError::InvalidData)
        );
    }
    assert!(ranks.get_rank_info(0).is_none());
    assert!(ranks.get_rank_info(2).is_none());

    // 5. SpecialPower duplicate / unknown science / null checks.
    let sp1 = b"SpecialPower SP_BASE\n  Enum = SPECIAL_DAISY_CUTTER\n  ReloadTime = 1000\n  AcademyClassify = ACT_SUPERPOWER\nEnd\n";
    let mut ini_sp1 = IniReader::new("sp1.ini", sp1, IniLoadType::Overwrite);
    sp.load_from_ini(&mut ini_sp1, &sci, &mut name_keys).unwrap();
    let sp_dup = b"SpecialPower SP_BASE\n  ReloadTime = 2000\nEnd\n";
    let mut ini_spd = IniReader::new("spd.ini", sp_dup, IniLoadType::Overwrite);
    assert_eq!(sp.load_from_ini(&mut ini_spd, &sci, &mut name_keys), Err(IniError::InvalidData));
    let sp_bad_sci = b"SpecialPower SP_BAD\n  RequiredScience = SCI_UNKNOWN\nEnd\n";
    let mut ini_spbs = IniReader::new("spbs.ini", sp_bad_sci, IniLoadType::Overwrite);
    assert_eq!(sp.load_from_ini(&mut ini_spbs, &sci, &mut name_keys), Err(IniError::InvalidData));

    let empty_player = MockPlayer { money: 0, owned_sciences: Vec::new() };
    let empty_obj = MockObject { player: &empty_player, supported_sp_ids: Vec::new() };
    assert_eq!(sp.can_use_special_power(None, sp.get_special_power_template_by_index(0)), FALSE);
    assert_eq!(sp.can_use_special_power(Some(&empty_obj), None), FALSE);

    let upg_center = UpgradeCenter::new();
    let mut ui_count = 0;
    assert_eq!(upg_center.can_afford_upgrade(None, None, true, |_| ui_count += 1), FALSE);
    assert_eq!(upg_center.can_afford_upgrade(Some(&empty_player), None, true, |_| ui_count += 1), FALSE);
    assert_eq!(ui_count, 0);
}

#[test]
fn test_wp3_2b_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    let mut text = MockTextLookup;
    let probe_names = [
        "INVALID",
        "SCI_ROOT_A",
        "SCI_ROOT_B",
        "SCI_MID",
        "SCI_LEAF",
        "SCI_OV_NEW",
    ];

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let money = get_u32(line, "player_money").unwrap();
        let owned_csv = get_str(line, "owned_sciences_csv").unwrap();
        let sp_sup_csv = get_str(line, "sp_supported_csv").unwrap();
        let do_ov = get_u32(line, "do_ov").unwrap() != 0;
        let do_reset = get_u32(line, "do_reset").unwrap() != 0;

        let mut name_keys = NameKeyGenerator::new();
        name_keys.init();
        let mut sci_store = ScienceStore::new();
        sci_store.init();
        let mut rank_store = RankInfoStore::new();
        rank_store.init();
        let mut upg_center = UpgradeCenter::new();
        let mut sp_store = SpecialPowerStore::new();
        sp_store.init();

        // 1. ScienceStore load + optional overrides.
        let sci_base = decode_hex_bytes(get_str(line, "sci_base_hex").unwrap()).unwrap();
        let mut ini_sb = IniReader::new("sci_base.ini", &sci_base, IniLoadType::Overwrite);
        sci_store
            .load_from_ini(&mut ini_sb, &mut name_keys, &mut text)
            .unwrap_or_else(|e| panic!("seed {seed}: sci base load failed: {e:?}"));

        if do_ov {
            let sci_ov = decode_hex_bytes(get_str(line, "sci_ov_hex").unwrap()).unwrap();
            let mut ini_so = IniReader::new("sci_ov.ini", &sci_ov, IniLoadType::CreateOverrides);
            sci_store
                .load_from_ini(&mut ini_so, &mut name_keys, &mut text)
                .unwrap_or_else(|e| panic!("seed {seed}: sci ov load failed: {e:?}"));
        }

        let mut owned_sciences = Vec::new();
        if !owned_csv.is_empty() {
            for name in owned_csv.split(',') {
                owned_sciences.push(sci_store.get_science_from_internal_name(name, &mut name_keys));
            }
        }
        let player = MockPlayer { money, owned_sciences };

        check_science_snapshot(line, seed, "ov", &sci_store, &mut name_keys, &player, &probe_names);

        // 2. RankInfoStore load + optional overrides.
        let rank_base = decode_hex_bytes(get_str(line, "rank_base_hex").unwrap()).unwrap();
        let mut ini_rb = IniReader::new("rank_base.ini", &rank_base, IniLoadType::Overwrite);
        rank_store
            .load_from_ini(&mut ini_rb, &sci_store, &mut name_keys, &mut text)
            .unwrap_or_else(|e| panic!("seed {seed}: rank base load failed: {e:?}"));

        if do_ov {
            let rank_ov = decode_hex_bytes(get_str(line, "rank_ov_hex").unwrap()).unwrap();
            let mut ini_ro = IniReader::new("rank_ov.ini", &rank_ov, IniLoadType::CreateOverrides);
            rank_store
                .load_from_ini(&mut ini_ro, &sci_store, &mut name_keys, &mut text)
                .unwrap_or_else(|e| panic!("seed {seed}: rank ov load failed: {e:?}"));
        }

        assert_eq!(rank_store.get_rank_level_count(), get_i32(line, "rank_cnt_ov").unwrap(), "seed {seed}: rank_cnt_ov");
        assert_eq!(dump_ranks(&rank_store, &sci_store, &name_keys), get_str(line, "rank_dump_ov").unwrap(), "seed {seed}: rank_dump_ov");

        // 3. UpgradeCenter + Upgrade Xfer.
        upg_center.init(&mut name_keys);
        let upg_base = decode_hex_bytes(get_str(line, "upg_base_hex").unwrap()).unwrap();
        let mut ini_ub = IniReader::new("upg_base.ini", &upg_base, IniLoadType::Overwrite);
        upg_center
            .load_from_ini(&mut ini_ub, &mut name_keys)
            .unwrap_or_else(|e| panic!("seed {seed}: upg base load failed: {e:?}"));
        upg_center.reset(true, |img| img == "BTN_FOUND");
        upg_center.reset(true, |img| img == "BTN_FOUND");

        let lifo_names = upg_center.get_upgrade_names();
        assert_eq!(lifo_names.join(","), get_str(line, "upg_names_lifo").unwrap(), "seed {seed}: upg_names_lifo");

        let disp_reason = get_u32(line, "upg_disp_reason").unwrap() != 0;
        let mut ui_msgs: Int = 0;
        let mut upg_rows = Vec::new();
        for uname in &lifo_names {
            let ut = upg_center.find_upgrade(uname, &mut name_keys).unwrap();
            let mask_bit = find_first_set_bit(ut.upgrade_mask());
            let can_aff = upg_center.can_afford_upgrade(Some(&player), Some(ut), disp_reason, |_msg| {
                ui_msgs += 1;
            });
            upg_rows.push(format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                ut.upgrade_name(),
                ut.upgrade_name_key().0,
                ut.upgrade_type().as_i32(),
                ut.display_name_label(),
                ut.calc_time_to_build(Some(&player)),
                ut.calc_cost_to_build(Some(&player)),
                mask_bit,
                ut.research_complete_sound(),
                ut.unit_specific_sound(),
                ut.academy_classification_type().as_i32(),
                can_aff
            ));
        }
        assert_eq!(upg_rows.join(";"), get_str(line, "upg_summary").unwrap(), "seed {seed}: upg_summary");
        assert_eq!(ui_msgs, get_i32(line, "upg_ui_msgs").unwrap(), "seed {seed}: upg_ui_msgs");

        // Upgrade XferSave / XferLoad / XferCRC parity.
        let upg_status = UpgradeStatusType::from_i32(get_i32(line, "upg_status").unwrap()).unwrap();
        let vet_name = upg_center
            .find_veterancy_upgrade(VeterancyLevel::Elite, &mut name_keys)
            .unwrap()
            .upgrade_name()
            .to_owned();
        let mut upg_inst = Upgrade::new(vet_name);
        upg_inst.set_status(upg_status);

        let expected_save = decode_hex_bytes(get_str(line, "upg_xfer_hex").unwrap()).unwrap();
        let expected_crc = zh_common::golden_json::get_hex_u32(line, "upg_crc_hex").unwrap();

        let mut saver = XferSave::new();
        upg_inst.xfer(&mut saver).unwrap();
        assert_eq!(saver.as_bytes(), expected_save.as_slice(), "seed {seed}: upg xfer_save mismatch");

        let mut loaded_upg = Upgrade::default();
        let mut loader = XferLoad::new(&expected_save);
        loaded_upg.xfer(&mut loader).unwrap();
        assert_eq!(loader.position(), expected_save.len());
        assert_eq!(loaded_upg.status(), upg_status);

        let mut xcrc = XferCrc::new();
        xcrc.xfer_snapshot(&mut upg_inst).unwrap();
        assert_eq!(xcrc.get_crc(), expected_crc, "seed {seed}: upg xfer_crc mismatch");

        // 4. SpecialPowerStore load + optional overrides.
        let sp_base = decode_hex_bytes(get_str(line, "sp_base_hex").unwrap()).unwrap();
        let mut ini_spb = IniReader::new("sp_base.ini", &sp_base, IniLoadType::Overwrite);
        sp_store
            .load_from_ini(&mut ini_spb, &sci_store, &mut name_keys)
            .unwrap_or_else(|e| panic!("seed {seed}: sp base load failed: {e:?}"));

        if do_ov {
            let sp_ov = decode_hex_bytes(get_str(line, "sp_ov_hex").unwrap()).unwrap();
            let mut ini_spo = IniReader::new("sp_ov.ini", &sp_ov, IniLoadType::CreateOverrides);
            sp_store
                .load_from_ini(&mut ini_spo, &sci_store, &mut name_keys)
                .unwrap_or_else(|e| panic!("seed {seed}: sp ov load failed: {e:?}"));
        }

        let supported_sp_ids: Vec<UnsignedInt> = if sp_sup_csv.is_empty() {
            Vec::new()
        } else {
            sp_sup_csv
                .split(',')
                .map(|s| s.parse::<UnsignedInt>().unwrap())
                .collect()
        };
        let obj = MockObject {
            player: &player,
            supported_sp_ids,
        };

        assert_eq!(sp_store.get_num_special_powers(), get_i32(line, "sp_cnt_ov").unwrap(), "seed {seed}: sp_cnt_ov");
        assert_eq!(dump_special_powers(&sp_store, &sci_store, &name_keys, &obj), get_str(line, "sp_summary_ov").unwrap(), "seed {seed}: sp_summary_ov");

        // Optional reset().
        if do_reset {
            sci_store.reset();
            rank_store.reset();
            sp_store.reset();
        }

        check_science_snapshot(line, seed, "rst", &sci_store, &mut name_keys, &player, &probe_names);
        assert_eq!(rank_store.get_rank_level_count(), get_i32(line, "rank_cnt_rst").unwrap(), "seed {seed}: rank_cnt_rst");
        assert_eq!(dump_ranks(&rank_store, &sci_store, &name_keys), get_str(line, "rank_dump_rst").unwrap(), "seed {seed}: rank_dump_rst");
        assert_eq!(sp_store.get_num_special_powers(), get_i32(line, "sp_cnt_rst").unwrap(), "seed {seed}: sp_cnt_rst");
        assert_eq!(dump_special_powers(&sp_store, &sci_store, &name_keys, &obj), get_str(line, "sp_summary_rst").unwrap(), "seed {seed}: sp_summary_rst");
    }
}

fn find_first_set_bit(mask: &UpgradeMaskType) -> Int {
    for b in 0..UPGRADE_MAX_COUNT {
        if mask.test(b as Int) != 0 {
            return b as Int;
        }
    }
    -1
}

fn check_science_snapshot(
    line: &str,
    seed: u32,
    suffix: &str,
    store: &ScienceStore,
    name_keys: &mut NameKeyGenerator,
    player: &MockPlayer,
    probes: &[&str],
) {
    let names_key = format!("sci_names_{suffix}");
    let costs_key = format!("sci_costs_{suffix}");
    let grant_key = format!("sci_grant_{suffix}");
    let valid_key = format!("sci_valid_{suffix}");
    let purch_key = format!("sci_purchasable_{suffix}");
    let pot_key = format!("sci_potential_{suffix}");
    let text_key = format!("sci_text_leaf_{suffix}");

    let names = store.friend_get_science_names(name_keys);
    assert_eq!(names.join(","), get_str(line, &names_key).unwrap(), "seed {seed}: {names_key}");

    let mut got_costs = Vec::new();
    let mut got_grant = Vec::new();
    let mut got_valid = Vec::new();
    let mut got_prereq = Vec::new();
    let mut got_roots = Vec::new();
    for &p in probes {
        let st = if p == "INVALID" {
            SCIENCE_INVALID
        } else {
            store.get_science_from_internal_name(p, name_keys)
        };
        got_costs.push(store.get_science_purchase_cost(st));
        got_grant.push(i32::from(store.is_science_grantable(st)));
        got_valid.push(i32::from(store.is_valid_science(st)));
        got_prereq.push(i32::from(store.player_has_prereqs_for_science(player, st)));
        got_roots.push(i32::from(store.player_has_root_prereqs_for_science(player, st)));
    }

    assert_eq!(got_costs, get_i32_vec(line, &costs_key).unwrap(), "seed {seed}: {costs_key}");
    assert_eq!(got_grant, get_i32_vec(line, &grant_key).unwrap(), "seed {seed}: {grant_key}");
    assert_eq!(got_valid, get_i32_vec(line, &valid_key).unwrap(), "seed {seed}: {valid_key}");
    if suffix == "ov" {
        assert_eq!(got_prereq, get_i32_vec(line, "sci_has_prereqs_ov").unwrap(), "seed {seed}: prereqs_ov");
        assert_eq!(got_roots, get_i32_vec(line, "sci_has_roots_ov").unwrap(), "seed {seed}: roots_ov");
    }

    let (purch, pot) = store.get_purchasable_sciences(player);
    let p_csv: Vec<&str> = purch
        .iter()
        .map(|&st| store.get_internal_name_for_science(st, name_keys))
        .collect();
    let pot_csv: Vec<&str> = pot
        .iter()
        .map(|&st| store.get_internal_name_for_science(st, name_keys))
        .collect();
    assert_eq!(p_csv.join(","), get_str(line, &purch_key).unwrap(), "seed {seed}: {purch_key}");
    assert_eq!(pot_csv.join(","), get_str(line, &pot_key).unwrap(), "seed {seed}: {pot_key}");

    let leaf_st = store.get_science_from_internal_name("SCI_LEAF", name_keys);
    let leaf_str = match store.get_name_and_description(leaf_st) {
        Some((dn, desc)) => format!("{dn}|{desc}"),
        None => "NONE".to_owned(),
    };
    assert_eq!(leaf_str, get_str(line, &text_key).unwrap(), "seed {seed}: {text_key}");
}

fn dump_ranks(
    store: &RankInfoStore,
    sci_store: &ScienceStore,
    name_keys: &NameKeyGenerator,
) -> String {
    let cnt = store.get_rank_level_count();
    let mut out = Vec::new();
    for lvl in 1..=cnt {
        let ri = store.get_rank_info(lvl).unwrap();
        let scis: Vec<&str> = ri
            .sciences_granted
            .iter()
            .map(|&st| sci_store.get_internal_name_for_science(st, name_keys))
            .collect();
        out.push(format!(
            "{}:{}:{}:{}:{}",
            lvl,
            ri.rank_name,
            ri.skill_points_needed,
            ri.science_purchase_points_granted,
            scis.join(",")
        ));
    }
    out.join(";")
}

fn dump_special_powers(
    sp_store: &SpecialPowerStore,
    sci_store: &ScienceStore,
    name_keys: &NameKeyGenerator,
    obj: &MockObject<'_>,
) -> String {
    let cnt = sp_store.get_num_special_powers();
    let mut rows = Vec::new();
    for i in 0..cnt {
        let sp = sp_store
            .get_special_power_template_by_index(i as UnsignedInt)
            .unwrap();
        let req = sp.required_science();
        let req_name = if req == SCIENCE_INVALID {
            "INVALID"
        } else {
            sci_store.get_internal_name_for_science(req, name_keys)
        };
        let can_use: Bool = sp_store.can_use_special_power(Some(obj), Some(sp));
        let act: AcademyClassificationType = sp.academy_classification_type();
        let look = sp_store.find_special_power_template(sp.name()).unwrap();
        assert_eq!(look.id(), sp.id());
        assert_eq!(sp_store.find_special_power_template_by_id(sp.id()).unwrap().name(), sp.name());

        rows.push(format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|0x{:08x}|0x{:08x}|{}|{}|{}|{}",
            sp.name(),
            sp.id(),
            sp.special_power_type().as_i32(),
            sp.reload_time(),
            req_name,
            sp.initiate_sound(),
            sp.initiate_at_target_sound(),
            act.as_i32(),
            sp.detection_time(),
            sp.view_object_duration(),
            sp.view_object_range().to_bits(),
            sp.radius_cursor_radius().to_bits(),
            sp.has_public_timer(),
            sp.is_shared_n_sync(),
            sp.is_shortcut_power(),
            can_use
        ));
    }
    rows.join(";")
}
