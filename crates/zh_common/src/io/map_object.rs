//! PKT-458 `MapObject`: one entry of a map's object list
//! (`Core/GameEngine/Include/Common/MapObject.h`; the bodies are in
//! `WorldHeightMap.cpp`). The render objects, the editor colour and the tree id
//! are client data and are left out. `TheMapObjectListPtr` is a `Vec` in file
//! order, and the thing template is looked up by [`MapObject::name`].
//!
//! Golden test: `crates/zh_logic/tests/golden_s1_terrain.rs`.

use crate::core::dict::Dict;
use crate::core::name_key_generator::NameKeyGenerator;
use crate::math::Coord3D;
use crate::types::{Bool, Int, Real, FALSE, TRUE};

/// `MAP_XY_FACTOR`: the side of a height map cell in world units.
pub const MAP_XY_FACTOR: Real = 10.0;
/// `MAP_HEIGHT_SCALE`.
pub const MAP_HEIGHT_SCALE: Real = MAP_XY_FACTOR / 16.0;

pub const FLAG_DRAWS_IN_MIRROR: Int = 0x0000_0001;
pub const FLAG_ROAD_POINT1: Int = 0x0000_0002;
pub const FLAG_ROAD_POINT2: Int = 0x0000_0004;
pub const FLAG_ROAD_FLAGS: Int = FLAG_ROAD_POINT1 | FLAG_ROAD_POINT2;
pub const FLAG_ROAD_CORNER_ANGLED: Int = 0x0000_0008;
pub const FLAG_BRIDGE_POINT1: Int = 0x0000_0010;
pub const FLAG_BRIDGE_POINT2: Int = 0x0000_0020;
pub const FLAG_BRIDGE_FLAGS: Int = FLAG_BRIDGE_POINT1 | FLAG_BRIDGE_POINT2;
pub const FLAG_ROAD_CORNER_TIGHT: Int = 0x0000_0040;
pub const FLAG_ROAD_JOIN: Int = 0x0000_0080;
pub const FLAG_DONT_RENDER: Int = 0x0000_0100;

const MO_SELECTED: Int = 0x01;
const MO_LIGHT: Int = 0x02;
const MO_WAYPOINT: Int = 0x04;
const MO_SCORCH: Int = 0x08;

/// `normalizeAngle` of `GameCommon.cpp`, into `(-PI, PI]` by repeated steps.
fn normalize_angle(mut angle: Real) -> Real {
    /// `PI`.
    const PI: Real = core::f32::consts::PI;
    if angle.is_nan() {
        return 0.0;
    }
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle <= -PI {
        angle += 2.0 * PI;
    }
    angle
}

/// A map object (`MapObject`).
#[derive(Debug, Clone)]
pub struct MapObject {
    location: Coord3D,
    name: String,
    angle: Real,
    flags: Int,
    properties: Dict,
    runtime_flags: Int,
}

impl MapObject {
    /// The constructor. `props` of `None` fills in the default property sheet.
    pub fn new(
        location: Coord3D,
        name: &str,
        angle: Real,
        flags: Int,
        props: Option<Dict>,
        name_keys: &mut NameKeyGenerator,
    ) -> Self {
        let properties = props.unwrap_or_else(|| {
            let mut d = Dict::new();
            d.set_int(name_keys.name_to_key("objectInitialHealth"), 100);
            d.set_bool(name_keys.name_to_key("objectEnabled"), TRUE);
            d.set_bool(name_keys.name_to_key("objectIndestructible"), FALSE);
            d.set_bool(name_keys.name_to_key("objectUnsellable"), FALSE);
            d.set_bool(name_keys.name_to_key("objectPowered"), TRUE);
            d.set_bool(name_keys.name_to_key("objectRecruitableAI"), TRUE);
            d.set_bool(name_keys.name_to_key("objectTargetable"), FALSE);
            d
        });
        Self {
            location,
            name: name.to_owned(),
            angle: normalize_angle(angle),
            flags,
            properties,
            runtime_flags: 0,
        }
    }

    pub fn location(&self) -> &Coord3D {
        &self.location
    }

    pub fn set_location(&mut self, location: Coord3D) {
        self.location = location;
    }

    pub fn angle(&self) -> Real {
        self.angle
    }

    pub fn set_angle(&mut self, angle: Real) {
        self.angle = normalize_angle(angle);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn properties(&self) -> &Dict {
        &self.properties
    }

    pub fn properties_mut(&mut self) -> &mut Dict {
        &mut self.properties
    }

    pub fn flags(&self) -> Int {
        self.flags
    }

    pub fn flag(&self, flag: Int) -> Bool {
        Bool::from(self.flags & flag != 0)
    }

    pub fn set_flag(&mut self, flag: Int) {
        self.flags |= flag;
    }

    pub fn clear_flag(&mut self, flag: Int) {
        self.flags &= !flag;
    }

    pub fn is_selected(&self) -> Bool {
        Bool::from(self.runtime_flags & MO_SELECTED != 0)
    }

    pub fn set_selected(&mut self, selected: Bool) {
        if selected != FALSE {
            self.runtime_flags |= MO_SELECTED;
        } else {
            self.runtime_flags &= !MO_SELECTED;
        }
    }

    pub fn is_light(&self) -> Bool {
        Bool::from(self.runtime_flags & MO_LIGHT != 0)
    }

    pub fn is_waypoint(&self) -> Bool {
        Bool::from(self.runtime_flags & MO_WAYPOINT != 0)
    }

    pub fn is_scorch(&self) -> Bool {
        Bool::from(self.runtime_flags & MO_SCORCH != 0)
    }

    pub fn set_is_light(&mut self) {
        self.runtime_flags |= MO_LIGHT;
    }

    pub fn set_is_waypoint(&mut self) {
        self.runtime_flags |= MO_WAYPOINT;
    }

    pub fn set_is_scorch(&mut self) {
        self.runtime_flags |= MO_SCORCH;
    }

    pub fn waypoint_id(&self, name_keys: &mut NameKeyGenerator) -> Int {
        self.properties
            .get_int(name_keys.name_to_key("waypointID"), None)
    }

    pub fn waypoint_name(&self, name_keys: &mut NameKeyGenerator) -> String {
        self.properties
            .get_ascii_string(name_keys.name_to_key("waypointName"), None)
    }

    pub fn set_waypoint_id(&mut self, id: Int, name_keys: &mut NameKeyGenerator) {
        self.properties
            .set_int(name_keys.name_to_key("waypointID"), id);
    }

    pub fn set_waypoint_name(&mut self, name: &str, name_keys: &mut NameKeyGenerator) {
        self.properties
            .set_ascii_string(name_keys.name_to_key("waypointName"), name);
    }
}
