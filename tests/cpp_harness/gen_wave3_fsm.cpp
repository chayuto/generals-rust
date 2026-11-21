#include "PreRTS.h"
#include "Common/Snapshot.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "GameLogic/Object.h"
#include "GameLogic/GameLogic.h"
#include "Common/StateMachine.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferCRC.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferSave.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferLoad.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/StateMachine.cpp"

const AsciiString AsciiString::TheEmptyString("");
const UnicodeString UnicodeString::TheEmptyString;

static GameStateStub g_game_state;
GameStateStub* TheGameState = &g_game_state;
static ScienceStoreStub g_science_store;
ScienceStoreStub* TheScienceStore = &g_science_store;
static UpgradeCenterStub g_upgrade_center;
UpgradeCenterStub* TheUpgradeCenter = &g_upgrade_center;
static GlobalData g_global_data;
GlobalData* TheGlobalData = &g_global_data;
static GameLogic g_game_logic;
GameLogic* TheGameLogic = &g_game_logic;

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t fbits(Real v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, 4);
    return b;
}

static uint32_t rot_crc(uint32_t crc, uint32_t word) {
    return ((crc << 1) | (crc >> 31)) + word;
}

struct SimTraceCtx {
    uint32_t trace_crc = 0;
    Int ticks = 0;
    Int max_ticks = 2;
    Int enter_sleep = 0;
    Int update_sleep = 0;
    Bool jump_to_5 = FALSE;
    Bool wake_early = FALSE;
    Bool wake_from_5 = FALSE;
    Int exit_now = 0;
    Bool reentrant_in_enter = FALSE;
    Bool reentrant_in_update = FALSE;
    Int chain_depth = 0;
    Int state4_counter = 100;

    void log_hook(uint32_t kind, uint32_t state_id, Int arg) {
        trace_crc = rot_crc(trace_crc, kind);
        trace_crc = rot_crc(trace_crc, state_id);
        trace_crc = rot_crc(trace_crc, static_cast<uint32_t>(arg));
    }
};

static Bool cond_jump_to_5(State*, void* ud) {
    auto* ctx = static_cast<SimTraceCtx*>(ud);
    if (ctx->jump_to_5) { ctx->jump_to_5 = FALSE; return TRUE; }
    return FALSE;
}
static Bool cond_wake_early(State*, void* ud) {
    auto* ctx = static_cast<SimTraceCtx*>(ud);
    if (ctx->wake_early) { ctx->wake_early = FALSE; return TRUE; }
    return FALSE;
}
static Bool cond_exit_success(State*, void* ud) {
    return static_cast<SimTraceCtx*>(ud)->exit_now == 1 ? TRUE : FALSE;
}
static Bool cond_exit_fail(State*, void* ud) {
    return static_cast<SimTraceCtx*>(ud)->exit_now == 2 ? TRUE : FALSE;
}
static Bool cond_wake_to_4(State*, void* ud) {
    auto* ctx = static_cast<SimTraceCtx*>(ud);
    if (ctx->wake_from_5) { ctx->wake_from_5 = FALSE; return TRUE; }
    return FALSE;
}

class TraceState : public State {
    MEMORY_POOL_GLUE_ABC(TraceState)
protected:
    Int m_kind;
    SimTraceCtx* m_ctx;
public:
    TraceState(StateMachine* m, Int kind, SimTraceCtx* ctx, const char* name)
        : State(m, name), m_kind(kind), m_ctx(ctx) {}

    void crc(Xfer*) override {}
    void xfer(Xfer* xfer) override {
        XferVersion cv = 1;
        XferVersion v = cv;
        xfer->xferVersion(&v, cv);
        if (m_kind == 4) {
            xfer->xferInt(&m_ctx->state4_counter);
        }
    }
    void loadPostProcess() override {}

    Bool isIdle() const override { return m_kind == 2 ? TRUE : FALSE; }
    Bool isAttack() const override { return m_kind == 2 ? TRUE : FALSE; }
    Bool isGuardIdle() const override { return m_kind == 2 ? TRUE : FALSE; }
    Bool isBusy() const override { return m_kind == 1 ? TRUE : FALSE; }

    StateReturnType onEnter() override {
        if (m_kind == 0) {
            m_ctx->log_hook(1, 0, 0);
            if (m_ctx->reentrant_in_enter) {
                m_ctx->reentrant_in_enter = FALSE;
                getMachine()->setState(2);
                return STATE_SLEEP(999);
            }
            if (m_ctx->enter_sleep > 0) {
                Int s = m_ctx->enter_sleep;
                m_ctx->enter_sleep = 0;
                return STATE_SLEEP(s);
            }
            return STATE_CONTINUE;
        } else if (m_kind == 1) {
            m_ctx->log_hook(1, 1, 0);
            if (m_ctx->enter_sleep > 0) {
                Int s = m_ctx->enter_sleep;
                m_ctx->enter_sleep = 0;
                return STATE_SLEEP(s);
            }
            return STATE_CONTINUE;
        } else if (m_kind == 2 || m_kind == 3) {
            m_ctx->log_hook(1, static_cast<uint32_t>(m_kind), m_ctx->chain_depth);
            if (m_ctx->chain_depth > 0) {
                --m_ctx->chain_depth;
                return STATE_SUCCESS;
            }
            return STATE_CONTINUE;
        } else if (m_kind == 4) {
            m_ctx->log_hook(1, 4, m_ctx->state4_counter);
            ++m_ctx->state4_counter;
            return STATE_CONTINUE;
        } else {
            m_ctx->log_hook(1, 5, 0);
            return STATE_SLEEP_FOREVER;
        }
    }

    StateReturnType update() override {
        if (m_kind == 0) {
            m_ctx->log_hook(2, 0, m_ctx->ticks);
            ++m_ctx->ticks;
            if (m_ctx->reentrant_in_update) {
                m_ctx->reentrant_in_update = FALSE;
                getMachine()->setState(3);
                return STATE_SLEEP(500);
            }
            if (m_ctx->ticks >= m_ctx->max_ticks) return STATE_SUCCESS;
            if (m_ctx->update_sleep > 0) {
                Int s = m_ctx->update_sleep;
                m_ctx->update_sleep = 0;
                return STATE_SLEEP(s);
            }
            return STATE_CONTINUE;
        } else if (m_kind == 1) {
            m_ctx->log_hook(2, 1, m_ctx->ticks);
            ++m_ctx->ticks;
            if (m_ctx->ticks >= m_ctx->max_ticks + 2) return STATE_SUCCESS;
            return STATE_CONTINUE;
        } else if (m_kind == 2) {
            m_ctx->log_hook(2, 2, 0);
            return STATE_SUCCESS;
        } else if (m_kind == 3) {
            m_ctx->log_hook(2, 3, 0);
            if (m_ctx->exit_now == 3) return STATE_FAILURE;
            return STATE_CONTINUE;
        } else if (m_kind == 4) {
            m_ctx->log_hook(2, 4, m_ctx->state4_counter);
            return STATE_SUCCESS;
        } else {
            m_ctx->log_hook(2, 5, 0);
            return STATE_SLEEP_FOREVER;
        }
    }

    void onExit(StateExitType status) override {
        m_ctx->log_hook(3, static_cast<uint32_t>(m_kind), static_cast<Int>(status));
    }
};
EMPTY_DTOR(TraceState)

class HarnessStateMachine : public StateMachine {
    MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(HarnessStateMachine, "HarnessSMPool")
public:
    HarnessStateMachine(Object* owner, SimTraceCtx* ctx)
        : StateMachine(owner, AsciiString("HarnessSM")) {
        static const StateConditionInfo c0[] = {
            { cond_jump_to_5, 5, nullptr },
            { cond_exit_success, EXIT_MACHINE_WITH_SUCCESS, nullptr },
            { nullptr, 0, nullptr }
        };
        static const StateConditionInfo c1[] = {
            { cond_wake_early, 3, nullptr },
            { cond_exit_fail, EXIT_MACHINE_WITH_FAILURE, nullptr },
            { nullptr, 0, nullptr }
        };
        static const StateConditionInfo c5[] = {
            { cond_wake_to_4, 4, nullptr },
            { nullptr, 0, nullptr }
        };
        StateConditionInfo sc0[] = { { cond_jump_to_5, 5, ctx }, { cond_exit_success, EXIT_MACHINE_WITH_SUCCESS, ctx }, { nullptr, 0, nullptr } };
        StateConditionInfo sc1[] = { { cond_wake_early, 3, ctx }, { cond_exit_fail, EXIT_MACHINE_WITH_FAILURE, ctx }, { nullptr, 0, nullptr } };
        StateConditionInfo sc5[] = { { cond_wake_to_4, 4, ctx }, { nullptr, 0, nullptr } };
        (void)c0; (void)c1; (void)c5;
        defineState(0, new TraceState(this, 0, ctx, "S0"), 1, 4, sc0);
        defineState(1, new TraceState(this, 1, ctx, "S1"), 2, EXIT_MACHINE_WITH_FAILURE, sc1);
        defineState(2, new TraceState(this, 2, ctx, "S2"), 3, 0);
        defineState(3, new TraceState(this, 3, ctx, "S3"), 2, 4);
        defineState(4, new TraceState(this, 4, ctx, "S4"), EXIT_MACHINE_WITH_SUCCESS, EXIT_MACHINE_WITH_FAILURE);
        defineState(5, new TraceState(this, 5, ctx, "S5"), 0, 0, sc5);
    }

    void callXfer(Xfer* x) { xfer(x); }
    void callCrc(Xfer* x) { crc(x); }
};
EMPTY_DTOR(HarnessStateMachine)

static void record_sm_snapshot(SimTraceCtx& ctx, const HarnessStateMachine& sm, Int ret) {
    ctx.trace_crc = rot_crc(ctx.trace_crc, static_cast<uint32_t>(ret));
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.getCurrentStateID());
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.isInIdleState());
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.isInAttackState());
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.isInGuardIdleState());
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.isInBusyState());
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.isLocked());
    ctx.trace_crc = rot_crc(ctx.trace_crc, sm.isGoalObjectDestroyed());
    ctx.trace_crc = rot_crc(ctx.trace_crc, fbits(sm.getGoalPosition()->x));
    ctx.trace_crc = rot_crc(ctx.trace_crc, fbits(sm.getGoalPosition()->y));
    ctx.trace_crc = rot_crc(ctx.trace_crc, fbits(sm.getGoalPosition()->z));
}

static std::vector<uint8_t> read_file_bytes(const char* path) {
    FILE* f = std::fopen(path, "rb");
    if (!f) { std::perror(path); std::exit(2); }
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    std::vector<uint8_t> buf(static_cast<size_t>(sz));
    if (sz > 0) std::fread(buf.data(), 1, static_cast<size_t>(sz), f);
    std::fclose(f);
    return buf;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_wave3_fsm <out.json>\n");
        return 1;
    }
    const char* tmp_save = "/tmp/pkt077_fsm_save.bin";
    FILE* fp = std::fopen(argv[1], "wb");
    if (!fp) { std::perror(argv[1]); return 1; }

    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-077\",\n  \"module\": \"StateMachine\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified StateMachine.cpp + Xfer*.cpp\",\n"
                     "  \"vector_count\": 1000,\n  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x85ebca6bu + 0x12345678u;
        TheGameLogic->m_frame = 10u + (seed % 50u);
        Object owner_obj(ObjectID(100), Coord3D{1.5f, -2.5f, 0.0f});
        Object goal_obj1(ObjectID(201), Coord3D{12.25f, -8.5f, 4.0f});
        Object goal_obj2(ObjectID(202), Coord3D{-30.0f, 64.5f, 1.25f});
        TheGameLogic->m_objects.clear();
        TheGameLogic->m_objects[ObjectID(100)] = &owner_obj;
        TheGameLogic->m_objects[ObjectID(201)] = &goal_obj1;
        TheGameLogic->m_objects[ObjectID(202)] = &goal_obj2;

        SimTraceCtx ctx;
        ctx.max_ticks = static_cast<Int>((splitmix32(rng) % 3u) + 1u);
        ctx.enter_sleep = static_cast<Int>(splitmix32(rng) % 4u);
        ctx.update_sleep = static_cast<Int>(splitmix32(rng) % 4u);
        if (seed % 29 == 0) ctx.reentrant_in_enter = TRUE;
        if (seed % 31 == 0) ctx.reentrant_in_update = TRUE;
        if (seed % 37 == 0) ctx.chain_depth = 23; // Exercises depth >= 20 guard!
        else if (seed % 11 == 0) ctx.chain_depth = 3;

        Int init_enter_sleep = ctx.enter_sleep;
        Int init_update_sleep = ctx.update_sleep;
        Bool init_re_enter = ctx.reentrant_in_enter;
        Bool init_re_update = ctx.reentrant_in_update;
        Int init_chain_depth = ctx.chain_depth;

        auto* sm = new HarnessStateMachine(&owner_obj, &ctx);

        // Record script ops so Rust test executes the exact same operations:
        // ops: [op_kind, arg]
        // 0: initDefaultState
        // 1: updateStateMachine (advances frame by arg)
        // 2: setState(arg)
        // 3: lock / unlock (arg 1/0)
        // 4: setGoalObject(arg: 201, 202, 0=null)
        // 5: setGoalPosition(fixed from arg)
        // 6: destroyGoalObject(arg)
        // 7: resetToDefaultState
        // 8: clear
        // 9: toggle ctx flag (arg: 1=jump5, 2=wake_early, 3=wake_5, 4=exit1, 5=exit2, 6=exit3)
        // 10: halt
        uint32_t ops[12][2];
        ops[0][0] = 0; ops[0][1] = 0;
        for (int step = 1; step < 6; ++step) {
            uint32_t kind = splitmix32(rng) % 10u;
            uint32_t arg = 0;
            if (kind == 0) {
                kind = (seed % 17 == 0 && step == 1) ? 0u : 1u;
            }
            if (kind == 1) arg = (splitmix32(rng) % 4u) + 1u;
            else if (kind == 2) {
                uint32_t picks[] = {0, 1, 2, 3, 4, 5, INVALID_STATE_ID, MACHINE_DONE_STATE_ID};
                arg = picks[splitmix32(rng) % 8u];
            } else if (kind == 3) {
                arg = splitmix32(rng) & 1u;
            } else if (kind == 4) {
                uint32_t ids[] = {0, 201, 202};
                arg = ids[splitmix32(rng) % 3u];
            } else if (kind == 5) {
                arg = splitmix32(rng) % 100u;
            } else if (kind == 6) {
                arg = (splitmix32(rng) & 1u) ? 201u : 202u;
            } else if (kind == 9) {
                arg = (splitmix32(rng) % 6u) + 1u;
            }
            ops[step][0] = kind;
            ops[step][1] = arg;
        }
        for (int step = 6; step < 12; ++step) {
            uint32_t kind = (step == 11 && (seed % 13 == 0)) ? 10u : ((splitmix32(rng) % 9u) + 1u);
            if (kind == 8 && (seed % 5 != 0)) kind = 1u;
            uint32_t arg = 0;
            if (kind == 1) arg = (splitmix32(rng) % 5u) + 1u;
            else if (kind == 2) arg = splitmix32(rng) % 6u;
            else if (kind == 3) arg = splitmix32(rng) & 1u;
            else if (kind == 4) arg = (splitmix32(rng) & 1u) ? 201u : 0u;
            else if (kind == 5) arg = splitmix32(rng) % 100u;
            else if (kind == 6) arg = 201u;
            else if (kind == 9) arg = (splitmix32(rng) % 6u) + 1u;
            ops[step][0] = kind;
            ops[step][1] = arg;
        }

        auto exec_step = [&](HarnessStateMachine* target, int step) {
            uint32_t k = ops[step][0];
            uint32_t a = ops[step][1];
            Int ret = 0;
            switch (k) {
                case 0: ret = static_cast<Int>(target->initDefaultState()); break;
                case 1:
                    TheGameLogic->m_frame += a;
                    ret = static_cast<Int>(target->updateStateMachine());
                    break;
                case 2: ret = static_cast<Int>(target->setState(a)); break;
                case 3:
                    if (a) target->lock("test");
                    else target->unlock();
                    break;
                case 4:
                    target->setGoalObject(a ? TheGameLogic->findObjectByID(static_cast<ObjectID>(a)) : nullptr);
                    break;
                case 5: {
                    Coord3D p{static_cast<Real>(a) * 0.5f, -static_cast<Real>(a) * 0.25f, 2.0f};
                    target->setGoalPosition(&p);
                    break;
                }
                case 6:
                    TheGameLogic->m_objects.erase(static_cast<ObjectID>(a));
                    break;
                case 7: ret = static_cast<Int>(target->resetToDefaultState()); break;
                case 8: target->clear(); break;
                case 9:
                    if (a == 1) ctx.jump_to_5 = TRUE;
                    else if (a == 2) ctx.wake_early = TRUE;
                    else if (a == 3) ctx.wake_from_5 = TRUE;
                    else if (a == 4) ctx.exit_now = 1;
                    else if (a == 5) ctx.exit_now = 2;
                    else if (a == 6) ctx.exit_now = 3;
                    break;
                case 10: target->halt(); break;
                default: break;
            }
            record_sm_snapshot(ctx, *target, ret);
        };

        for (int step = 0; step < 6; ++step) exec_step(sm, step);

        UnsignedInt mid_frame = TheGameLogic->m_frame;
        Bool obj201_alive = TheGameLogic->findObjectByID(ObjectID(201)) != nullptr ? TRUE : FALSE;
        Bool obj202_alive = TheGameLogic->findObjectByID(ObjectID(202)) != nullptr ? TRUE : FALSE;

        XferCRC xcrc;
        xcrc.open("sm_crc");
        xcrc.xferSnapshot(sm);
        UnsignedInt crc_val = xcrc.getCRC();

        {
            XferSave xsave;
            xsave.open(tmp_save);
            xsave.xferSnapshot(sm);
            xsave.close();
        }
        std::vector<uint8_t> save_bytes = read_file_bytes(tmp_save);

        XferCRC save_xcrc;
        save_xcrc.open("save_crc");
        save_xcrc.xferUser(save_bytes.data(), static_cast<Int>(save_bytes.size()));
        UnsignedInt save_crc_val = save_xcrc.getCRC();

        auto* sm2 = new HarnessStateMachine(&owner_obj, &ctx);
        {
            XferLoad xload;
            xload.open(tmp_save);
            xload.xferSnapshot(sm2);
            xload.close();
        }

        for (int step = 6; step < 12; ++step) exec_step(sm2, step);

        std::fprintf(fp, "    {\"seed\":%u,\"init_frame\":%u,\"max_ticks\":%d,\"enter_sleep\":%d,"
                         "\"update_sleep\":%d,\"re_enter\":%u,\"re_update\":%u,\"chain_depth\":%d,"
                         "\"mid_frame\":%u,\"obj201_alive\":%u,\"obj202_alive\":%u,"
                         "\"ops\":[",
            seed, 10u + (seed % 50u), ctx.max_ticks,
            init_enter_sleep, init_update_sleep, init_re_enter, init_re_update, init_chain_depth,
            mid_frame, obj201_alive, obj202_alive);
        for (int step = 0; step < 12; ++step) {
            std::fprintf(fp, "%u,%u%s", ops[step][0], ops[step][1], (step + 1 < 12) ? "," : "");
        }
        std::fprintf(fp, "],\"xfer_crc\":%u,\"save_crc\":%u,\"trace_crc\":%u,\"xfer_save_hex\":\"",
            crc_val, save_crc_val, ctx.trace_crc);
        for (uint8_t b : save_bytes) {
            std::fprintf(fp, "%02x", b);
        }
        std::fprintf(fp, "\"}%s\n", (seed + 1u < 1000u) ? "," : "");

        sm->Release_Ref();
        sm2->Release_Ref();
    }

    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
    std::remove(tmp_save);
    std::fprintf(stderr, "Generated 1000 PKT-077 vectors to %s\n", argv[1]);
    return 0;
}
