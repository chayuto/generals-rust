//! `PKT-109`: `INI::parseTerrainBridgeDefinition` (`Common/INI/INITerrainBridge.cpp`) and
//! the bridge field table of `TerrainRoadType` (`GameClient/Terrain/TerrainRoads.cpp`).
//! The types are in [`super::ini_terrain_road`].
//!
//! No packet document; the oracle is `tests/cpp_harness/gen_game_data_load.cpp` and the
//! golden test `crates/zh_logic/tests/golden_game_data_load.rs`.

use super::ini_terrain_road::{TerrainRoadCollection, TerrainRoadType, MAX_BRIDGE_BODY_FX};
use super::{IniError, IniReader, SEPS_DEFAULT};

/// `TheBodyDamageTypeNames`.
const BODY_DAMAGE_NAMES: [&str; 4] = ["PRISTINE", "DAMAGED", "REALLYDAMAGED", "RUBBLE"];
/// `BODY_DAMAGED`.
const BODY_DAMAGED: usize = 1;

/// The part `parseTransitionToOCL` and `parseTransitionToFX` share:
/// `Transition:<Damage|Repair> ToState:<state> EffectNum:<n> <kind>:<name>`.
fn parse_transition(
    ini: &mut IniReader<'_>,
    kind: &str,
) -> Result<(bool, usize, usize, String), IniError> {
    let transition = ini.next_sub_token("Transition")?;
    let damage = if transition.eq_ignore_ascii_case("Damage") {
        true
    } else if transition.eq_ignore_ascii_case("Repair") {
        false
    } else {
        return Err(IniError::InvalidData);
    };
    let state = IniReader::scan_index_list(ini.next_sub_token("ToState")?, &BODY_DAMAGE_NAMES)?;
    let effect = IniReader::scan_int(ini.next_sub_token("EffectNum")?)?.wrapping_sub(1);
    if effect < 0 || effect >= MAX_BRIDGE_BODY_FX as i32 {
        return Err(IniError::InvalidData);
    }
    let name = ini.next_sub_token(kind)?.to_owned();
    Ok((damage, state as usize, effect as usize, name))
}

fn parse_bridge_field(
    bridge: &mut TerrainRoadType,
    key: &str,
    ini: &mut IniReader<'_>,
) -> Result<bool, IniError> {
    let text = match key {
        "ScaffoldObjectName" => &mut bridge.scaffold_object_name,
        "ScaffoldSupportObjectName" => &mut bridge.scaffold_support_object_name,
        "BridgeModelName" => &mut bridge.bridge_model_name,
        "Texture" => &mut bridge.texture,
        "BridgeModelNameDamaged" => &mut bridge.bridge_model_name_damaged,
        "TextureDamaged" => &mut bridge.texture_damaged,
        "BridgeModelNameReallyDamaged" => &mut bridge.bridge_model_name_really_damaged,
        "TextureReallyDamaged" => &mut bridge.texture_really_damaged,
        "BridgeModelNameBroken" => &mut bridge.bridge_model_name_broken,
        "TextureBroken" => &mut bridge.texture_broken,
        "TowerObjectNameFromLeft" => &mut bridge.tower_object_name[0],
        "TowerObjectNameFromRight" => &mut bridge.tower_object_name[1],
        "TowerObjectNameToLeft" => &mut bridge.tower_object_name[2],
        "TowerObjectNameToRight" => &mut bridge.tower_object_name[3],
        "DamagedToSound" => &mut bridge.damage_to_sound[BODY_DAMAGED],
        "RepairedToSound" => &mut bridge.repaired_to_sound[BODY_DAMAGED],
        _ => {
            match key {
                "BridgeScale" => ini.parse_real(&mut bridge.bridge_scale)?,
                "RadarColor" => ini.parse_rgb_color(&mut bridge.radar_color)?,
                "TransitionEffectsHeight" => {
                    ini.parse_real(&mut bridge.transition_effects_height)?
                }
                "NumFXPerType" => ini.parse_int(&mut bridge.num_fx_per_type)?,
                "TransitionToOCL" => {
                    let (damage, state, effect, name) = parse_transition(ini, "OCL")?;
                    let table = if damage {
                        &mut bridge.damage_to_ocl
                    } else {
                        &mut bridge.repaired_to_ocl
                    };
                    table[state][effect] = name;
                }
                "TransitionToFX" => {
                    let (damage, state, effect, name) = parse_transition(ini, "FX")?;
                    let table = if damage {
                        &mut bridge.damage_to_fx
                    } else {
                        &mut bridge.repaired_to_fx
                    };
                    table[state][effect] = name;
                }
                _ => return Ok(false),
            }
            return Ok(true);
        }
    };
    ini.parse_ascii_string(text)?;
    Ok(true)
}

impl TerrainRoadCollection {
    /// `INI::parseTerrainBridgeDefinition`, from the token after `Bridge`. A second
    /// definition of a bridge is an error whatever the load type.
    pub fn parse_bridge_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        if self.find_bridge(&name).is_some() {
            return Err(IniError::InvalidData);
        }
        let bridge = self.new_bridge(&name);
        ini.read_block_fields(|key, ini| {
            if !parse_bridge_field(bridge, key, ini)? {
                ignored.push(key.to_owned());
            }
            Ok(())
        })
    }
}
