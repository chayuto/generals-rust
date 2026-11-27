//! Entry points that run the ported `DefaultProductionExitUpdate` (`PKT-315`) for an
//! engine object and the unit leaving it.
//!
//! The rally point and its `crc` and `xfer` stay in C++, which passes them in. The
//! producer's transform, facing and layer are read through value accessors.

use crate::engine_sim::{EngineSim, ZhEngineSimCallbacks, GUEST};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, TRUE};
use zh_logic::modules::module::{ExitDoorType, ExitFacet};
use zh_logic::modules::update::default_production_exit_update::{
    DefaultProductionExitUpdate, DefaultProductionExitUpdateModuleData,
};

/// Mirrors `ZhExitData` in `zh_ffi.h`.
#[repr(C)]
pub struct ZhExitData {
    pub unit_create_point: [f32; 3],
    pub natural_rally_point: [f32; 3],
}

fn coord([x, y, z]: [f32; 3]) -> Coord3D {
    Coord3D { x, y, z }
}

fn module(data: &ZhExitData) -> DefaultProductionExitUpdate {
    let mut module_data =
        DefaultProductionExitUpdateModuleData::new(Default::default(), Default::default());
    module_data.unit_create_point = coord(data.unit_create_point);
    module_data.natural_rally_point = coord(data.natural_rally_point);
    DefaultProductionExitUpdate::new(module_data)
}

/// `exitObjectViaDoor`.
///
/// # Safety
/// `callbacks` and `data` must be non-null and valid for the call; `rally_point`
/// is null or points to three floats.
#[no_mangle]
pub unsafe extern "C" fn zh_exit_object_via_door(
    callbacks: *const ZhEngineSimCallbacks,
    data: *const ZhExitData,
    rally_point: *const [f32; 3],
) {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let mut exit = module(unsafe { &*data });
    if let Some(&point) = unsafe { rally_point.as_ref() } {
        exit.set_rally_point(&coord(point));
    }
    exit.exit_object_via_door(&mut sim, EngineSim::context(), GUEST, ExitDoorType::Door1);
}

/// `getExitPosition`.
///
/// # Safety
/// All pointers must be non-null and valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_exit_position(
    callbacks: *const ZhEngineSimCallbacks,
    data: *const ZhExitData,
    position: *mut [f32; 3],
) -> Bool {
    let sim = EngineSim::new(unsafe { &*callbacks });
    let mut out = Coord3D::default();
    let found = module(unsafe { &*data }).get_exit_position(&sim, EngineSim::context(), &mut out);
    if found == TRUE {
        unsafe { *position = [out.x, out.y, out.z] };
    }
    found
}

/// `getNaturalRallyPoint`.
///
/// # Safety
/// All pointers must be non-null and valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_exit_natural_rally_point(
    callbacks: *const ZhEngineSimCallbacks,
    data: *const ZhExitData,
    offset: Bool,
    point: *mut [f32; 3],
) -> Bool {
    let sim = EngineSim::new(unsafe { &*callbacks });
    let mut out = Coord3D::default();
    let found = module(unsafe { &*data }).get_natural_rally_point(
        &sim,
        EngineSim::context(),
        &mut out,
        offset,
    );
    if found == TRUE {
        unsafe { *point = [out.x, out.y, out.z] };
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_engine::{callbacks, Engine};
    use zh_common::types::FALSE;

    const DATA: ZhExitData = ZhExitData {
        unit_create_point: [4.0, 5.0, 6.0],
        natural_rally_point: [30.0, 0.0, 0.0],
    };

    // The stand-in producer sits at (100, 200, 10), unrotated, on layer 2.

    #[test]
    fn exit_places_the_unit_and_orders_it_out() {
        let mut e = Engine {
            ai: true,
            ground: true,
            ..Engine::default()
        };
        let cb = callbacks(&mut e);
        let rally = [7.0, 8.0, 9.0];
        unsafe { zh_exit_object_via_door(&cb, &DATA, &rally) };
        // The height comes from the layer, not the create point.
        assert_eq!(
            e.exit_log,
            [
                "position 104 205 52",
                "orientation 1.5",
                "layer 2",
                "pathfinder"
            ]
        );
        // The natural point is pushed two path cells out; the adjusted rally follows.
        assert_eq!(e.paths, [vec![150.0, 200.0, 10.0, 8.0, 8.0, 9.0]]);
    }

    #[test]
    fn a_unit_without_ground_movement_gets_the_natural_point_only() {
        let mut e = Engine {
            ai: true,
            ..Engine::default()
        };
        let cb = callbacks(&mut e);
        unsafe { zh_exit_object_via_door(&cb, &DATA, &[7.0, 8.0, 9.0]) };
        assert_eq!(e.paths, [vec![150.0, 200.0, 10.0]]);
    }

    #[test]
    fn a_unit_without_ai_gets_no_order() {
        let mut e = Engine::default();
        let cb = callbacks(&mut e);
        unsafe { zh_exit_object_via_door(&cb, &DATA, core::ptr::null()) };
        assert_eq!(e.exit_log.len(), 4);
        assert!(e.paths.is_empty());
    }

    #[test]
    fn the_getters_transform_the_template_points() {
        let mut e = Engine::default();
        let cb = callbacks(&mut e);
        let mut out = [0.0; 3];
        assert_eq!(unsafe { zh_exit_position(&cb, &DATA, &mut out) }, TRUE);
        assert_eq!(out, [104.0, 205.0, 16.0]);
        unsafe { zh_exit_natural_rally_point(&cb, &DATA, FALSE, &mut out) };
        assert_eq!(out, [130.0, 200.0, 10.0]);
        unsafe { zh_exit_natural_rally_point(&cb, &DATA, TRUE, &mut out) };
        assert_eq!(out, [150.0, 200.0, 10.0]);
        assert!(e.exit_log.is_empty() && e.paths.is_empty());
    }
}
