#!/usr/bin/env python3
"""Records the reference world during a game start (`env:local-full`).

The stock headless engine plays each replay under lldb, which loads
`gen_game_start.cpp` as a dylib into the stopped process, at two stages:

- `before_units`: the first call of `placeNetworkBuildingsForPlayer`, with the
  map's objects made;
- `before_ai`: entry of `PlayerList::newMap`, with every map object and starting
  unit made and the computer players' `AIPlayer::newMap` not yet run;
- `end`: return of the loading `GameLogic::tryStartNewGame`.

The raw dumps name map content, so they go to a scratch directory and are never
committed. Without replay arguments the counts and digests of all replays are
written to `docs/golden_vectors/GameStart.json`.

usage: gen_game_start.py <scratch dir> [replay.rep ...]
"""

import os
import pathlib
import shlex
import subprocess
import sys
import urllib.parse

ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = pathlib.Path(__file__).with_suffix(".cpp")
BINARY = ROOT / "vendor/GeneralsX/build/macos-vulkan/GeneralsMD/GeneralsXZH"
ASSETS = pathlib.Path.home() / "Games/generalszh"
REPLAYS = pathlib.Path.home() / "Library/Application Support/GeneralsX/GeneralsZH/Replays"


def build_dylib(out: pathlib.Path) -> None:
    """Compiles the dumper with the engine's own flags."""
    printed = subprocess.run(
        [sys.executable, str(ROOT / "scripts/build_oracle.py"), str(SOURCE),
         "-o", str(out), "--dry-run", "--print-cmd"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout
    line = next(l for l in printed.splitlines() if " -D" in l)
    tokens = shlex.split(line)
    flags = []
    index = 1
    while index < len(tokens):
        token = tokens[index]
        if token in ("-isystem", "-include", "-arch", "-isysroot"):
            flags += [token, tokens[index + 1]]
            index += 2
            continue
        if token.startswith(("-D", "-I", "-std=", "-f", "-U", "-m")):
            flags.append(token)
        index += 1
    subprocess.run(
        [tokens[0], *flags, "-O1", "-w", "-dynamiclib", "-undefined", "dynamic_lookup",
         str(SOURCE), "-o", str(out)],
        cwd=ROOT, check=True,
    )


STAGES = {
    "before_units": ["breakpoint set --name placeNetworkBuildingsForPlayer -N stop", "continue"],
    "before_ai": ["breakpoint set --name PlayerList::newMap -N stop", "continue"],
    "end": ["thread step-out"],
}


# `(case, map, seed, slots)`: games no replay on disk plays. Each runs the macOS
# vanilla replay with its options string swapped, from a scratch `HOME`. A human slot
# is `H<name>,ip,port,flags,color,template,start,team,nat`, a computer one
# `C<E|M|H>,color,template,start,team`, `X` closed.
HUMAN = "Hprobe,0,0,TT,"
VARIANTS = [
    ("teams_random", "defcon6", 12345,
     HUMAN + "-1,-1,-1,0,1:CE,-1,-1,-1,0:CE,-1,-1,-1,1:CE,-1,-1,-1,1:X:X:X:X:"),
    ("observer", "defcon6", 777,
     HUMAN + "2,-2,-1,-1,1:CE,-1,-1,-1,-1:CM,-1,-1,-1,-1:X:X:X:X:X:"),
    ("chosen_spots", "defcon6", 4242,
     HUMAN + "0,2,1,1,1:CE,3,3,3,1:CH,-1,-1,-1,0:CE,-1,-1,-1,0:X:X:X:X:"),
    ("defcon6", "defcon6", 99, HUMAN + "-1,-1,-1,2,1:CE,-1,-1,-1,2:CE,-1,-1,-1,3:CH,-1,-1,-1,3:X:X:X:X:"),
    ("old_factions", "defcon6", 31337,
     HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:CE,-1,-1,-1,-1:CE,-1,-1,-1,-1:X:X:X:X:"),
    ("same_colors", "defcon6", 2024,
     HUMAN + "3,-1,-1,-1,1:CE,3,-1,-1,-1:CE,3,-1,-1,-1:X:X:X:X:X:"),
    ("alpine_assault", "alpine assault", 1, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("bitter_winter", "bitter winter", 2, HUMAN + "-1,-1,-1,-1,1:CM,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("dark_night", "dark night", 3, HUMAN + "-1,-1,-1,-1,1:CH,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("eastern_everglades", "eastern everglades", 4, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("fallen_empire", "fallen empire", 5, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("flash_fire", "flash fire", 6, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("leipzig_lowlands", "leipzig lowlands", 7, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("seaside_mutiny", "seaside mutiny", 8, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("silent_river", "silent river", 9, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("tournament_city", "tournament city", 10, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("whiteout", "whiteout", 11, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
    ("winding_river", "winding river", 13, HUMAN + "-1,-1,-1,-1,1:CE,-1,-1,-1,-1:X:X:X:X:X:X:"),
]


def variant_options(map_name: str, seed: int, slots: str) -> str:
    old = "Y" if seed == 31337 else "N"
    return f"US=1;M=07maps/{map_name};MC=0;MS=0;SD={seed};C=100;SR=0;SC=10000;O={old};S={slots};"


def write_variant(home: pathlib.Path, name: str, options: str) -> None:
    data = (REPLAYS / "macos_2p_vanilla_map.rep").read_bytes()
    start = data.index(b"US=1;M=")
    end = data.index(b"\0", start)
    folder = home / REPLAYS.relative_to(pathlib.Path.home())
    folder.mkdir(parents=True, exist_ok=True)
    (folder / f"{name}.rep").write_bytes(data[:start] + options.encode() + data[end:])


def dump(dylib: pathlib.Path, replay: str, stage: str, out: pathlib.Path,
         home: pathlib.Path | None = None) -> None:
    env = dict(os.environ)
    if home:
        env["HOME"] = str(home)
    env.update(SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy",
               CNC_GENERALS_ZH_PATH=str(ASSETS))
    out.unlink(missing_ok=True)
    env["ZH_DUMP_PATH"] = str(out)
    commands = [
        # The first call of a game only asks to be run again; the second loads.
        "breakpoint set --name GameLogic::tryStartNewGame",
        "run",
        "continue",
        "breakpoint disable",
        *STAGES[stage],
        f"process load {dylib}",
        "kill",
        "quit",
    ]
    argv = ["lldb", "--batch"]
    for command in commands:
        argv += ["-o", command]
    argv += ["-k", "bt 15", "-k", "kill", "-k", "quit", "--", str(BINARY), "-headless",
             "-assetPath", str(ASSETS), "-ignoreReplaySyncErrors", "-replay", replay]
    done = subprocess.run(argv, env=env, cwd=BINARY.parent, capture_output=True,
                          text=True, errors="replace", timeout=600)
    if not out.exists():
        sys.stderr.write(done.stdout[-3000:] + done.stderr[-3000:])
        raise SystemExit(f"{replay}: no dump")


# `(column, 0-based tab fields)` of an object row, as `golden_game_start.rs` reads them.
COLUMNS = [("template", (2,)), ("team", (3, 4, 5)), ("position", (6, 7, 8)), ("angle", (9,)),
           ("layer", (10,)), ("health", (11, 12)), ("status", (13, 14)), ("vision", (15, 16))]


def digest(rows) -> str:
    value = 0xCBF29CE484222325
    for row in rows:
        for byte in row + b"\n":
            value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016x}"


def summarize(path: pathlib.Path) -> str:
    """One dump's counts and digests. Objects are compared in id order, the map's
    (`map_`) apart from those of the playing sides' starting teams (`start_`)."""
    rows = path.read_bytes().split(b"\n")[:-1]
    kinds = {k: [r for r in rows if r.split(b"\t")[0] == k.encode()]
             for k in ("seed", "game", "player", "team", "obj")}
    words = []
    for kind in ("seed", "game", "player", "team"):
        words.append(f"{kind}_count={len(kinds[kind])}")
        words.append(f"{kind}_digest={digest(kinds[kind])}")
    objects = [r.split(b"\t") for r in kinds["obj"]]
    objects.sort(key=lambda c: int(c[1]))
    playing = (b"teamplayer", b"teamSkirmish")
    for group, started in (("map", False), ("start", True)):
        cells = [c for c in objects if c[4].startswith(playing) == started]
        words.append(f"{group}_count={len(cells)}")
        if started:
            words.append("start_positions=" + ",".join(
                digest([b"\t".join([c[1], c[6], c[7], c[8]])]) for c in cells))
        for name, fields in COLUMNS:
            words.append(f"{group}_{name}_digest=" + digest(
                b"\t".join([c[1]] + [c[f] for f in fields]) for c in cells))
    # What can still be compared on a map with bridges, whose objects shift every id:
    # the map's other objects, by content.
    loose = sorted(
        b"\t".join([c[2], c[4], c[5]] + c[6:14] + c[15:17]) for c in objects
        if not c[4].startswith(playing) and b"Bridge" not in c[2])
    words.append(f"unbridged_count={len(loose)}")
    words.append(f"unbridged_digest={digest(loose)}")
    return " ".join(words)


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    scratch = pathlib.Path(sys.argv[1]).resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    dylib = scratch / "zh_game_start.dylib"
    build_dylib(dylib)
    wanted = sys.argv[2:]
    home = scratch / "home"
    games = [(p.name, None) for p in sorted(REPLAYS.glob("*.rep"))]
    for name, map_name, seed, slots in VARIANTS:
        options = variant_options(map_name, seed, slots)
        write_variant(home, name, options)
        games.append((f"{name}.rep", options))
    cases = []
    for replay, options in games:
        if wanted and replay not in wanted:
            continue
        for stage in STAGES:
            out = scratch / f"{replay.removesuffix('.rep')}.{stage}.ref.txt"
            dump(dylib, replay, stage, out, home if options else None)
            lines = out.read_text(errors="replace").splitlines()
            print(replay, stage, {k: sum(l.startswith(k + "\t") for l in lines)
                                  for k in ("player", "team", "obj")}, flush=True)
            words = f" options={urllib.parse.quote(options)}" if options else ""
            cases.append(f"{replay} stage={stage}{words} {summarize(out)}")
    if not wanted:
        body = ",\n".join(f'    "{c}"' for c in cases)
        golden = ROOT / "docs/golden_vectors/GameStart.json"
        golden.write_text(f'{{\n  "cases": [\n{body}\n  ]\n}}\n')
    return 0


if __name__ == "__main__":
    sys.exit(main())
