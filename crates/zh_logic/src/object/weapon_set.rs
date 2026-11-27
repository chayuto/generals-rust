//! `PKT-400`: Weapon template sets and weapon-set slot/lock state machines.

use super::weapon::{
    Weapon, WeaponReloadType, WeaponTemplate, WEAPON_ANTI_AIRBORNE_INFANTRY,
    WEAPON_ANTI_AIRBORNE_VEHICLE, WEAPON_ANTI_BALLISTIC_MISSILE, WEAPON_ANTI_GROUND,
    WEAPON_ANTI_MINE, WEAPON_ANTI_PARACHUTE, WEAPON_ANTI_PROJECTILE, WEAPON_ANTI_SMALL_MISSILE,
};
use super::weapon_queries;
use crate::ai::seams::ai_seams;
use crate::arena::ObjectSlot;
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::turrets;
use crate::object::object_script_status_bits::OBJECT_STATUS_SCRIPT_TARGETABLE;
use crate::rules::combat_types::{
    DamageType, DamageTypeFlags, WeaponSetFlags, WeaponSetType, WeaponStatus, WEAPON_SET_NAMES,
};
use crate::rules::kind_of::{KindOfMaskType, KindOfType, KINDOF_NAMES};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use zh_common::math::Coord3D;
use zh_common::types::{ObjectId, Real, Relationship};
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
    pub preferred_against: [KindOfMaskType; WEAPONSLOT_COUNT],
    pub is_reload_time_shared: Bool,
    pub is_weapon_lock_shared_across_sets: Bool,
}

impl Default for WeaponTemplateSet {
    fn default() -> Self {
        Self {
            thing_template_name: String::new(),
            types: WeaponSetFlags::new(),
            templates: [None, None, None],
            // `WeaponTemplateSet::clear`: any command source may choose.
            auto_choose_mask: [0xffff_ffff; WEAPONSLOT_COUNT],
            preferred_against: Default::default(),
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
                let mut mask = KindOfMaskType::new();
                mask.parse_from_ini(reader, &KINDOF_NAMES)?;
                self.preferred_against[slot_idx as usize] = mask;
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
                    let mut kind = KindOfMaskType::new();
                    for name in p {
                        if let Some(i) = KINDOF_NAMES.iter().position(|n| *n == name) { kind.set(i as i32, 1); }
                    }
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
    /// `clearLeechRangeModeForAllWeapons`.
    pub fn clear_leech_range_mode_for_all_weapons(&mut self) {
        for w in self.weapons.iter_mut().flatten() {
            w.set_leech_range_active(FALSE);
        }
    }
    pub fn release_weapon_lock(&mut self, lock: WeaponLockType) {
        // Releasing a permanent lock releases any lock.
        if lock == WeaponLockType::LockedPermanently || self.cur_weapon_locked_status == lock {
            self.cur_weapon_locked_status = WeaponLockType::NotLocked;
        }
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
    /// `isOutOfAmmo`: every weapon held is out of ammo for good.
    pub fn is_out_of_ammo(&mut self, now: UnsignedInt) -> Bool {
        for w in self.weapons.iter_mut().flatten() {
            if w.get_status(now) != WeaponStatus::OutOfAmmo {
                return FALSE;
            }
        }
        TRUE
    }
    /// `getModelConditionForWeaponSlot`: the slot's flag for the condition, and
    /// its using-weapon flag for any condition.
    pub fn get_model_condition_for_weapon_slot(wslot: WeaponSlotType, condition: WeaponSetConditionType) -> ModelConditionFlags {
        use ModelConditionFlagType as M;
        let mut f = ModelConditionFlags::new();
        let slot = wslot as usize;
        let flags = match condition {
            WeaponSetConditionType::Firing => [M::FiringA, M::FiringB, M::FiringC],
            WeaponSetConditionType::Between => [
                M::BetweenFiringShotsA,
                M::BetweenFiringShotsB,
                M::BetweenFiringShotsC,
            ],
            WeaponSetConditionType::Reloading => [M::ReloadingA, M::ReloadingB, M::ReloadingC],
            WeaponSetConditionType::Preattack => [M::PreAttackA, M::PreAttackB, M::PreAttackC],
            _ => return f,
        };
        f.set(flags[slot].as_i32(), 1);
        f.set([M::UsingWeaponA, M::UsingWeaponB, M::UsingWeaponC][slot].as_i32(), 1);
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

// -----------------------------------------------------------------------------
// What the set says about a target
// -----------------------------------------------------------------------------

/// `AbleToAttackType` bits.
pub const ATTACK_FORCED_BIT: UnsignedInt = 0x01;
pub const ATTACK_NEW_TARGET: UnsignedInt = 0;
pub const ATTACK_CONTINUED_TARGET: UnsignedInt = 0x02;
pub const ATTACK_CONTINUED_TARGET_FORCED: UnsignedInt = 0x03;
pub const ATTACK_TUNNEL_NETWORK_GUARD: UnsignedInt = 0x04;

/// The reference tests the command source mask against the value of the
/// `CMD_DEFAULT_SWITCH_WEAPON` enumerator, not its bit: it is the bit of
/// `FROM_AI`.
const DEFAULT_SWITCH_WEAPON_BIT: UnsignedInt = CommandSourceType::DefaultSwitchWeapon as UnsignedInt;

fn is_kind(sim: &dyn SimView, slot: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(slot, kind) != FALSE
}

fn has_status(sim: &dyn SimView, slot: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.status.test(status as Int) != FALSE)
}

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

/// `getVictimAntiMask`.
pub fn victim_anti_mask(sim: &dyn SimView, victim: ObjectSlot) -> Int {
    let is = |kind| is_kind(sim, victim, kind);
    if is(KindOfType::SmallMissile) {
        WEAPON_ANTI_SMALL_MISSILE
    } else if is(KindOfType::BallisticMissile) {
        WEAPON_ANTI_BALLISTIC_MISSILE
    } else if is(KindOfType::Projectile) {
        WEAPON_ANTI_PROJECTILE
    } else if is(KindOfType::Mine) || is(KindOfType::Demotrap) {
        WEAPON_ANTI_MINE | WEAPON_ANTI_GROUND
    } else if has_status(sim, victim, ObjectStatusType::AirborneTarget) {
        if is(KindOfType::Vehicle) {
            WEAPON_ANTI_AIRBORNE_VEHICLE
        } else if is(KindOfType::Infantry) {
            WEAPON_ANTI_AIRBORNE_INFANTRY
        } else if is(KindOfType::Parachute) {
            WEAPON_ANTI_PARACHUTE
        } else {
            0
        }
    } else {
        WEAPON_ANTI_GROUND
    }
}

/// `isAnyWithinTargetPitch`.
fn is_any_within_target_pitch(sim: &dyn SimView, source: ObjectSlot, victim: ObjectSlot) -> bool {
    let Some(set) = sim.object(source).map(|o| &o.weapon_set) else {
        return false;
    };
    set.has_pitch_limit == FALSE
        || set
            .weapons
            .iter()
            .flatten()
            .any(|w| weapon_queries::is_within_target_pitch(w, sim, source, victim) != FALSE)
}

/// `Object::getAbleToAttackSpecificObject` (`WeaponSet`) with no weapon slot named.
pub fn able_to_attack_specific_object(
    sim: &dyn SimView,
    source: ObjectSlot,
    attack_type: UnsignedInt,
    victim: ObjectSlot,
    command_source: CommandSourceType,
) -> CanAttackResult {
    use CanAttackResult::NotPossible;
    let (Some(me), Some(them)) = (sim.object(source), sim.object(victim)) else {
        return NotPossible;
    };
    if me.is_effectively_dead() != FALSE
        || them.is_effectively_dead() != FALSE
        || me.is_destroyed()
        || them.is_destroyed()
        || source == victim
    {
        return NotPossible;
    }
    let forced = attack_type & ATTACK_FORCED_BIT != 0;
    let same_owner_force_attack =
        sim.player_index(me.team_id) == sim.player_index(them.team_id) && forced;
    let has = |s| has_status(sim, victim, s);
    if has(ObjectStatusType::Masked) || is_kind(sim, victim, KindOfType::Unattackable) {
        return NotPossible;
    }
    if has(ObjectStatusType::NoAttackFromAi) && command_source == CommandSourceType::FromAi {
        return NotPossible;
    }

    // Stealth keeps a unit from being attacked unless it is the owner forcing
    // the attack, or the attacker ignores it.
    let mut stealth_prevents = !(has_status(sim, source, ObjectStatusType::IgnoringStealth)
        || same_owner_force_attack);
    if forced && is_kind(sim, victim, KindOfType::Disguiser) && has(ObjectStatusType::Disguised) {
        stealth_prevents = false;
    }
    if stealth_prevents && has(ObjectStatusType::Stealthed) && !has(ObjectStatusType::Detected) {
        if !is_kind(sim, victim, KindOfType::Disguiser) {
            return NotPossible;
        }
        if let Some(stealth) = sim.stealth_params(victim) {
            if stealth.disguised_template.is_some()
                && sim.team_relationship_to_player(me.team_id, stealth.disguised_player_index)
                    != Relationship::Enemies
            {
                return NotPossible;
            }
        }
    }

    let relationship = sim.player_relationship(source, victim);
    // Only a person's order is refused here, unless a script made the victim a target.
    let refused_for_player = || {
        command_source == CommandSourceType::FromPlayer
            && (them.script_status & OBJECT_STATUS_SCRIPT_TARGETABLE == 0
                || relationship == Relationship::Allies)
    };
    if relationship != Relationship::Enemies
        && !forced
        && !(is_kind(sim, victim, KindOfType::Mine) && relationship != Relationship::Allies)
        && refused_for_player()
    {
        return NotPossible;
    }

    if let Some(container) = them.contained_by {
        if sim.contain_is_enclosing_container_for(container, victim) != FALSE {
            return NotPossible;
        }
    }

    if !forced && sim.object_has_contain(victim) != FALSE {
        let observer = sim.player_index(me.team_id);
        if let Some(controller) = sim.contain_apparent_controlling_player(victim, observer) {
            if sim.team_relationship_to_player(me.team_id, controller) != Relationship::Enemies
                && refused_for_player()
            {
                return NotPossible;
            }
        }
    }

    let pos = position_of(sim, victim);
    able_to_use_weapon_against_target(
        sim,
        source,
        attack_type,
        Some(victim),
        &pos,
        command_source,
    )
}

/// `Object::getAbleToUseWeaponAgainstTarget` (`WeaponSet`) with no weapon slot
/// named. The weapons are asked on copies, so the status `getStatus` caches is
/// not stored here.
pub fn able_to_use_weapon_against_target(
    sim: &dyn SimView,
    source: ObjectSlot,
    attack_type: UnsignedInt,
    victim: Option<ObjectSlot>,
    pos: &Coord3D,
    command_source: CommandSourceType,
) -> CanAttackResult {
    use CanAttackResult::{InvalidShot, Possible, PossibleAfterMoving};
    let Some(me) = sim.object(source) else {
        return CanAttackResult::NotPossible;
    };
    let set = &me.weapon_set;
    let (target_anti_mask, pos) = match victim {
        Some(v) => (victim_anti_mask(sim, v), position_of(sim, v)),
        None => (WEAPON_ANTI_GROUND, *pos),
    };

    let mut within_attack_range = false;
    let contained_by = me.contained_by;
    let mut has_a_weapon_in_range = false;
    let mut has_a_weapon = false;
    for _ in 0..WEAPONSLOT_COUNT - 1 {
        // The reference tests the current weapon on every round.
        let Some(weapon) = set.get_weapon_in_weapon_slot(set.cur_weapon) else {
            continue;
        };
        has_a_weapon = true;
        if set.total_anti_mask & target_anti_mask == 0 {
            continue;
        }
        let garrisoned = contained_by.filter(|&c| {
            sim.contain_is_garrisonable(c) != FALSE
                && sim.contain_is_enclosing_container_for(c, source) != FALSE
        });
        if let Some(container) = garrisoned {
            let container = sim.object(container).map_or(ObjectId::INVALID, |o| o.id);
            if let Some(goal) = ai_seams(sim).garrison_fire_position(container, &pos) {
                within_attack_range = weapon_queries::is_goal_pos_within_attack_range(
                    weapon, sim, source, &goal, victim, &pos,
                ) != FALSE;
            }
        } else if let Some(v) = victim {
            within_attack_range =
                weapon_queries::is_within_attack_range(weapon, sim, source, v) != FALSE;
        } else {
            within_attack_range =
                weapon_queries::is_within_attack_range_of_position(weapon, sim, source, &pos)
                    != FALSE;
        }
        if within_attack_range {
            has_a_weapon_in_range = true;
            break;
        }
    }

    // What cannot move must have the target in range already.
    if (is_kind(sim, source, KindOfType::Immobile)
        || is_kind(sim, source, KindOfType::SpawnsAreTheWeapons)
        || contained_by.is_some())
        && has_a_weapon
        && !has_a_weapon_in_range
        && attack_type != ATTACK_TUNNEL_NETWORK_GUARD
    {
        return InvalidShot;
    }

    let mut ok_result = if within_attack_range {
        Possible
    } else {
        PossibleAfterMoving
    };

    if set.has_any_damage_weapon() != FALSE {
        if set.total_anti_mask & target_anti_mask == 0 {
            return InvalidShot;
        }
        let Some(victim) = victim else {
            return ok_result;
        };
        if !is_any_within_target_pitch(sim, source, victim) {
            return InvalidShot;
        }
        let (first, last) = if set.is_cur_weapon_locked() != FALSE {
            (set.cur_weapon as usize, set.cur_weapon as usize)
        } else {
            (WEAPONSLOT_COUNT - 1, 0)
        };
        for i in (last..=first).rev() {
            let Some(mut weapon) = set.weapons[i].clone() else {
                continue;
            };
            if weapon_queries::estimate_weapon_damage(&mut weapon, sim, source, Some(victim)) != 0.0
            {
                // A hero does not volunteer a pilot kill with the primary selected.
                if weapon.template.damage_type == DamageType::KillPilot
                    && is_kind(sim, source, KindOfType::Hero)
                    && set.cur_weapon == WeaponSlotType::Primary
                {
                    continue;
                }
                return ok_result;
            }
        }
    }

    // Passengers who may shoot from inside.
    if sim.object_has_contain(source) != FALSE
        && sim.contain_is_passenger_allowed_to_fire(source, ObjectId::INVALID) != FALSE
    {
        for id in sim.contain_items(source) {
            let Some(member) = sim.find_object(id) else {
                continue;
            };
            if sim.object_is_able_to_attack(member) != FALSE {
                let result = sim.able_to_use_weapon_against_target(
                    member,
                    attack_type,
                    victim,
                    &pos,
                    command_source,
                );
                if result == Possible || result == PossibleAfterMoving {
                    return result;
                }
            }
        }
    }

    // And slaves that fight for their owner.
    if sim.spawn_slaves_use_weapon_against_target(source, attack_type, victim, &pos, command_source)
        == Some(Possible)
    {
        if is_kind(sim, source, KindOfType::Immobile)
            && is_kind(sim, source, KindOfType::SpawnsAreTheWeapons)
            && ok_result == PossibleAfterMoving
        {
            ok_result = Possible;
        }
        return ok_result;
    }

    InvalidShot
}

/// `Object::chooseBestWeaponForTarget`: makes the best weapon against `victim`
/// the current one. `false` when none can hurt it.
pub fn choose_best_weapon_for_target(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    victim: Option<ObjectSlot>,
    criteria: WeaponChoiceCriteria,
    command_source: CommandSourceType,
) -> bool {
    let Some(set) = sim.object(owner).map(|o| &o.weapon_set) else {
        return false;
    };
    if set.is_cur_weapon_locked() != FALSE {
        return true;
    }
    let Some(victim) = victim else {
        if let Some(o) = sim.object_mut(owner) {
            o.weapon_set.cur_weapon = WeaponSlotType::Primary;
        }
        return true;
    };

    let mut found = false;
    let mut found_backup = false;
    let mut longest_range: Real = 0.0;
    let mut best_damage: Real = 0.0;
    let mut longest_range_backup: Real = 0.0;
    let mut best_damage_backup: Real = 0.0;
    let mut decision = WeaponSlotType::Primary;
    let mut decision_backup = WeaponSlotType::Primary;
    let now = sim.frame();

    for i in (0..WEAPONSLOT_COUNT).rev() {
        let slot = WeaponSlotType::from_i32(i as Int).unwrap_or(WeaponSlotType::Primary);
        let Some(set) = sim.object(owner).map(|o| &o.weapon_set) else {
            return false;
        };
        let Some(mut weapon) = set.weapons[i].clone() else {
            continue;
        };
        let (sources, preferred) = set.cur_weapon_template_set.as_ref().map_or_else(
            || (0xffff_ffff, KindOfMaskType::new()),
            |t| (t.auto_choose_mask[i], t.preferred_against[i].clone()),
        );
        if sources & (1 << command_source as u32) == 0 && sources & DEFAULT_SWITCH_WEAPON_BIT == 0 {
            continue;
        }
        let mut usable = judge_weapon(sim, owner, victim, &mut weapon);
        // A weapon whose turret is already aiming there does not count as ready,
        // and one that would do nothing is passed over.
        if let Some((damage, _, ready)) = &mut usable {
            let victim_id = sim.object(victim).map_or(ObjectId::INVALID, |o| o.id);
            if turrets::weapon_slot_aiming_at(sim, owner, slot, victim_id) {
                *ready = false;
            }
            if *damage <= 0.0 && weapon.template.damage_type != DamageType::Unresistable {
                usable = None;
            }
        }
        // A weapon preferred against the victim's kind beats the others.
        let preferred_here = usable.is_some()
            && preferred.any() != FALSE
            && sim.object(victim).is_some_and(|o| {
                o.thing.is_kind_of_multi(
                    &sim.rules().thing_factory,
                    &preferred,
                    &KindOfMaskType::new(),
                ) != FALSE
            });
        let ready_if_preferred =
            preferred_here && weapon.get_status(now) != WeaponStatus::OutOfAmmo;
        // `getStatus` stores what it works out.
        let status = weapon.status;
        if let Some(stored) = sim
            .object_mut(owner)
            .and_then(|o| o.weapon_set.get_weapon_in_weapon_slot_mut(slot))
        {
            stored.status = status;
        }
        let Some((mut damage, mut range, mut ready)) = usable else {
            continue;
        };
        if preferred_here {
            damage = 1e10;
            range = 1e10;
            ready = ready_if_preferred;
        }

        match criteria {
            WeaponChoiceCriteria::PreferMostDamage => {
                if !ready {
                    if damage >= best_damage_backup {
                        best_damage_backup = damage;
                        decision_backup = slot;
                        found_backup = true;
                    }
                } else if damage >= best_damage {
                    best_damage = damage;
                    decision = slot;
                    found = true;
                }
            }
            WeaponChoiceCriteria::PreferLongestRange => {
                if !ready {
                    if range > longest_range_backup {
                        longest_range_backup = range;
                        decision_backup = slot;
                        found_backup = true;
                    }
                } else if range > longest_range {
                    longest_range = range;
                    decision = slot;
                    found = true;
                }
            }
        }
    }

    let chosen = if found {
        decision
    } else if found_backup {
        found = true;
        decision_backup
    } else {
        WeaponSlotType::Primary
    };
    if let Some(o) = sim.object_mut(owner) {
        o.weapon_set.cur_weapon = chosen;
    }
    found
}

/// What `chooseBestWeaponForTarget` finds out about one weapon: damage, range
/// and readiness, or `None` to pass it over.
fn judge_weapon(
    sim: &dyn SimView,
    owner: ObjectSlot,
    victim: ObjectSlot,
    weapon: &mut Weapon,
) -> Option<(Real, Real, bool)> {
    let now = sim.frame();
    if weapon.get_status(now) == WeaponStatus::OutOfAmmo
        && weapon.template.reload_type != WeaponReloadType::AutoReload
    {
        return None;
    }
    if weapon.template.anti_mask & victim_anti_mask(sim, victim) == 0 {
        return None;
    }
    if weapon_queries::is_within_target_pitch(weapon, sim, owner, victim) == FALSE {
        return None;
    }
    let damage = weapon_queries::estimate_weapon_damage(weapon, sim, owner, Some(victim));
    let range = weapon_queries::attack_range(weapon, sim, owner);
    let ready = weapon.get_status(now) == WeaponStatus::ReadyToFire;
    Some((damage, range, ready))
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
