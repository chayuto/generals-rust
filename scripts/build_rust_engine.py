#!/usr/bin/env python3
"""
Builds `GeneralsXZH-rust`: the reference engine with Rust code swapped in
(`env:local-full` only).

Nothing in `vendor/GeneralsX` is edited and `libz_gameengine.a` is not rebuilt,
because the golden-vector oracles link against that library and must stay stock.
Instead a copy of each swapped source file is patched under `target/rust_seam/`,
compiled with that file's own flags, and put ahead of the library on the game's
link line, so the linker never pulls the original member.

  python3 scripts/build_rust_engine.py
  python3 scripts/check_replays.py --check --expect-rust --binary <printed path>

Swapped so far: the random number generator (`RandomValue.cpp`, PKT-002) and
experience and veterancy (`ExperienceTracker.cpp`, PKT-003).
`--mutate <swap>` builds a deliberately wrong variant to prove the replay check fails.
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import pathlib
import shlex
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import build_oracle  # noqa: E402

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
BUILD_DIR = REPO_ROOT / "vendor/GeneralsX/build/macos-vulkan"
GAME_TARGET = "GeneralsMD/GeneralsXZH"
VENDOR = REPO_ROOT / "vendor/GeneralsX"


@dataclasses.dataclass(frozen=True)
class Swap:
    """One engine source file whose functions are replaced by calls into Rust."""

    name: str
    source: pathlib.Path
    # (first line replaced, first line kept again, replacement text)
    regions: tuple[tuple[str, str, str], ...]


SWAPS = (
    Swap(
        "random",
        VENDOR / "Core/GameEngine/Source/Common/RandomValue.cpp",
        (
            (
                "static const Real theMultFactor",
                "const char *const GameClientRandomVariable::DistributionTypeNames[] =",
                '#include "zh_ffi_random_value.inc"',
            ),
        ),
    ),
    Swap(
        "experience",
        VENDOR / "GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ExperienceTracker.cpp",
        (
            (
                "Int ExperienceTracker::getExperienceValue(",
                "Bool ExperienceTracker::isTrainable() const",
                '#include "zh_ffi_experience_tracker.inc"',
            ),
            (
                "void ExperienceTracker::setMinVeterancyLevel(",
                "void ExperienceTracker::crc(",
                "// the veterancy functions are in the included file above",
            ),
        ),
    ),
)


class BuildError(Exception):
    pass


def swap_region(source: str, begin: str, end: str, replacement: str) -> str:
    """Replaces everything from the line starting `begin` up to the line starting `end`."""
    lines = source.splitlines(keepends=True)
    starts = [i for i, line in enumerate(lines) if line.startswith(begin)]
    ends = [i for i, line in enumerate(lines) if line.startswith(end)]
    if len(starts) != 1 or len(ends) != 1 or starts[0] >= ends[0]:
        raise BuildError(
            f"expected one '{begin}' before one '{end}', found {len(starts)} and {len(ends)}"
        )
    return "".join(lines[: starts[0]]) + replacement + "\n\n" + "".join(lines[ends[0] :])


def compile_args(
    command: list[str], source: pathlib.Path, output: pathlib.Path, extra: list[str]
) -> list[str]:
    """Turns a `compile_commands.json` entry into one compiling `source` to `output`."""
    args: list[str] = []
    skip = False
    for token in command:
        if skip:
            skip = False
        elif token in ("-o", "-c"):
            skip = True
        else:
            args.append(token)
    return args + extra + ["-o", str(output), "-c", str(source)]


def link_args(command: list[str], objects: list[pathlib.Path], output: str) -> list[str]:
    """Puts `objects` ahead of the first static library and renames the output."""
    args = list(command)
    while args and args[0] in (":", "&&"):
        args.pop(0)
    if "&&" in args:
        args = args[: args.index("&&")]
    if "-o" not in args:
        raise BuildError("the link command has no -o")
    args[args.index("-o") + 1] = output
    first_lib = next((i for i, token in enumerate(args) if token.endswith(".a")), None)
    if first_lib is None:
        raise BuildError("the link command lists no static library")
    if not any(token.endswith("libzh_ffi.a") for token in args):
        raise BuildError("libzh_ffi.a is not on the link line; run scripts/apply_ffi_seam.py")
    return args[:first_lib] + [str(obj) for obj in objects] + args[first_lib:]


def run(args: list[str], cwd: pathlib.Path) -> None:
    done = subprocess.run(args, cwd=str(cwd), capture_output=True, text=True, errors="replace")
    if done.returncode != 0:
        raise BuildError(f"{args[0]} failed:\n{done.stderr[-4000:]}")


def build(mutate: str | None) -> pathlib.Path:
    """Builds the swapped engine; `mutate` names one swap to compile with a fault."""
    if not (BUILD_DIR / "compile_commands.json").is_file():
        raise BuildError(f"no reference build tree at {BUILD_DIR}")
    run(["cargo", "build", "--release", "-p", "zh_ffi"], REPO_ROOT)

    suffix = f"-mutant-{mutate}" if mutate else ""
    work = REPO_ROOT / "target" / f"rust_seam{suffix}"
    work.mkdir(parents=True, exist_ok=True)

    entries = json.loads((BUILD_DIR / "compile_commands.json").read_text())
    raw_link = subprocess.run(
        ["ninja", "-t", "commands", GAME_TARGET],
        cwd=str(BUILD_DIR),
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip().splitlines()[-1]
    link_tokens = shlex.split(raw_link)

    objects = []
    stale: list[str] = []
    for swap in SWAPS:
        text = swap.source.read_text(encoding="utf-8", errors="surrogateescape")
        for begin, end, replacement in swap.regions:
            text = swap_region(text, begin, end, replacement)
        patched = work / swap.source.name
        patched.write_text(text, encoding="utf-8", errors="surrogateescape")

        tail = "/" + "/".join(swap.source.parts[-3:])
        matches = [e for e in entries if e["file"].endswith(tail) and "z_gameengine" in e["command"]]
        if len(matches) != 1:
            raise BuildError(f"{tail}: {len(matches)} compile commands, expected one")
        raw_compile = shlex.split(matches[0]["command"])
        # The tree may have been configured under another folder name.
        stale = build_oracle.detect_stale_prefixes(raw_compile + link_tokens)
        command = [build_oracle.rewrite_repo_path(t, stale, REPO_ROOT) for t in raw_compile]

        extra = [f"-I{REPO_ROOT / 'crates/zh_ffi/include'}"]
        if mutate == swap.name:
            extra.append("-DZH_SEAM_MUTATE")
        obj = work / f"{swap.source.name}.o"
        run(compile_args(command, patched, obj, extra), BUILD_DIR)
        objects.append(obj)

    link = [build_oracle.rewrite_repo_path(t, stale, REPO_ROOT) for t in link_tokens]
    output = f"{GAME_TARGET}-rust{suffix}"
    run(link_args(link, objects, output), BUILD_DIR)
    return BUILD_DIR / output


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build the engine with Rust code swapped in.")
    parser.add_argument(
        "--mutate",
        choices=[swap.name for swap in SWAPS],
        help="build a variant with a deliberate fault in this swap",
    )
    args = parser.parse_args(argv)
    try:
        binary = build(args.mutate)
    except (BuildError, subprocess.CalledProcessError, OSError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    print(binary)
    return 0


if __name__ == "__main__":
    sys.exit(main())
