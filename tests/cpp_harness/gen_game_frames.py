#!/usr/bin/env python3
"""Records the reference world over the first logic frames of a game
(`env:local-full`).

The games, the stock headless engine and the player commands are those of
`gen_game_start.py`: each baseline replay plays its own commands, and each option
variant plays those of `macos_2p_vanilla_map.rep` under its own options. Under lldb
the engine stops at every `GameLogic::update`, and `gen_game_frames.cpp` writes the
world as it is at the entry of frame N, which is after N logic frames.

The raw dumps name map content, so they go to the scratch directory and are never
committed. `docs/golden_vectors/GameFrames.json` holds one line per game and field:
`<game> <field> <frame>:<digest> ...`, listing the frames at which the field's digest
changes. Object fields are `<group>.<field>` (see `GROUPS`).

The rows are not an oracle build over the library: they are read out of the running
stock engine, so there is no `-O0` or sanitizer build to compare.

usage: gen_game_frames.py <scratch dir> [--no-run] [--jobs N] [replay.rep ...]
"""

import argparse
import concurrent.futures
import os
import pathlib
import shutil
import subprocess
import sys

import gen_game_start as start

ROOT = pathlib.Path(__file__).resolve().parents[2]
GOLDEN = ROOT / "docs/golden_vectors/GameFrames.json"
LAST_FRAME = 300

# Runs inside lldb: stop at every logic update and call the dumper there.
DRIVER = """\
import lldb

def drive(debugger, command, result, internal_dict):
    last = int(command)
    debugger.SetAsync(False)
    target = debugger.GetSelectedTarget()
    process = target.GetProcess()
    symbol = target.FindSymbols("zh_dump_frame")[0].symbol
    address = symbol.GetStartAddress().GetLoadAddress(target)
    target.BreakpointCreateByName("GameLogic::update")
    options = lldb.SBExpressionOptions()
    # The dumper can need a lock another thread holds, and on a loaded machine a call
    # can overrun lldb's default limits: run one thread briefly, then all of them.
    options.SetIgnoreBreakpoints(True)
    options.SetOneThreadTimeoutInMicroSeconds(1000 * 1000)
    options.SetTimeoutInMicroSeconds(120 * 1000 * 1000)
    while True:
        process.Continue()
        thread = process.GetSelectedThread()
        if (process.GetState() != lldb.eStateStopped
                or thread.GetStopReason() != lldb.eStopReasonBreakpoint):
            print("zh: stopped early")
            return
        value = thread.GetFrameAtIndex(0).EvaluateExpression(
            "((unsigned(*)())0x%x)()" % address, options)
        if value.GetError().Fail():
            print("zh: %s" % value.GetError())
            return
        if value.GetValueAsUnsigned() >= last:
            return
"""


def run_game(dylib: pathlib.Path, driver: pathlib.Path, replay: str,
             out: pathlib.Path, home: pathlib.Path | None) -> None:
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir(parents=True)
    env = dict(os.environ)
    if home:
        env["HOME"] = str(home)
    env.update(SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy",
               CNC_GENERALS_ZH_PATH=str(start.ASSETS), ZH_FRAMES_DIR=str(out))
    env.pop("ZH_DUMP_PATH", None)
    commands = [
        # The first call of a game only asks to be run again; the second loads.
        "breakpoint set --name GameLogic::tryStartNewGame",
        "run",
        "continue",
        "breakpoint disable",
        "thread step-out",
        f"process load {dylib}",
        f"command script import {driver}",
        f"command script add -f {driver.stem}.drive zh_drive",
        f"zh_drive {LAST_FRAME}",
        "kill",
        "quit",
    ]
    argv = ["lldb", "--batch"]
    for command in commands:
        argv += ["-o", command]
    argv += ["-k", "bt 15", "-k", "kill", "-k", "quit", "--", str(start.BINARY), "-headless",
             "-assetPath", str(start.ASSETS), "-ignoreReplaySyncErrors", "-replay", replay]
    done = subprocess.run(argv, env=env, cwd=start.BINARY.parent, capture_output=True,
                          text=True, errors="replace", timeout=3600)
    if not (out / f"f{LAST_FRAME:04}.txt").exists():
        sys.stderr.write(done.stdout[-3000:] + done.stderr[-3000:])
        raise SystemExit(f"{replay}: stopped before frame {LAST_FRAME}")


# `(field, row kind, 0-based tab fields)`, as `golden_game_frames.rs` reads them. An
# empty field list takes the whole row.
WORLD_FIELDS = [
    ("seed", "seed", ()), ("game", "game", ()), ("player", "player", ()),
    ("energy", "energy", ()), ("team", "team", ()),
    ("bridge", "bridge", ()), ("deck", "deck", ()),
]
# Taken apart by who owns the object, so that a difference among one group's objects
# does not hide the others': the starting teams of the human players (`teamplayer*`)
# and of the computer players (`teamSkirmish*`), and everything else.
OBJECT_FIELDS = [
    ("template", "obj", (1, 2)), ("owner", "obj", (1, 3, 4, 5)),
    ("position", "obj", (1, 6, 7, 8)), ("angle", "obj", (1, 9)), ("layer", "obj", (1, 10)),
    ("health", "obj", (1, 11, 12)), ("status", "obj", (1, 13, 14)),
    ("vision", "obj", (1, 15, 16)), ("wake", "wake", ()), ("ai", "ai", ()),
]
GROUPS = (("map", None), ("computer", b"teamSkirmish"), ("human", b"teamplayer"))
# Rows the engine lists in object list order; `module` rows are not digested.
BY_ID = ("obj", "wake", "ai")


def digest32(rows) -> str:
    """The low half of FNV-1a over the rows, each ended by a newline."""
    value = 0xCBF29CE484222325
    for row in rows:
        for byte in row + b"\n":
            value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value & 0xFFFFFFFF:08x}"


def group_of(team: bytes) -> str:
    return next((name for name, prefix in GROUPS if prefix and team.startswith(prefix)), "map")


def pick(rows, columns):
    return (b"\t".join([c[i] for i in columns] if columns else c) for c in rows)


def frame_fields(path: pathlib.Path) -> dict:
    kinds = {}
    for row in path.read_bytes().split(b"\n")[:-1]:
        cells = row.split(b"\t")
        kinds.setdefault(cells[0].decode(), []).append(cells)
    for kind in BY_ID:
        kinds.get(kind, []).sort(key=lambda c: int(c[1]))
    owners = {c[1]: group_of(c[4]) for c in kinds.get("obj", [])}
    out = {}
    for field, kind, columns in WORLD_FIELDS:
        out[field] = digest32(pick(kinds.get(kind, []), columns))
    for group, _ in GROUPS:
        out[f"{group}.count"] = str(sum(g == group for g in owners.values()))
        for field, kind, columns in OBJECT_FIELDS:
            rows = [c for c in kinds.get(kind, []) if owners.get(c[1]) == group]
            out[f"{group}.{field}"] = digest32(pick(rows, columns))
    return out


def summarize(name: str, out: pathlib.Path) -> list:
    lines = {}
    last = {}
    for frame in range(1, LAST_FRAME + 1):
        for field, value in frame_fields(out / f"f{frame:04}.txt").items():
            if last.get(field) != value:
                lines.setdefault(field, []).append(f"{frame}:{value}")
                last[field] = value
    return [f"{name} {field} " + " ".join(changes) for field, changes in lines.items()]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scratch", type=pathlib.Path)
    parser.add_argument("replays", nargs="*")
    parser.add_argument("--no-run", action="store_true", help="summarize existing dumps")
    parser.add_argument("--jobs", type=int, default=1)
    args = parser.parse_args()
    scratch = args.scratch.resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    dylib = scratch / "zh_game_frames.dylib"
    driver = scratch / "zh_frames_driver.py"
    if not args.no_run:
        start.SOURCE = pathlib.Path(__file__).resolve().with_suffix(".cpp")
        start.build_dylib(dylib)
        driver.write_text(DRIVER)
    home = scratch / "home"
    games = [(p.name, None) for p in sorted(start.REPLAYS.glob("*.rep"))]
    for name, map_name, seed, slots in start.VARIANTS:
        start.write_variant(home, name, start.variant_options(map_name, seed, slots))
        games.append((f"{name}.rep", home))
    if args.replays:
        games = [g for g in games if g[0] in args.replays]

    def one(game):
        replay, game_home = game
        out = scratch / replay.removesuffix(".rep")
        if not args.no_run:
            run_game(dylib, driver, replay, out, game_home)
        print(replay, flush=True)
        return summarize(replay, out)

    with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool:
        lines = [line for game in pool.map(one, games) for line in game]
    if not args.replays:
        body = ",\n".join(f'    "{line}"' for line in lines)
        GOLDEN.write_text(f'{{\n  "frames": {LAST_FRAME},\n  "fields": [\n{body}\n  ]\n}}\n')
    return 0


if __name__ == "__main__":
    sys.exit(main())
