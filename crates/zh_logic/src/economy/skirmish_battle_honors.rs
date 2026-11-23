//! `PKT-123`: Skirmish battle honors persistence and INI preferences
//! (`GeneralsMD/Code/GameEngine/Source/Common/SkirmishBattleHonors.cpp`
//!  and `UserPreferences` subset from `Core/GameEngine/Source/Common/UserPreferences.cpp`).

use std::collections::BTreeMap;

use crate::gamelogic::global_data::GameDifficulty;
use zh_common::types::{Bool, Int, Real, UnsignedInt, FALSE, TRUE};

/// Storage abstraction for `UserPreferences` INI persistence, keeping simulation
/// deterministic and decoupled from host filesystem side effects.
pub trait PreferencesStorage {
    fn load_file(&self, rel_name: &str) -> Option<String>;
    fn write_file(&mut self, rel_name: &str, contents: &str) -> bool;
}

/// In-memory [`PreferencesStorage`] backend for deterministic tests and headless runs.
#[derive(Debug, Clone, Default)]
pub struct MemoryPreferencesStorage {
    pub files: BTreeMap<String, String>,
}

impl PreferencesStorage for MemoryPreferencesStorage {
    fn load_file(&self, rel_name: &str) -> Option<String> {
        self.files.get(rel_name).cloned()
    }

    fn write_file(&mut self, rel_name: &str, contents: &str) -> bool {
        self.files.insert(rel_name.to_string(), contents.to_string());
        true
    }
}

/// Deterministic `UserPreferences` base map matching C++ `std::map<AsciiString, AsciiString>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserPreferences {
    pub entries: BTreeMap<String, String>,
    pub filename: String,
}

impl UserPreferences {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_with_storage(&mut self, fname: &str, storage: &dyn PreferencesStorage) -> Bool {
        let normalized = fname.replace('\\', "/");
        self.filename = normalized.clone();
        if let Some(contents) = storage.load_file(&normalized) {
            self.parse_ini_contents(&contents);
            TRUE
        } else {
            FALSE
        }
    }

    pub fn parse_ini_contents(&mut self, contents: &str) {
        for raw_line in contents.lines() {
            let line = ascii_trim(raw_line);
            if let Some(eq_idx) = line.find('=') {
                let key = ascii_trim(&line[..eq_idx]);
                let val = ascii_trim(&line[eq_idx + 1..]);
                if !key.is_empty() && !val.is_empty() {
                    self.entries.insert(key.to_string(), val.to_string());
                }
            }
        }
    }

    pub fn write_with_storage(&self, storage: &mut dyn PreferencesStorage) -> Bool {
        if self.filename.is_empty() {
            return FALSE;
        }
        let body = self.to_ini_string();
        if storage.write_file(&self.filename, &body) {
            TRUE
        } else {
            FALSE
        }
    }

    /// Formats entries identically to C++ `UserPreferences::write()` (`"%s = %s\n"`).
    pub fn to_ini_string(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.entries {
            out.push_str(k);
            out.push_str(" = ");
            out.push_str(v);
            out.push('\n');
        }
        out
    }

    pub fn get_bool(&self, key: &str, default_value: Bool) -> Bool {
        let val = self.get_ascii_string(key, "");
        if val.is_empty() {
            return default_value;
        }
        let lower = val.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "1" | "t" | "true" | "y" | "yes" | "ok"
        ) {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn get_int(&self, key: &str, default_value: Int) -> Int {
        let val = self.get_ascii_string(key, "");
        if val.is_empty() {
            return default_value;
        }
        c_atoi(&val)
    }

    pub fn get_real(&self, key: &str, default_value: Real) -> Real {
        let val = self.get_ascii_string(key, "");
        if val.is_empty() {
            return default_value;
        }
        c_atof_simple(&val)
    }

    pub fn get_ascii_string(&self, key: &str, default_value: &str) -> String {
        self.entries
            .get(key)
            .cloned()
            .unwrap_or_else(|| default_value.to_string())
    }

    pub fn set_bool(&mut self, key: &str, val: Bool) {
        self.entries
            .insert(key.to_string(), format!("{}", if val != FALSE { 1 } else { 0 }));
    }

    pub fn set_int(&mut self, key: &str, val: Int) {
        self.entries.insert(key.to_string(), format!("{val}"));
    }

    pub fn set_real(&mut self, key: &str, val: Real) {
        self.entries.insert(key.to_string(), format!("{val}"));
    }

    pub fn set_ascii_string(&mut self, key: &str, val: &str) {
        self.entries.insert(key.to_string(), val.to_string());
    }
}

/// Persistent skirmish battle honors preferences (`SkirmishBattleHonors.h/.cpp`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkirmishBattleHonors {
    pub prefs: UserPreferences,
}

impl SkirmishBattleHonors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_storage(instance_id: UnsignedInt, storage: &dyn PreferencesStorage) -> Self {
        let mut out = Self::default();
        out.load_from_ini_file(instance_id, storage);
        out
    }

    pub fn ini_filename(instance_id: UnsignedInt) -> String {
        if instance_id > 1 {
            format!("SkirmishStats_Instance{instance_id:02}.ini")
        } else {
            "SkirmishStats.ini".to_string()
        }
    }

    pub fn load_from_ini_file(
        &mut self,
        instance_id: UnsignedInt,
        storage: &dyn PreferencesStorage,
    ) -> Bool {
        let fname = Self::ini_filename(instance_id);
        self.prefs.load_with_storage(&fname, storage)
    }

    pub fn write_to_storage(&self, storage: &mut dyn PreferencesStorage) -> Bool {
        self.prefs.write_with_storage(storage)
    }

    pub fn set_wins(&mut self, val: Int) {
        self.prefs.set_int("Wins", val);
    }

    pub fn get_wins(&self) -> Int {
        self.prefs.get_int("Wins", 0)
    }

    pub fn set_losses(&mut self, val: Int) {
        self.prefs.set_int("Losses", val);
    }

    pub fn get_losses(&self) -> Int {
        self.prefs.get_int("Losses", 0)
    }

    pub fn set_win_streak(&mut self, val: Int) {
        self.prefs.set_int("WinStreak", val);
    }

    pub fn get_win_streak(&self) -> Int {
        self.prefs.get_int("WinStreak", 0)
    }

    pub fn set_best_win_streak(&mut self, val: Int) {
        self.prefs.set_int("BestWinStreak", val);
    }

    pub fn get_best_win_streak(&self) -> Int {
        self.prefs.get_int("BestWinStreak", 0)
    }

    pub fn set_challenge_medals(&mut self, val: Int) {
        self.prefs.set_int("Challenge", val);
    }

    pub fn get_challenge_medals(&self) -> Int {
        self.prefs.get_int("Challenge", 0)
    }

    pub fn set_built_scud(&mut self) {
        self.prefs.set_bool("SCUD", TRUE);
    }

    pub fn built_scud(&self) -> Bool {
        self.prefs.get_bool("SCUD", FALSE)
    }

    pub fn set_built_particle_cannon(&mut self) {
        self.prefs.set_bool("PPC", TRUE);
    }

    pub fn built_particle_cannon(&self) -> Bool {
        self.prefs.get_bool("PPC", FALSE)
    }

    pub fn set_built_nuke(&mut self) {
        self.prefs.set_bool("Nuke", TRUE);
    }

    pub fn built_nuke(&self) -> Bool {
        self.prefs.get_bool("Nuke", FALSE)
    }

    pub fn set_honors(&mut self, mask: Int) {
        let honors = self.prefs.get_int("Honors", 0);
        self.prefs.set_int("Honors", honors | mask);
    }

    pub fn get_honors(&self) -> Int {
        self.prefs.get_int("Honors", 0)
    }

    pub fn set_endurance_medal(&mut self, map_name: &str, difficulty: Int, num_ais: Int) {
        let key = format!("{map_name}_{difficulty}");
        self.prefs.set_int(&key, num_ais);
    }

    pub fn get_endurance_medal(&self, map_name: &str, difficulty: Int) -> Int {
        let key = format!("{map_name}_{difficulty}");
        self.prefs.get_int(&key, 0)
    }

    pub fn set_last_general(&mut self, val: &str) {
        self.prefs.set_ascii_string("LastHouse", val);
    }

    pub fn get_last_general(&self) -> String {
        self.prefs.get_ascii_string("LastHouse", "")
    }

    pub fn set_num_games_loyal(&mut self, val: Int) {
        self.prefs.set_int("LoyalGames", val);
    }

    pub fn get_num_games_loyal(&self) -> Int {
        self.prefs.get_int("LoyalGames", 0)
    }

    pub fn set_usa_campaign_complete(&mut self, difficulty: GameDifficulty) {
        let key = format!("USACampaign_{}", difficulty as Int);
        self.prefs.set_int(&key, 1);
    }

    pub fn get_usa_campaign_complete(&self, difficulty: GameDifficulty) -> Bool {
        let key = format!("USACampaign_{}", difficulty as Int);
        if self.prefs.get_int(&key, 0) != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn set_china_campaign_complete(&mut self, difficulty: GameDifficulty) {
        let key = format!("CHINACampaign_{}", difficulty as Int);
        self.prefs.set_int(&key, 1);
    }

    pub fn get_china_campaign_complete(&self, difficulty: GameDifficulty) -> Bool {
        let key = format!("CHINACampaign_{}", difficulty as Int);
        if self.prefs.get_int(&key, 0) != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn set_gla_campaign_complete(&mut self, difficulty: GameDifficulty) {
        let key = format!("GLACampaign_{}", difficulty as Int);
        self.prefs.set_int(&key, 1);
    }

    pub fn get_gla_campaign_complete(&self, difficulty: GameDifficulty) -> Bool {
        let key = format!("GLACampaign_{}", difficulty as Int);
        if self.prefs.get_int(&key, 0) != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn set_challenge_campaign_complete(&mut self, challenge: Int, difficulty: GameDifficulty) {
        let key = format!("ChallengeCampaign{challenge}_{}", difficulty as Int);
        self.prefs.set_int(&key, 1);
    }

    pub fn get_challenge_campaign_complete(
        &self,
        challenge: Int,
        difficulty: GameDifficulty,
    ) -> Bool {
        let key = format!("ChallengeCampaign{challenge}_{}", difficulty as Int);
        if self.prefs.get_int(&key, 0) != 0 {
            TRUE
        } else {
            FALSE
        }
    }
}

fn ascii_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_ascii() && (c as u8) <= b' ')
}

fn c_atoi(s: &str) -> Int {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let mut neg = false;
    if i < bytes.len() {
        if bytes[i] == b'-' {
            neg = true;
            i += 1;
        } else if bytes[i] == b'+' {
            i += 1;
        }
    }
    let mut acc: i64 = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        acc = acc.wrapping_mul(10).wrapping_add((bytes[i] - b'0') as i64);
        i += 1;
    }
    if neg {
        acc.wrapping_neg() as Int
    } else {
        acc as Int
    }
}

fn c_atof_simple(s: &str) -> Real {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let mut neg = false;
    if i < bytes.len() {
        if bytes[i] == b'-' {
            neg = true;
            i += 1;
        } else if bytes[i] == b'+' {
            i += 1;
        }
    }
    let mut val = 0.0_f64;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        val = val * 10.0 + ((bytes[i] - b'0') as f64);
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let mut scale = 0.1_f64;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            val += ((bytes[i] - b'0') as f64) * scale;
            scale *= 0.1;
            i += 1;
        }
    }
    if neg {
        -(val as Real)
    } else {
        val as Real
    }
}
