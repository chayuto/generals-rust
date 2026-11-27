//! A recording stand-in for the engine, shared by the module entry-point tests.

use crate::engine_sim::ZhEngineSimCallbacks;
use core::ffi::c_void;
use zh_logic::rules::kind_of::KindOfType;

#[derive(Default)]
pub(crate) struct Engine {
    pub hulk: bool,
    pub hulk_override: i32,
    pub drawable: bool,
    pub draws: Vec<(i32, i32)>,
    pub wakes: Vec<u32>,
    pub damage: Vec<(i32, i32, i32, f32, u8)>,
    /// `(flag, set)`, in call order.
    pub conditions: Vec<(i32, bool)>,
    pub armor: Vec<(i32, u8)>,
    pub warehouses: u32,
    pub ai: bool,
    pub ground: bool,
    /// Changes to the second object, in call order.
    pub exit_log: Vec<String>,
    pub paths: Vec<Vec<f32>>,
}

unsafe fn engine<'a>(ctx: *mut c_void) -> &'a mut Engine {
    unsafe { &mut *ctx.cast::<Engine>() }
}

extern "C" fn frame(_: *mut c_void) -> u32 {
    100
}
extern "C" fn rng(ctx: *mut c_void, lo: i32, hi: i32) -> i32 {
    unsafe { engine(ctx) }.draws.push((lo, hi));
    hi
}
extern "C" fn wake(ctx: *mut c_void, sleep: u32) {
    unsafe { engine(ctx) }.wakes.push(sleep);
}
extern "C" fn hulk_override(ctx: *mut c_void) -> i32 {
    unsafe { engine(ctx) }.hulk_override
}
extern "C" fn kind(ctx: *mut c_void, kindof: i32) -> u8 {
    u8::from(kindof == KindOfType::Hulk as i32 && unsafe { engine(ctx) }.hulk)
}
extern "C" fn health(_: *mut c_void) -> f32 {
    250.0
}
extern "C" fn damage(ctx: *mut c_void, kind: i32, death: i32, source: i32, amount: f32, kill: u8) {
    unsafe { engine(ctx) }
        .damage
        .push((kind, death, source, amount, kill));
}
extern "C" fn has_drawable(ctx: *mut c_void) -> u8 {
    u8::from(unsafe { engine(ctx) }.drawable)
}
extern "C" fn set_condition(ctx: *mut c_void, flag: i32) {
    unsafe { engine(ctx) }.conditions.push((flag, true));
}
extern "C" fn clear_condition(ctx: *mut c_void, flag: i32) {
    unsafe { engine(ctx) }.conditions.push((flag, false));
}
extern "C" fn armor(ctx: *mut c_void, flag: i32, set: u8) {
    unsafe { engine(ctx) }.armor.push((flag, set));
}
extern "C" fn warehouse(ctx: *mut c_void) {
    unsafe { engine(ctx) }.warehouses += 1;
}

extern "C" fn transform(_: *mut c_void, rows: *mut f32) {
    let unrotated: [f32; 12] = [1.0, 0.0, 0.0, 100.0, 0.0, 1.0, 0.0, 200.0, 0.0, 0.0, 1.0, 10.0];
    unsafe { core::ptr::copy_nonoverlapping(unrotated.as_ptr(), rows, 12) };
}
extern "C" fn orientation(_: *mut c_void) -> f32 {
    1.5
}
extern "C" fn layer(_: *mut c_void) -> i32 {
    2
}
extern "C" fn layer_height(_: *mut c_void, _: f32, _: f32, layer: i32) -> f32 {
    50.0 + layer as f32
}
unsafe fn log(ctx: *mut c_void, line: String) {
    unsafe { engine(ctx) }.exit_log.push(line);
}
extern "C" fn set_position(ctx: *mut c_void, x: f32, y: f32, z: f32) {
    unsafe { log(ctx, format!("position {x} {y} {z}")) };
}
extern "C" fn set_orientation(ctx: *mut c_void, angle: f32) {
    unsafe { log(ctx, format!("orientation {angle}")) };
}
extern "C" fn set_layer(ctx: *mut c_void, layer: i32) {
    unsafe { log(ctx, format!("layer {layer}")) };
}
extern "C" fn pathfinder_add(ctx: *mut c_void) {
    unsafe { log(ctx, "pathfinder".to_string()) };
}
extern "C" fn has_ai(ctx: *mut c_void) -> u8 {
    u8::from(unsafe { engine(ctx) }.ai)
}
extern "C" fn ground(ctx: *mut c_void) -> u8 {
    u8::from(unsafe { engine(ctx) }.ground)
}
extern "C" fn adjust(_: *mut c_void, xyz: *mut f32) -> u8 {
    unsafe { *xyz += 1.0 };
    1
}
extern "C" fn follow(ctx: *mut c_void, xyz: *const f32, count: i32) {
    let points = unsafe { core::slice::from_raw_parts(xyz, count as usize * 3) };
    unsafe { engine(ctx) }.paths.push(points.to_vec());
}

pub(crate) fn callbacks(engine: &mut Engine) -> ZhEngineSimCallbacks {
    ZhEngineSimCallbacks {
        ctx: (engine as *mut Engine).cast(),
        frame,
        rng_logic_int: rng,
        set_wake_frame: wake,
        hulk_max_lifetime_override: hulk_override,
        object_is_kind_of: kind,
        object_max_health: health,
        attempt_damage: damage,
        object_has_drawable: has_drawable,
        set_model_condition_state: set_condition,
        clear_model_condition_state: clear_condition,
        set_armor_set_flag: armor,
        register_supply_warehouse: warehouse,
        object_transform: transform,
        object_orientation: orientation,
        object_layer: layer,
        get_layer_height: layer_height,
        set_object_position: set_position,
        set_object_orientation: set_orientation,
        set_object_layer: set_layer,
        pathfinder_add_object: pathfinder_add,
        has_ai,
        ai_is_doing_ground_movement: ground,
        pathfinder_adjust_destination: adjust,
        ai_follow_exit_production_path: follow,
    }
}

/// Asserts that `zh_ffi.h` defines `name` as `value`; the `.inc` files assert the
/// same names against the engine's enums.
pub(crate) fn assert_header_number(name: &str, value: i32) {
    let header = include_str!("../include/zh_ffi.h");
    let line = format!("#define {name} {value}");
    assert!(header.lines().any(|l| l == line), "missing: {line}");
}
