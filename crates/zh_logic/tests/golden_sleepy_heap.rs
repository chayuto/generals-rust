//! Gate V2-Golden test suite for `WP-5.0` / `WP-7.1` Core Spine:
//! - `SleepyUpdateHeap` & `UpdateModuleScheduleState` (1,000 C++ reference sequences)
//! - `ModuleArena` Take-and-Restore re-entrancy (sibling awaken, nested damage -> die, self-destroy)
//! - `ObjectArena` intrusive world list & deterministic lookup parity

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{extract_vector_lines, get_i32, get_i32_vec, get_u32};
use zh_common::types::{ObjectId, UnsignedInt, UpdateSleepTime};
use zh_common::xfer::{XferLoad, XferSave};
use zh_logic::arena::{
    with_module_dispatch, with_module_dispatch_and_drain, DeferredSelfOp, FacetMask, ModuleArena,
    ModuleDataId, ModuleMeta, ModuleSidecar, ModuleSlot, ObjectArena, TakeError,
};
use zh_logic::scheduler::{
    should_process_disabled_update, ScheduleStore, SleepyUpdateHeap, SleepyUpdatePhase,
    UpdateModuleScheduleState, UPDATE_SLEEP_FOREVER,
};

const WP5_0_JSON: &str = include_str!("../../../docs/golden_vectors/WP5_0_SleepyHeap.json");
const DISPLACED_JSON: &str =
    include_str!("../../../docs/golden_vectors/WP5_0_SleepyHeap_displaced.json");

#[inline]
fn rot_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

#[inline]
fn splitmix32(state: &mut u32) -> u32 {
    *state = state.wrapping_add(0x9e37_79b9);
    let mut z = *state;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

#[derive(Debug, Clone)]
struct MockModPayload {
    id: UnsignedInt,
    process_mask: u64,
    disabled_mask: u64,
}

#[test]
fn test_wp5_0_sleepy_heap_1000_golden_vectors() {
    let lines = extract_vector_lines(WP5_0_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 vectors in WP5_0_SleepyHeap.json");

    let mut nk = NameKeyGenerator::new();
    nk.init();
    let tag_key = nk.name_to_key("MockUpdate");

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed");
        assert_eq!(seed as usize, idx);

        let num_mods = get_i32(line, "num_mods").expect("num_mods") as usize;
        let init_frame = get_u32(line, "init_frame").expect("init_frame");
        let phases = get_i32_vec(line, "phases").expect("phases");
        let pmasks = get_i32_vec(line, "pmasks").expect("pmasks");
        let ops = get_i32_vec(line, "ops").expect("ops");
        assert_eq!(phases.len(), num_mods);
        assert_eq!(pmasks.len(), num_mods);
        assert_eq!(ops.len(), 240);

        let mut objects: ObjectArena<()> = ObjectArena::new();
        let mut obj_slots = Vec::with_capacity(num_mods);
        let mut modules: ModuleArena<MockModPayload> = ModuleArena::new();
        let mut mod_slots = Vec::with_capacity(num_mods);

        for i in 0..num_mods {
            let oid = ObjectId((i as i32) + 1);
            let oslot = objects.spawn(oid, (), false);
            obj_slots.push(oslot);

            let phase = SleepyUpdatePhase::from_bits(phases[i] as u32);
            let meta = ModuleMeta::new(
                oslot,
                oid,
                tag_key,
                ModuleDataId((i as u32) + 1),
                FacetMask::UPDATE,
                phase,
            );
            let payload = MockModPayload {
                id: i as u32,
                process_mask: (pmasks[i] as u32) as u64,
                disabled_mask: 0,
            };
            mod_slots.push(modules.allocate(meta, payload));
        }

        let mut heap: SleepyUpdateHeap<ModuleSlot> = SleepyUpdateHeap::new();
        let mut now: UnsignedInt = init_frame;
        let mut layout_crc: u32 = 0;
        let mut tick_trace_crc: u32 = 0;
        let mut helper_crc: u32 = 0;

        for s in 0..60 {
            let kind = ops[s * 4];
            let target = ops[s * 4 + 1] as usize;
            let a1 = ops[s * 4 + 2] as u32;
            let a2 = ops[s * 4 + 3] as u32;

            let target_mod = mod_slots[target];
            let target_obj = obj_slots[target];

            match kind {
                0 => {
                    if !objects.is_in_world_list(target_obj) {
                        if a1 > 0 {
                            modules.schedule_mut(target_mod).set_next_call_frame(a1);
                        }
                        objects.link_into_world(target_obj);
                        heap.register_update_module(&mut modules, target_mod, now);
                        heap.validate_invariants(&modules).expect("invariants op 0");
                    }
                }
                1 => {
                    let in_world = objects.is_in_world_list(target_obj);
                    let changed = heap.awaken_module(&mut modules, target_mod, in_world, now, a1);
                    heap.validate_invariants(&modules).expect("invariants op 1");
                    layout_crc = rot_crc(layout_crc, u32::from(changed));
                }
                2 => {
                    if objects.is_in_world_list(target_obj) {
                        let heap_idx = modules.schedule(target_mod).index_in_logic() as usize;
                        heap.erase(&mut modules, heap_idx);
                        objects.unlink_from_world(target_obj);
                        heap.validate_invariants(&modules).expect("invariants op 2");
                    }
                }
                3 => {
                    if let Some(root) = heap.pop(&mut modules) {
                        let oslot = modules.meta(root).owner_slot;
                        let rid = modules.payload(root).expect("payload").id;
                        objects.unlink_from_world(oslot);
                        heap.validate_invariants(&modules).expect("invariants op 3");
                        layout_crc = rot_crc(layout_crc, rid);
                    }
                }
                4 => {
                    let span = a2 as usize;
                    for k in 0..span {
                        let idx_m = (target + k) % num_mods;
                        modules
                            .schedule_mut(mod_slots[idx_m])
                            .set_next_call_frame(a1 + (k as u32));
                    }
                    heap.remake(&mut modules);
                    heap.validate_invariants(&modules).expect("invariants op 4");
                }
                5 => {
                    now = now.wrapping_add(a1);
                    let mut step_rng = a2;
                    let mut guard = 0;
                    while guard < 128 {
                        let Some(u) = heap.peek_due(&modules, now) else {
                            break;
                        };
                        guard += 1;
                        let mut sleep_len = UpdateSleepTime::NONE;
                        let (dis_mask, proc_mask, uid) = {
                            let p = modules.payload(u).expect("payload");
                            (p.disabled_mask, p.process_mask, p.id)
                        };
                        if should_process_disabled_update(dis_mask, proc_mask) {
                            heap.set_cur_update_module(Some(u));
                            let self_aw = heap.awaken_module(
                                &mut modules,
                                u,
                                true,
                                now,
                                now.wrapping_add(99),
                            );
                            tick_trace_crc = rot_crc(tick_trace_crc, if self_aw { 999 } else { uid });
                            tick_trace_crc = rot_crc(tick_trace_crc, now);
                            tick_trace_crc = rot_crc(tick_trace_crc, modules.schedule(u).priority());

                            if (splitmix32(&mut step_rng) % 4) == 0 && num_mods > 1 {
                                let sib = (splitmix32(&mut step_rng) as usize) % num_mods;
                                let sib_mod = mod_slots[sib];
                                if sib_mod != u
                                    && objects.is_in_world_list(obj_slots[sib])
                                    && modules.schedule(sib_mod).next_call_frame() > now + 1
                                {
                                    let aw = heap.awaken_module(
                                        &mut modules,
                                        sib_mod,
                                        true,
                                        now,
                                        now + 1,
                                    );
                                    let sid = modules.payload(sib_mod).expect("payload").id;
                                    tick_trace_crc = rot_crc(
                                        tick_trace_crc,
                                        if aw { 100 + sid } else { 0 },
                                    );
                                }
                            }

                            let mode = splitmix32(&mut step_rng) % 6;
                            sleep_len = if mode == 0 {
                                UpdateSleepTime(0)
                            } else if mode == 1 {
                                UpdateSleepTime::NONE
                            } else if mode == 2 {
                                UpdateSleepTime::FOREVER
                            } else {
                                UpdateSleepTime(2 + (splitmix32(&mut step_rng) % 10))
                            };
                        }
                        heap.finish_due_update_at_root(&mut modules, now, sleep_len);
                    }
                    heap.validate_invariants(&modules).expect("invariants op 5");
                }
                6 => {
                    let st1 = UpdateModuleScheduleState::frame_to_sleep_time(
                        now,
                        a1,
                        a2,
                        UPDATE_SLEEP_FOREVER,
                        UPDATE_SLEEP_FOREVER,
                    );
                    let st2 = modules.schedule(target_mod).get_wake_frame(now);
                    helper_crc = rot_crc(helper_crc, st1.0);
                    helper_crc = rot_crc(helper_crc, st2.0);
                }
                7 => {
                    modules.payload_mut(target_mod).expect("payload").disabled_mask = a1 as u64;
                }
                _ => panic!("unknown op kind {kind}"),
            }

            layout_crc = rot_crc(layout_crc, heap.len() as u32);
            for &h in heap.as_slice() {
                let id = modules.payload(h).expect("payload").id;
                let st = modules.schedule(h);
                layout_crc = rot_crc(layout_crc, id);
                layout_crc = rot_crc(layout_crc, st.priority());
                layout_crc = rot_crc(layout_crc, st.index_in_logic() as u32);
            }
        }

        assert_eq!(helper_crc, get_u32(line, "helper_crc").expect("helper_crc"), "seed {seed}");
        #[cfg(not(feature = "retail-crc"))]
        {
            assert_eq!(layout_crc, get_u32(line, "layout_crc").expect("layout_crc"), "seed {seed}");
            assert_eq!(
                tick_trace_crc,
                get_u32(line, "tick_trace_crc").expect("tick_trace_crc"),
                "seed {seed}"
            );
            let actual_heap: Vec<i32> = heap
                .as_slice()
                .iter()
                .map(|&h| modules.payload(h).expect("payload").id as i32)
                .collect();
            assert_eq!(
                actual_heap,
                get_i32_vec(line, "final_heap").expect("final_heap"),
                "seed {seed}"
            );
        }
    }
}

#[derive(Debug, Clone)]
enum SimModuleKind {
    Updater { ticks: u32, wake_sibling: ModuleSlot, self_damage: i32 },
    SleepySibling { ticks: u32 },
    BodyDie { hp: i32, died: bool, wake_on_die: ModuleSlot },
    ReplacedDummy { tag: &'static str },
}

struct SimObj {
    body_slot: ModuleSlot,
}

struct SimWorld {
    frame: UnsignedInt,
    objects: ObjectArena<SimObj>,
    modules: ModuleArena<SimModuleKind>,
    heap: SleepyUpdateHeap<ModuleSlot>,
    log: Vec<&'static str>,
}

fn apply_damage(world: &mut SimWorld, target: zh_logic::arena::ObjectSlot, dmg: i32) {
    let body_slot = match world.objects.get(target) {
        Some(o) => o.body_slot,
        None => return,
    };
    let _ = with_module_dispatch(
        world,
        body_slot,
        |w| &mut w.modules,
        |kind, w, _ctx| {
            if let SimModuleKind::BodyDie { hp, died, wake_on_die } = kind {
                *hp -= dmg;
                w.log.push("on_damage");
                if *hp <= 0 && !*died {
                    *died = true;
                    w.log.push("on_die");
                    w.heap.awaken_module(&mut w.modules, *wake_on_die, true, w.frame, w.frame + 1);
                }
            }
        },
    )
    .expect("dispatched");
}

#[test]
fn test_take_and_restore_reentrancy_and_sibling_awaken() {
    let mut nk = NameKeyGenerator::new();
    nk.init();

    let mut world = SimWorld {
        frame: 10,
        objects: ObjectArena::new(),
        modules: ModuleArena::new(),
        heap: SleepyUpdateHeap::new(),
        log: Vec::new(),
    };

    let obj_id = ObjectId(101);
    let obj_slot = world.objects.spawn(
        obj_id,
        SimObj {
            body_slot: ModuleSlot::INVALID,
        },
        true,
    );

    let mod_b = world.modules.allocate(
        ModuleMeta::new(
            obj_slot,
            obj_id,
            nk.name_to_key("Sibling"),
            ModuleDataId(2),
            FacetMask::UPDATE,
            SleepyUpdatePhase::Normal,
        ),
        SimModuleKind::SleepySibling { ticks: 0 },
    );
    world.modules.schedule_mut(mod_b).set_next_call_frame(999);
    world.heap.register_update_module(&mut world.modules, mod_b, world.frame);

    let mod_c = world.modules.allocate(
        ModuleMeta::new(
            obj_slot,
            obj_id,
            nk.name_to_key("BodyDie"),
            ModuleDataId(3),
            FacetMask::BODY.union(FacetMask::DAMAGE).union(FacetMask::DIE),
            SleepyUpdatePhase::Normal,
        ),
        SimModuleKind::BodyDie {
            hp: 50,
            died: false,
            wake_on_die: mod_b,
        },
    );
    world.objects.get_mut(obj_slot).expect("obj").body_slot = mod_c;

    let mod_a = world.modules.allocate(
        ModuleMeta::new(
            obj_slot,
            obj_id,
            nk.name_to_key("SelfDamageUpdater"),
            ModuleDataId(1),
            FacetMask::UPDATE,
            SleepyUpdatePhase::Initial,
        ),
        SimModuleKind::Updater {
            ticks: 0,
            wake_sibling: mod_b,
            self_damage: 60,
        },
    );
    world.heap.register_update_module(&mut world.modules, mod_a, world.frame);

    // Frame 10: mod_a runs and safely re-enters world to self-awaken (ignored), damage own object (body->die),
    // and awaken sibling mod_b to frame 11 without any borrow conflict.
    let root = world.heap.peek_due(&world.modules, world.frame).expect("mod_a due");
    assert_eq!(root, mod_a);
    world.heap.set_cur_update_module(Some(root));
    let mut self_aw_res = true;
    let sleep_len = with_module_dispatch(
        &mut world,
        root,
        |w| &mut w.modules,
        |kind, w, ctx| {
            let SimModuleKind::Updater { ticks, wake_sibling, self_damage } = kind else {
                unreachable!();
            };
            *ticks += 1;
            w.log.push("mod_a_update");
            self_aw_res = w.heap.awaken_module(&mut w.modules, ctx.mod_slot, true, w.frame, w.frame + 50);
            w.heap.awaken_module(&mut w.modules, *wake_sibling, true, w.frame, w.frame + 2);
            apply_damage(w, ctx.owner_slot, *self_damage);
            UpdateSleepTime(20)
        },
    )
    .expect("dispatched");
    world.heap.finish_due_update_at_root(&mut world.modules, world.frame, sleep_len);

    assert!(!self_aw_res, "self-awaken during update must be ignored");
    assert_eq!(world.log, vec!["mod_a_update", "on_damage", "on_die"]);
    assert!(matches!(world.modules.payload(mod_a), Some(SimModuleKind::Updater { ticks: 1, .. })));
    assert!(matches!(world.modules.payload(mod_c), Some(SimModuleKind::BodyDie { hp: -10, died: true, .. })));
    // on_die woke mod_b to frame 11 (earlier than frame 12 from mod_a!).
    assert_eq!(world.modules.schedule(mod_b).next_call_frame(), 11);

    // Advance to frame 11 -> mod_b wakes and updates.
    world.frame = 11;
    let next_due = world.heap.peek_due(&world.modules, world.frame).expect("mod_b due");
    assert_eq!(next_due, mod_b);
    world.heap.set_cur_update_module(Some(next_due));
    let s2 = with_module_dispatch(
        &mut world,
        next_due,
        |w| &mut w.modules,
        |kind, w, _| {
            if let SimModuleKind::SleepySibling { ticks } = kind {
                *ticks += 1;
                w.log.push("mod_b_update");
            }
            UpdateSleepTime::FOREVER
        },
    )
    .expect("dispatched");
    world.heap.finish_due_update_at_root(&mut world.modules, world.frame, s2);
    assert_eq!(world.log, vec!["mod_a_update", "on_damage", "on_die", "mod_b_update"]);
    assert!(matches!(world.modules.payload(mod_b), Some(SimModuleKind::SleepySibling { ticks: 1 })));
    world.heap.validate_invariants(&world.modules).expect("final heap invariants");
}

#[test]
fn test_take_and_restore_self_destroy_and_slot_reuse_safety() {
    let mut nk = NameKeyGenerator::new();
    nk.init();

    let mut world = SimWorld {
        frame: 5,
        objects: ObjectArena::new(),
        modules: ModuleArena::new(),
        heap: SleepyUpdateHeap::new(),
        log: Vec::new(),
    };

    let obj_id = ObjectId(7);
    let obj_slot = world.objects.spawn(
        obj_id,
        SimObj {
            body_slot: ModuleSlot::INVALID,
        },
        true,
    );

    let meta_a = ModuleMeta::new(
        obj_slot,
        obj_id,
        nk.name_to_key("SelfDestruct"),
        ModuleDataId(10),
        FacetMask::UPDATE,
        SleepyUpdatePhase::Normal,
    );
    let mod_a = world.modules.allocate(
        meta_a.clone(),
        SimModuleKind::Updater {
            ticks: 0,
            wake_sibling: ModuleSlot::INVALID,
            self_damage: 0,
        },
    );
    world.heap.register_update_module(&mut world.modules, mod_a, world.frame);

    // Case 1: Self-free without immediate reuse -> slot stays dead, payload is not restored.
    let _ = with_module_dispatch(
        &mut world,
        mod_a,
        |w| &mut w.modules,
        |_kind, w, ctx| {
            let idx = w.modules.schedule(ctx.mod_slot).index_in_logic() as usize;
            w.heap.erase(&mut w.modules, idx);
            w.modules.free(ctx.mod_slot);
        },
    )
    .expect("dispatched");
    assert!(!world.modules.is_alive(mod_a));
    assert!(world.modules.payload(mod_a).is_none());

    // Case 2: Self-free followed immediately by slot reallocation in the same callback ->
    // generation mismatch prevents restoring old payload over new module!
    let mod_b = world.modules.allocate(
        meta_a.clone(),
        SimModuleKind::Updater {
            ticks: 42,
            wake_sibling: ModuleSlot::INVALID,
            self_damage: 0,
        },
    );
    let mut reused_slot = ModuleSlot::INVALID;
    let _ = with_module_dispatch(
        &mut world,
        mod_b,
        |w| &mut w.modules,
        |_kind, w, ctx| {
            w.modules.free(ctx.mod_slot);
            reused_slot = w.modules.allocate(meta_a.clone(), SimModuleKind::ReplacedDummy { tag: "fresh" });
        },
    )
    .expect("dispatched");
    assert_eq!(reused_slot, mod_b);
    assert!(matches!(
        world.modules.payload(reused_slot),
        Some(SimModuleKind::ReplacedDummy { tag: "fresh" })
    ));
}

#[test]
fn test_object_arena_world_list_and_schedule_xfer_normalization() {
    let mut arena: ObjectArena<u32> = ObjectArena::new();
    let s1 = arena.spawn(ObjectId(10), 100, true);
    let gen1 = arena.generation(s1).expect("gen1");
    let s2 = arena.spawn(ObjectId(20), 200, false);
    let s3 = arena.spawn(ObjectId(30), 300, true);
    arena.link_into_world(s2);

    assert_eq!(arena.slot_by_id(ObjectId(20)), Some(s2));
    // link_into_world prepends to world_head (`GameLogic::registerObject`).
    assert_eq!(arena.world_iter().collect::<Vec<_>>(), vec![s2, s3, s1]);

    arena.unlink_from_world(s3);
    assert_eq!(arena.world_iter().collect::<Vec<_>>(), vec![s2, s1]);
    assert_eq!(arena.despawn(s1), Some(100));
    assert_eq!(arena.world_iter().collect::<Vec<_>>(), vec![s2]);
    assert!(arena.slot_by_id(ObjectId(10)).is_none());
    assert!(!arena.is_slot_current(s1, gen1));
    assert!(arena.get_checked(s1, gen1).is_none());

    // Reusing s1 increments generation to prevent ABA bugs.
    let s4 = arena.spawn(ObjectId(40), 400, true);
    assert_eq!(s4, s1);
    assert!(!arena.is_slot_current(s4, gen1));
    assert_ne!(arena.generation(s4), Some(gen1));
    assert!(arena.get_checked(s4, gen1).is_none());
    assert_eq!(arena.get_checked(s4, arena.generation(s4).unwrap()), Some(&400));

    let mut sched_save = UpdateModuleScheduleState::new(SleepyUpdatePhase::Physics);
    sched_save.set_next_call_frame(123);
    let mut xsave = XferSave::new();
    sched_save.xfer_schedule(&mut xsave).expect("save");

    // Simulate legacy phase bits in save buffer and verify normalization on load
    let mut sched_load = UpdateModuleScheduleState::new(SleepyUpdatePhase::Final);
    sched_load.set_index_in_logic(9);
    let mut xload = XferLoad::new(xsave.as_bytes());
    sched_load.xfer_schedule(&mut xload).expect("load");
    assert_eq!(sched_load.next_call_frame(), 123);
    assert_eq!(sched_load.next_call_phase(), SleepyUpdatePhase::Final);
    assert_eq!(sched_load.index_in_logic(), -1);
}

#[test]
fn test_sleepy_heap_root_displacement_and_erase_for_owner() {
    let mut nk = NameKeyGenerator::new();
    nk.init();
    let tag_key = nk.name_to_key("TestMod");

    let mut objects: ObjectArena<()> = ObjectArena::new();
    let obj1 = objects.spawn(ObjectId(1), (), true);
    let obj2 = objects.spawn(ObjectId(2), (), true);

    let mut modules: ModuleArena<u32> = ModuleArena::new();
    let m_norm = modules.allocate(
        ModuleMeta::new(obj1, ObjectId(1), tag_key, ModuleDataId(1), FacetMask::UPDATE, SleepyUpdatePhase::Normal),
        10,
    );
    let m_init = modules.allocate(
        ModuleMeta::new(obj2, ObjectId(2), tag_key, ModuleDataId(2), FacetMask::UPDATE, SleepyUpdatePhase::Initial),
        20,
    );
    let m_other = modules.allocate(
        ModuleMeta::new(obj1, ObjectId(1), tag_key, ModuleDataId(3), FacetMask::UPDATE, SleepyUpdatePhase::Final),
        30,
    );

    let mut heap: SleepyUpdateHeap<ModuleSlot> = SleepyUpdateHeap::new();
    let now = 5;
    modules.schedule_mut(m_norm).set_next_call_frame(now);
    modules.schedule_mut(m_init).set_next_call_frame(now + 20);
    modules.schedule_mut(m_other).set_next_call_frame(now + 2);
    heap.register_update_module(&mut modules, m_norm, now);
    heap.register_update_module(&mut modules, m_init, now);
    heap.register_update_module(&mut modules, m_other, now);

    // m_norm (frame 5, Normal) is at root; during its update, m_init (Initial) is awakened to frame 5,
    // displacing m_norm from index 0. finish_due_update_at_root must reschedule m_norm, not m_init,
    // and rebalance slot 0 as C++ does (see the displaced-root vectors below).
    assert_eq!(heap.peek_due(&modules, now), Some(m_norm));
    heap.set_cur_update_module(Some(m_norm));
    assert!(heap.awaken_module(&mut modules, m_init, true, now, now));
    assert_eq!(heap.peek(), Some(m_init));
    heap.finish_due_update_at_root(&mut modules, now, UpdateSleepTime(10));

    assert_eq!(modules.schedule(m_init).next_call_frame(), now);
    assert_eq!(modules.schedule(m_norm).next_call_frame(), now + 10);
    assert_eq!(heap.peek_due(&modules, now), Some(m_init));
    heap.validate_invariants(&modules).expect("valid after displacement");

    // Disabled-skip path: cur_update_module is None, so root (m_init) is rescheduled to now + 1.
    assert!(heap.cur_update_module().is_none());
    heap.finish_due_update_at_root(&mut modules, now, UpdateSleepTime::NONE);
    assert_eq!(modules.schedule(m_init).next_call_frame(), now + 1);
    heap.validate_invariants(&modules).expect("valid after disabled skip");

    // Two-pass erase_for_owner removes all modules belonging to obj1 while preserving obj2's module.
    let erased = heap.erase_for_owner(&mut modules, |m, ms| m.meta(ms).owner_slot == obj1);
    assert_eq!(erased, 2);
    assert_eq!(heap.len(), 1);
    assert_eq!(heap.peek(), Some(m_init));
    assert_eq!(modules.schedule(m_norm).index_in_logic(), -1);
    assert_eq!(modules.schedule(m_other).index_in_logic(), -1);
    heap.validate_invariants(&modules).expect("valid after erase_for_owner");
}

#[test]
fn test_module_arena_reentered_error_and_deferred_ops() {
    let mut nk = NameKeyGenerator::new();
    nk.init();
    let tag_key = nk.name_to_key("BodyTag");

    let mut arena: ModuleArena<i32> = ModuleArena::new();
    let slot = arena.allocate(
        ModuleMeta::new(
            zh_logic::arena::ObjectSlot(0),
            ObjectId(1),
            tag_key,
            ModuleDataId(1),
            FacetMask::BODY,
            SleepyUpdatePhase::Normal,
        ),
        100,
    );
    arena.set_sidecar(
        slot,
        ModuleSidecar::Body {
            health: 100.0,
            max_health: 100.0,
            prev_health: 100.0,
            initial_health: 100.0,
            damage_scalar: 1.0,
            damage_state: 0,
            front_crushed: 0,
            back_crushed: 0,
            indestructible: 0,
            current_subdual_damage: 0.0,
            subdued: 0,
            last_damage_ts: u32::MAX,
            last_healing_ts: u32::MAX,
            armor_set_bits: 0,
        },
    );

    let mut reenter_err = None;
    let res = with_module_dispatch_and_drain(
        &mut arena,
        slot,
        |a| a,
        |hp, a, ctx| {
            reenter_err = a.take_payload(ctx.mod_slot).err();
            assert!(a.is_executing(ctx.mod_slot));
            assert!(a.push_deferred_op(
                ctx.mod_slot,
                DeferredSelfOp::ChangeHealth {
                    delta: -25.0,
                    change_type: 0,
                },
            ));
            *hp
        },
        |hp, _a, _ctx, op| {
            if let DeferredSelfOp::ChangeHealth { delta, .. } = op {
                *hp += delta as i32;
            }
        },
    );

    assert_eq!(res, Ok(100));
    assert_eq!(reenter_err, Some(TakeError::Reentered));
    assert_eq!(arena.payload(slot).copied(), Some(75));

    let _ = arena.free(slot);
    assert_eq!(arena.take_payload(slot).err(), Some(TakeError::Dead));
}

/// `update()` takes slot 0 away from its own module, by waking another one to the
/// current frame or by registering a new one. C++ then rebalances slot 0 all the
/// same (`GameLogic.cpp:4108-4109`), and the replayed layout must match that.
#[test]
fn test_sleepy_heap_displaced_root_golden_vectors() {
    let lines = extract_vector_lines(DISPLACED_JSON);
    assert_eq!(lines.len(), 300);
    let mut nk = NameKeyGenerator::new();
    nk.init();
    let tag_key = nk.name_to_key("MockUpdate");
    let mut displaced_total = 0;

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let mut now = get_u32(line, "first_frame").expect("first_frame");
        let phases = get_i32_vec(line, "phases").expect("phases");
        let frames = get_i32_vec(line, "frames").expect("frames");
        let steps = get_i32_vec(line, "steps").expect("steps");

        let mut objects: ObjectArena<()> = ObjectArena::new();
        let mut modules: ModuleArena<u32> = ModuleArena::new();
        let slots: Vec<ModuleSlot> = (0..phases.len())
            .map(|i| {
                let oid = ObjectId(i as i32 + 1);
                let oslot = objects.spawn(oid, (), false);
                let phase = SleepyUpdatePhase::from_bits(phases[i] as u32);
                let meta = ModuleMeta::new(
                    oslot,
                    oid,
                    tag_key,
                    ModuleDataId(i as u32 + 1),
                    FacetMask::UPDATE,
                    phase,
                );
                modules.allocate(meta, i as u32)
            })
            .collect();

        let mut heap: SleepyUpdateHeap<ModuleSlot> = SleepyUpdateHeap::new();
        for (i, &frame) in frames.iter().enumerate() {
            modules
                .schedule_mut(slots[i])
                .set_next_call_frame(frame as u32);
            heap.push(&mut modules, slots[i]);
        }

        let mut layout_crc = 0;
        let mut displaced = 0;
        for (step_index, step) in steps.chunks_exact(4).enumerate() {
            let (run, action, target, sleep) = (step[0], step[1], step[2], step[3]);
            if run < 0 {
                now += 1;
                continue;
            }
            let u = heap.peek_due(&modules, now).expect("a module is due");
            assert_eq!(
                u, slots[run as usize],
                "seed {seed} step {step_index}: wrong module runs"
            );
            heap.set_cur_update_module(Some(u));
            match action {
                1 => {
                    heap.awaken_module(&mut modules, slots[target as usize], true, now, now);
                }
                2 => heap.register_update_module(&mut modules, slots[target as usize], now),
                _ => {}
            }
            if heap.peek() != Some(u) {
                displaced += 1;
            }
            heap.finish_due_update(&mut modules, u, now, UpdateSleepTime(sleep as u32));
            for &h in heap.as_slice() {
                layout_crc = rot_crc(layout_crc, *modules.payload(h).expect("payload"));
                layout_crc = rot_crc(layout_crc, modules.schedule(h).priority());
            }
        }

        let final_heap: Vec<i32> = heap
            .as_slice()
            .iter()
            .map(|&h| *modules.payload(h).expect("payload") as i32)
            .collect();
        assert_eq!(displaced, get_u32(line, "displaced").unwrap(), "seed {seed}");
        assert_eq!(final_heap, get_i32_vec(line, "final_heap").unwrap(), "seed {seed}");
        assert_eq!(layout_crc, get_u32(line, "layout_crc").unwrap(), "seed {seed}");
        displaced_total += displaced;
    }
    assert!(displaced_total > 500, "the vectors no longer displace the root");
}
