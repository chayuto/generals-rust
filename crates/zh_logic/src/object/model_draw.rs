//! What the simulation reads of `W3DModelDrawModuleData`: the condition states with
//! their models, animations and bone names (`buildFieldParse`,
//! `parseConditionState`, `parseAnimation`, `parseWeaponBoneName`,
//! `parseBoneNameKey` in `W3DModelDraw.cpp`).
//!
//! No name keys are made here; `ModuleRegistry` interns them as it skips the block.
//! Transition states are not kept: logic queries go through `findBestInfo`.
//!
//! This is device code with no packet. Golden test:
//! `crates/zh_logic/tests/golden_model_bones.rs`.

use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Int, NameKeyType, Real, UnsignedInt};

use crate::arena::FacetMask;
use crate::modules::module::{ModuleData, StubModuleData};
use crate::player::sparse_match_finder::{SparseMatchFinder, SparseMatchable};
use crate::rules::model_state::{ModelConditionFlags, MODEL_CONDITION_NAMES};
use crate::scheduler::SleepyUpdatePhase;

/// `WEAPONSLOT_COUNT`.
pub const WEAPON_SLOTS: usize = 3;
/// `MAX_TURRETS`.
pub const MAX_TURRETS: usize = 2;

const WEAPON_SLOT_NAMES: [&str; WEAPON_SLOTS] = ["PRIMARY", "SECONDARY", "TERTIARY"];

const ANIM_FLAG_NAMES: [&str; 10] = [
    "RANDOMSTART",
    "START_FRAME_FIRST",
    "START_FRAME_LAST",
    "ADJUST_HEIGHT_BY_CONSTRUCTION_PERCENT",
    "PRISTINE_BONE_POS_IN_FINAL_FRAME",
    "MAINTAIN_FRAME_ACROSS_STATES",
    "RESTART_ANIM_WHEN_COMPLETE",
    "MAINTAIN_FRAME_ACROSS_STATES2",
    "MAINTAIN_FRAME_ACROSS_STATES3",
    "MAINTAIN_FRAME_ACROSS_STATES4",
];
const PRISTINE_BONE_POS_IN_FINAL_FRAME: UnsignedInt = 1 << 4;

/// The draw modules derived from `W3DModelDraw`.
pub const MODEL_DRAW_MODULES: [&str; 11] = [
    "W3DModelDraw",
    "W3DTankDraw",
    "W3DTruckDraw",
    "W3DTankTruckDraw",
    "W3DPoliceCarDraw",
    "W3DOverlordTankDraw",
    "W3DOverlordTruckDraw",
    "W3DOverlordAircraftDraw",
    "W3DScienceModelDraw",
    "W3DSupplyDraw",
    "W3DDependencyModelDraw",
];

/// The bone names of one weapon slot; empty when not given.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WeaponSlotBones {
    pub fire_fx: String,
    pub recoil: String,
    pub muzzle_flash: String,
    pub launch: String,
}

/// `ModelConditionInfo::TurretInfo`, the parts read from INI.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TurretSpec {
    /// Empty for `NAMEKEY_INVALID`.
    pub angle_bone: String,
    pub pitch_bone: String,
    pub art_angle: Real,
    pub art_pitch: Real,
}

/// One `ModelConditionInfo`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelConditionBoneSpec {
    pub conditions_yes: Vec<ModelConditionFlags>,
    /// Lower case; empty for `NONE`.
    pub model_name: String,
    /// `m_animations.front()`.
    pub first_animation: Option<String>,
    flags: UnsignedInt,
    /// `m_publicBones`, without the module's `ExtraPublicBone`s.
    pub public_bones: Vec<String>,
    pub weapons: [WeaponSlotBones; WEAPON_SLOTS],
    pub turrets: [TurretSpec; MAX_TURRETS],
    anims_copied_from_default: bool,
}

impl ModelConditionBoneSpec {
    /// `PRISTINE_BONE_POS_IN_FINAL_FRAME`.
    pub fn pristine_bones_in_final_frame(&self) -> bool {
        self.flags & PRISTINE_BONE_POS_IN_FINAL_FRAME != 0
    }

    /// `addPublicBone`.
    fn add_public_bone(&mut self, name: &str) {
        add_public_bone(&mut self.public_bones, name);
    }
}

/// `ModelConditionInfo::addPublicBone`.
pub fn add_public_bone(list: &mut Vec<String>, name: &str) {
    if name.is_empty() || name.eq_ignore_ascii_case("none") {
        return;
    }
    let name = name.to_ascii_lowercase();
    if !list.contains(&name) {
        list.push(name);
    }
}

impl SparseMatchable<ModelConditionFlags> for ModelConditionBoneSpec {
    fn get_conditions_yes_count(&self) -> Int {
        self.conditions_yes.len() as Int
    }

    fn get_nth_conditions_yes(&self, i: Int) -> &ModelConditionFlags {
        &self.conditions_yes[i as usize]
    }
}

/// The bone related data of one `W3DModelDrawModuleData`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelDrawSpec {
    pub extra_public_bones: Vec<String>,
    pub ignore_condition_states: ModelConditionFlags,
    pub default_state: Option<usize>,
    pub condition_states: Vec<ModelConditionBoneSpec>,
    /// `AttachToBoneInAnotherModule`, lower case.
    pub attach_to_drawable_bone: String,
}

impl ModelDrawSpec {
    /// Reads a draw module block up to its `End`.
    pub fn parse(ini: &mut IniReader<'_>) -> Result<Self, IniError> {
        let mut spec = Self::default();
        ini.read_block_fields(|field, ini| match field {
            "DefaultConditionState" => spec.parse_default_state(ini),
            "ConditionState" => spec.parse_state(ini),
            "AliasConditionState" => spec.parse_alias(ini),
            "TransitionState" => ini.skip_block(),
            "ExtraPublicBone" => ini.parse_ascii_string_vector_append(&mut spec.extra_public_bones),
            "IgnoreConditionStates" => spec
                .ignore_condition_states
                .parse_from_ini(ini, &MODEL_CONDITION_NAMES),
            "AttachToBoneInAnotherModule" => {
                spec.attach_to_drawable_bone = ini.next_ascii_string()?.to_ascii_lowercase();
                Ok(())
            }
            _ => Ok(()),
        })?;
        Ok(spec)
    }

    /// `findBestInfo`: the index of the state for these condition flags.
    pub fn find_best_info(&self, conditions: &ModelConditionFlags) -> Option<usize> {
        let mut bits = conditions.clone();
        bits.clear_flags(&self.ignore_condition_states);
        SparseMatchFinder::<ModelConditionBoneSpec, ModelConditionFlags>::find_best_info_slow_index(
            &self.condition_states,
            &bits,
        )
    }

    // The reference also rejects ill-formed module data (a second default state,
    // repeated or ignored condition bits, a state without a model); retail data has
    // none, and the loader does not judge unported module data.
    fn parse_default_state(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        self.default_state = Some(self.condition_states.len());
        let mut info = ModelConditionBoneSpec::default();
        info.conditions_yes.push(ModelConditionFlags::default());
        self.finish_state(ini, info)
    }

    fn parse_state(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        let mut info = match self
            .default_state
            .and_then(|i| self.condition_states.get(i))
        {
            Some(default) => {
                let mut info = default.clone();
                info.anims_copied_from_default = true;
                info.conditions_yes.clear();
                info
            }
            None => ModelConditionBoneSpec::default(),
        };
        info.conditions_yes.push(parse_conditions(ini)?);
        self.finish_state(ini, info)
    }

    fn parse_alias(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        let conditions = parse_conditions(ini)?;
        if let Some(previous) = self.condition_states.last_mut() {
            previous.conditions_yes.push(conditions);
        }
        Ok(())
    }

    fn finish_state(
        &mut self,
        ini: &mut IniReader<'_>,
        mut info: ModelConditionBoneSpec,
    ) -> Result<(), IniError> {
        ini.read_block_fields(|field, ini| parse_state_field(&mut info, field, ini))?;
        if info.model_name == "none" {
            info.model_name.clear();
        }
        self.condition_states.push(info);
        Ok(())
    }
}

/// The flags of a state line.
fn parse_conditions(ini: &mut IniReader<'_>) -> Result<ModelConditionFlags, IniError> {
    let mut flags = ModelConditionFlags::default();
    flags.parse_from_ini(ini, &MODEL_CONDITION_NAMES)?;
    Ok(flags)
}

fn parse_state_field(
    info: &mut ModelConditionBoneSpec,
    field: &str,
    ini: &mut IniReader<'_>,
) -> Result<(), IniError> {
    match field {
        "Model" => info.model_name = ini.next_ascii_string()?.to_ascii_lowercase(),
        "Turret" => info.turrets[0].angle_bone = bone_name_key(info, ini)?,
        "TurretPitch" => info.turrets[0].pitch_bone = bone_name_key(info, ini)?,
        "AltTurret" => info.turrets[1].angle_bone = bone_name_key(info, ini)?,
        "AltTurretPitch" => info.turrets[1].pitch_bone = bone_name_key(info, ini)?,
        "TurretArtAngle" => ini.parse_angle_real(&mut info.turrets[0].art_angle)?,
        "TurretArtPitch" => ini.parse_angle_real(&mut info.turrets[0].art_pitch)?,
        "AltTurretArtAngle" => ini.parse_angle_real(&mut info.turrets[1].art_angle)?,
        "AltTurretArtPitch" => ini.parse_angle_real(&mut info.turrets[1].art_pitch)?,
        "WeaponFireFXBone" => {
            let (slot, name) = weapon_bone_name(info, ini)?;
            info.weapons[slot].fire_fx = name;
        }
        "WeaponRecoilBone" => {
            let (slot, name) = weapon_bone_name(info, ini)?;
            info.weapons[slot].recoil = name;
        }
        "WeaponMuzzleFlash" => {
            let (slot, name) = weapon_bone_name(info, ini)?;
            info.weapons[slot].muzzle_flash = name;
        }
        "WeaponLaunchBone" => {
            let (slot, name) = weapon_bone_name(info, ini)?;
            info.weapons[slot].launch = name;
        }
        // Only its public bone matters here.
        "WeaponHideShowBone" => {
            weapon_bone_name(info, ini)?;
        }
        "Animation" | "IdleAnimation" => {
            let name = ini.next_ascii_string()?.to_ascii_lowercase();
            if info.anims_copied_from_default {
                info.anims_copied_from_default = false;
                info.first_animation = None;
            }
            if info.first_animation.is_none() && !name.is_empty() && name != "none" {
                info.first_animation = Some(name);
            }
        }
        "Flags" => ini.parse_bit_string32(&mut info.flags, &ANIM_FLAG_NAMES)?,
        _ => {}
    }
    Ok(())
}

/// `parseBoneNameKey`.
fn bone_name_key(
    info: &mut ModelConditionBoneSpec,
    ini: &mut IniReader<'_>,
) -> Result<String, IniError> {
    let name = ini.next_token(SEPS_DEFAULT)?.to_ascii_lowercase();
    info.add_public_bone(&name);
    Ok(if name == "none" { String::new() } else { name })
}

/// `parseWeaponBoneName`: the slot and the lower-case bone name.
fn weapon_bone_name(
    info: &mut ModelConditionBoneSpec,
    ini: &mut IniReader<'_>,
) -> Result<(usize, String), IniError> {
    let slot = IniReader::scan_index_list(ini.next_token(SEPS_DEFAULT)?, &WEAPON_SLOT_NAMES)?;
    let mut name = ini.next_ascii_string()?.to_ascii_lowercase();
    if name == "none" {
        name.clear();
    }
    info.add_public_bone(&name);
    Ok((slot as usize, name))
}

/// The module data of a `W3DModelDraw` family module.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDrawModuleData {
    pub base: StubModuleData,
    pub spec: ModelDrawSpec,
}

impl ModuleData for ModelDrawModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.base.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        self.base.facet_mask
    }

    fn update_phase(&self) -> SleepyUpdatePhase {
        self.base.update_phase
    }
}
