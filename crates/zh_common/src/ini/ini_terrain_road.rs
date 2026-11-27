//! `PKT-110`: `INI::parseTerrainRoadDefinition` (`Common/INI/INITerrainRoad.cpp`) with
//! the `TerrainRoadType` and `TerrainRoadCollection` it fills
//! (`GameClient/Terrain/TerrainRoads.cpp`). Bridges: [`super::ini_terrain_bridge`].
//!
//! No packet document; the oracle is `tests/cpp_harness/gen_game_data_load.cpp` and the
//! golden test `crates/zh_logic/tests/golden_game_data_load.rs`.
//!
//! A key outside a field table is skipped, as the reference build does (`DEBUG_CRASH`
//! only), and appended to `ignored`.

use super::{IniError, IniReader, SEPS_DEFAULT};
use crate::math::RGBColor;
use crate::types::{Int, Real, UnsignedInt};

/// `BODYDAMAGETYPE_COUNT`.
pub const BRIDGE_BODY_STATES: usize = 4;
/// `MAX_BRIDGE_BODY_FX`.
pub const MAX_BRIDGE_BODY_FX: usize = 3;
/// `BRIDGE_MAX_TOWERS`: from left, from right, to left, to right.
pub const BRIDGE_MAX_TOWERS: usize = 4;

/// One effect name per body damage state and effect number.
pub type BridgeTransitionNames = [[String; MAX_BRIDGE_BODY_FX]; BRIDGE_BODY_STATES];

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainRoadType {
    pub name: String,
    pub is_bridge: bool,
    pub id: UnsignedInt,
    pub texture: String,
    pub road_width: Real,
    pub road_width_in_texture: Real,
    pub bridge_scale: Real,
    pub scaffold_object_name: String,
    pub scaffold_support_object_name: String,
    pub radar_color: RGBColor,
    pub transition_effects_height: Real,
    pub num_fx_per_type: Int,
    pub bridge_model_name: String,
    pub bridge_model_name_damaged: String,
    pub bridge_model_name_really_damaged: String,
    pub bridge_model_name_broken: String,
    pub texture_damaged: String,
    pub texture_really_damaged: String,
    pub texture_broken: String,
    pub tower_object_name: [String; BRIDGE_MAX_TOWERS],
    pub damage_to_sound: [String; BRIDGE_BODY_STATES],
    pub repaired_to_sound: [String; BRIDGE_BODY_STATES],
    pub damage_to_ocl: BridgeTransitionNames,
    pub damage_to_fx: BridgeTransitionNames,
    pub repaired_to_ocl: BridgeTransitionNames,
    pub repaired_to_fx: BridgeTransitionNames,
}

impl TerrainRoadType {
    /// `TerrainRoadType::TerrainRoadType` with the name, ID and kind set.
    fn new(name: &str, id: UnsignedInt, is_bridge: bool) -> Self {
        Self {
            name: name.to_owned(),
            is_bridge,
            id,
            texture: String::new(),
            road_width: 0.0,
            road_width_in_texture: 0.0,
            bridge_scale: 1.0,
            scaffold_object_name: String::new(),
            scaffold_support_object_name: String::new(),
            radar_color: RGBColor {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            },
            transition_effects_height: 0.0,
            num_fx_per_type: 0,
            bridge_model_name: String::new(),
            bridge_model_name_damaged: String::new(),
            bridge_model_name_really_damaged: String::new(),
            bridge_model_name_broken: String::new(),
            texture_damaged: String::new(),
            texture_really_damaged: String::new(),
            texture_broken: String::new(),
            tower_object_name: Default::default(),
            damage_to_sound: Default::default(),
            repaired_to_sound: Default::default(),
            damage_to_ocl: Default::default(),
            damage_to_fx: Default::default(),
            repaired_to_ocl: Default::default(),
            repaired_to_fx: Default::default(),
        }
    }

    fn parse_road_field(&mut self, key: &str, ini: &mut IniReader<'_>) -> Result<bool, IniError> {
        match key {
            "Texture" => ini.parse_ascii_string(&mut self.texture)?,
            "RoadWidth" => ini.parse_real(&mut self.road_width)?,
            "RoadWidthInTexture" => ini.parse_real(&mut self.road_width_in_texture)?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

/// `TheTerrainRoads`. Both lists are kept from the head, the newest first.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainRoadCollection {
    pub roads: Vec<TerrainRoadType>,
    pub bridges: Vec<TerrainRoadType>,
    id_counter: UnsignedInt,
}

impl Default for TerrainRoadCollection {
    fn default() -> Self {
        Self {
            roads: Vec::new(),
            bridges: Vec::new(),
            id_counter: 1,
        }
    }
}

impl TerrainRoadCollection {
    pub fn find_road(&self, name: &str) -> Option<&TerrainRoadType> {
        self.roads.iter().find(|r| r.name == name)
    }

    pub fn find_bridge(&self, name: &str) -> Option<&TerrainRoadType> {
        self.bridges.iter().find(|r| r.name == name)
    }

    fn next_id(&mut self) -> UnsignedInt {
        let id = self.id_counter;
        self.id_counter = self.id_counter.wrapping_add(1);
        id
    }

    /// `newRoad`: starts from `DefaultRoad`'s texture and widths.
    fn new_road(&mut self, name: &str) -> &mut TerrainRoadType {
        let mut road = TerrainRoadType::new(name, self.next_id(), false);
        if let Some(default) = self.find_road("DefaultRoad") {
            road.texture = default.texture.clone();
            road.road_width = default.road_width;
            road.road_width_in_texture = default.road_width_in_texture;
        }
        self.roads.insert(0, road);
        &mut self.roads[0]
    }

    /// `newBridge`. Of `DefaultBridge` it copies the looks only: the loops meant to copy
    /// the sounds, effects and creation lists read the new bridge itself.
    pub(super) fn new_bridge(&mut self, name: &str) -> &mut TerrainRoadType {
        let mut bridge = TerrainRoadType::new(name, self.next_id(), true);
        if let Some(default) = self.find_bridge("DefaultBridge") {
            bridge.texture = default.texture.clone();
            bridge.bridge_scale = default.bridge_scale;
            bridge.bridge_model_name = default.bridge_model_name.clone();
            bridge.bridge_model_name_damaged = default.bridge_model_name_damaged.clone();
            bridge.bridge_model_name_really_damaged =
                default.bridge_model_name_really_damaged.clone();
            bridge.bridge_model_name_broken = default.bridge_model_name_broken.clone();
            bridge.texture_damaged = default.texture_damaged.clone();
            bridge.texture_really_damaged = default.texture_really_damaged.clone();
            bridge.texture_broken = default.texture_broken.clone();
            bridge.transition_effects_height = default.transition_effects_height;
            bridge.num_fx_per_type = default.num_fx_per_type;
        }
        self.bridges.insert(0, bridge);
        &mut self.bridges[0]
    }

    /// `INI::parseTerrainRoadDefinition`, from the token after `Road`. A second
    /// definition of a road is an error whatever the load type.
    pub fn parse_road_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        if self.find_road(&name).is_some() {
            return Err(IniError::InvalidData);
        }
        let road = self.new_road(&name);
        ini.read_block_fields(|key, ini| {
            if !road.parse_road_field(key, ini)? {
                ignored.push(key.to_owned());
            }
            Ok(())
        })
    }
}
