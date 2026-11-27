// Writes the numbers the running reference engine's start handed out, to
// `$ZH_DUMP_PATH`: one `key<TAB>name` row per `NameKeyGenerator` key in key order,
// then one `template<TAB>id<TAB>name` row per thing template in id order, the ids a
// replay's build orders carry. Loaded into the stopped engine by
// `gen_name_keys.py`. The names are game content, so the dump stays in the scratch
// directory.

#include <cstdio>
#include <cstdlib>
#include <map>
#include <string>

#define private public
#include "Common/NameKeyGenerator.h"
#undef private
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"

namespace {

struct Dump {
    Dump() {
        const char* path = std::getenv("ZH_DUMP_PATH");
        if (!path) return;
        std::map<int, std::string> keys;
        for (int i = 0; i < NameKeyGenerator::SOCKET_COUNT; ++i) {
            for (Bucket* b = TheNameKeyGenerator->m_sockets[i]; b; b = b->m_nextInSocket) {
                keys[static_cast<int>(b->m_key)] = b->m_nameString.str();
            }
        }
        FILE* out = std::fopen(path, "w");
        if (!out) return;
        std::fprintf(out, "next\t%u\n", static_cast<unsigned>(TheNameKeyGenerator->m_nextID));
        for (const auto& k : keys) std::fprintf(out, "%d\t%s\n", k.first, k.second.c_str());
        std::map<int, std::string> templates;
        for (const ThingTemplate* t = TheThingFactory->firstTemplate(); t;
             t = t->friend_getNextTemplate()) {
            templates[static_cast<int>(t->getTemplateID())] = t->getName().str();
        }
        for (const auto& t : templates) {
            std::fprintf(out, "template\t%d\t%s\n", t.first, t.second.c_str());
        }
        std::fclose(out);
    }
} dump_on_load;

}  // namespace
