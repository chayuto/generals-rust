#!/usr/bin/env python3
"""
Counts the module classes the engine creates while playing the local replays, and
writes `docs/MODULE_USAGE.md` ranking the ones not yet ported (`env:local-full` only).

Builds `GeneralsXZH-census` (the stock engine plus one log line per module), checks
that it still matches the replay baseline, and joins the counts with
`docs/MODULE_MANIFEST.json` and `docs/PORT_MAP.md`.

  python3 scripts/module_census.py
"""

from __future__ import annotations

import collections
import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import build_rust_engine  # noqa: E402
import check_replays  # noqa: E402

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
VENDOR = REPO_ROOT / "vendor/GeneralsX"
OUTPUT = REPO_ROOT / "docs/MODULE_USAGE.md"
COUNTS = REPO_ROOT / "docs/module_instances.json"
LINE = re.compile(r"^ZH_CENSUS (\d+) (\S+)$", re.MULTILINE)
KINDS = {0: "behavior", 1: "draw", 2: "client update"}
PORT_ROW = re.compile(r"^\| `(PKT-\d+)` \| `[^`]*` \|[^|]*\|[^|]*\| `(\w+)` \|", re.MULTILINE)


def parse_census(stderr: str) -> collections.Counter:
    """Instances per (module type, name) in one run's stderr."""
    return collections.Counter((int(kind), name) for kind, name in LINE.findall(stderr))


def port_status(port_map: str) -> dict[str, str]:
    return dict(PORT_ROW.findall(port_map))


def defining_files(names: set[str], sources: dict[str, str]) -> dict[str, str]:
    """The source file holding each class's constructor; `sources` is path -> text."""
    found: dict[str, str] = {}
    constructor = re.compile(r"^(\w+)::\1\s*\(", re.MULTILINE)
    # Zero Hour's copy wins over the base game's.
    for path in sorted(sources, key=lambda p: (not p.startswith("GeneralsMD/"), p)):
        for name in constructor.findall(sources[path]):
            if name in names:
                found.setdefault(name, path)
    return found


def packet_of(path: str | None, name: str, manifest: dict) -> str | None:
    """The packet listing `path`, else the packet named after the class."""
    for entry in manifest["packets"]:
        if path and any(f["path"] == path for f in entry["cpp_files"]):
            return entry["packet_id"]
    for entry in manifest["packets"]:
        if entry["module_name"] == name:
            return entry["packet_id"]
    return None


def rank(runs: list[collections.Counter]) -> list[tuple[str, int, int]]:
    """Behavior modules as (name, replays that create it, instances), most used first."""
    replays: collections.Counter = collections.Counter()
    instances: collections.Counter = collections.Counter()
    for run in runs:
        for (kind, name), count in run.items():
            if kind == 0:
                replays[name] += 1
                instances[name] += count
    return sorted(
        ((n, replays[n], instances[n]) for n in instances), key=lambda r: (-r[1], -r[2], r[0])
    )


def render(rows: list[tuple], total_replays: int) -> str:
    """`rows` are (name, replays, instances, packet, status, bytes)."""
    done = [r for r in rows if r[4] == "implemented"]
    todo = [r for r in rows if r[4] != "implemented"]
    total = sum(r[2] for r in rows)
    ported = sum(r[2] for r in done)
    every = [r for r in rows if r[1] == total_replays]
    out = [
        "# Behavior modules the replays create",
        "",
        "Written by `python3 scripts/module_census.py`. Do not edit by hand.",
        "",
        f"Counts are module instances over the {total_replays} local replays. They show what is",
        "created, not how often it runs, and cover behavior modules only (not AI, pathfinding",
        "or weapon code outside modules). The replays do not match their recordings",
        "(`BASELINE.md`), so they are a sample of play, not the original games.",
        "",
        f"- Classes created: {len(rows)}; in every replay: {len(every)}.",
        f"- Classes with a ported packet: {len(done)} of {len(rows)}; "
        f"in every replay: {sum(r[4] == 'implemented' for r in every)} of {len(every)}.",
        f"- Instances with a ported packet: {ported} of {total}.",
        "",
        "## Not ported",
        "",
        "| Module | Packet | Replays | Instances | C++ bytes in packet |",
        "|---|---|---|---|---|",
    ]
    for name, replays, instances, pkt, _, size in todo:
        out.append(f"| `{name}` | `{pkt or '-'}` | {replays} | {instances} | {size} |")
    return "\n".join(out) + "\n"


def counts_json(rows: list[tuple]) -> str:
    """Every class the replays create, for scripts that need the ported ones too."""
    classes = {r[0]: {"replays": r[1], "instances": r[2], "packet": r[3]} for r in rows}
    return json.dumps(classes, indent=1, sort_keys=True) + "\n"


def main() -> int:
    binary = build_rust_engine.build(None, rust=False, census=True)
    baseline = json.loads(check_replays.DEFAULT_BASELINE.read_text())["replays"]
    names = check_replays.replay_names(check_replays.DEFAULT_REPLAYS)
    if sorted(names) != sorted(baseline):
        raise check_replays.ReplayError("the local replays differ from the baseline's")
    runs = []
    for name in names:
        sink: list[str] = []
        result = check_replays.run_replay(
            binary, check_replays.DEFAULT_ASSETS, name, 600, stderr_sink=sink
        )
        problems = check_replays.compare(name, baseline[name], result)
        if problems:
            raise check_replays.ReplayError(problems[0])
        runs.append(parse_census(sink[0]))
        if not runs[-1]:
            raise check_replays.ReplayError(f"{name}: the census engine logged no module")

    ranked = rank(runs)
    sources = {
        str(p.relative_to(VENDOR)): p.read_text(errors="replace")
        for top in ("GeneralsMD/Code", "Core")
        for p in (VENDOR / top).rglob("*.cpp")
    }
    files = defining_files({r[0] for r in ranked}, sources)
    manifest = json.loads((REPO_ROOT / "docs/MODULE_MANIFEST.json").read_text())
    status = port_status((REPO_ROOT / "docs/PORT_MAP.md").read_text())
    sizes = {e["packet_id"]: e["total_bytes"] for e in manifest["packets"]}
    rows = []
    for name, replays, instances in ranked:
        pkt = packet_of(files.get(name), name, manifest)
        rows.append((name, replays, instances, pkt, status.get(pkt, "unported"), sizes.get(pkt, 0)))
    OUTPUT.write_text(render(rows, len(names)))
    COUNTS.write_text(counts_json(rows))
    print("\n".join(OUTPUT.read_text().splitlines()[9:12]))
    print(f"wrote {OUTPUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
