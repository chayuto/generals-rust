//! `PKT-022`: Deterministic `GeometryType` and `GeometryInfo` (`GeneralsMD/Code/GameEngine/Source/Common/System/Geometry.cpp`).
//!
//! Reference C++ packet: `packets/phase2/PKT-022_Geometry.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt022.rs` (`docs/golden_vectors/PKT-022.json`)

#![allow(unused_variables, dead_code)]

use crate::math::{clamp_f32, gm_atan2f, gm_cosf, gm_sinf, gm_sqrt, gm_sqrtf, Coord3D, Region2D, PI};
use crate::random_value::PrngState;
use crate::types::{Bool, Int, Real, XferVersion, FALSE, TRUE};
use crate::xfer::{Snapshot, Xfer, XferError};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GeometryType {
    #[default]
    Sphere = 0,
    Cylinder = 1,
    Box = 2,
}

impl GeometryType {
    #[inline]
    pub const fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Sphere,
            1 => Self::Cylinder,
            2 => Self::Box,
            _ => Self::Sphere,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

pub const GEOMETRY_NAMES: [&str; 3] = ["SPHERE", "CYLINDER", "BOX"];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GeometryInfo {
    geom_type: GeometryType,
    is_small: Bool,
    height: Real,
    major_radius: Real,
    minor_radius: Real,
    bounding_circle_radius: Real,
    bounding_sphere_radius: Real,
}

impl GeometryInfo {
    pub fn new(
        geom_type: GeometryType,
        is_small: Bool,
        height: Real,
        major_radius: Real,
        minor_radius: Real,
    ) -> Self {
        let mut info = Self::default();
        info.set(geom_type, is_small, height, major_radius, minor_radius);
        info
    }

    pub fn set(
        &mut self,
        geom_type: GeometryType,
        is_small: Bool,
        height: Real,
        major_radius: Real,
        minor_radius: Real,
    ) {
        let _ = (geom_type, is_small, height, major_radius, minor_radius);
        todo!("PKT-022: implement GeometryInfo::set & calcBoundingStuff")
    }

    pub fn set_major_radius(&mut self, major_radius: Real) {
        todo!("PKT-022: implement GeometryInfo::setMajorRadius")
    }

    pub fn set_minor_radius(&mut self, minor_radius: Real) {
        todo!("PKT-022: implement GeometryInfo::setMinorRadius")
    }

    #[inline]
    pub const fn get_geom_type(&self) -> GeometryType {
        self.geom_type
    }

    #[inline]
    pub const fn get_is_small(&self) -> Bool {
        self.is_small
    }

    #[inline]
    pub const fn get_major_radius(&self) -> Real {
        self.major_radius
    }

    #[inline]
    pub const fn get_minor_radius(&self) -> Real {
        self.minor_radius
    }

    #[inline]
    pub const fn get_bounding_circle_radius(&self) -> Real {
        self.bounding_circle_radius
    }

    #[inline]
    pub const fn get_bounding_sphere_radius(&self) -> Real {
        self.bounding_sphere_radius
    }

    pub fn is_intersected_by_line_segment(
        &self,
        loc: &Coord3D,
        from: &Coord3D,
        to: &Coord3D,
    ) -> Bool {
        let _ = (loc, from, to, FALSE, TRUE);
        todo!("PKT-022: implement GeometryInfo::isIntersectedByLineSegment")
    }

    pub fn get_footprint_area(&self) -> Real {
        let _ = PI;
        todo!("PKT-022: implement GeometryInfo::getFootprintArea")
    }

    pub fn get_max_height_above_position(&self) -> Real {
        todo!("PKT-022: implement GeometryInfo::getMaxHeightAbovePosition")
    }

    pub fn get_max_height_below_position(&self) -> Real {
        todo!("PKT-022: implement GeometryInfo::getMaxHeightBelowPosition")
    }

    pub fn get_z_delta_to_center_position(&self) -> Real {
        todo!("PKT-022: implement GeometryInfo::getZDeltaToCenterPosition")
    }

    pub fn get_center_position(&self, pos: &Coord3D, center: &mut Coord3D) {
        let _ = (pos, center);
        todo!("PKT-022: implement GeometryInfo::getCenterPosition")
    }

    pub fn calc_pitches(
        &self,
        this_pos: &Coord3D,
        that: &GeometryInfo,
        that_pos: &Coord3D,
        min_pitch: &mut Real,
        max_pitch: &mut Real,
    ) {
        let _ = (this_pos, that, that_pos, min_pitch, max_pitch, gm_atan2f, gm_sqrtf);
        todo!("PKT-022: implement GeometryInfo::calcPitches")
    }

    pub fn expand_footprint(&mut self, radius: Real) {
        let _ = radius;
        todo!("PKT-022: implement GeometryInfo::expandFootprint")
    }

    pub fn get_2d_bounds(&self, geom_center: &Coord3D, angle: Real, bounds: &mut Region2D) {
        let _ = (geom_center, angle, bounds, gm_cosf, gm_sinf);
        todo!("PKT-022: implement GeometryInfo::get2DBounds")
    }

    pub fn make_random_offset_within_footprint(&self, pt: &mut Coord3D, rng: &mut PrngState) {
        let _ = (pt, rng);
        todo!("PKT-022: implement GeometryInfo::makeRandomOffsetWithinFootprint")
    }

    pub fn make_random_offset_on_perimeter(&self, pt: &mut Coord3D, rng: &mut PrngState) {
        let _ = (pt, rng);
        todo!("PKT-022: implement GeometryInfo::makeRandomOffsetOnPerimeter")
    }

    pub fn clip_point_to_footprint(&self, geom_center: &Coord3D, pt_to_clip: &mut Coord3D) {
        let _ = (geom_center, pt_to_clip, clamp_f32);
        todo!("PKT-022: implement GeometryInfo::clipPointToFootprint")
    }

    pub fn is_point_in_footprint(&self, geom_center: &Coord3D, pt: &Coord3D) -> Bool {
        let _ = (geom_center, pt);
        todo!("PKT-022: implement GeometryInfo::isPointInFootprint")
    }

    pub fn set_max_height_above_position(&mut self, z: Real) {
        let _ = (z, gm_sqrt);
        todo!("PKT-022: implement GeometryInfo::setMaxHeightAbovePosition")
    }
}

impl Snapshot for GeometryInfo {
    fn crc(&self, _xfer: &mut dyn Xfer) {
        // GeometryInfo::crc is empty in C++
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut type_raw: Int = self.geom_type.as_i32();
        xfer.xfer_enum_i32(&mut type_raw)?;
        self.geom_type = GeometryType::from_i32(type_raw);
        xfer.xfer_bool(&mut self.is_small)?;
        xfer.xfer_real(&mut self.height)?;
        xfer.xfer_real(&mut self.major_radius)?;
        xfer.xfer_real(&mut self.minor_radius)?;
        xfer.xfer_real(&mut self.bounding_circle_radius)?;
        xfer.xfer_real(&mut self.bounding_sphere_radius)?;
        Ok(())
    }
}
