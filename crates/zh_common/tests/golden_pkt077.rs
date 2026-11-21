//! Gate V2-Golden test suite for `PKT-077` (`StateMachine`, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-077`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

#![allow(dead_code)]

use std::collections::BTreeMap;

use zh_common::core::state_machine::{
    StateCallbacks, StateCondition, StateExitType, StateId, StateMachine, StateReturnType,
    StateWorldView, EXIT_MACHINE_WITH_FAILURE, EXIT_MACHINE_WITH_SUCCESS, INVALID_STATE_ID,
    MACHINE_DONE_STATE_ID,
};
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32,
};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, ObjectId, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError, XferLoad, XferSave};
#[cfg(not(feature = "retail-crc"))]
use zh_common::xfer::XferCrc;

const PKT077_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-077.json");

fn rot_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

struct SimTraceCtx {
    frame: UnsignedInt,
    objects: BTreeMap<ObjectId, Coord3D>,
    trace_crc: u32,
    ticks: Int,
    max_ticks: Int,
    enter_sleep: Int,
    update_sleep: Int,
    jump_to_5: bool,
    wake_early: bool,
    wake_from_5: bool,
    exit_now: Int,
    reentrant_in_enter: bool,
    reentrant_in_update: bool,
    chain_depth: Int,
    state4_counter: Int,
}

impl StateWorldView for SimTraceCtx {
    fn current_frame(&self) -> UnsignedInt {
        self.frame
    }

    fn object_exists(&self, id: ObjectId) -> bool {
        self.objects.contains_key(&id)
    }
}

impl SimTraceCtx {
    fn log_hook(&mut self, kind: u32, state_id: u32, arg: Int) {
        self.trace_crc = rot_crc(self.trace_crc, kind);
        self.trace_crc = rot_crc(self.trace_crc, state_id);
        self.trace_crc = rot_crc(self.trace_crc, arg as u32);
    }
}

fn cond_jump_to_5(ctx: &mut SimTraceCtx) -> Bool {
    if ctx.jump_to_5 {
        ctx.jump_to_5 = false;
        TRUE
    } else {
        FALSE
    }
}

fn cond_wake_early(ctx: &mut SimTraceCtx) -> Bool {
    if ctx.wake_early {
        ctx.wake_early = false;
        TRUE
    } else {
        FALSE
    }
}

fn cond_exit_success(ctx: &mut SimTraceCtx) -> Bool {
    Bool::from(ctx.exit_now == 1)
}

fn cond_exit_fail(ctx: &mut SimTraceCtx) -> Bool {
    Bool::from(ctx.exit_now == 2)
}

fn cond_wake_to_4(ctx: &mut SimTraceCtx) -> Bool {
    if ctx.wake_from_5 {
        ctx.wake_from_5 = false;
        TRUE
    } else {
        FALSE
    }
}

fn s_xfer_state(id: StateId, xfer: &mut dyn Xfer, ctx: &mut SimTraceCtx) -> Result<(), XferError> {
    let mut v: XferVersion = 1;
    xfer.xfer_version(&mut v, 1)?;
    if id == 4 {
        xfer.xfer_int(&mut ctx.state4_counter)?;
    }
    Ok(())
}

fn s0_enter(sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(1, 0, 0);
    if ctx.reentrant_in_enter {
        ctx.reentrant_in_enter = false;
        sm.set_state(2, ctx);
        return StateReturnType::Sleep(999);
    }
    if ctx.enter_sleep > 0 {
        let s = ctx.enter_sleep;
        ctx.enter_sleep = 0;
        return StateReturnType::Sleep(s as UnsignedInt);
    }
    StateReturnType::Continue
}
fn s0_update(sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(2, 0, ctx.ticks);
    ctx.ticks += 1;
    if ctx.reentrant_in_update {
        ctx.reentrant_in_update = false;
        sm.set_state(3, ctx);
        return StateReturnType::Sleep(500);
    }
    if ctx.ticks >= ctx.max_ticks {
        return StateReturnType::Success;
    }
    if ctx.update_sleep > 0 {
        let s = ctx.update_sleep;
        ctx.update_sleep = 0;
        return StateReturnType::Sleep(s as UnsignedInt);
    }
    StateReturnType::Continue
}
fn s0_exit(st: StateExitType, ctx: &mut SimTraceCtx) {
    ctx.log_hook(3, 0, st as Int);
}

fn s1_enter(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(1, 1, 0);
    if ctx.enter_sleep > 0 {
        let s = ctx.enter_sleep;
        ctx.enter_sleep = 0;
        return StateReturnType::Sleep(s as UnsignedInt);
    }
    StateReturnType::Continue
}
fn s1_update(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(2, 1, ctx.ticks);
    ctx.ticks += 1;
    if ctx.ticks >= ctx.max_ticks + 2 {
        StateReturnType::Success
    } else {
        StateReturnType::Continue
    }
}
fn s1_exit(st: StateExitType, ctx: &mut SimTraceCtx) {
    ctx.log_hook(3, 1, st as Int);
}

fn s2_enter(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(1, 2, ctx.chain_depth);
    if ctx.chain_depth > 0 {
        ctx.chain_depth -= 1;
        return StateReturnType::Success;
    }
    StateReturnType::Continue
}
fn s2_update(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(2, 2, 0);
    StateReturnType::Success
}
fn s2_exit(st: StateExitType, ctx: &mut SimTraceCtx) {
    ctx.log_hook(3, 2, st as Int);
}

fn s3_enter(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(1, 3, ctx.chain_depth);
    if ctx.chain_depth > 0 {
        ctx.chain_depth -= 1;
        return StateReturnType::Success;
    }
    StateReturnType::Continue
}
fn s3_update(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(2, 3, 0);
    if ctx.exit_now == 3 {
        StateReturnType::Failure
    } else {
        StateReturnType::Continue
    }
}
fn s3_exit(st: StateExitType, ctx: &mut SimTraceCtx) {
    ctx.log_hook(3, 3, st as Int);
}

fn s4_enter(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(1, 4, ctx.state4_counter);
    ctx.state4_counter += 1;
    StateReturnType::Continue
}
fn s4_update(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(2, 4, ctx.state4_counter);
    StateReturnType::Success
}
fn s4_exit(st: StateExitType, ctx: &mut SimTraceCtx) {
    ctx.log_hook(3, 4, st as Int);
}

fn s5_enter(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(1, 5, 0);
    StateReturnType::sleep_forever()
}
fn s5_update(_sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx) -> StateReturnType {
    ctx.log_hook(2, 5, 0);
    StateReturnType::sleep_forever()
}
fn s5_exit(st: StateExitType, ctx: &mut SimTraceCtx) {
    ctx.log_hook(3, 5, st as Int);
}

fn build_harness_sm() -> StateMachine<SimTraceCtx> {
    let mut sm = StateMachine::new(ObjectId(100));
    sm.define_state(
        0,
        StateCallbacks {
            on_enter: s0_enter,
            on_exit: s0_exit,
            update: s0_update,
            xfer_state: s_xfer_state,
            is_idle: FALSE,
            is_attack: FALSE,
            is_guard_idle: FALSE,
            is_busy: FALSE,
        },
        1,
        4,
        &[
            StateCondition { test: cond_jump_to_5, to_state_id: 5 },
            StateCondition { test: cond_exit_success, to_state_id: EXIT_MACHINE_WITH_SUCCESS },
        ],
    );
    sm.define_state(
        1,
        StateCallbacks {
            on_enter: s1_enter,
            on_exit: s1_exit,
            update: s1_update,
            xfer_state: s_xfer_state,
            is_idle: FALSE,
            is_attack: FALSE,
            is_guard_idle: FALSE,
            is_busy: TRUE,
        },
        2,
        EXIT_MACHINE_WITH_FAILURE,
        &[
            StateCondition { test: cond_wake_early, to_state_id: 3 },
            StateCondition { test: cond_exit_fail, to_state_id: EXIT_MACHINE_WITH_FAILURE },
        ],
    );
    sm.define_state(
        2,
        StateCallbacks {
            on_enter: s2_enter,
            on_exit: s2_exit,
            update: s2_update,
            xfer_state: s_xfer_state,
            is_idle: TRUE,
            is_attack: TRUE,
            is_guard_idle: TRUE,
            is_busy: FALSE,
        },
        3,
        0,
        &[],
    );
    sm.define_state(
        3,
        StateCallbacks {
            on_enter: s3_enter,
            on_exit: s3_exit,
            update: s3_update,
            xfer_state: s_xfer_state,
            is_idle: FALSE,
            is_attack: FALSE,
            is_guard_idle: FALSE,
            is_busy: FALSE,
        },
        2,
        4,
        &[],
    );
    sm.define_state(
        4,
        StateCallbacks {
            on_enter: s4_enter,
            on_exit: s4_exit,
            update: s4_update,
            xfer_state: s_xfer_state,
            is_idle: FALSE,
            is_attack: FALSE,
            is_guard_idle: FALSE,
            is_busy: FALSE,
        },
        EXIT_MACHINE_WITH_SUCCESS,
        EXIT_MACHINE_WITH_FAILURE,
        &[],
    );
    sm.define_state(
        5,
        StateCallbacks {
            on_enter: s5_enter,
            on_exit: s5_exit,
            update: s5_update,
            xfer_state: s_xfer_state,
            is_idle: FALSE,
            is_attack: FALSE,
            is_guard_idle: FALSE,
            is_busy: FALSE,
        },
        0,
        0,
        &[StateCondition { test: cond_wake_to_4, to_state_id: 4 }],
    );
    sm
}

fn record_sm_snapshot(ctx: &mut SimTraceCtx, sm: &StateMachine<SimTraceCtx>, ret: StateReturnType) {
    ctx.trace_crc = rot_crc(ctx.trace_crc, ret.as_raw() as u32);
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.get_current_state_id());
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.is_in_idle_state() as u32);
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.is_in_attack_state() as u32);
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.is_in_guard_idle_state() as u32);
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.is_in_busy_state() as u32);
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.is_locked() as u32);
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.is_goal_object_destroyed(ctx) as u32);
    let gp = sm.get_goal_position();
    ctx.trace_crc = rot_crc(ctx.trace_crc, gp.x.to_bits());
    ctx.trace_crc = rot_crc(ctx.trace_crc, gp.y.to_bits());
    ctx.trace_crc = rot_crc(ctx.trace_crc, gp.z.to_bits());
}

fn exec_step(sm: &mut StateMachine<SimTraceCtx>, ctx: &mut SimTraceCtx, kind: i32, arg: i32) {
    let mut ret = StateReturnType::Continue;
    match kind {
        0 => ret = sm.init_default_state(ctx),
        1 => {
            ctx.frame += arg as UnsignedInt;
            ret = sm.update_state_machine(ctx);
        }
        2 => ret = sm.set_state(arg as StateId, ctx),
        3 => {
            if arg != 0 {
                sm.lock();
            } else {
                sm.unlock();
            }
        }
        4 => {
            if arg == 0 {
                sm.set_goal_object(None);
            } else {
                let id = ObjectId(arg);
                let obj = ctx.objects.get(&id).copied().map(|p| (id, p));
                sm.set_goal_object(obj);
            }
        }
        5 => {
            let p = Coord3D::new((arg as f32) * 0.5, -(arg as f32) * 0.25, 2.0);
            sm.set_goal_position(Some(&p));
        }
        6 => {
            ctx.objects.remove(&ObjectId(arg));
        }
        7 => ret = sm.reset_to_default_state(ctx),
        8 => sm.clear(ctx),
        9 => match arg {
            1 => ctx.jump_to_5 = true,
            2 => ctx.wake_early = true,
            3 => ctx.wake_from_5 = true,
            4 => ctx.exit_now = 1,
            5 => ctx.exit_now = 2,
            6 => ctx.exit_now = 3,
            _ => {}
        },
        10 => sm.halt(),
        _ => {}
    }
    record_sm_snapshot(ctx, sm, ret);
}

#[test]
#[ignore = "Un-ignore when implementing PKT-077 (crates/zh_common/src/core/state_machine.rs)"]
fn test_pkt077_state_machine_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT077_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-077.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let mut objects = BTreeMap::new();
        objects.insert(ObjectId(100), Coord3D::new(1.5, -2.5, 0.0));
        objects.insert(ObjectId(201), Coord3D::new(12.25, -8.5, 4.0));
        objects.insert(ObjectId(202), Coord3D::new(-30.0, 64.5, 1.25));

        let mut ctx = SimTraceCtx {
            frame: get_u32(line, "init_frame").expect("init_frame"),
            objects,
            trace_crc: 0,
            ticks: 0,
            max_ticks: get_i32(line, "max_ticks").expect("max_ticks"),
            enter_sleep: get_i32(line, "enter_sleep").expect("enter_sleep"),
            update_sleep: get_i32(line, "update_sleep").expect("update_sleep"),
            jump_to_5: false,
            wake_early: false,
            wake_from_5: false,
            exit_now: 0,
            reentrant_in_enter: get_u32(line, "re_enter").unwrap() != 0,
            reentrant_in_update: get_u32(line, "re_update").unwrap() != 0,
            chain_depth: get_i32(line, "chain_depth").expect("chain_depth"),
            state4_counter: 100,
        };

        let ops = get_i32_vec(line, "ops").expect("ops");
        assert_eq!(ops.len(), 24, "seed {} ops length", seed);

        let mut sm = build_harness_sm();
        for step in 0..6 {
            exec_step(&mut sm, &mut ctx, ops[step * 2], ops[step * 2 + 1]);
        }

        #[cfg(not(feature = "retail-crc"))]
        {
            let mut xcrc = XferCrc::new();
            sm.crc(&mut xcrc).unwrap();
            assert_eq!(xcrc.get_crc(), get_u32(line, "xfer_crc").unwrap(), "seed {} xfer_crc", seed);
        }

        let mut xsave = XferSave::new();
        sm.xfer(&mut xsave, &mut ctx).unwrap();
        #[allow(unused_mut)]
        let mut save_bytes = xsave.into_bytes();
        let expected_save = decode_hex_bytes(get_str(line, "xfer_save_hex").unwrap()).unwrap();
        assert_eq!(save_bytes, expected_save, "seed {} xfer_save_hex", seed);

        #[cfg(not(feature = "retail-crc"))]
        {
            let mut scrc = XferCrc::new();
            scrc.xfer_user(&mut save_bytes).unwrap();
            assert_eq!(scrc.get_crc(), get_u32(line, "save_crc").unwrap(), "seed {} save_crc", seed);
        }

        let mut sm2 = build_harness_sm();
        let mut xload = XferLoad::new(&save_bytes);
        sm2.xfer(&mut xload, &mut ctx).unwrap();
        assert_eq!(INVALID_STATE_ID, 999_999);
        assert_eq!(MACHINE_DONE_STATE_ID, 999_998);

        for step in 6..12 {
            exec_step(&mut sm2, &mut ctx, ops[step * 2], ops[step * 2 + 1]);
        }
        assert_eq!(ctx.trace_crc, get_u32(line, "trace_crc").unwrap(), "seed {} trace_crc", seed);
    }
}
