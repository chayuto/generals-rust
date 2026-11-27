//! `PKT-396`: Base simulation/client object (`Thing.h` / `Thing.cpp`).

use std::cell::Cell;

use crate::arena::ThingTemplateId;
use crate::combat::turret_ai::normalize_angle_cpp;
use crate::object::r#override::Override;
use crate::object::thing_factory::ThingFactory;
use crate::object::thing_template::ThingTemplate;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, Matrix3D};
use zh_common::types::{Bool, Int, Real, FALSE, TRUE};

pub const INITIAL_THING_POS_X: Real = 0.0;
pub const INITIAL_THING_POS_Y: Real = 0.0;

const VALID_DIRVECTOR: Int = 0x01;
const VALID_ALTITUDE_TERRAIN: Int = 0x02;
const VALID_ALTITUDE_SEALEVEL: Int = 0x04;

/// Terrain height & slope alignment callbacks required by [`Thing`] (`TerrainLogic`).
pub trait ThingTerrainAccess {
    /// The map's `TerrainLogic`, where the bridges are; `None` for other terrain.
    fn terrain_logic(&self) -> Option<&crate::terrain::TerrainLogic> {
        None
    }
    fn terrain_logic_mut(&mut self) -> Option<&mut crate::terrain::TerrainLogic> {
        None
    }
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real;
    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum;
    fn is_cliff_cell(&self, _x: Real, _y: Real) -> Bool {
        FALSE
    }
    /// `TerrainLogic::isClearLineOfSight`. Without a height map every line is clear.
    fn is_clear_line_of_sight(&self, _pos: &Coord3D, _other: &Coord3D) -> Bool {
        TRUE
    }
    fn is_underwater(
        &self,
        x: Real,
        y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool;
    fn get_layer_for_destination(&self, _pos: &Coord3D) -> PathfindLayerEnum {
        PathfindLayerEnum::Ground
    }
    fn get_layer_height(
        &self,
        x: Real,
        y: Real,
        _layer: PathfindLayerEnum,
        normal: Option<&mut Coord3D>,
        _clip: Bool,
    ) -> Real {
        self.get_ground_height(x, y, normal)
    }
    fn get_highest_layer_for_destination(
        &self,
        pos: &Coord3D,
        _only_healthy_bridges: Bool,
    ) -> PathfindLayerEnum {
        self.get_layer_for_destination(pos)
    }
}

/// Subclass hook equivalent to virtual `Thing::reactToTransformChange`.
pub trait ThingTransformReactor {
    fn react_to_transform_change(&mut self, old_mtx: &Matrix3D, old_pos: &Coord3D, old_angle: Real);
}

#[derive(Debug, Default)]
pub struct NoopTransformReactor;

impl ThingTransformReactor for NoopTransformReactor {
    fn react_to_transform_change(
        &mut self,
        _old_mtx: &Matrix3D,
        _old_pos: &Coord3D,
        _old_angle: Real,
    ) {
    }
}

/// Builds a terrain-normal-aligned transform matrix (`TerrainLogic::makeAlignToNormalMatrix`).
pub fn make_align_to_normal_matrix(
    angle: Real,
    pos: &Coord3D,
    normal: &Coord3D,
    mtx: &mut Matrix3D,
) {
    let mut x = Coord3D::new(gm_cosf(angle), gm_sinf(angle), 0.0);
    let z = *normal;
    let mut y = Coord3D::default();

    if z.z != 0.0 {
        x.z = -(x.x * z.x + x.y * z.y) / z.z;
        x.normalize();
    }
    Coord3D::cross_product(&z, &x, &mut y);
    y.normalize();

    mtx.set_axes(&x, &y, &z, pos);
}

/// Shared transform, template-override, and terrain-altitude base state (`Thing.h/.cpp`).
#[derive(Debug, Clone)]
pub struct Thing {
    template: Override<ThingTemplateId>,
    transform: Matrix3D,
    cached_pos: Coord3D,
    cached_angle: Real,
    cached_dir_vector: Cell<Coord3D>,
    cached_altitude_above_terrain: Cell<Real>,
    cached_altitude_above_terrain_or_water: Cell<Real>,
    cache_flags: Cell<Int>,
    /// `Object::m_layer`, which `Object::calculateHeightAboveTerrain` measures from.
    height_layer: PathfindLayerEnum,
}

impl Thing {
    pub fn new(template_id: ThingTemplateId) -> Self {
        Self {
            template: Override::from_key(template_id),
            transform: Matrix3D::identity(),
            cached_pos: Coord3D::new(INITIAL_THING_POS_X, INITIAL_THING_POS_Y, 0.0),
            cached_angle: 0.0,
            cached_dir_vector: Cell::new(Coord3D::default()),
            cached_altitude_above_terrain: Cell::new(0.0),
            cached_altitude_above_terrain_or_water: Cell::new(0.0),
            cache_flags: Cell::new(0),
            height_layer: PathfindLayerEnum::Ground,
        }
    }

    #[inline]
    pub const fn template_override(&self) -> Override<ThingTemplateId> {
        self.template
    }

    #[inline]
    pub fn get_template<'a>(&self, factory: &'a ThingFactory) -> &'a ThingTemplate {
        self.template
            .get_final(factory)
            .expect("Thing initialized with invalid ThingTemplateId")
    }

    #[inline]
    pub fn is_kind_of(&self, factory: &ThingFactory, t: KindOfType) -> Bool {
        self.get_template(factory).is_kind_of(t)
    }

    #[inline]
    pub fn is_kind_of_multi(
        &self,
        factory: &ThingFactory,
        must_be_set: &KindOfMaskType,
        must_be_clear: &KindOfMaskType,
    ) -> Bool {
        self.get_template(factory)
            .is_kind_of_multi(must_be_set, must_be_clear)
    }

    #[inline]
    pub fn is_any_kind_of(&self, factory: &ThingFactory, any_kind_of: &KindOfMaskType) -> Bool {
        self.get_template(factory).is_any_kind_of(any_kind_of)
    }

    #[inline]
    pub const fn get_position(&self) -> &Coord3D {
        &self.cached_pos
    }

    #[inline]
    pub const fn get_orientation(&self) -> Real {
        self.cached_angle
    }

    #[inline]
    pub const fn get_transform_matrix(&self) -> &Matrix3D {
        &self.transform
    }

    #[inline]
    pub fn is_positioned(&self) -> Bool {
        if self.cached_pos.x != INITIAL_THING_POS_X || self.cached_pos.y != INITIAL_THING_POS_Y {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn get_unit_direction_vector_2d(&self) -> Coord3D {
        let flags = self.cache_flags.get();
        if (flags & VALID_DIRVECTOR) == 0 {
            let angle = self.get_orientation();
            self.cached_dir_vector
                .set(Coord3D::new(gm_cosf(angle), gm_sinf(angle), 0.0));
            self.cache_flags.set(flags | VALID_DIRVECTOR);
        }
        self.cached_dir_vector.get()
    }

    pub fn get_unit_direction_vector_3d(&self) -> Coord3D {
        let mut vdir = self.transform.get_x_vector();
        vdir.normalize_ww_vector3();
        vdir
    }

    pub fn set_position(
        &mut self,
        factory: &ThingFactory,
        pos: &Coord3D,
        terrain: &dyn ThingTerrainAccess,
        reactor: &mut dyn ThingTransformReactor,
    ) {
        if self.is_kind_of(factory, KindOfType::StickToTerrainSlope) == FALSE {
            let old_angle = self.cached_angle;
            let old_pos = self.cached_pos;
            let old_mtx = self.transform;

            self.transform.set_x_translation(pos.x);
            self.transform.set_y_translation(pos.y);
            self.transform.set_z_translation(pos.z);
            self.cached_pos = *pos;
            self.cache_flags
                .set(self.cache_flags.get() & !(VALID_ALTITUDE_TERRAIN | VALID_ALTITUDE_SEALEVEL));

            reactor.react_to_transform_change(&old_mtx, &old_pos, old_angle);
        } else {
            let mut mtx = Matrix3D::identity();
            terrain.align_on_terrain(self.get_orientation(), pos, TRUE, &mut mtx);
            self.set_transform_matrix(&mtx, reactor);
        }
    }

    pub fn set_position_z(
        &mut self,
        factory: &ThingFactory,
        z: Real,
        terrain: &dyn ThingTerrainAccess,
        reactor: &mut dyn ThingTransformReactor,
    ) {
        if self.is_kind_of(factory, KindOfType::StickToTerrainSlope) == FALSE {
            let old_angle = self.cached_angle;
            let old_pos = self.cached_pos;
            let old_mtx = self.transform;

            self.transform.set_z_translation(z);
            self.cached_pos.z = z;

            let flags = self.cache_flags.get();
            if (flags & VALID_ALTITUDE_TERRAIN) != 0 {
                self.cached_altitude_above_terrain
                    .set(self.cached_altitude_above_terrain.get() + (z - old_pos.z));
            }
            if (flags & VALID_ALTITUDE_SEALEVEL) != 0 {
                self.cached_altitude_above_terrain_or_water
                    .set(self.cached_altitude_above_terrain_or_water.get() + (z - old_pos.z));
            }

            reactor.react_to_transform_change(&old_mtx, &old_pos, old_angle);
        } else {
            let mut mtx = Matrix3D::identity();
            let mut pos = self.cached_pos;
            pos.z = z;
            terrain.align_on_terrain(self.get_orientation(), &pos, TRUE, &mut mtx);
            self.set_transform_matrix(&mtx, reactor);
        }
    }

    pub fn set_orientation(
        &mut self,
        factory: &ThingFactory,
        angle: Real,
        terrain: &dyn ThingTerrainAccess,
        reactor: &mut dyn ThingTransformReactor,
    ) {
        let old_angle = self.cached_angle;
        let old_pos = self.cached_pos;
        let old_mtx = self.transform;

        let pos = self.transform.get_translation();
        if self.is_kind_of(factory, KindOfType::StickToTerrainSlope) != FALSE {
            terrain.align_on_terrain(angle, &pos, TRUE, &mut self.transform);
        } else {
            let z = Coord3D::new(0.0, 0.0, 1.0);
            let u = Coord3D::new(gm_cosf(angle), gm_sinf(angle), 0.0);
            let mut y = Coord3D::default();
            let mut x = Coord3D::default();
            Coord3D::cross_product(&z, &u, &mut y);
            Coord3D::cross_product(&y, &z, &mut x);
            self.transform.set_axes(&x, &y, &z, &pos);
        }

        // The engine's `normalizeAngle`, range (-PI, PI], not the `WWMath` one.
        self.cached_angle = normalize_angle_cpp(angle);
        self.cached_pos = pos;
        self.cache_flags
            .set(self.cache_flags.get() & !VALID_DIRVECTOR);

        reactor.react_to_transform_change(&old_mtx, &old_pos, old_angle);
    }

    pub fn set_transform_matrix(
        &mut self,
        mx: &Matrix3D,
        reactor: &mut dyn ThingTransformReactor,
    ) {
        let old_angle = self.cached_angle;
        let old_pos = self.cached_pos;
        let old_mtx = self.transform;

        self.transform = *mx;
        self.cached_pos = self.transform.get_translation();
        self.cached_angle = self.transform.get_z_rotation();
        self.cache_flags.set(0);

        reactor.react_to_transform_change(&old_mtx, &old_pos, old_angle);
    }

    #[inline]
    pub fn calculate_height_above_terrain(&self, terrain: &dyn ThingTerrainAccess) -> Real {
        let pos = self.get_position();
        let terrain_z = match self.height_layer {
            PathfindLayerEnum::Ground => terrain.get_ground_height(pos.x, pos.y, None),
            layer => terrain.get_layer_height(pos.x, pos.y, layer, None, TRUE),
        };
        pos.z - terrain_z
    }

    /// Follows `Object::setLayer`. The cached altitude is kept, as in the reference.
    pub fn set_height_layer(&mut self, layer: PathfindLayerEnum) {
        self.height_layer = layer;
    }

    pub fn get_height_above_terrain(&self, terrain: &dyn ThingTerrainAccess) -> Real {
        let flags = self.cache_flags.get();
        if (flags & VALID_ALTITUDE_TERRAIN) == 0 {
            self.cached_altitude_above_terrain
                .set(self.calculate_height_above_terrain(terrain));
            self.cache_flags.set(flags | VALID_ALTITUDE_TERRAIN);
        }
        self.cached_altitude_above_terrain.get()
    }

    pub fn get_height_above_terrain_or_water(&self, terrain: &dyn ThingTerrainAccess) -> Real {
        let flags = self.cache_flags.get();
        if (flags & VALID_ALTITUDE_SEALEVEL) == 0 {
            let pos = self.get_position();
            let mut water_z = 0.0;
            let alt = if terrain.is_underwater(pos.x, pos.y, Some(&mut water_z), None) != FALSE {
                pos.z - water_z
            } else {
                self.get_height_above_terrain(terrain)
            };
            self.cached_altitude_above_terrain_or_water.set(alt);
            self.cache_flags.set(self.cache_flags.get() | VALID_ALTITUDE_SEALEVEL);
        }
        self.cached_altitude_above_terrain_or_water.get()
    }

    #[inline]
    pub fn is_above_terrain(&self, terrain: &dyn ThingTerrainAccess) -> Bool {
        if self.get_height_above_terrain(terrain) > 0.0 {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn is_above_terrain_or_water(&self, terrain: &dyn ThingTerrainAccess) -> Bool {
        if self.get_height_above_terrain_or_water(terrain) > 0.0 {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn is_significantly_above_terrain(
        &self,
        terrain: &dyn ThingTerrainAccess,
        gravity: Real,
    ) -> Bool {
        if self.get_height_above_terrain(terrain) > -(3.0_f32 * 3.0_f32) * gravity {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn convert_bone_pos_to_world_pos(
        &self,
        bone_pos: Option<&Coord3D>,
        bone_transform: Option<&Matrix3D>,
        world_pos: Option<&mut Coord3D>,
        world_transform: Option<&mut Matrix3D>,
    ) {
        if let (Some(wt), Some(bt)) = (world_transform, bone_transform) {
            *wt = Matrix3D::mul(&self.transform, bt);
        }
        if let (Some(wp), Some(bp)) = (world_pos, bone_pos) {
            *wp = Matrix3D::transform_vector(&self.transform, bp);
        }
    }

    #[inline]
    pub fn transform_point(&self, input: &Coord3D) -> Coord3D {
        Matrix3D::transform_vector(&self.transform, input)
    }
}
