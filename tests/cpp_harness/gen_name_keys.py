#!/usr/bin/env python3
"""Records the key table of the running reference engine's `NameKeyGenerator`
(`env:local-full`).

The stock headless engine starts a replay under lldb, which loads
`gen_name_keys.cpp` as a dylib into the process as the game start is entered, when
every name of the engine start has been interned and none of a game's. The table
is the same for every replay.

The names include game content, so the raw table stays in the scratch directory.
`docs/golden_vectors/NameKeys.json` holds the number of keys and a digest of the
names of every run of `CHUNK` keys, and likewise for the thing templates by id (a
digest of `id<TAB>name` rows), which `golden_name_keys.rs` compares with the port's
loader.

usage: gen_name_keys.py <scratch dir> [replay.rep]
"""

import pathlib
import sys

import gen_game_start as start

ROOT = pathlib.Path(__file__).resolve().parents[2]
GOLDEN = ROOT / "docs/golden_vectors/NameKeys.json"
CHUNK = 64
#: The window and module names, which no game data enters: keys 97 to 608.
FIXED = (97, 608)


def names_of(path: pathlib.Path) -> tuple:
    """The names in key order, and the `id<TAB>name` rows of the templates."""
    rows = path.read_bytes().split(b"\n")[:-1]
    templates = [r.removeprefix(b"template\t") for r in rows if r.startswith(b"template\t")]
    rows = [r for r in rows if not r.startswith(b"template\t")]
    assert rows[0].startswith(b"next\t")
    names = []
    for key, row in enumerate(rows[1:], start=1):
        number, name = row.split(b"\t", 1)
        assert int(number) == key, f"key {key} is missing"
        names.append(name)
    assert int(rows[0].split(b"\t")[1]) == len(names) + 1
    return names, templates


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    scratch = pathlib.Path(sys.argv[1]).resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    replay = sys.argv[2] if len(sys.argv) > 2 else "macos_2p_vanilla_map.rep"
    dylib = scratch / "zh_name_keys.dylib"
    table = scratch / "name_keys.txt"
    start.SOURCE = pathlib.Path(__file__).resolve().with_suffix(".cpp")
    start.build_dylib(dylib)
    start.STAGES["loaded"] = []
    start.dump(dylib, replay, "loaded", table)
    names, templates = names_of(table)
    cases = [f"count {len(names)}"]
    cases.append(f"fixed {FIXED[0]}-{FIXED[1]} {start.digest(names[FIXED[0] - 1:FIXED[1]])}")
    for first in range(0, len(names), CHUNK):
        chunk = names[first:first + CHUNK]
        cases.append(f"chunk {first + 1}-{first + len(chunk)} {start.digest(chunk)}")
    cases.append(f"templates {len(templates)}")
    for first in range(0, len(templates), CHUNK):
        chunk = templates[first:first + CHUNK]
        cases.append(f"ids {first + 1}-{first + len(chunk)} {start.digest(chunk)}")
    body = ",\n".join(f'    "{case}"' for case in cases)
    GOLDEN.write_text('{\n  "cases": [\n' + body + "\n  ]\n}\n")
    print(f"{GOLDEN}: {len(names)} keys, {len(templates)} templates")
    return 0


if __name__ == "__main__":
    sys.exit(main())
