#!/usr/bin/env python3
"""
Differential replay check against a local CRC baseline (`env:local-full` only).

The public replays do not match this machine's retail data, so their recorded
CRCs cannot be used as the oracle. The engine still plays them deterministically,
and prints its own logic CRC each time it differs from the recording (every 100
frames). This script records that trace from the stock C++ build and compares a
later build (for example one with Rust code linked in) against it.

  python3 scripts/check_replays.py --write                 # stock build: record
  python3 scripts/check_replays.py --check                 # compare
  python3 scripts/check_replays.py --check --expect-rust   # and require Rust symbols

The game is always started headless with dummy video and audio drivers.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
REFERENCE_COMMIT = "84a7a20769aa5749f388b1f49bd542c79b3c33e8"
DEFAULT_BINARY = REPO_ROOT / "vendor/GeneralsX/build/macos-vulkan/GeneralsMD/GeneralsXZH"
DEFAULT_BASELINE = REPO_ROOT / "docs/baselines/replay_crc.json"
DEFAULT_ASSETS = pathlib.Path.home() / "Games/generalszh"
DEFAULT_REPLAYS = (
    pathlib.Path.home() / "Library/Application Support/GeneralsX/GeneralsZH/Replays"
)

CRC_LINE = re.compile(
    r"^CRC Mismatch in Frame (\d+) \(ignored\) InGame:([0-9A-F]{8}) Replay:([0-9A-F]{8})\s*$"
)
TIME_LINE = re.compile(r"Game Time: (\d+:\d+)/(\d+:\d+)")


class ReplayError(Exception):
    pass


def parse_log(text: str) -> dict:
    """Extracts the CRC trace and the final game time from one engine log."""
    crcs = []
    game_time = None
    for line in text.splitlines():
        match = CRC_LINE.match(line)
        if match:
            crcs.append([int(match.group(1)), match.group(2), match.group(3)])
            continue
        times = TIME_LINE.search(line)
        if times:
            game_time = [times.group(1), times.group(2)]
    return {"crcs": crcs, "game_time": game_time}


def run_replay(binary: pathlib.Path, assets: pathlib.Path, name: str, timeout: int) -> dict:
    env = dict(os.environ)
    env["SDL_VIDEODRIVER"] = "dummy"
    env["SDL_AUDIODRIVER"] = "dummy"
    env["CNC_GENERALS_ZH_PATH"] = str(assets)
    cmd = [
        str(binary),
        "-headless",
        "-assetPath",
        str(assets),
        "-ignoreReplaySyncErrors",
        "-replay",
        name,
    ]
    try:
        done = subprocess.run(
            cmd,
            env=env,
            cwd=str(binary.parent),
            capture_output=True,
            text=True,
            errors="replace",
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as exc:
        raise ReplayError(f"{name}: no exit within {timeout}s") from exc
    if done.returncode != 0:
        raise ReplayError(f"{name}: engine exited {done.returncode}")
    result = parse_log(done.stdout)
    if not result["crcs"]:
        raise ReplayError(f"{name}: the log has no CRC lines")
    if result["game_time"] is None or result["game_time"][0] != result["game_time"][1]:
        raise ReplayError(f"{name}: playback stopped early at {result['game_time']}")
    return result


def compare(name: str, expected: dict, actual: dict) -> list[str]:
    """Returns the differences between a baseline entry and a run (empty if equal)."""
    exp, act = expected["crcs"], actual["crcs"]
    for index, (want, got) in enumerate(zip(exp, act)):
        if want != got:
            return [
                f"{name}: diverges at entry {index}: baseline frame {want[0]} crc {want[1]}, "
                f"this build frame {got[0]} crc {got[1]}"
            ]
    if len(exp) != len(act):
        return [f"{name}: {len(act)} CRC entries, baseline has {len(exp)}"]
    if expected["game_time"] != actual["game_time"]:
        return [f"{name}: game time {actual['game_time']}, baseline {expected['game_time']}"]
    return []


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def count_rust_symbols(binary: pathlib.Path) -> int:
    done = subprocess.run(["nm", str(binary)], capture_output=True, text=True, errors="replace")
    return sum(1 for line in done.stdout.splitlines() if re.search(r"\b[Tt] _?zh_", line))


def replay_names(replay_dir: pathlib.Path) -> list[str]:
    return sorted(p.name for p in replay_dir.glob("*.rep"))


def write_baseline(args: argparse.Namespace) -> int:
    names = replay_names(args.replay_dir)
    if not names:
        raise ReplayError(f"no replays in {args.replay_dir}")
    replays = {}
    for name in names:
        first = run_replay(args.binary, args.assets, name, args.timeout)
        second = run_replay(args.binary, args.assets, name, args.timeout)
        problems = compare(name, first, second)
        if problems:
            raise ReplayError("two runs of one build differ: " + problems[0])
        replays[name] = {"sha256": sha256(args.replay_dir / name), **first}
        print(f"recorded {name}: {len(first['crcs'])} CRC points")
    args.baseline.parent.mkdir(parents=True, exist_ok=True)
    lines = ["{", f'  "reference_commit": "{REFERENCE_COMMIT}",', '  "replays": {']
    for index, (name, entry) in enumerate(replays.items()):
        comma = "," if index + 1 < len(replays) else ""
        lines.append(f"    {json.dumps(name)}: {json.dumps(entry, separators=(',', ':'))}{comma}")
    lines += ["  }", "}"]
    args.baseline.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {args.baseline}")
    return 0


def check_baseline(args: argparse.Namespace) -> int:
    baseline = json.loads(args.baseline.read_text(encoding="utf-8"))["replays"]
    if not baseline:
        raise ReplayError("the baseline lists no replays")
    if args.expect_rust:
        found = count_rust_symbols(args.binary)
        if found == 0:
            raise ReplayError(f"{args.binary} contains no zh_ symbols, so no Rust code is linked")
        print(f"{found} Rust symbol(s) linked into the binary")
    failures = []
    for name, expected in baseline.items():
        path = args.replay_dir / name
        if not path.is_file():
            failures.append(f"{name}: missing from {args.replay_dir}")
            continue
        if sha256(path) != expected["sha256"]:
            failures.append(f"{name}: replay file differs from the one in the baseline")
            continue
        try:
            actual = run_replay(args.binary, args.assets, name, args.timeout)
        except ReplayError as exc:
            failures.append(str(exc))
            continue
        problems = compare(name, expected, actual)
        failures += problems
        if not problems:
            print(f"ok {name}: {len(actual['crcs'])} CRC points match")
    for failure in failures:
        print(f"FAIL {failure}", file=sys.stderr)
    return 1 if failures else 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0].strip())
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="record the baseline")
    mode.add_argument("--check", action="store_true", help="compare against the baseline")
    parser.add_argument("--expect-rust", action="store_true")
    parser.add_argument("--binary", type=pathlib.Path, default=DEFAULT_BINARY)
    parser.add_argument("--assets", type=pathlib.Path, default=DEFAULT_ASSETS)
    parser.add_argument("--replay-dir", type=pathlib.Path, default=DEFAULT_REPLAYS)
    parser.add_argument("--baseline", type=pathlib.Path, default=DEFAULT_BASELINE)
    parser.add_argument("--timeout", type=int, default=600)
    args = parser.parse_args(argv)
    # The engine runs from its own folder, so paths must not depend on ours.
    args.binary = args.binary.resolve()
    args.assets = args.assets.resolve()
    try:
        if not args.binary.is_file():
            raise ReplayError(f"engine binary not found: {args.binary}")
        return write_baseline(args) if args.write else check_baseline(args)
    except ReplayError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
