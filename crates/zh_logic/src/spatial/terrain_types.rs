//! `PKT-157` & `PKT-108`: `TerrainType`, `TerrainTypeCollection`, `INITerrain`
//! (`Core/GameEngine/Include/Common/TerrainTypes.h`,
//! `Core/GameEngine/Source/Common/TerrainTypes.cpp`, &
//! `Core/GameEngine/Source/Common/INI/INITerrain.cpp`).

use zh_common::ini::{IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, Int, FALSE};

pub const TERRAIN_NUM_CLASSES: usize = 38;

/// Zero Hour `terrainTypeNames[]` lookup table (`TerrainTypes.h`).
pub const TERRAIN_TYPE_NAMES: [&str; TERRAIN_NUM_CLASSES] = [
    "NONE",
    "DESERT_1",
    "DESERT_2",
    "DESERT_3",
    "EASTERN_EUROPE_1",
    "EASTERN_EUROPE_2",
    "EASTERN_EUROPE_3",
    "SWISS_1",
    "SWISS_2",
    "SWISS_3",
    "SNOW_1",
    "SNOW_2",
    "SNOW_3",
    "DIRT",
    "GRASS",
    "TRANSITION",
    "ROCK",
    "SAND",
    "CLIFF",
    "WOOD",
    "BLEND_EDGE",
    "DESERT_LIVE",
    "DESERT_DRY",
    "SAND_ACCENT",
    "BEACH_TROPICAL",
    "BEACH_PARK",
    "MOUNTAIN_RUGGED",
    "GRASS_COBBLESTONE",
    "GRASS_ACCENT",
    "RESIDENTIAL",
    "SNOW_RUGGED",
    "SNOW_FLAT",
    "FIELD",
    "ASPHALT",
    "CONCRETE",
    "CHINA",
    "ROCK_ACCENT",
    "URBAN",
];

/// Terrain classification enum matching `TerrainClass` (`TerrainTypes.h`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TerrainClass {
    #[default]
    None = 0,
    Desert1 = 1,
    Desert2 = 2,
    Desert3 = 3,
    EasternEurope1 = 4,
    EasternEurope2 = 5,
    EasternEurope3 = 6,
    Swiss1 = 7,
    Swiss2 = 8,
    Swiss3 = 9,
    Snow1 = 10,
    Snow2 = 11,
    Snow3 = 12,
    Dirt = 13,
    Grass = 14,
    Transition = 15,
    Rock = 16,
    Sand = 17,
    Cliff = 18,
    Wood = 19,
    BlendEdges = 20,
    LiveDesert = 21,
    DryDesert = 22,
    AccentSand = 23,
    TropicalBeach = 24,
    BeachPark = 25,
    RuggedMountain = 26,
    CobblestoneGrass = 27,
    AccentGrass = 28,
    Residential = 29,
    RuggedSnow = 30,
    FlatSnow = 31,
    Field = 32,
    Asphalt = 33,
    Concrete = 34,
    China = 35,
    AccentRock = 36,
    Urban = 37,
}

impl TerrainClass {
    #[inline]
    pub const fn from_i32(value: Int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Desert1),
            2 => Some(Self::Desert2),
            3 => Some(Self::Desert3),
            4 => Some(Self::EasternEurope1),
            5 => Some(Self::EasternEurope2),
            6 => Some(Self::EasternEurope3),
            7 => Some(Self::Swiss1),
            8 => Some(Self::Swiss2),
            9 => Some(Self::Swiss3),
            10 => Some(Self::Snow1),
            11 => Some(Self::Snow2),
            12 => Some(Self::Snow3),
            13 => Some(Self::Dirt),
            14 => Some(Self::Grass),
            15 => Some(Self::Transition),
            16 => Some(Self::Rock),
            17 => Some(Self::Sand),
            18 => Some(Self::Cliff),
            19 => Some(Self::Wood),
            20 => Some(Self::BlendEdges),
            21 => Some(Self::LiveDesert),
            22 => Some(Self::DryDesert),
            23 => Some(Self::AccentSand),
            24 => Some(Self::TropicalBeach),
            25 => Some(Self::BeachPark),
            26 => Some(Self::RuggedMountain),
            27 => Some(Self::CobblestoneGrass),
            28 => Some(Self::AccentGrass),
            29 => Some(Self::Residential),
            30 => Some(Self::RuggedSnow),
            31 => Some(Self::FlatSnow),
            32 => Some(Self::Field),
            33 => Some(Self::Asphalt),
            34 => Some(Self::Concrete),
            35 => Some(Self::China),
            36 => Some(Self::AccentRock),
            37 => Some(Self::Urban),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Terrain definition entry matching C++ `TerrainType` (`TerrainTypes.h`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainType {
    pub name: String,
    pub texture: String,
    pub blend_edge_texture: Bool,
    pub class: TerrainClass,
    pub restrict_construction: Bool,
}

impl Default for TerrainType {
    fn default() -> Self {
        Self::new()
    }
}

impl TerrainType {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            texture: String::new(),
            blend_edge_texture: FALSE,
            class: TerrainClass::None,
            restrict_construction: FALSE,
        }
    }

    pub fn parse_from_ini(&mut self, reader: &mut IniReader<'_>) -> Result<(), IniError> {
        loop {
            reader.read_line(None)?;
            let field = reader
                .next_token_or_null(SEPS_DEFAULT)
                .map(str::to_string);

            if let Some(field_name) = field {
                if field_name.eq_ignore_ascii_case("End")
                    || IniReader::is_end_of_block(reader.current_line_bytes()) != FALSE
                {
                    if reader.is_eof() && !field_name.eq_ignore_ascii_case("End") {
                        return Err(IniError::MissingEndToken);
                    }
                    break;
                }
                if field_name.eq_ignore_ascii_case("Texture") {
                    reader.parse_ascii_string(&mut self.texture)?;
                } else if field_name.eq_ignore_ascii_case("BlendEdges") {
                    reader.parse_bool(&mut self.blend_edge_texture)?;
                } else if field_name.eq_ignore_ascii_case("Class") {
                    let mut idx: Int = 0;
                    reader.parse_index_list(&mut idx, &TERRAIN_TYPE_NAMES)?;
                    self.class = TerrainClass::from_i32(idx).ok_or(IniError::InvalidData)?;
                } else if field_name.eq_ignore_ascii_case("RestrictConstruction") {
                    reader.parse_bool(&mut self.restrict_construction)?;
                } else {
                    return Err(IniError::UnknownToken);
                }
            }

            if reader.is_eof() {
                return Err(IniError::MissingEndToken);
            }
        }
        Ok(())
    }
}

/// Collection of terrain types matching `TerrainTypeCollection` (`TerrainTypes.cpp`).
/// Preserves C++ head-insertion LIFO list order (`newTerrain` inserts at index 0).
#[derive(Debug, Clone, Default)]
pub struct TerrainTypeCollection {
    terrains: Vec<TerrainType>,
}

impl TerrainTypeCollection {
    pub fn new() -> Self {
        Self {
            terrains: Vec::new(),
        }
    }

    pub fn init(&mut self) {}
    pub fn reset(&mut self) {}
    pub fn update(&mut self) {}

    pub fn find_terrain(&self, name: &str) -> Option<&TerrainType> {
        self.terrains.iter().find(|t| t.name == name)
    }

    pub fn find_terrain_mut(&mut self, name: &str) -> Option<&mut TerrainType> {
        self.terrains.iter_mut().find(|t| t.name == name)
    }

    /// Allocates a new terrain entry copying `"DefaultTerrain"` if present and inserting at head.
    pub fn new_terrain(&mut self, name: &str) -> &mut TerrainType {
        let mut entry = if let Some(def) = self.find_terrain("DefaultTerrain") {
            def.clone()
        } else {
            TerrainType::new()
        };
        entry.name = name.to_string();
        self.terrains.insert(0, entry);
        &mut self.terrains[0]
    }

    /// Returns all entries in head-to-tail order matching `firstTerrain()` / `nextTerrain()`.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, TerrainType> {
        self.terrains.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.terrains.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.terrains.is_empty()
    }

    /// Parses one `Terrain <name> ... End` block matching `INI::parseTerrainDefinition` (`INITerrain.cpp`).
    pub fn parse_terrain_definition(&mut self, reader: &mut IniReader<'_>) -> Result<(), IniError> {
        let name = reader.next_token(SEPS_DEFAULT)?.to_string();
        if let Some(idx) = self.terrains.iter().position(|t| t.name == name) {
            self.terrains[idx].parse_from_ini(reader)
        } else {
            let entry = self.new_terrain(&name);
            entry.parse_from_ini(reader)
        }
    }

    /// Loads an INI buffer containing `"Terrain"` blocks.
    pub fn load_from_ini(&mut self, ini_bytes: &[u8]) -> Result<(), IniError> {
        let mut reader = IniReader::new("terrain.ini", ini_bytes, IniLoadType::Overwrite);
        reader.load_blocks(None, |block, r| {
            let _block_kw = r.next_token(SEPS_DEFAULT)?;
            if block.eq_ignore_ascii_case("Terrain") {
                self.parse_terrain_definition(r)
            } else {
                Err(IniError::UnknownToken)
            }
        })?;
        Ok(())
    }
}
