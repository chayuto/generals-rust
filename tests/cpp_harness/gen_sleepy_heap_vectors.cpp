// WP-5.0 / WP-7.1 C++ Golden Vector Generator for GameLogic::m_sleepyUpdates & UpdateModule scheduling.
// Embeds verbatim heap & scheduler logic from GameLogic.cpp (3073-3359, 4062-4113) and UpdateModule.h/.cpp.

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <algorithm>
#include <vector>

using Int = int32_t;
using UnsignedInt = uint32_t;
using Bool = uint8_t;
constexpr Bool TRUE = 1;
constexpr Bool FALSE = 0;

enum UpdateSleepTime : Int {
    UPDATE_SLEEP_INVALID = 0,
    UPDATE_SLEEP_NONE = 1,
    UPDATE_SLEEP_FOREVER = 0x3fffffff
};
#define UPDATE_SLEEP(numFrames) ((UpdateSleepTime)(numFrames))

enum SleepyUpdatePhase : Int {
    PHASE_INITIAL = 0,
    PHASE_PHYSICS = 1,
    PHASE_NORMAL = 2,
    PHASE_FINAL = 3
};

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t rot_crc(uint32_t crc, uint32_t word) {
    return ((crc << 1) | (crc >> 31)) + word;
}

struct MockModule {
    UnsignedInt id = 0;
    UnsignedInt m_nextCallFrameAndPhase = 0;
    Int m_indexInLogic = -1;
    SleepyUpdatePhase m_phase = PHASE_NORMAL;
    bool m_inObjList = false;
    uint64_t m_disabledMask = 0;
    uint64_t m_processMask = 0;

    UnsignedInt friend_getPriority() const { return m_nextCallFrameAndPhase; }
    UnsignedInt friend_getNextCallFrame() const { return m_nextCallFrameAndPhase >> 2; }
    SleepyUpdatePhase friend_getNextCallPhase() const { return (SleepyUpdatePhase)(m_nextCallFrameAndPhase & 3u); }
    SleepyUpdatePhase getUpdatePhase() const { return m_phase; }

    void friend_setNextCallFrame(UnsignedInt frame) {
        if (frame > (UnsignedInt)UPDATE_SLEEP_FOREVER)
            frame = (UnsignedInt)UPDATE_SLEEP_FOREVER;
        m_nextCallFrameAndPhase = (frame << 2) | (UnsignedInt)getUpdatePhase();
    }

    Int friend_getIndexInLogic() const { return m_indexInLogic; }
    void friend_setIndexInLogic(Int i) { m_indexInLogic = i; }

    UpdateSleepTime getWakeFrame(UnsignedInt now) const {
        UnsignedInt nextCallFrame = m_nextCallFrameAndPhase >> 2;
        if (nextCallFrame > now)
            return UPDATE_SLEEP(nextCallFrame - now);
        else
            return UPDATE_SLEEP_NONE;
    }

    static UpdateSleepTime frameToSleepTime(UnsignedInt now, UnsignedInt f1, UnsignedInt f2, UnsignedInt f3, UnsignedInt f4) {
        if (f1 > f2) f1 = f2;
        if (f1 > f3) f1 = f3;
        if (f1 > f4) f1 = f4;
        if (f1 > now)
            return UPDATE_SLEEP(f1 - now);
        else
            return UPDATE_SLEEP_NONE;
    }

    bool shouldProcessDisabled() const {
        return (m_disabledMask & ~m_processMask) == 0;
    }
};

using UpdateModulePtr = MockModule*;

inline Bool isLowerPriority(const UpdateModulePtr a, const UpdateModulePtr b) {
    UnsignedInt f1 = a->friend_getPriority();
    UnsignedInt f2 = b->friend_getPriority();
    return f1 > f2;
}

class HarnessLogic {
public:
    UnsignedInt m_frame = 1;
    std::vector<UpdateModulePtr> m_sleepyUpdates;
    UpdateModulePtr m_curUpdateModule = nullptr;

    void validateSleepyUpdate() const {
        int sz = static_cast<int>(m_sleepyUpdates.size());
        if (sz == 0) return;
        for (int i = 0; i < sz; ++i) {
            if (m_sleepyUpdates[i]->friend_getIndexInLogic() != i) {
                std::fprintf(stderr, "FATAL: index mismatch at %d\n", i);
                std::exit(2);
            }
            UnsignedInt pri = m_sleepyUpdates[i]->friend_getPriority();
            if (i > 0) {
                Int i0 = (i + 1) / 2 - 1;
                if (pri < m_sleepyUpdates[i0]->friend_getPriority()) {
                    std::fprintf(stderr, "FATAL: heap parent violation at %d\n", i);
                    std::exit(2);
                }
            }
            Int i1 = 2 * i + 1;
            Int i2 = 2 * i + 2;
            if (i1 < sz && pri > m_sleepyUpdates[i1]->friend_getPriority()) {
                std::fprintf(stderr, "FATAL: heap child1 violation at %d\n", i);
                std::exit(2);
            }
            if (i2 < sz && pri > m_sleepyUpdates[i2]->friend_getPriority()) {
                std::fprintf(stderr, "FATAL: heap child2 violation at %d\n", i);
                std::exit(2);
            }
        }
    }

    void eraseSleepyUpdate(Int i) {
        m_sleepyUpdates[i]->friend_setIndexInLogic(-1);
        Int last = static_cast<Int>(m_sleepyUpdates.size()) - 1;
        if (i < last) {
            m_sleepyUpdates[i] = m_sleepyUpdates[last];
            m_sleepyUpdates[i]->friend_setIndexInLogic(i);
            m_sleepyUpdates.pop_back();
            rebalanceSleepyUpdate(i);
        } else {
            m_sleepyUpdates.pop_back();
        }
    }

    Int rebalanceParentSleepyUpdate(Int i) {
        Int parent = ((i + 1) >> 1) - 1;
        while (parent >= 0 && isLowerPriority(m_sleepyUpdates[parent], m_sleepyUpdates[i])) {
            UpdateModulePtr a = m_sleepyUpdates[parent];
            UpdateModulePtr b = m_sleepyUpdates[i];
            m_sleepyUpdates[i] = a;
            m_sleepyUpdates[parent] = b;
            a->friend_setIndexInLogic(i);
            b->friend_setIndexInLogic(parent);
            i = parent;
            parent = ((parent + 1) >> 1) - 1;
        }
        return i;
    }

    Int rebalanceChildSleepyUpdate(Int i) {
        Int sz = static_cast<Int>(m_sleepyUpdates.size());
        Int child = ((i) << 1) + 1;
        while (child < sz) {
            if (child < sz - 1 && isLowerPriority(m_sleepyUpdates[child], m_sleepyUpdates[child + 1]))
                ++child;
            if (!isLowerPriority(m_sleepyUpdates[i], m_sleepyUpdates[child]))
                break;
            UpdateModulePtr a = m_sleepyUpdates[child];
            UpdateModulePtr b = m_sleepyUpdates[i];
            m_sleepyUpdates[i] = a;
            m_sleepyUpdates[child] = b;
            a->friend_setIndexInLogic(i);
            b->friend_setIndexInLogic(child);
            i = child;
            child = ((i) << 1) + 1;
        }
        return i;
    }

    void rebalanceSleepyUpdate(Int i) {
        i = rebalanceParentSleepyUpdate(i);
        rebalanceChildSleepyUpdate(i);
    }

    void remakeSleepyUpdate() {
        if (m_sleepyUpdates.empty()) return;
        Int parent = static_cast<Int>(m_sleepyUpdates.size()) / 2;
        while (true) {
            rebalanceChildSleepyUpdate(parent);
            if (parent == 0) break;
            --parent;
        }
        validateSleepyUpdate();
    }

    void pushSleepyUpdate(UpdateModulePtr u) {
        m_sleepyUpdates.push_back(u);
        u->friend_setIndexInLogic(static_cast<Int>(m_sleepyUpdates.size()) - 1);
        rebalanceParentSleepyUpdate(static_cast<Int>(m_sleepyUpdates.size()) - 1);
    }

    UpdateModulePtr peekSleepyUpdate() const {
        return m_sleepyUpdates.front();
    }

    void popSleepyUpdate() {
        Int sz = static_cast<Int>(m_sleepyUpdates.size());
        if (sz == 0) return;
        m_sleepyUpdates[0]->friend_setIndexInLogic(-1);
        if (sz > 1) {
            m_sleepyUpdates[0] = m_sleepyUpdates[sz - 1];
            m_sleepyUpdates[0]->friend_setIndexInLogic(0);
            m_sleepyUpdates.pop_back();
            rebalanceChildSleepyUpdate(0);
        } else {
            m_sleepyUpdates.pop_back();
        }
    }

    bool friend_awakenUpdateModule(UpdateModulePtr u, UnsignedInt whenToWakeUp) {
        UnsignedInt now = m_frame;
        if (u == m_curUpdateModule)
            return false;
        if (whenToWakeUp == u->friend_getNextCallFrame())
            return false;
        if ((now > 0) && (u->friend_getNextCallFrame() == now) && (whenToWakeUp == now + 1))
            return false;

        Int idx = u->friend_getIndexInLogic();
        if (u->m_inObjList) {
            u->friend_setNextCallFrame(whenToWakeUp);
            rebalanceSleepyUpdate(idx);
            validateSleepyUpdate();
            return true;
        } else {
            u->friend_setNextCallFrame(whenToWakeUp);
            return true;
        }
    }
};

struct Op {
    Int kind;
    Int target;
    UnsignedInt a1;
    UnsignedInt a2;
};

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "Usage: %s <out.json>\n", argv[0]);
        return 1;
    }

    FILE* fp = std::fopen(argv[1], "wb");
    if (!fp) { std::perror(argv[1]); return 1; }

    std::fprintf(fp, "{\n"
                     "  \"work_package\": \"WP-5.0/WP-7.1\",\n"
                     "  \"module\": \"SleepyUpdateHeap\",\n"
                     "  \"oracle\": \"GameLogic.cpp (3073-3359, 4062-4113) + UpdateModule.h\",\n"
                     "  \"vector_count\": 1000,\n"
                     "  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x243f6a88u;
        Int num_mods = 4 + static_cast<Int>(splitmix32(rng) % 25u); // 4..28 modules
        std::vector<MockModule> mods(static_cast<size_t>(num_mods));
        for (Int i = 0; i < num_mods; ++i) {
            mods[i].id = static_cast<UnsignedInt>(i);
            mods[i].m_phase = static_cast<SleepyUpdatePhase>(splitmix32(rng) % 4u);
            mods[i].m_processMask = static_cast<uint64_t>(splitmix32(rng) & 0xfu);
            mods[i].m_disabledMask = 0;
            mods[i].m_inObjList = false;
        }

        HarnessLogic logic;
        logic.m_frame = 1u + (seed % 40u);
        uint32_t init_frame = logic.m_frame;

        auto fold_layout = [&](uint32_t& crc) {
            crc = rot_crc(crc, static_cast<uint32_t>(logic.m_sleepyUpdates.size()));
            for (size_t i = 0; i < logic.m_sleepyUpdates.size(); ++i) {
                UpdateModulePtr u = logic.m_sleepyUpdates[i];
                crc = rot_crc(crc, u->id);
                crc = rot_crc(crc, u->friend_getPriority());
                crc = rot_crc(crc, static_cast<uint32_t>(u->friend_getIndexInLogic()));
            }
        };

        std::vector<Op> ops(60);
        uint32_t layout_crc = 0;
        uint32_t tick_trace_crc = 0;
        uint32_t helper_crc = 0;

        for (int s = 0; s < 60; ++s) {
            Int kind = static_cast<Int>(splitmix32(rng) % 8u);
            Int target = static_cast<Int>(splitmix32(rng) % static_cast<UnsignedInt>(num_mods));
            UnsignedInt a1 = 0, a2 = 0;

            if (s < 10 && !mods[target].m_inObjList) {
                kind = 0; // quickly populate early steps
            }

            if (kind == 0) {
                // push / register if idle
                UnsignedInt mode = splitmix32(rng) % 5u;
                a1 = (mode == 0) ? 0u :
                     (mode == 1) ? static_cast<UnsignedInt>(UPDATE_SLEEP_FOREVER) :
                     (mode == 2) ? static_cast<UnsignedInt>(UPDATE_SLEEP_FOREVER) + 50u :
                     (logic.m_frame + (splitmix32(rng) % 12u));
                if (!mods[target].m_inObjList) {
                    if (a1 > 0) mods[target].friend_setNextCallFrame(a1);
                    UnsignedInt now = logic.m_frame ? logic.m_frame : 1u;
                    if (mods[target].friend_getNextCallFrame() == 0)
                        mods[target].friend_setNextCallFrame(now);
                    mods[target].m_inObjList = true;
                    logic.pushSleepyUpdate(&mods[target]);
                    logic.validateSleepyUpdate();
                }
            } else if (kind == 1) {
                // awaken
                UnsignedInt r = splitmix32(rng) % 6u;
                if (r == 0) a1 = mods[target].friend_getNextCallFrame(); // same frame no-op
                else if (r == 1) a1 = logic.m_frame + 1u; // tests now==cur && when==now+1!
                else if (r == 2) a1 = static_cast<UnsignedInt>(UPDATE_SLEEP_FOREVER);
                else if (r == 3) a1 = static_cast<UnsignedInt>(UPDATE_SLEEP_FOREVER) + 77u;
                else a1 = logic.m_frame + (splitmix32(rng) % 20u);
                bool changed = logic.friend_awakenUpdateModule(&mods[target], a1);
                layout_crc = rot_crc(layout_crc, changed ? 1u : 0u);
            } else if (kind == 2) {
                // erase if active
                if (mods[target].m_inObjList) {
                    logic.eraseSleepyUpdate(mods[target].friend_getIndexInLogic());
                    mods[target].m_inObjList = false;
                    logic.validateSleepyUpdate();
                }
            } else if (kind == 3) {
                // pop root if non-empty
                if (!logic.m_sleepyUpdates.empty()) {
                    UpdateModulePtr r = logic.peekSleepyUpdate();
                    logic.popSleepyUpdate();
                    r->m_inObjList = false;
                    logic.validateSleepyUpdate();
                    layout_crc = rot_crc(layout_crc, r->id);
                }
            } else if (kind == 4) {
                // bulk raw priority mutation + remakeSleepyUpdate
                a1 = logic.m_frame + (splitmix32(rng) % 15u);
                a2 = (splitmix32(rng) % 3u) + 1u;
                for (UnsignedInt k = 0; k < a2; ++k) {
                    Int idx_m = (target + static_cast<Int>(k)) % num_mods;
                    mods[idx_m].friend_setNextCallFrame(a1 + k);
                }
                logic.remakeSleepyUpdate();
            } else if (kind == 5) {
                // step_frame: advance by a1 (0..3) and run GameLogic sleepy update loop
                a1 = splitmix32(rng) % 4u;
                a2 = splitmix32(rng); // sleep behavior seed
                logic.m_frame += a1;
                UnsignedInt now = logic.m_frame;
                UnsignedInt step_rng = a2;
                int guard = 0;
                while (!logic.m_sleepyUpdates.empty() && guard < 128) {
                    UpdateModulePtr u = logic.peekSleepyUpdate();
                    if (u->friend_getNextCallFrame() > now) break;
                    ++guard;
                    UpdateSleepTime sleepLen = UPDATE_SLEEP_NONE;
                    if (u->shouldProcessDisabled()) {
                        logic.m_curUpdateModule = u;
                        // Self-awaken inside own update() must be ignored:
                        bool self_aw = logic.friend_awakenUpdateModule(u, now + 99u);
                        tick_trace_crc = rot_crc(tick_trace_crc, self_aw ? 999u : u->id);
                        tick_trace_crc = rot_crc(tick_trace_crc, now);
                        tick_trace_crc = rot_crc(tick_trace_crc, u->friend_getPriority());

                        // Occasionally awaken another future module safely (>= now + 1):
                        if ((splitmix32(step_rng) % 4u) == 0u && num_mods > 1) {
                            Int sib = static_cast<Int>(splitmix32(step_rng) % static_cast<UnsignedInt>(num_mods));
                            if (&mods[sib] != u && mods[sib].m_inObjList && mods[sib].friend_getNextCallFrame() > now + 1u) {
                                bool aw = logic.friend_awakenUpdateModule(&mods[sib], now + 1u);
                                tick_trace_crc = rot_crc(tick_trace_crc, aw ? (100u + mods[sib].id) : 0u);
                            }
                        }

                        UnsignedInt mode = splitmix32(step_rng) % 6u;
                        if (mode == 0) sleepLen = UPDATE_SLEEP_INVALID;
                        else if (mode == 1) sleepLen = UPDATE_SLEEP_NONE;
                        else if (mode == 2) sleepLen = UPDATE_SLEEP_FOREVER;
                        else sleepLen = UPDATE_SLEEP(2u + (splitmix32(step_rng) % 10u));

                        if (sleepLen < 1)
                            sleepLen = UPDATE_SLEEP_NONE;

                        logic.m_curUpdateModule = nullptr;
                    }
                    u->friend_setNextCallFrame(now + static_cast<UnsignedInt>(sleepLen));
                    logic.rebalanceSleepyUpdate(0);
                }
                logic.validateSleepyUpdate();
            } else if (kind == 6) {
                // frameToSleepTime + getWakeFrame
                UnsignedInt delta1 = splitmix32(rng) % 25u;
                UnsignedInt delta2 = splitmix32(rng) % 25u;
                a1 = (delta1 < 5u && logic.m_frame < 5u - delta1) ? 1u : (logic.m_frame + delta1 - 5u);
                a2 = (delta2 < 5u && logic.m_frame < 5u - delta2) ? 1u : (logic.m_frame + delta2 - 5u);
                UnsignedInt f3 = static_cast<UnsignedInt>(UPDATE_SLEEP_FOREVER);
                UnsignedInt f4 = static_cast<UnsignedInt>(UPDATE_SLEEP_FOREVER);
                UpdateSleepTime st1 = MockModule::frameToSleepTime(logic.m_frame, a1, a2, f3, f4);
                UpdateSleepTime st2 = mods[target].getWakeFrame(logic.m_frame);
                helper_crc = rot_crc(helper_crc, static_cast<uint32_t>(st1));
                helper_crc = rot_crc(helper_crc, static_cast<uint32_t>(st2));
            } else {
                // kind == 7: update disabled mask bits on module target
                a1 = splitmix32(rng) & 0xfu;
                mods[target].m_disabledMask = a1;
            }

            ops[s] = Op{kind, target, a1, a2};
            fold_layout(layout_crc);
        }

        std::fprintf(fp,
            "    {\"seed\":%u,\"num_mods\":%d,\"init_frame\":%u,\"phases\":[",
            seed, num_mods, init_frame);
        for (Int i = 0; i < num_mods; ++i) {
            std::fprintf(fp, "%d%s", static_cast<Int>(mods[i].m_phase), (i + 1 < num_mods) ? "," : "");
        }
        std::fprintf(fp, "],\"pmasks\":[");
        for (Int i = 0; i < num_mods; ++i) {
            std::fprintf(fp, "%u%s", static_cast<UnsignedInt>(mods[i].m_processMask), (i + 1 < num_mods) ? "," : "");
        }
        std::fprintf(fp, "],\"ops\":[");
        for (int s = 0; s < 60; ++s) {
            std::fprintf(fp, "%d,%d,%u,%u%s", ops[s].kind, ops[s].target, ops[s].a1, ops[s].a2,
                         (s + 1 < 60) ? "," : "");
        }
        std::fprintf(fp,
            "],\"layout_crc\":%u,\"tick_trace_crc\":%u,\"helper_crc\":%u,\"final_heap\":[",
            layout_crc, tick_trace_crc, helper_crc);
        for (size_t i = 0; i < logic.m_sleepyUpdates.size(); ++i) {
            std::fprintf(fp, "%u%s", logic.m_sleepyUpdates[i]->id,
                         (i + 1 < logic.m_sleepyUpdates.size()) ? "," : "");
        }
        std::fprintf(fp, "]}%s\n", (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
    std::fprintf(stderr, "Generated 1000 WP-5.0/WP-7.1 SleepyUpdateHeap vectors to %s\n", argv[1]);
    return 0;
}
