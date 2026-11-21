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
        self.geom_type = geom_type;
        self.is_small = is_small;

        match geom_type {
            GeometryType::Sphere => {
                self.major_radius = major_radius;
                self.minor_radius = major_radius;
                self.height = major_radius;
            }
            GeometryType::Cylinder => {
                self.major_radius = major_radius;
                self.minor_radius = major_radius;
                self.height = height;
            }
            GeometryType::Box => {
                self.major_radius = major_radius;
                self.minor_radius = minor_radius;
                self.height = height;
            }
        }

        self.calc_bounding_stuff();
    }

    pub fn set_major_radius(&mut self, major_radius: Real) {
        self.major_radius = major_radius;
        self.calc_bounding_stuff();
    }

    pub fn set_minor_radius(&mut self, minor_radius: Real) {
        self.minor_radius = minor_radius;
        self.calc_bounding_stuff();
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
        let line = Coord3D::new(to.x - from.x, to.y - from.y, to.z - from.z);
        let line_to_pt = Coord3D::new(loc.x - from.x, loc.y - from.y, loc.z - from.z);

        let dot = calc_dot_product(&line_to_pt, &line);
        let dist_squared = if dot <= 0.0 {
            calc_dist_squared(loc, from)
        } else {
            let line_len_squared = calc_dist_squared(from, to);
            if line_len_squared <= dot {
                calc_dist_squared(loc, to)
            } else {
                let tmp = dot / line_len_squared;
                let closest = Coord3D::new(
                    from.x + tmp * line.x,
                    from.y + tmp * line.y,
                    from.z + tmp * line.z,
                );
                calc_dist_squared(loc, &closest)
            }
        };

        if dist_squared <= self.bounding_sphere_radius * self.bounding_sphere_radius {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn get_footprint_area(&self) -> Real {
        match self.geom_type {
            GeometryType::Sphere | GeometryType::Cylinder => {
                PI * (self.bounding_circle_radius * self.bounding_circle_radius)
            }
            GeometryType::Box => 4.0 * self.major_radius * self.minor_radius,
        }
    }

    pub fn get_max_height_above_position(&self) -> Real {
        match self.geom_type {
            GeometryType::Sphere => self.major_radius,
            GeometryType::Cylinder | GeometryType::Box => self.height,
        }
    }

    pub fn get_max_height_below_position(&self) -> Real {
        match self.geom_type {
            GeometryType::Sphere => self.major_radius,
            GeometryType::Cylinder | GeometryType::Box => 0.0,
        }
    }

    pub fn get_z_delta_to_center_position(&self) -> Real {
        if self.geom_type == GeometryType::Sphere {
            0.0
        } else {
            self.height * 0.5
        }
    }

    pub fn get_center_position(&self, pos: &Coord3D, center: &mut Coord3D) {
        *center = *pos;
        center.z += self.get_z_delta_to_center_position();
    }

    pub fn calc_pitches(
        &self,
        this_pos: &Coord3D,
        that: &GeometryInfo,
        that_pos: &Coord3D,
        min_pitch: &mut Real,
        max_pitch: &mut Real,
    ) {
        let mut this_center = Coord3D::default();
        self.get_center_position(this_pos, &mut this_center);

        let dx = that_pos.x - this_center.x;
        let dy = that_pos.y - this_center.y;
        let dxy = gm_sqrtf((dx * dx) + (dy * dy));

        let dz = (that_pos.z + that.get_max_height_above_position()) - this_center.z;
        *max_pitch = gm_atan2f(dz, dxy);

        let dz = (that_pos.z - that.get_max_height_below_position()) - this_center.z;
        *min_pitch = gm_atan2f(dz, dxy);
    }

    pub fn expand_footprint(&mut self, radius: Real) {
        self.major_radius += radius;
        self.minor_radius += radius;
        self.calc_bounding_stuff();
    }

    pub fn get_2d_bounds(&self, geom_center: &Coord3D, angle: Real, bounds: &mut Region2D) {
        match self.geom_type {
            GeometryType::Sphere | GeometryType::Cylinder => {
                bounds.lo.x = geom_center.x - self.major_radius;
                bounds.lo.y = geom_center.y - self.major_radius;
                bounds.hi.x = geom_center.x + self.major_radius;
                bounds.hi.y = geom_center.y + self.major_radius;
            }
            GeometryType::Box => {
                let c = gm_cosf(angle);
                let s = gm_sinf(angle);
                let exc = self.major_radius * c;
                let eyc = self.minor_radius * c;
                let exs = self.major_radius * s;
                let eys = self.minor_radius * s;

                let mut x = geom_center.x - exc - eys;
                let mut y = geom_center.y + eyc - exs;
                bounds.lo.x = x;
                bounds.lo.y = y;
                bounds.hi.x = x;
                bounds.hi.y = y;

                x = geom_center.x + exc - eys;
                y = geom_center.y + eyc + exs;
                if bounds.lo.x > x { bounds.lo.x = x; }
                if bounds.lo.y > y { bounds.lo.y = y; }
                if bounds.hi.x < x { bounds.hi.x = x; }
                if bounds.hi.y < y { bounds.hi.y = y; }

                x = geom_center.x + exc + eys;
                y = geom_center.y - eyc + exs;
                if bounds.lo.x > x { bounds.lo.x = x; }
                if bounds.lo.y > y { bounds.lo.y = y; }
                if bounds.hi.x < x { bounds.hi.x = x; }
                if bounds.hi.y < y { bounds.hi.y = y; }

                x = geom_center.x - exc + eys;
                y = geom_center.y - eyc - exs;
                if bounds.lo.x > x { bounds.lo.x = x; }
                if bounds.lo.y > y { bounds.lo.y = y; }
                if bounds.hi.x < x { bounds.hi.x = x; }
                if bounds.hi.y < y { bounds.hi.y = y; }
            }
        }
    }

    pub fn make_random_offset_within_footprint(&self, pt: &mut Coord3D, rng: &mut PrngState) {
        match self.geom_type {
            GeometryType::Sphere | GeometryType::Cylinder => {
                let max_dist_squared = self.major_radius * self.major_radius;
                loop {
                    pt.x = rng.get_game_logic_random_value_real(-self.major_radius, self.major_radius);
                    pt.y = rng.get_game_logic_random_value_real(-self.major_radius, self.major_radius);
                    pt.z = 0.0;
                    let dist_squared = (pt.x * pt.x) + (pt.y * pt.y);
                    if dist_squared <= max_dist_squared {
                        break;
                    }
                }
            }
            GeometryType::Box => {
                pt.x = rng.get_game_logic_random_value_real(-self.major_radius, self.major_radius);
                pt.y = rng.get_game_logic_random_value_real(-self.minor_radius, self.minor_radius);
                pt.z = 0.0;
            }
        }
    }

    pub fn make_random_offset_on_perimeter(&self, pt: &mut Coord3D, rng: &mut PrngState) {
        match self.geom_type {
            GeometryType::Sphere | GeometryType::Cylinder => {
                pt.x = 0.0;
                pt.y = 0.0;
            }
            GeometryType::Box => {
                if rng.get_game_logic_random_value_real(0.0, 1.0) < 0.5 {
                    pt.x = rng.get_game_logic_random_value_real(-self.major_radius, self.major_radius);
                    if rng.get_game_logic_random_value_real(0.0, 1.0) < 0.5 {
                        pt.y = -self.minor_radius;
                    } else {
                        pt.y = self.minor_radius;
                    }
                } else {
                    pt.y = rng.get_game_logic_random_value_real(-self.minor_radius, self.minor_radius);
                    if rng.get_game_logic_random_value_real(0.0, 1.0) < 0.5 {
                        pt.x = -self.major_radius;
                    } else {
                        pt.x = self.major_radius;
                    }
                }
                pt.z = 0.0;
            }
        }
    }

    pub fn clip_point_to_footprint(&self, geom_center: &Coord3D, pt_to_clip: &mut Coord3D) {
        match self.geom_type {
            GeometryType::Sphere | GeometryType::Cylinder => {
                let dx = pt_to_clip.x - geom_center.x;
                let dy = pt_to_clip.y - geom_center.y;
                let radius = gm_sqrtf((dx * dx) + (dy * dy));
                if radius > self.major_radius {
                    let ratio = self.major_radius / radius;
                    pt_to_clip.x = geom_center.x + dx * ratio;
                    pt_to_clip.y = geom_center.y + dy * ratio;
                }
            }
            GeometryType::Box => {
                pt_to_clip.x = clamp_f32(pt_to_clip.x, geom_center.x - self.major_radius, geom_center.x + self.major_radius);
                pt_to_clip.y = clamp_f32(pt_to_clip.y, geom_center.y - self.minor_radius, geom_center.y + self.minor_radius);
            }
        }
    }

    pub fn is_point_in_footprint(&self, geom_center: &Coord3D, pt: &Coord3D) -> Bool {
        match self.geom_type {
            GeometryType::Sphere | GeometryType::Cylinder => {
                let dx = pt.x - geom_center.x;
                let dy = pt.y - geom_center.y;
                let radius = gm_sqrtf((dx * dx) + (dy * dy));
                if radius <= self.major_radius { TRUE } else { FALSE }
            }
            GeometryType::Box => {
                if is_within(geom_center.x - self.major_radius, pt.x, geom_center.x + self.major_radius)
                    && is_within(geom_center.y - self.minor_radius, pt.y, geom_center.y + self.minor_radius)
                { TRUE } else { FALSE }
            }
        }
    }

    pub fn set_max_height_above_position(&mut self, z: Real) {
        match self.geom_type {
            GeometryType::Sphere => self.major_radius = z,
            GeometryType::Cylinder | GeometryType::Box => self.height = z,
        }
        self.calc_bounding_stuff();
    }

    fn calc_bounding_stuff(&mut self) {
        match self.geom_type {
            GeometryType::Sphere => {
                self.bounding_sphere_radius = self.major_radius;
                self.bounding_circle_radius = self.major_radius;
            }
            GeometryType::Cylinder => {
                self.bounding_circle_radius = self.major_radius;
                let half_h = (self.height as f64) * 0.5;
                self.bounding_sphere_radius = half_h as Real;
                if self.bounding_sphere_radius < self.major_radius {
                    self.bounding_sphere_radius = self.major_radius;
                }
            }
            GeometryType::Box => {
                let circle_sum = (self.major_radius * self.major_radius) + (self.minor_radius * self.minor_radius);
                self.bounding_circle_radius = gm_sqrtf(circle_sum);
                let half_h = (self.height as f64) * 0.5;
                let sum = ((self.major_radius * self.major_radius + self.minor_radius * self.minor_radius) as f64)
                    + half_h * half_h;
                self.bounding_sphere_radius = gm_sqrt(sum) as Real;
            }
        }
    }
}

#[inline]
fn calc_dot_product(a: &Coord3D, b: &Coord3D) -> Real {
    (a.x * b.x) + (a.y * b.y) + (a.z * b.z)
}

#[inline]
fn calc_dist_squared(a: &Coord3D, b: &Coord3D) -> Real {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    (dx * dx) + (dy * dy) + (dz * dz)
}

#[inline]
fn is_within(a: Real, b: Real, c: Real) -> bool {
    a <= b && b <= c
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
