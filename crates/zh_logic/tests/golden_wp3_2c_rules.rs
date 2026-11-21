//! Gate V2 Golden Vector parity tests for `WP-3.2c`
//! (`PKT-128` `MultiplayerSettings`, `PKT-104` `INIMultiplayer`,
//! `PKT-130` `PlayerTemplate`, `PKT-131` `ProductionPrerequisite`).

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_u32, parse_i64_token,
};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::types::{Int, FALSE, TRUE};
use zh_logic::rules::{
    load_multiplayer_ini, parse_multiplayer_starting_money_choice_definition, MultiplayerSettings,
    PlayerTemplateImageLookup, PlayerTemplateStore, PrereqPlayerView, PrereqTextLookup,
    ProductionPrerequisite, ScienceStore, ScienceTextLookup, ScienceType, ThingTemplateRef,
    ThingTemplateRegistry, GAME_COLOR_UNDEFINED, MAX_MP_STARTING_UNITS, MAX_PREREQ,
    PLAYERTEMPLATE_MIN, PLAYERTEMPLATE_OBSERVER, PLAYERTEMPLATE_RANDOM,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/WP3_2c_Rules.json");

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

struct MockImages;

impl PlayerTemplateImageLookup for MockImages {
    fn has_image(&self, name: &str) -> bool {
        matches!(
            name,
            "BTN_FOUND" | "IMG_HW_1" | "IMG_FW_1" | "IMG_SIDE_1" | "IMG_GEN_1"
        )
    }
}

struct MockThingFactory {
    entries: Vec<(String, Vec<u16>)>,
}

impl MockThingFactory {
    fn new() -> Self {
        let mut entries = Vec::new();
        for i in 0..12 {
            let name = format!("TMPL_{i:02}");
            let disp = format!("DISP_{i:02}");
            entries.push((name, disp.bytes().map(u16::from).collect()));
        }
        Self { entries }
    }
}

impl ThingTemplateRegistry for MockThingFactory {
    fn find_template(&self, name: &str) -> Option<ThingTemplateRef> {
        self.entries
            .iter()
            .position(|(n, _)| n == name)
            .map(ThingTemplateRef)
    }

    fn display_name_utf16(&self, tmpl: ThingTemplateRef) -> &[u16] {
        &self.entries[tmpl.0].1
    }
}

struct MockPlayer {
    owned_sciences: Vec<ScienceType>,
    owned_counts: [Int; 12],
}

impl PrereqPlayerView for MockPlayer {
    fn has_science(&self, st: ScienceType) -> bool {
        self.owned_sciences.contains(&st)
    }

    fn count_objects_by_thing_template(
        &self,
        tmpls: &[Option<ThingTemplateRef>],
        counts: &mut [Int],
    ) {
        for (i, slot) in tmpls.iter().enumerate() {
            counts[i] = match slot {
                Some(t) if t.0 < self.owned_counts.len() => self.owned_counts[t.0],
                _ => 0,
            };
        }
    }
}

struct MockPrereqText;

impl PrereqTextLookup for MockPrereqText {
    fn or_requirement_utf16(&mut self) -> Vec<u16> {
        "OR_REQ".bytes().map(u16::from).collect()
    }

    fn generals_promotion_utf16(&mut self) -> Vec<u16> {
        "GEN_PROMO".bytes().map(u16::from).collect()
    }
}

fn find_json_value_start<'a>(obj: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\":");
    let pos = obj.find(&pat)? + pat.len();
    Some(obj[pos..].trim_start())
}

fn get_u32_vec(obj: &str, key: &str) -> Option<Vec<u32>> {
    let rest = find_json_value_start(obj, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    let end = rest.find(']')?;
    let inner = rest[1..end].trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    for part in inner.split(',') {
        out.push(parse_i64_token(part)? as u32);
    }
    Some(out)
}

fn get_unescaped_json_str(obj: &str, key: &str) -> Option<String> {
    let rest = find_json_value_start(obj, key)?;
    let bytes = rest.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut out = String::new();
    let mut i = 1usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            return Some(out);
        }
        if b == b'\\' {
            i += 1;
            if i >= bytes.len() {
                return None;
            }
            match bytes[i] {
                b'n' => out.push('\n'),
                b'r' => out.push('\r'),
                b't' => out.push('\t'),
                b'\\' => out.push('\\'),
                b'"' => out.push('"'),
                other => out.push(other as char),
            }
        } else {
            out.push(b as char);
        }
        i += 1;
    }
    None
}

fn parse_str_array(rest: &str) -> Option<(Vec<String>, usize)> {
    let bytes = rest.as_bytes();
    if bytes.first() != Some(&b'[') {
        return None;
    }
    let mut i = 1usize;
    let mut out = Vec::new();
    while i < bytes.len() {
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b',') {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        if bytes[i] == b']' {
            return Some((out, i + 1));
        }
        if bytes[i] != b'"' {
            return None;
        }
        i += 1;
        let mut s = String::new();
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'\\' {
                i += 1;
            }
            if i >= bytes.len() {
                return None;
            }
            s.push(bytes[i] as char);
            i += 1;
        }
        i += 1;
        out.push(s);
    }
    None
}

fn get_str_vec(obj: &str, key: &str) -> Option<Vec<String>> {
    let rest = find_json_value_start(obj, key)?;
    parse_str_array(rest).map(|(v, _)| v)
}

fn parse_i32_pairs(obj: &str, key: &str) -> Option<Vec<(i32, i32)>> {
    let rest = find_json_value_start(obj, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    let mut s = &rest[1..];
    let mut pairs = Vec::new();
    loop {
        s = s.trim_start_matches([' ', ',']);
        if s.starts_with(']') {
            return Some(pairs);
        }
        if !s.starts_with('[') {
            return None;
        }
        let end = s.find(']')?;
        let inner = &s[1..end];
        let mut parts = inner.split(',');
        let a = parts.next()?.trim().parse::<i32>().ok()?; // determinism-ok: test JSON parser
        let b = parts.next()?.trim().parse::<i32>().ok()?; // determinism-ok: test JSON parser
        pairs.push((a, b));
        s = &s[end + 1..];
    }
}

fn parse_i32_triples(obj: &str, key: &str) -> Option<Vec<(i32, i32, i32)>> {
    let rest = find_json_value_start(obj, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    let mut s = &rest[1..];
    let mut out = Vec::new();
    loop {
        s = s.trim_start_matches([' ', ',']);
        if s.starts_with(']') {
            return Some(out);
        }
        if !s.starts_with('[') {
            return None;
        }
        let end = s.find(']')?;
        let inner = &s[1..end];
        let mut parts = inner.split(',');
        let a = parts.next()?.trim().parse::<i32>().ok()?; // determinism-ok: test JSON parser
        let b = parts.next()?.trim().parse::<i32>().ok()?; // determinism-ok: test JSON parser
        let c = parts.next()?.trim().parse::<i32>().ok()?; // determinism-ok: test JSON parser
        out.push((a, b, c));
        s = &s[end + 1..];
    }
}

#[derive(Debug, PartialEq)]
struct PtEntryExpected {
    name: String,
    side: String,
    base_side: String,
    money: u32,
    pref_color: i32,
    playable: u8,
    observer: u8,
    old_faction: u8,
    intrinsic_spp: i32,
    hw_hit: u32,
    en_hit: u32,
}

fn parse_pt_entries(obj: &str, key: &str) -> Option<Vec<PtEntryExpected>> {
    let rest = find_json_value_start(obj, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    let mut s = &rest[1..];
    let mut out = Vec::new();
    loop {
        s = s.trim_start_matches([' ', ',']);
        if s.starts_with(']') {
            return Some(out);
        }
        if !s.starts_with('[') {
            return None;
        }
        let end = s.find(']')?;
        let inner = &s[1..end];
        let mut parts = inner.split(',');
        let trim_str = |p: &str| p.trim().trim_matches('"').to_owned();
        let name = trim_str(parts.next()?);
        let side = trim_str(parts.next()?);
        let base_side = trim_str(parts.next()?);
        let money = parts.next()?.trim().parse::<u32>().ok()?; // determinism-ok: test JSON parser
        let pref_color = parts.next()?.trim().parse::<i32>().ok()?; // determinism-ok: test JSON parser
        let playable = parts.next()?.trim().parse::<u8>().ok()?; // determinism-ok: test JSON parser
        let observer = parts.next()?.trim().parse::<u8>().ok()?; // determinism-ok: test JSON parser
        let old_faction = parts.next()?.trim().parse::<u8>().ok()?; // determinism-ok: test JSON parser
        let intrinsic_spp = parts.next()?.trim().parse::<i32>().ok()?; // determinism-ok: test JSON parser
        let hw_hit = parts.next()?.trim().parse::<u32>().ok()?; // determinism-ok: test JSON parser
        let en_hit = parts.next()?.trim().parse::<u32>().ok()?; // determinism-ok: test JSON parser
        out.push(PtEntryExpected {
            name,
            side,
            base_side,
            money,
            pref_color,
            playable,
            observer,
            old_faction,
            intrinsic_spp,
            hw_hit,
            en_hit,
        });
        s = &s[end + 1..];
    }
}

#[test]
fn test_wp3_2c_oracle_structure_and_metadata() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert_eq!(PLAYERTEMPLATE_RANDOM, -1);
    assert_eq!(PLAYERTEMPLATE_OBSERVER, -2);
    assert_eq!(PLAYERTEMPLATE_MIN, -2);
    assert_eq!(GAME_COLOR_UNDEFINED, 0x00FF_FFFF);
    assert_eq!(MAX_MP_STARTING_UNITS, 10);
    assert_eq!(MAX_PREREQ, 32);

    for required in [
        "MultiplayerSettings.h",
        "MultiplayerSettings.cpp",
        "INIMultiplayer.cpp",
        "PlayerTemplate.h",
        "PlayerTemplate.cpp",
        "ProductionPrerequisite.h",
        "ProductionPrerequisite.cpp",
    ] {
        assert!(VECTORS_JSON.contains(required), "missing sha256 for {required}");
    }
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
fn test_wp3_2c_boundaries_and_error_paths() {
    // 1. MultiplayerSettings constructor defaults & boundary queries.
    let mut ms = MultiplayerSettings::new();
    let monies: Vec<u32> = ms
        .starting_money_list()
        .iter()
        .map(|m| m.count_money())
        .collect();
    assert_eq!(monies, vec![100_000, 150_000, 200_000]);
    assert!(ms.get_color(PLAYERTEMPLATE_RANDOM).is_some());
    assert!(ms.get_color(PLAYERTEMPLATE_OBSERVER).is_some());
    assert!(ms.get_color(0).is_none());
    assert!(ms.get_color(-3).is_none());

    let mut bad_money_ini =
        IniReader::new("m.ini", b"Value = 1000\nDefault = Yes\nEnd\n", IniLoadType::CreateOverrides);
    assert_eq!(
        parse_multiplayer_starting_money_choice_definition(&mut ms, &mut bad_money_ini),
        Err(IniError::InvalidData)
    );

    // 2. PlayerTemplate remap + reset() no-op + out-of-range starting_unit.
    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let sci_store = ScienceStore::new();
    let mut pt_store = PlayerTemplateStore::new();
    pt_store.init();
    let mut text = MockTextLookup;
    let pt_src = b"PlayerTemplate FactionAmerica\n  Side = America\n  BaseSide = America\n  PlayableSide = Yes\nEnd\nPlayerTemplate FactionAmericaTankCommand\n  Side = TankGen\nEnd\n";
    let mut ini = IniReader::new("pt.ini", pt_src, IniLoadType::Overwrite);
    pt_store
        .load_from_ini(&mut ini, &sci_store, &mut name_keys, &mut text)
        .unwrap();
    assert_eq!(pt_store.player_template_count(), 1);
    let remapped = pt_store.get_nth_player_template(0).unwrap();
    assert_eq!(remapped.name(&name_keys), "FactionAmericaTankCommand");
    assert_eq!(remapped.side(), "TankGen");
    assert_eq!(remapped.base_side(), "America");
    assert_eq!(remapped.starting_unit(-1), "");
    assert_eq!(remapped.starting_unit(10), "");
    pt_store.reset();
    assert_eq!(pt_store.player_template_count(), 1);
    assert!(pt_store.get_nth_player_template(-1).is_none());
    assert!(pt_store.get_nth_player_template(1).is_none());

    // 3. ProductionPrerequisite unresolved names + nullptr player + MAX_PREREQ=32 cap.
    let mut factory = MockThingFactory::new();
    let mut cap_names = Vec::new();
    for i in 0..36 {
        let nm = format!("CapUnit{i}");
        factory
            .entries
            .push((nm.clone(), nm.bytes().map(u16::from).collect()));
        cap_names.push(nm);
    }

    let mut pp_missing = ProductionPrerequisite::new();
    pp_missing.add_unit_prereq("MissingTemplate", FALSE);
    pp_missing.add_unit_prereq("TMPL_00", TRUE);
    pp_missing.resolve_names(&factory);

    struct CapPlayer {
        owned: Vec<usize>,
    }
    impl PrereqPlayerView for CapPlayer {
        fn has_science(&self, _: ScienceType) -> bool {
            false
        }
        fn count_objects_by_thing_template(
            &self,
            tmpls: &[Option<ThingTemplateRef>],
            counts: &mut [Int],
        ) {
            for (i, slot) in tmpls.iter().enumerate() {
                counts[i] = match slot {
                    Some(t) if self.owned.contains(&t.0) => 1,
                    _ => 0,
                };
            }
        }
    }
    let player_with_0 = CapPlayer { owned: vec![0] };
    assert_eq!(pp_missing.is_satisfied(Some(&player_with_0)), TRUE);
    assert_eq!(pp_missing.is_satisfied(None), FALSE);
    let mut req_text = MockPrereqText;
    assert!(pp_missing
        .get_requires_list(None, &factory, &mut req_text)
        .is_empty());

    let mut pp_cap = ProductionPrerequisite::new();
    for nm in &cap_names {
        pp_cap.add_unit_prereq(nm, FALSE);
    }
    pp_cap.resolve_names(&factory);
    let cap_player = CapPlayer {
        owned: (12..12 + 32).collect(),
    };
    assert_eq!(pp_cap.is_satisfied(Some(&cap_player)), TRUE);
}

#[test]
fn test_wp3_2c_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    let legacy_remaps = [
        "FactionAmericaChooseAGeneral",
        "FactionAmericaTankCommand",
        "FactionAmericaSpecialForces",
        "FactionAmericaAirForce",
        "FactionChinaChooseAGeneral",
        "FactionChinaRedArmy",
        "FactionChinaSpecialWeapons",
        "FactionChinaSecretPolice",
        "FactionGLAChooseAGeneral",
        "FactionGLATerrorCell",
        "FactionGLABiowarCommand",
        "FactionGLAWarlordCommand",
    ];
    let base_factions = ["FactionAmerica", "FactionChina", "FactionGLA"];

    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let images = MockImages;
    let factory = MockThingFactory::new();
    let mut text = MockTextLookup;
    let mut prereq_text = MockPrereqText;

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();

        // --- Part 1: MultiplayerSettings + INIMultiplayer ---
        let mp_ini = get_unescaped_json_str(line, "mp_ini").unwrap();
        let mut settings_opt: Option<MultiplayerSettings> = None;
        let mut mp_reader = IniReader::new("mp.ini", mp_ini.as_bytes(), IniLoadType::Overwrite);
        load_multiplayer_ini(&mut settings_opt, &mut mp_reader).unwrap();
        let mut settings = settings_opt.expect("MultiplayerSettings initialized");

        assert_eq!(
            settings.start_countdown_timer_seconds(),
            get_i32(line, "mp_timer").unwrap(),
            "seed {seed} mp_timer"
        );
        assert_eq!(
            settings.max_beacons_per_player(),
            get_i32(line, "mp_beacons").unwrap(),
            "seed {seed} mp_beacons"
        );
        let flags = get_i32_vec(line, "mp_flags").unwrap();
        assert_eq!(
            [
                settings.is_shroud_in_multiplayer() as i32,
                settings.show_random_player_template() as i32,
                settings.show_random_start_pos() as i32,
                settings.show_random_color() as i32,
            ],
            flags.as_slice(),
            "seed {seed} mp_flags"
        );
        let num_colors = settings.get_num_colors();
        assert_eq!(num_colors, get_i32(line, "mp_num_colors").unwrap());

        let got_money: Vec<u32> = settings
            .starting_money_list()
            .iter()
            .map(|m| m.count_money())
            .collect();
        assert_eq!(got_money, get_u32_vec(line, "mp_money_list").unwrap());
        assert_eq!(
            settings.default_starting_money().count_money(),
            get_u32(line, "mp_default_money").unwrap()
        );

        let exp_colors = parse_i32_pairs(line, "mp_colors").unwrap();
        for (idx_pos, which) in (PLAYERTEMPLATE_OBSERVER..=num_colors).enumerate() {
            let (ec, enc) = exp_colors[idx_pos];
            match settings.get_color(which) {
                None => assert_eq!((ec, enc), (-1, 0), "seed {seed} color[{which}]"),
                Some(cd) => assert_eq!((cd.color(), cd.night_color()), (ec, enc), "seed {seed} color[{which}]"),
            }
        }

        // --- Part 2: PlayerTemplate + PlayerTemplateStore ---
        let mut sci_store = ScienceStore::new();
        sci_store.init();
        let sci_src = b"Science SCI_A\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\nEnd\nScience SCI_B\n  PrerequisiteSciences = SCI_A\n  SciencePurchasePointCost = 2\nEnd\n";
        let mut sci_ini = IniReader::new("sci.ini", sci_src, IniLoadType::Overwrite);
        sci_store
            .load_from_ini(&mut sci_ini, &mut name_keys, &mut text)
            .unwrap();

        let pt_ini = get_unescaped_json_str(line, "pt_ini").unwrap();
        let mut pt_store = PlayerTemplateStore::new();
        pt_store.init();
        let mut pt_reader = IniReader::new("pt.ini", pt_ini.as_bytes(), IniLoadType::Overwrite);
        pt_store
            .load_from_ini(&mut pt_reader, &sci_store, &mut name_keys, &mut text)
            .unwrap();
        pt_store.reset();

        assert_eq!(
            pt_store.player_template_count(),
            get_i32(line, "pt_count").unwrap(),
            "seed {seed} pt_count"
        );

        let mut sides_out = vec!["SeedSide".to_owned()];
        pt_store.get_all_side_strings(&mut sides_out);
        assert_eq!(
            sides_out,
            get_str_vec(line, "pt_sides").unwrap(),
            "seed {seed} pt_sides"
        );

        let remap_q = legacy_remaps[(seed % 12) as usize];
        let remap_key = name_keys.name_to_key(remap_q);
        let remap_name = pt_store
            .find_player_template(remap_key, &name_keys)
            .map(|pt| pt.name(&name_keys))
            .unwrap_or("");
        let exp_remap = get_unescaped_json_str(line, "pt_remap_hit").unwrap();
        assert_eq!(remap_name, exp_remap, "seed {seed} pt_remap_hit");

        let lookup = get_i32_vec(line, "pt_lookup").unwrap();
        assert_eq!(
            pt_store.get_template_num_by_name(base_factions[(seed % 3) as usize], &name_keys),
            lookup[0],
            "seed {seed} lookup hit"
        );
        assert_eq!(
            pt_store.get_template_num_by_name("DOES_NOT_EXIST", &name_keys),
            lookup[1],
            "seed {seed} lookup miss"
        );

        let exp_entries = parse_pt_entries(line, "pt_entries").unwrap();
        for (i, exp) in exp_entries.iter().enumerate() {
            let pt = pt_store.get_nth_player_template(i as Int).unwrap();
            let got = PtEntryExpected {
                name: pt.name(&name_keys).to_owned(),
                side: pt.side().to_owned(),
                base_side: pt.base_side().to_owned(),
                money: pt.money().count_money(),
                pref_color: pt.preferred_color().get_as_int(),
                playable: pt.is_playable_side(),
                observer: pt.is_observer(),
                old_faction: pt.is_old_faction(),
                intrinsic_spp: pt.intrinsic_science_purchase_points(),
                hw_hit: u32::from(pt.has_head_water_mark_image(&images)),
                en_hit: u32::from(pt.has_enabled_image(&images)),
            };
            assert_eq!(got, *exp, "seed {seed} pt_entries[{i}]");
        }

        let pt0 = pt_store.get_nth_player_template(0).unwrap();
        let pt0_maps = get_u32_vec(line, "pt0_maps").unwrap();
        assert_eq!(
            vec![
                pt0.production_cost_changes().len() as u32,
                pt0.production_time_changes().len() as u32,
                pt0.production_veterancy_levels().len() as u32,
                pt0.intrinsic_sciences().len() as u32,
            ],
            pt0_maps,
            "seed {seed} pt0_maps"
        );
        let mut cost_sum = 0u32;
        for (&k, &v) in pt0.production_cost_changes() {
            cost_sum ^= (k.0 as u32) ^ v.to_bits();
        }
        let mut vet_sum = 0u32;
        for (&k, &v) in pt0.production_veterancy_levels() {
            vet_sum ^= (k.0 as u32) ^ (v.as_i32() as u32);
        }
        assert_eq!(
            vec![cost_sum, vet_sum],
            get_u32_vec(line, "pt0_hashes").unwrap(),
            "seed {seed} pt0_hashes"
        );

        // --- Part 3: ProductionPrerequisite ---
        let sci_req = get_i32(line, "pp_sci_req").unwrap();
        let sci_own = get_u32(line, "pp_sci_own").unwrap();
        let owned_mask = get_u32(line, "pp_owned_mask").unwrap();
        let fac_cap = get_i32(line, "pp_fac_cap").unwrap() as usize;
        let ops = parse_i32_triples(line, "pp_ops").unwrap();

        let mut pp = ProductionPrerequisite::new();
        if (sci_req & 1) != 0 {
            pp.add_science_prereq(sci_store.friend_lookup_science("SCI_A", &mut name_keys).unwrap());
        }
        if (sci_req & 2) != 0 {
            pp.add_science_prereq(sci_store.friend_lookup_science("SCI_B", &mut name_keys).unwrap());
        }
        let mut idx_pos = 0;
        while idx_pos < ops.len() {
            let (idx0, or_prev, mode) = ops[idx_pos];
            if mode == 1 {
                let (idx1, _, mode1) = ops[idx_pos + 1];
                assert_eq!(mode1, 2);
                let n0 = format!("TMPL_{idx0:02}");
                let n1 = format!("TMPL_{idx1:02}");
                pp.add_unit_prereq_list(&[&n0, &n1]);
                idx_pos += 2;
            } else {
                let n0 = format!("TMPL_{idx0:02}");
                pp.add_unit_prereq(&n0, if or_prev != 0 { TRUE } else { FALSE });
                idx_pos += 1;
            }
        }
        pp.resolve_names(&factory);

        let mut owned_sci = Vec::new();
        if (sci_own & 1) != 0 {
            owned_sci.push(sci_store.friend_lookup_science("SCI_A", &mut name_keys).unwrap());
        }
        if (sci_own & 2) != 0 {
            owned_sci.push(sci_store.friend_lookup_science("SCI_B", &mut name_keys).unwrap());
        }
        let mut owned_counts = [0; 12];
        for (i, c) in owned_counts.iter_mut().enumerate() {
            if (owned_mask & (1 << i)) != 0 {
                *c = 1;
            }
        }
        let pp_player = MockPlayer {
            owned_sciences: owned_sci,
            owned_counts,
        };

        assert_eq!(
            pp.is_satisfied(Some(&pp_player)) as u32,
            get_u32(line, "pp_sat").unwrap(),
            "seed {seed} pp_sat"
        );
        let got_fac = pp
            .get_existing_build_facility_template(&pp_player)
            .map(|r| factory.entries[r.0].0.as_str())
            .unwrap_or("");
        assert_eq!(
            got_fac,
            get_unescaped_json_str(line, "pp_fac").unwrap(),
            "seed {seed} pp_fac"
        );

        let mut all_facs = vec![None; fac_cap];
        let cnt = pp.get_all_possible_build_facility_templates(&mut all_facs) as usize;
        let got_all_facs: Vec<String> = all_facs[..cnt]
            .iter()
            .map(|opt| {
                opt.map(|r| factory.entries[r.0].0.clone())
                    .unwrap_or_default()
            })
            .collect();
        assert_eq!(
            got_all_facs,
            get_str_vec(line, "pp_all_fac").unwrap(),
            "seed {seed} pp_all_fac"
        );

        let got_req_u16 = pp.get_requires_list(Some(&pp_player), &factory, &mut prereq_text);
        let mut got_req_bytes = Vec::with_capacity(got_req_u16.len() * 2);
        for unit in got_req_u16 {
            got_req_bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let exp_req_bytes =
            decode_hex_bytes(&get_unescaped_json_str(line, "pp_req_u16_hex").unwrap()).unwrap();
        assert_eq!(got_req_bytes, exp_req_bytes, "seed {seed} pp_req_u16");
    }
}
