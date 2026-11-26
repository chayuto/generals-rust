//! `PKT-400`: Weapon template sets and weapon-set slot/lock state machines.

use super::weapon::{Weapon, WeaponTemplate};
use crate::rules::combat_types::{DamageTypeFlags, WeaponSetFlags, WeaponSetType, WEAPON_SET_NAMES};
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use zh_common::types::{Bool, Int, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::math::real_to_int_floor;
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub const WEAPONSLOT_COUNT: usize = 3;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponSlotType {
    #[default]
    Primary = 0,
    Secondary = 1,
    Tertiary = 2,
}

impl WeaponSlotType {
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v { 0=>Some(Self::Primary), 1=>Some(Self::Secondary), 2=>Some(Self::Tertiary), _=>None }
    }
    pub const fn as_i32(self) -> Int { self as Int }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponSetConditionType {
    #[default]
    Invalid = -1,
    None = 0,
    Firing = 1,
    Between = 2,
    Reloading = 3,
    Preattack = 4,
}

impl WeaponSetConditionType {
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v { -1=>Some(Self::Invalid), 0=>Some(Self::None), 1=>Some(Self::Firing), 2=>Some(Self::Between), 3=>Some(Self::Reloading), 4=>Some(Self::Preattack), _=>None }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponChoiceCriteria { #[default] PreferMostDamage = 0, PreferLongestRange = 1 }

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponLockType { #[default] NotLocked = 0, LockedTemporarily = 1, LockedPermanently = 2 }

impl WeaponLockType {
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v { 0=>Some(Self::NotLocked), 1=>Some(Self::LockedTemporarily), 2=>Some(Self::LockedPermanently), _=>None }
    }
    pub const fn as_i32(self) -> Int { self as Int }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanAttackResult { #[default] NotPossible = 0, InvalidShot = 1, PossibleAfterMoving = 2, Possible = 3 }

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponTemplateSet {
    pub thing_template_name: String,
    pub types: WeaponSetFlags,
    pub templates: [Option<WeaponTemplate>; WEAPONSLOT_COUNT],
    pub auto_choose_mask: [UnsignedInt; WEAPONSLOT_COUNT],
    pub preferred_against: [UnsignedInt; WEAPONSLOT_COUNT],
    pub is_reload_time_shared: Bool,
    pub is_weapon_lock_shared_across_sets: Bool,
}

impl Default for WeaponTemplateSet {
    fn default() -> Self {
        Self {
            thing_template_name: String::new(),
            types: WeaponSetFlags::new(),
            templates: [None, None, None],
            auto_choose_mask: [0; WEAPONSLOT_COUNT],
            preferred_against: [0; WEAPONSLOT_COUNT],
            is_reload_time_shared: FALSE,
            is_weapon_lock_shared_across_sets: FALSE,
        }
    }
}

impl WeaponTemplateSet {
    pub fn new() -> Self { Self::default() }
    pub fn clear(&mut self) { *self = Self::default(); }
    pub fn test_weapon_set_flag(&self, wst: WeaponSetType) -> Bool {
        self.types.test(wst.as_i32())
    }
    pub fn is_shared_reload_time(&self) -> Bool { self.is_reload_time_shared }
    pub fn is_weapon_lock_shared_across_sets(&self) -> Bool { self.is_weapon_lock_shared_across_sets }
    pub fn get_nth(&self, slot: WeaponSlotType) -> Option<&WeaponTemplate> { self.templates[slot.as_i32() as usize].as_ref() }
    pub fn get_nth_command_source_mask(&self, slot: WeaponSlotType) -> UnsignedInt { self.auto_choose_mask[slot.as_i32() as usize] }

    pub fn has_any_weapons(&self) -> Bool {
        if self.templates.iter().any(|t| t.is_some()) {
            TRUE
        } else {
            FALSE
        }
    }

    /// Parses a `WeaponSet ... End` INI block using `IniReader` and resolving weapon names against `weapon_store`.
    pub fn parse_weapon_template_set_ini(
        &mut self,
        ini: &mut zh_common::ini::IniReader<'_>,
        weapon_store: &super::weapon::WeaponStore,
        thing_template_name: &str,
    ) -> Result<(), zh_common::ini::IniError> {
        const WEAPON_SLOT_NAMES: [&str; WEAPONSLOT_COUNT] = ["PRIMARY", "SECONDARY", "TERTIARY"];
        const COMMAND_SOURCE_MASK_NAMES: [&str; 5] = [
            "FROM_PLAYER",
            "FROM_SCRIPT",
            "FROM_AI",
            "FROM_DOZER",
            "DEFAULT_SWITCH_WEAPON",
        ];

        self.clear();
        self.auto_choose_mask = [0xffff_ffff; WEAPONSLOT_COUNT];
        self.thing_template_name = thing_template_name.to_owned();

        ini.read_block_fields(|field, reader| {
            if field.eq_ignore_ascii_case("Conditions") {
                self.types.parse_from_ini(reader, &WEAPON_SET_NAMES)
            } else if field.eq_ignore_ascii_case("Weapon") {
                let slot_tok = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
                let slot_idx =
                    zh_common::ini::IniReader::scan_index_list(slot_tok, &WEAPON_SLOT_NAMES)?;
                let wpn_tok = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
                if wpn_tok.eq_ignore_ascii_case("None") {
                    self.templates[slot_idx as usize] = None;
                } else {
                    self.templates[slot_idx as usize] =
                        weapon_store.find_weapon_template(wpn_tok).cloned();
                }
                Ok(())
            } else if field.eq_ignore_ascii_case("AutoChooseSources") {
                let slot_tok = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
                let slot_idx =
                    zh_common::ini::IniReader::scan_index_list(slot_tok, &WEAPON_SLOT_NAMES)?;
                reader.parse_bit_string32(
                    &mut self.auto_choose_mask[slot_idx as usize],
                    &COMMAND_SOURCE_MASK_NAMES,
                )
            } else if field.eq_ignore_ascii_case("PreferredAgainst") {
                let slot_tok = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
                let slot_idx =
                    zh_common::ini::IniReader::scan_index_list(slot_tok, &WEAPON_SLOT_NAMES)?;
                let mut mask = crate::rules::kind_of::KindOfMaskType::new();
                mask.parse_from_ini(reader, &crate::rules::kind_of::KINDOF_NAMES)?;
                self.preferred_against[slot_idx as usize] = mask.to_unsigned_int();
                Ok(())
            } else if field.eq_ignore_ascii_case("ShareWeaponReloadTime") {
                reader.parse_bool(&mut self.is_reload_time_shared)
            } else if field.eq_ignore_ascii_case("WeaponLockSharedAcrossSets") {
                reader.parse_bool(&mut self.is_weapon_lock_shared_across_sets)
            } else {
                Err(zh_common::ini::IniError::UnknownToken)
            }
        })
    }

    pub fn parse_weapon_template_set(&mut self, text: &str) {
        let mut active_slot: Option<WeaponSlotType> = None;
        for line in text.lines().skip(1) {
            let line = line.trim();
            if line.eq_ignore_ascii_case("End") { break; }
            let Some((key, value)) = line.split_once('=') else { continue };
            match key.trim() {
                "Conditions" => {
                    self.types = WeaponSetFlags::new();
                    for name in value.trim().split_whitespace() {
                        if let Some(i) = WEAPON_SET_NAMES.iter().position(|n| *n == name) { self.types.set(i as i32, 1); }
                    }
                }
                "Weapon" => {
                    let mut p = value.trim().split_whitespace();
                    let slot_name = p.next().unwrap_or("");
                    let template_name = p.next().unwrap_or("");
                    let slot = match slot_name { "PRIMARY"=>WeaponSlotType::Primary, "SECONDARY"=>WeaponSlotType::Secondary, "TERTIARY"=>WeaponSlotType::Tertiary, _=>continue };
                    active_slot = Some(slot);
                    let _ = template_name;
                }
                "AutoChooseSources" => {
                    let mut p = value.trim().split_whitespace();
                    let slot_name = p.next().unwrap_or("");
                    let slot = match slot_name { "PRIMARY"=>WeaponSlotType::Primary, "SECONDARY"=>WeaponSlotType::Secondary, "TERTIARY"=>WeaponSlotType::Tertiary, _=>continue };
                    let mut mask = 0u32;
                    for name in p {
                        mask |= match name { "FROM_SCRIPT"=>2, "FROM_PLAYER"=>1, "FROM_AI"=>4, "DEFAULT_SWITCH_WEAPON"=>16, _=>0 };
                    }
                    self.auto_choose_mask[slot.as_i32() as usize] = mask;
                    active_slot = Some(slot);
                }
                "PreferredAgainst" => {
                    let mut p = value.trim().split_whitespace();
                    let slot_name = p.next().unwrap_or("");
                    let slot = match slot_name { "PRIMARY"=>WeaponSlotType::Primary, "SECONDARY"=>WeaponSlotType::Secondary, "TERTIARY"=>WeaponSlotType::Tertiary, _=>continue };
                    let mut kind = 0u32;
                    for name in p { kind |= match name { "INFANTRY"=>1, "VEHICLE"=>2, "STRUCTURE"=>4, "AIRCRAFT"=>8, "SMALL_MISSILE"=>16, _=>0 }; }
                    self.preferred_against[slot.as_i32() as usize] = kind;
                    active_slot = Some(slot);
                }
                "ShareWeaponReloadTime" => self.is_reload_time_shared = if value.trim().eq_ignore_ascii_case("Yes") { TRUE } else { FALSE },
                "WeaponLockSharedAcrossSets" => self.is_weapon_lock_shared_across_sets = if value.trim().eq_ignore_ascii_case("Yes") { TRUE } else { FALSE },
                _ => {}
            }
        }
        let _ = active_slot;
    }
}

impl crate::player::sparse_match_finder::SparseMatchable<WeaponSetFlags> for WeaponTemplateSet {
    #[inline]
    fn get_conditions_yes_count(&self) -> Int {
        1
    }

    #[inline]
    fn get_nth_conditions_yes(&self, _i: Int) -> &WeaponSetFlags {
        &self.types
    }
}

pub const WEAPON_SET_TYPE_TO_MODEL_CONDITION_TYPE_MAP: [ModelConditionFlagType; 17] = [
    ModelConditionFlagType::WeaponSetVeteran,
    ModelConditionFlagType::WeaponSetElite,
    ModelConditionFlagType::WeaponSetHero,
    ModelConditionFlagType::WeaponSetPlayerUpgrade,
    ModelConditionFlagType::WeaponSetCrateUpgradeOne,
    ModelConditionFlagType::WeaponSetCrateUpgradeTwo,
    ModelConditionFlagType::Invalid,
    ModelConditionFlagType::Invalid,
    ModelConditionFlagType::Invalid,
    ModelConditionFlagType::Rider1,
    ModelConditionFlagType::Rider2,
    ModelConditionFlagType::Rider3,
    ModelConditionFlagType::Rider4,
    ModelConditionFlagType::Rider5,
    ModelConditionFlagType::Rider6,
    ModelConditionFlagType::Rider7,
    ModelConditionFlagType::Rider8,
];

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponSet {
    pub cur_weapon_template_set: Option<WeaponTemplateSet>,
    pub weapons: [Option<Weapon>; WEAPONSLOT_COUNT],
    pub cur_weapon: WeaponSlotType,
    pub cur_weapon_locked_status: WeaponLockType,
    pub filled_weapon_slot_mask: UnsignedInt,
    pub total_anti_mask: Int,
    pub total_damage_type_mask: DamageTypeFlags,
    pub has_pitch_limit: Bool,
    pub has_damage_weapon: Bool,
}

impl Default for WeaponSet {
    fn default() -> Self {
        Self {
            cur_weapon_template_set: None,
            weapons: [None, None, None],
            cur_weapon: WeaponSlotType::Primary,
            cur_weapon_locked_status: WeaponLockType::NotLocked,
            filled_weapon_slot_mask: 0,
            total_anti_mask: 0,
            total_damage_type_mask: DamageTypeFlags::new(),
            has_pitch_limit: FALSE,
            has_damage_weapon: FALSE,
        }
    }
}

impl WeaponSet {
    pub fn new() -> Self { Self::default() }
    pub fn with_template_set(template_set: WeaponTemplateSet) -> Self {
        let mut s = Self::default();
        s.cur_weapon_template_set = Some(template_set);
        s
    }
    pub fn get_cur_weapon_slot(&self) -> WeaponSlotType { self.cur_weapon }
    pub fn is_cur_weapon_locked(&self) -> Bool { if self.cur_weapon_locked_status != WeaponLockType::NotLocked { TRUE } else { FALSE } }
    pub fn has_any_weapon(&self) -> Bool { if self.filled_weapon_slot_mask != 0 { TRUE } else { FALSE } }
    pub fn has_any_damage_weapon(&self) -> Bool { self.has_damage_weapon }
    pub fn get_nth_command_source_mask(&self, slot: WeaponSlotType) -> UnsignedInt {
        self.cur_weapon_template_set.as_ref().map_or(0, |t| t.get_nth_command_source_mask(slot))
    }
    pub fn get_weapon_in_weapon_slot(&self, slot: WeaponSlotType) -> Option<&Weapon> {
        self.weapons[slot.as_i32() as usize].as_ref()
    }
    pub fn get_weapon_in_weapon_slot_mut(&mut self, slot: WeaponSlotType) -> Option<&mut Weapon> {
        self.weapons[slot.as_i32() as usize].as_mut()
    }
    pub fn set_weapon_lock(&mut self, slot: WeaponSlotType, lock: WeaponLockType) -> Bool {
        if lock == WeaponLockType::NotLocked {
            return FALSE;
        }
        let idx = slot.as_i32() as usize;
        if idx >= WEAPONSLOT_COUNT || self.weapons[idx].is_none() {
            return FALSE;
        }
        if lock == WeaponLockType::LockedPermanently {
            self.cur_weapon = slot;
            self.cur_weapon_locked_status = lock;
        } else if lock == WeaponLockType::LockedTemporarily
            && self.cur_weapon_locked_status != WeaponLockType::LockedPermanently
        {
            self.cur_weapon = slot;
            self.cur_weapon_locked_status = lock;
        }
        TRUE
    }
    pub fn init_from_template_set(&mut self, tmpl_set: &WeaponTemplateSet, now: UnsignedInt) {
        self.cur_weapon_template_set = Some(tmpl_set.clone());
        for (i, wt_opt) in tmpl_set.templates.iter().enumerate() {
            let slot = WeaponSlotType::from_i32(i as Int).unwrap_or(WeaponSlotType::Primary);
            self.weapons[i] = wt_opt.as_ref().map(|wt| {
                let mut weapon = Weapon::new(wt.clone(), slot, now);
                weapon.load_at_creation(now);
                weapon
            });
        }
        self.cur_weapon = WeaponSlotType::Primary;
        self.cur_weapon_locked_status = WeaponLockType::NotLocked;
        self.rebuild_masks();
    }
    pub fn release_weapon_lock(&mut self, lock: WeaponLockType) {
        if self.cur_weapon_locked_status == lock { self.cur_weapon_locked_status = WeaponLockType::NotLocked; }
    }
    pub fn is_shared_reload_time(&self) -> Bool {
        self.cur_weapon_template_set.as_ref().map_or(FALSE, |t| t.is_shared_reload_time())
    }
    pub fn get_most_percent_ready_to_fire_any_weapon(&mut self, now: UnsignedInt) -> UnsignedInt {
        let mut most = 0u32;
        for weapon in &mut self.weapons {
            if let Some(w) = weapon {
                let p = w.get_percent_ready_to_fire(now);
                let pct = real_to_int_floor(p * 100.0_f32) as UnsignedInt;
                if pct > most { most = pct; }
            }
        }
        most
    }
    pub fn is_out_of_ammo(&mut self, now: UnsignedInt) -> Bool {
        for weapon in &mut self.weapons {
            if let Some(w) = weapon {
                if w.get_status(now) != crate::rules::combat_types::WeaponStatus::OutOfAmmo
                    && w.get_remaining_ammo(now) != 0
                { return FALSE; }
            }
        }
        TRUE
    }
    pub fn get_model_condition_for_weapon_slot(wslot: WeaponSlotType, condition: WeaponSetConditionType) -> ModelConditionFlags {
        let mut f = ModelConditionFlags::new();
        let base = match wslot {
            WeaponSlotType::Primary => 34,
            WeaponSlotType::Secondary => 39,
            WeaponSlotType::Tertiary => 43,
        };
        let bit = match condition {
            WeaponSetConditionType::Firing => base + 2,
            WeaponSetConditionType::Between => base + 3,
            WeaponSetConditionType::Reloading => base + 4,
            WeaponSetConditionType::Preattack => base + 1,
            _ => -1,
        };
        if bit >= 0 { f.set(bit, 1); }
        f
    }
    pub fn rebuild_masks(&mut self) {
        self.filled_weapon_slot_mask = 0;
        self.total_anti_mask = 0;
        self.total_damage_type_mask = DamageTypeFlags::new();
        self.has_damage_weapon = FALSE;
        self.has_pitch_limit = FALSE;
        for (i, weapon) in self.weapons.iter().enumerate() {
            if let Some(w) = weapon {
                self.filled_weapon_slot_mask |= 1u32 << i;
                self.total_anti_mask |= w.template.anti_mask;
                self.total_damage_type_mask.set(w.template.damage_type.as_i32(), 1);
                if w.template.is_damage_weapon() == TRUE { self.has_damage_weapon = TRUE; }
                if w.is_pitch_limited() == TRUE { self.has_pitch_limit = TRUE; }
            }
        }
    }
}

impl Snapshot for WeaponSet {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        #[cfg(feature = "retail-xfer-save")]
        let current_version: XferVersion = 1;
        #[cfg(not(feature = "retail-xfer-save"))]
        let current_version: XferVersion = 1;
        let mut version = current_version;
        xfer.xfer_version(&mut version, current_version)?;

        if xfer.mode() == XferMode::Load {
            let mut tt_name = String::new();
            let mut ws_flags = WeaponSetFlags::new();
            xfer.xfer_ascii_string(&mut tt_name)?;
            let mut ws_bits = ws_flags.to_unsigned_int();
            xfer.xfer_unsigned_int(&mut ws_bits)?;
            ws_flags = WeaponSetFlags::from_unsigned_int(ws_bits);
            if !tt_name.is_empty() {
                if let Some(t) = self.cur_weapon_template_set.as_mut() { t.thing_template_name = tt_name; t.types = ws_flags; }
            } else {
                self.cur_weapon_template_set = None;
            }
        } else {
            let mut tt_name = self.cur_weapon_template_set.as_ref().map_or(String::new(), |t| t.thing_template_name.clone());
            if tt_name.is_empty() && self.cur_weapon_template_set.is_some() { tt_name = "DummyThing".to_string(); }
            let ws_flags = self.cur_weapon_template_set.as_ref().map_or_else(WeaponSetFlags::new, |t| t.types.clone());
            xfer.xfer_ascii_string(&mut tt_name)?;
            let mut ws_bits = ws_flags.to_unsigned_int();
            xfer.xfer_unsigned_int(&mut ws_bits)?;
        }

        for i in 0..WEAPONSLOT_COUNT {
            let mut has_weapon = self.weapons[i].is_some() as Bool;
            xfer.xfer_bool(&mut has_weapon)?;
            if has_weapon == TRUE {
                if xfer.mode() == XferMode::Load && self.weapons[i].is_none() {
                    let template = self.cur_weapon_template_set.as_ref().and_then(|t| t.templates[i].clone());
                    if let Some(template) = template { self.weapons[i] = Some(Weapon::new(template, WeaponSlotType::from_i32(i as i32).unwrap_or(WeaponSlotType::Primary), 0)); }
                }
                if let Some(w) = self.weapons[i].as_mut() {
                    let size = xfer.begin_block()?;
                    let _ = size;
                    xfer.xfer_snapshot(w)?;
                    xfer.end_block()?;
                }
            }
        }
        let mut cur = self.cur_weapon.as_i32(); xfer.xfer_int(&mut cur)?; if xfer.mode()==XferMode::Load { self.cur_weapon = WeaponSlotType::from_i32(cur).unwrap_or(WeaponSlotType::Primary); }
        let mut lock = self.cur_weapon_locked_status.as_i32(); xfer.xfer_int(&mut lock)?; if xfer.mode()==XferMode::Load { self.cur_weapon_locked_status = WeaponLockType::from_i32(lock).unwrap_or(WeaponLockType::NotLocked); }
        xfer.xfer_unsigned_int(&mut self.filled_weapon_slot_mask)?;
        xfer.xfer_int(&mut self.total_anti_mask)?;
        #[cfg(feature = "retail-xfer-save")]
        {
            let mut has = self.has_damage_weapon;
            xfer.xfer_bool(&mut has)?;
            self.has_damage_weapon = has;
            xfer.xfer_bool(&mut self.has_damage_weapon)?;
        }
        #[cfg(not(feature = "retail-xfer-save"))]
        {
            xfer.xfer_bool(&mut self.has_damage_weapon)?;
            xfer.xfer_bool(&mut self.has_damage_weapon)?;
        }
        let mut damage_bits = self.total_damage_type_mask.to_unsigned_int();
        xfer.xfer_unsigned_int(&mut damage_bits)?;
        if xfer.mode() == XferMode::Load {
            self.total_damage_type_mask = DamageTypeFlags::from_unsigned_int(damage_bits);
        }
        Ok(())
    }
}
