#!/usr/bin/env python3
"""Records what `AIPlayer::newMap` leaves behind (`env:local-full`).

Runs the games of `gen_game_start.py` under lldb with `gen_ai_new_map.cpp` loaded
into the stopped engine, before the players' `newMap` and at the end of the start.
Each game then runs `AIPlayer::newMap` once more over a build list marked as built
(stage `built`), and one game calls the map setup's functions with chosen inputs.

The state dumps name map content, so they go to a scratch directory; only their
counts and digests are written to `docs/golden_vectors/AiNewMap.json`, with the
calls made with chosen inputs in full.

usage: gen_ai_new_map.py <scratch dir> [replay.rep ...]
"""

import os
import pathlib
import sys

import gen_game_start as game_start

ROOT = game_start.ROOT
STAGES = ("before_ai", "end")
KINDS = ("seed", "flags", "ai", "build", "score", "energy", "objx", "unit", "path", "cells", "shroud")
# The game whose start also makes the calls with chosen inputs.
SYNTHETIC = "macos_2p_vanilla_map.rep"


def summarize(path: pathlib.Path) -> str:
    rows = [r for r in path.read_bytes().split(b"\n") if r and not r.startswith(b"synth ")]
    words = []
    for kind in KINDS:
        of = [r for r in rows if r.split(b"\t")[0] == kind.encode()]
        words.append(f"{kind}={len(of)}:{game_start.digest(of)}")
    return " ".join(words)


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    scratch = pathlib.Path(sys.argv[1]).resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    dylib = scratch / "zh_ai_new_map.dylib"
    game_start.SOURCE = pathlib.Path(__file__).resolve().with_suffix(".cpp")
    game_start.build_dylib(dylib)
    wanted = sys.argv[2:]
    home = scratch / "home"
    games = [(p.name, None) for p in sorted(game_start.REPLAYS.glob("*.rep"))]
    for name, map_name, seed, slots in game_start.VARIANTS:
        options = game_start.variant_options(map_name, seed, slots)
        game_start.write_variant(home, name, options)
        games.append((f"{name}.rep", options))
    cases = []
    synthetic = []
    for replay, options in games:
        if wanted and replay not in wanted:
            continue
        stem = replay.removesuffix(".rep")
        user = home if options else None
        for stage in STAGES:
            out = scratch / f"{stem}.{stage}.ai.txt"
            built = pathlib.Path(f"{out}.built")
            built.unlink(missing_ok=True)
            if stage == "end":
                os.environ["ZH_DUMP_BUILD"] = "1"
            game_start.dump(dylib, replay, stage, out, user)
            os.environ.pop("ZH_DUMP_BUILD", None)
            cases.append(f"{replay} stage={stage} {summarize(out)}")
            if stage == "end":
                cases.append(f"{replay} stage=built {summarize(built)}")
            print(replay, stage, flush=True)
        if replay == SYNTHETIC:
            out = scratch / f"{stem}.synth.ai.txt"
            os.environ["ZH_DUMP_SYNTHETIC"] = "1"
            game_start.dump(dylib, replay, "end", out, user)
            os.environ.pop("ZH_DUMP_SYNTHETIC", None)
            lines = out.read_text(errors="replace").splitlines()
            synthetic += [l for l in lines if l.startswith("synth ")]
    if not wanted:
        body = ",\n".join(f'    "{c}"' for c in cases + synthetic)
        golden = ROOT / "docs/golden_vectors/AiNewMap.json"
        golden.write_text(f'{{\n  "cases": [\n{body}\n  ]\n}}\n')
    return 0


if __name__ == "__main__":
    sys.exit(main())
