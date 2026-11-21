#pragma once
#include "Common/GameMemory.h"
#include "Common/Snapshot.h"
#include "Common/DataChunk.h"
#include "Common/Xfer.h"

struct Script { int dummy = 1; };
struct ScriptGroup { int dummy = 1; };

class ScriptList : public MemoryPoolObject, public Snapshot {
    MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ScriptList, "ScriptList")
public:
    Bool m_hasScript = FALSE;
    Bool m_hasGroup = FALSE;
    Int m_tag = 0;
    Script m_dummyScript{};
    ScriptGroup m_dummyGroup{};

    static ScriptList* s_readLists[MAX_PLAYER_COUNT];
    static Int s_numInReadList;

    ScriptList(Bool hasScript = FALSE, Bool hasGroup = FALSE, Int tag = 0)
        : m_hasScript(hasScript), m_hasGroup(hasGroup), m_tag(tag) {}

    ScriptGroup* getScriptGroup() { return m_hasGroup ? &m_dummyGroup : nullptr; }
    Script* getScript() { return m_hasScript ? &m_dummyScript : nullptr; }

    ScriptList* duplicate() const {
        return new ScriptList(m_hasScript, m_hasGroup, m_tag);
    }

    virtual void crc(Xfer*) override {}
    virtual void xfer(Xfer* x) override {
        XferVersion currentVersion = 1;
        XferVersion version = currentVersion;
        x->xferVersion(&version, currentVersion);
        x->xferByte(reinterpret_cast<Byte*>(&m_hasScript));
        x->xferByte(reinterpret_cast<Byte*>(&m_hasGroup));
        x->xferInt(&m_tag);
    }
    virtual void loadPostProcess() override {}

    static void resetReadLists() {
        for (Int i = 0; i < MAX_PLAYER_COUNT; ++i) {
            deleteInstance(s_readLists[i]);
            s_readLists[i] = nullptr;
        }
        s_numInReadList = 0;
    }

    static Bool ParseScriptsDataChunk(DataChunkInput& file, DataChunkInfo*, void*) {
        resetReadLists();
        Int count = file.readInt();
        for (Int i = 0; i < count; ++i) {
            Byte present = file.readByte();
            Bool hs = file.readByte() != 0 ? TRUE : FALSE;
            Bool hg = file.readByte() != 0 ? TRUE : FALSE;
            Int tag = file.readInt();
            if (i < MAX_PLAYER_COUNT) {
                if (present != 0) {
                    s_readLists[s_numInReadList++] = new ScriptList(hs, hg, tag);
                } else {
                    s_readLists[s_numInReadList++] = nullptr;
                }
            }
        }
        return TRUE;
    }

    static void WriteScriptsDataChunk(DataChunkOutput& chunkWriter, ScriptList* scriptLists[], Int numLists) {
        chunkWriter.openDataChunk("PlayerScriptsList", 1);
        chunkWriter.writeInt(numLists);
        for (Int i = 0; i < numLists; ++i) {
            ScriptList* sl = scriptLists[i];
            if (sl) {
                chunkWriter.writeByte(1);
                chunkWriter.writeByte(sl->m_hasScript ? 1 : 0);
                chunkWriter.writeByte(sl->m_hasGroup ? 1 : 0);
                chunkWriter.writeInt(sl->m_tag);
            } else {
                chunkWriter.writeByte(0);
                chunkWriter.writeByte(0);
                chunkWriter.writeByte(0);
                chunkWriter.writeInt(0);
            }
        }
        chunkWriter.closeDataChunk();
    }

    static Int getReadScripts(ScriptList* scriptLists[MAX_PLAYER_COUNT]) {
        Int n = s_numInReadList;
        for (Int i = 0; i < n; ++i) {
            scriptLists[i] = s_readLists[i];
            s_readLists[i] = nullptr;
        }
        s_numInReadList = 0;
        return n;
    }
};

inline ScriptList::~ScriptList() = default;
inline ScriptList* ScriptList::s_readLists[MAX_PLAYER_COUNT]{};
inline Int ScriptList::s_numInReadList = 0;
