//! `PKT-155`: Common map/terrain filename length limit
//! (`GeneralsMD/Code/GameEngine/Include/Common/Terrain.h`).

/// Maximum map filename length including extension (`MAX_TERRAIN_NAME_LEN`).
pub const MAX_TERRAIN_NAME_LEN: usize = 64;

/// `TerrainLogic::findClosestEdgePoint`.
pub fn find_closest_edge_point(
    terrain: &dyn crate::object::thing::ThingTerrainAccess,
    extent: &zh_common::math::Region3D,
    closest_to: &zh_common::math::Coord3D,
) -> zh_common::math::Coord3D {
    // Top, right, bottom, left; the first of equals wins.
    let distances = [
        (closest_to.y - extent.lo.y).abs(),
        (closest_to.x - extent.hi.x).abs(),
        (closest_to.y - extent.hi.y).abs(),
        (closest_to.x - extent.lo.x).abs(),
    ];
    let mut best = 0;
    for i in 1..4 {
        if distances[i] < distances[best] {
            best = i;
        }
    }
    let mut point = *closest_to;
    match best {
        0 => point.y = extent.lo.y,
        1 => point.x = extent.hi.x,
        2 => point.y = extent.hi.y,
        _ => point.x = extent.lo.x,
    }
    point.z = terrain.get_ground_height(point.x, point.y, None);
    point
}

/// `TerrainLogic::findFarthestEdgePoint`: a corner of the map. The halves are of
/// the width and height, not offset by the low corner.
pub fn find_farthest_edge_point(
    terrain: &dyn crate::object::thing::ThingTerrainAccess,
    extent: &zh_common::math::Region3D,
    farthest_from: &zh_common::math::Coord3D,
) -> zh_common::math::Coord3D {
    let mut point = *farthest_from;
    point.x = if farthest_from.x < (extent.hi.x - extent.lo.x) / 2.0 {
        extent.hi.x
    } else {
        extent.lo.x
    };
    point.y = if farthest_from.y < (extent.hi.y - extent.lo.y) / 2.0 {
        extent.hi.y
    } else {
        extent.lo.y
    };
    point.z = terrain.get_ground_height(point.x, point.y, None);
    point
}
