//! The bridges drawn into a map: `W3DBridgeBuffer::loadBridges`, `W3DBridge::load`
//! and `W3DBridge::getBridgeInfo` (`W3DBridgeBuffer.cpp`). The deck of such a bridge
//! takes its width from its model, so the headless engine reads the model too.
//!
//! This is device code with no packet. Golden test:
//! `crates/zh_logic/tests/golden_bridge_load.rs`; against the running reference:
//! `golden_game_start`.

use std::collections::BTreeMap;

use zh_assets::model::Model;
use zh_assets::w3d::{self, Object as W3dObject};
use zh_assets::wwmath::Matrix3D;
use zh_common::ini::ini_terrain_road::TerrainRoadType;
use zh_common::io::map_object::{MapObject, FLAG_BRIDGE_POINT1, FLAG_BRIDGE_POINT2};
use zh_common::math::Coord3D;
use zh_common::types::Real;

use crate::terrain::bridge::BridgeGeometry;

/// `BRIDGE_FLOAT_AMT`.
const BRIDGE_FLOAT_AMT: Real = 0.25;
/// `W3DBridgeBuffer::MAX_BRIDGES`.
pub const MAX_BRIDGES: usize = 200;

/// What the logic reads of a bridge model: the extent across the deck of its left
/// end piece (`m_minY`, `m_maxY`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BridgeModelShape {
    pub min_y: Real,
    pub max_y: Real,
}

/// The models of the bridge definitions, by lower-case model name. `None` is a model
/// the engine cannot load, whose bridges it leaves out.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BridgeModels(pub BTreeMap<String, Option<BridgeModelShape>>);

impl BridgeModels {
    /// The entry for a model name; the asset manager ignores case.
    pub fn get(&self, model: &str) -> Option<Option<BridgeModelShape>> {
        self.0.get(&model.to_ascii_lowercase()).copied()
    }

    /// Loads the pristine model of every bridge definition. `read` returns the bytes
    /// of a file by its archive path.
    pub fn load<'a>(
        bridges: impl Iterator<Item = &'a TerrainRoadType>,
        mut read: impl FnMut(&str) -> Option<Vec<u8>>,
    ) -> Self {
        let mut assets = Assets::default();
        let mut models = BTreeMap::new();
        for bridge in bridges {
            let name = bridge.bridge_model_name.to_ascii_lowercase();
            models
                .entry(name)
                .or_insert_with_key(|name| assets.shape(name, &mut read));
        }
        Self(models)
    }
}

/// The prototypes of the files loaded so far, by lower-case name.
#[derive(Default)]
struct Assets {
    files: BTreeMap<String, ()>,
    meshes: BTreeMap<String, Vec<[f32; 3]>>,
    hlods: BTreeMap<String, zh_assets::w3d::objects::HLod>,
    hierarchies: BTreeMap<String, zh_assets::w3d::anim::Hierarchy>,
}

impl Assets {
    /// `Load_3D_Assets` of `<name>.w3d`, once.
    fn load_file(&mut self, stem: &str, read: &mut impl FnMut(&str) -> Option<Vec<u8>>) {
        if self.files.insert(stem.to_owned(), ()).is_some() {
            return;
        }
        let Some(bytes) = read(&format!("Art\\W3D\\{stem}.w3d")) else {
            return;
        };
        for object in w3d::parse(&bytes).objects {
            match object {
                W3dObject::Mesh(mesh) => {
                    self.meshes
                        .entry(mesh.name().to_ascii_lowercase())
                        .or_insert(mesh.vertices);
                }
                W3dObject::HLod(hlod) => {
                    self.hlods
                        .entry(hlod.name.to_ascii_lowercase())
                        .or_insert(hlod);
                }
                W3dObject::Hierarchy(tree) => {
                    self.hierarchies
                        .entry(tree.name.to_ascii_lowercase())
                        .or_insert(tree);
                }
                _ => {}
            }
        }
    }

    /// Loads on demand the file a prototype name comes from: the part before `.`.
    fn demand(&mut self, name: &str, read: &mut impl FnMut(&str) -> Option<Vec<u8>>) {
        if !self.meshes.contains_key(name) && !self.hlods.contains_key(name) {
            let stem = name.split('.').next().unwrap_or(name);
            self.load_file(stem, read);
        }
    }

    /// The named sub-objects of a render object with their transforms, `None` if
    /// there is no such render object.
    fn sub_objects(
        &mut self,
        name: &str,
        read: &mut impl FnMut(&str) -> Option<Vec<u8>>,
    ) -> Option<Vec<(String, Matrix3D)>> {
        self.demand(name, read);
        let Some(hlod) = self.hlods.get(name).cloned() else {
            // A lone mesh has no sub-objects.
            return self.meshes.contains_key(name).then(Vec::new);
        };
        let tree_name = hlod.hierarchy_name.to_ascii_lowercase();
        if !self.hierarchies.contains_key(&tree_name) {
            self.load_file(&tree_name, read);
        }
        let model = Model::new(&hlod, self.hierarchies.get(&tree_name)?, |_| true);
        let pose = model.skeleton().pose(&Matrix3D::IDENTITY, None);
        Some(
            model
                .sub_objects()
                .iter()
                .filter_map(|(sub, bone)| Some((sub.clone(), pose.get(*bone)?.transform)))
                .collect(),
        )
    }

    /// `W3DBridge::load`: `None` where it returns false.
    fn shape(
        &mut self,
        model: &str,
        read: &mut impl FnMut(&str) -> Option<Vec<u8>>,
    ) -> Option<BridgeModelShape> {
        let subs = self.sub_objects(model, read)?;
        let (left, transform) = left_piece(model, &subs);
        self.demand(&left, read);
        let vertices = self.meshes.get(&left)?;
        Some(extent_across(&transform, vertices))
    }
}

/// The name of the left end piece and its transform in the model: each sub-object
/// whose name starts with the name sought takes its place, longer names included.
fn left_piece(model: &str, subs: &[(String, Matrix3D)]) -> (String, Matrix3D) {
    let mut left = format!("{model}.bridge_left").to_ascii_lowercase();
    let mut transform = Matrix3D::IDENTITY;
    for (sub, matrix) in subs {
        let sub = sub.to_ascii_lowercase();
        if sub.starts_with(&left) {
            transform = *matrix;
            left = sub;
        }
    }
    (left, transform)
}

/// The smallest and largest Y of the transformed vertices.
fn extent_across(transform: &Matrix3D, vertices: &[[f32; 3]]) -> BridgeModelShape {
    let row = transform.0[1];
    let mut shape = BridgeModelShape {
        min_y: f32::MAX,
        max_y: -f32::MAX,
    };
    for v in vertices {
        // `Matrix3D::Transform_Vector`.
        let y = row[0] * v[0] + row[1] * v[1] + row[2] * v[2] + row[3];
        if shape.min_y > y {
            shape.min_y = y;
        }
        if y > shape.max_y {
            shape.max_y = y;
        }
    }
    shape
}

/// `getBridgeInfo` of the bridge from `from` to `to`.
pub fn bridge_geometry(
    from: Coord3D,
    to: Coord3D,
    shape: BridgeModelShape,
    scale: Real,
) -> BridgeGeometry {
    let span = Coord3D::new(to.x - from.x, to.y - from.y, to.z - from.z);
    let mut normal = Coord3D::new(-span.y, span.x, 0.0);
    normal.normalize_ww_vector3();
    let corner = |end: &Coord3D, across: Real| {
        Coord3D::new(
            end.x + normal.x * across * scale,
            end.y + normal.y * across * scale,
            end.z + normal.z * across * scale,
        )
    };
    BridgeGeometry::from_corners(
        from,
        to,
        corner(&from, shape.max_y),
        corner(&from, shape.min_y),
        corner(&to, shape.max_y),
        corner(&to, shape.min_y),
    )
}

/// One bridge of the map: the indices of its first and second point. A first point
/// not followed by a second is passed over.
pub fn bridge_point_pairs(objects: &[MapObject]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    let mut i = 0;
    while let Some(object) = objects.get(i) {
        if object.flags() & FLAG_BRIDGE_POINT1 != 0 {
            let Some(next) = objects.get(i + 1) else {
                break;
            };
            if next.flags() & FLAG_BRIDGE_POINT2 != 0 {
                pairs.push((i, i + 1));
                i += 1;
            }
        }
        i += 1;
    }
    pairs
}

/// The ends of a map bridge: its points just above the ground, whatever height they
/// were given. `ground` is the terrain height.
pub fn bridge_ends(
    first: &Coord3D,
    second: &Coord3D,
    ground: impl Fn(Real, Real) -> Real,
) -> (Coord3D, Coord3D) {
    let end = |p: &Coord3D| Coord3D::new(p.x, p.y, ground(p.x, p.y) + BRIDGE_FLOAT_AMT);
    (end(first), end(second))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longer_names_take_the_left_piece_over() {
        let mut moved = Matrix3D::IDENTITY;
        moved.0[1][3] = 3.0;
        let subs = [
            ("M.BRIDGE_SPAN".to_owned(), Matrix3D::IDENTITY),
            ("M.BRIDGE_LEFT".to_owned(), Matrix3D::IDENTITY),
            ("M.BRIDGE_LEFT01".to_owned(), moved),
            // No longer a prefix match once the longer name has been taken.
            ("M.BRIDGE_LEFT02".to_owned(), Matrix3D::IDENTITY),
        ];
        let (name, transform) = left_piece("m", &subs);
        assert_eq!(name, "m.bridge_left01");
        assert_eq!(transform.0[1][3], 3.0);
        assert_eq!(left_piece("m", &[]).0, "m.bridge_left");
    }

    #[test]
    fn extent_is_of_the_transformed_vertices() {
        let mut turned = Matrix3D::IDENTITY;
        turned.0[1] = [0.5, -2.0, 4.0, 1.0];
        let shape = extent_across(&turned, &[[2.0, 1.0, 0.0], [0.0, 0.0, 3.0]]);
        assert_eq!((shape.min_y, shape.max_y), (0.0, 13.0));
        let empty = extent_across(&turned, &[]);
        assert_eq!((empty.min_y, empty.max_y), (f32::MAX, -f32::MAX));
    }
}
