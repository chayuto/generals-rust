#!/usr/bin/env python3
"""Zero-stub C++ Golden Vector oracle compiler (`scripts/build_oracle.py`).

Compiles C++ oracle drivers directly against the pre-built `vendor/GeneralsX`
CMake/Ninja tree (`compile_commands.json` + `build.ninja`) and `libz_gameengine.a`
so golden-vector harnesses require zero per-packet `stubs_pktXXX/` header trees.

Usage:
  python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt406_radar.cpp -o /tmp/gen_pkt406
  python3 scripts/build_oracle.py driver.cpp -o /tmp/gen --opt O2 --sanitize
  python3 scripts/build_oracle.py driver.cpp -o /tmp/gen --dry-run --print-cmd
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

DEFAULT_REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_BUILD_REL = Path("vendor/GeneralsX/build/macos-vulkan")

# Flags in compile_commands.json stripped when compiling standalone oracle drivers.
DROP_SINGLE_FLAGS = {
    "-c",
    "-fPIC",
    "-fPIE",
    "-Winvalid-pch",
    "-Wsuggest-override",
    "-DNDEBUG",
}

# Windows-only import libraries that can leak into CMake test targets without `if(WIN32)`.
WINDOWS_ONLY_LIBS = {
    "-lcomctl32",
    "-limm32",
    "-lvfw32",
    "-lwinmm",
    "-ld3d8",
    "-ldinput8",
    "-ldxguid",
}


def detect_stale_prefixes(tokens: Sequence[str]) -> List[str]:
    """Detects old absolute checkout prefixes ending before `vendor/GeneralsX` or `crates/`."""
    prefixes: List[str] = []
    markers = ("/vendor/GeneralsX", "/crates/", "/target/")
    for tok in tokens:
        for marker in markers:
            idx = tok.find(marker)
            if idx > 0:
                prefix = tok[:idx]
                if prefix.endswith(",-rpath"):
                    continue
                if prefix.startswith("-Wl,-rpath,"):
                    prefix = prefix[len("-Wl,-rpath,"):]
                elif prefix.startswith("-I"):
                    prefix = prefix[2:]
                if prefix.startswith("/") and prefix not in prefixes:
                    prefixes.append(prefix)
    # Longest prefixes first so more specific matches win cleanly.
    prefixes.sort(key=len, reverse=True)
    return prefixes


def rewrite_repo_path(token: str, stale_prefixes: Sequence[str], repo_root: Path) -> str:
    """Rewrites stale absolute repo-root prefixes in a compiler/linker argument."""
    root_str = repo_root.as_posix()
    out = token
    for old in stale_prefixes:
        if old != root_str and old in out:
            out = out.replace(old, root_str)
    return out


def find_gameengine_compile_entry(compile_commands: List[Dict[str, str]]) -> Dict[str, str]:
    """Selects a canonical `z_gameengine` translation unit entry from compile_commands.json."""
    preferred_suffixes = (
        "Source/Common/GlobalData.cpp",
        "Source/Common/GameEngine.cpp",
        "Source/Common/System/Radar.cpp",
    )
    for suffix in preferred_suffixes:
        for entry in compile_commands:
            f = entry.get("file", "")
            cmd = entry.get("command", "")
            if f.endswith(suffix) and ("z_gameengine" in cmd or "GameEngine/Include" in cmd):
                return entry
    for entry in compile_commands:
        cmd = entry.get("command", "")
        if "z_gameengine" in cmd or "GameEngine/Include" in cmd:
            return entry
    raise RuntimeError("No z_gameengine entry found in compile_commands.json")


def parse_compile_flags(compile_commands_path: Path, repo_root: Path) -> Tuple[str, List[str]]:
    """Extracts compiler executable and sanitized compile flags from compile_commands.json."""
    entries = json.loads(compile_commands_path.read_text(encoding="utf-8"))
    entry = find_gameengine_compile_entry(entries)
    tokens = shlex.split(entry["command"])
    if not tokens:
        raise RuntimeError(f"Empty compiler command in {compile_commands_path}")

    compiler = tokens[0]
    if os.path.basename(compiler) == "ccache" and len(tokens) > 1:
        compiler = tokens[1]
        raw_args = tokens[2:]
    else:
        raw_args = tokens[1:]

    stale = detect_stale_prefixes(raw_args)
    flags: List[str] = []
    i = 0
    while i < len(raw_args):
        tok = rewrite_repo_path(raw_args[i], stale, repo_root)
        # Drop output object / input source pairs
        if tok in ("-o", "-c"):
            if tok == "-o" and i + 1 < len(raw_args):
                i += 2
            else:
                i += 1
            continue
        # Drop CMake PCH directives so arbitrary driver .cpp files compile cleanly
        if tok in ("-include-pch", "-Xclang"):
            if i + 1 < len(raw_args):
                i += 2
            else:
                i += 1
            continue
        if tok == "-include":
            if i + 1 < len(raw_args):
                nxt = rewrite_repo_path(raw_args[i + 1], stale, repo_root)
                if "cmake_pch" in nxt:
                    i += 2
                    continue
                flags.extend(["-include", nxt])
                i += 2
                continue
        if tok.endswith(".cpp") or tok.endswith(".cxx") or tok.endswith(".o"):
            i += 1
            continue
        if tok in DROP_SINGLE_FLAGS or tok.startswith("-O") or tok == "-g":
            i += 1
            continue
        if tok == "-isystem":
            if i + 1 < len(raw_args):
                nxt = rewrite_repo_path(raw_args[i + 1], stale, repo_root)
                if "googletest" not in nxt:
                    flags.extend(["-isystem", nxt])
                i += 2
                continue
        if "googletest" in tok and tok.startswith("-I"):
            i += 1
            continue
        flags.append(tok)
        i += 1

    return compiler, flags


def extract_ninja_link_block(build_ninja_path: Path) -> Tuple[List[str], List[str], str]:
    """Extracts (link_flags, link_libraries, target_name) from build.ninja."""
    text = build_ninja_path.read_text(encoding="utf-8")
    target_patterns = (
        ("GeneralsMD/GeneralsXZH", r"(?ms)^build GeneralsMD/GeneralsXZH: CXX_EXECUTABLE_LINKER[^\n]*(?:\n  [^\n]*)*"),
        ("Tests/Google/z_googletest", r"(?ms)^build Tests/Google/z_googletest: CXX_EXECUTABLE_LINKER[^\n]*(?:\n  [^\n]*)*"),
    )
    block = None
    chosen = ""
    for name, pat in target_patterns:
        m = re.search(pat, text)
        if m:
            block = m.group(0)
            chosen = name
            break
    if block is None:
        raise RuntimeError(f"Could not find GeneralsXZH or z_googletest link rule in {build_ninja_path}")

    link_flags: List[str] = []
    link_libs: List[str] = []
    for line in block.splitlines():
        stripped = line.strip()
        if stripped.startswith("LINK_FLAGS = "):
            link_flags.extend(shlex.split(stripped[len("LINK_FLAGS = "):]))
        elif stripped.startswith("LINK_LIBRARIES = "):
            link_libs.extend(shlex.split(stripped[len("LINK_LIBRARIES = "):]))
    return link_flags, link_libs, chosen


def parse_link_args(build_dir: Path, repo_root: Path) -> List[str]:
    """Resolves linker flags, archives, dylibs, and rpaths relative to build_dir."""
    build_ninja = build_dir / "build.ninja"
    link_flags, raw_libs, _ = extract_ninja_link_block(build_ninja)
    stale = detect_stale_prefixes(link_flags + raw_libs)

    resolved: List[str] = []
    rpaths: List[str] = []

    def add_rpath(rdir: str) -> None:
        flag = f"-Wl,-rpath,{rdir}"
        if flag not in rpaths:
            rpaths.append(flag)

    for lf in link_flags:
        resolved.append(rewrite_repo_path(lf, stale, repo_root))

    for tok in raw_libs:
        rewritten = rewrite_repo_path(tok, stale, repo_root)
        if rewritten in WINDOWS_ONLY_LIBS:
            continue
        if "libgtest" in rewritten or "libgmock" in rewritten:
            continue
        if rewritten.endswith(".sym"):
            continue
        if rewritten.startswith("-Wl,-rpath,"):
            add_rpath(rewritten[len("-Wl,-rpath,"):])
            continue
        if rewritten.startswith("-"):
            resolved.append(rewritten)
            continue

        p = Path(rewritten)
        if not p.is_absolute():
            p = build_dir / p
        p_str = p.as_posix()
        resolved.append(p_str)
        if p_str.endswith(".dylib") or ".dylib." in p.name:
            add_rpath(p.parent.as_posix())

    return rpaths + resolved


# `// oracle-wrap: <archive under the build dir> <member> <mangled symbol>` in a driver
# asks to answer that library function in the driver, to record the calls other library
# code makes to it. The reference function stays reachable as `_oracle_real<symbol>`.
WRAP_RE = re.compile(r"^//[ \t]*oracle-wrap:[ \t]*(\S+)[ \t]+(\S+)[ \t]+(\S+)[ \t]*$", re.M)
WRAP_ALIAS_PREFIX = "_oracle_real"


def parse_wrap_directives(sources: Sequence[Path]) -> Dict[Tuple[str, str], List[str]]:
    """Maps (archive, member) to the symbols the drivers want to answer themselves."""
    wanted: Dict[Tuple[str, str], List[str]] = {}
    for source in sources:
        if not Path(source).is_file():
            continue
        text = Path(source).read_text(encoding="utf-8", errors="replace")
        for archive, member, symbol in WRAP_RE.findall(text):
            symbols = wanted.setdefault((archive, member), [])
            if symbol not in symbols:
                symbols.append(symbol)
    return wanted


def wrap_commands(
    wanted: Dict[Tuple[str, str], List[str]], build_dir: Path, work_dir: Path, arch: str
) -> List[Tuple[List[str], List[str], Path]]:
    """(extract, relink, object) per wrapped member. Both commands run inside `work_dir`.

    The copy of the member keeps its own calls to a wrapped function; only the symbol
    other objects bind to is hidden, so that the driver's definition takes its place.
    The archive itself is only read.
    """
    steps = []
    for (archive, member), symbols in sorted(wanted.items()):
        if "/" in member:
            raise ValueError(f"oracle-wrap member must be a plain name: {member}")
        wrapped = work_dir.resolve() / f"{member}.wrapped.o"
        extract = ["ar", "x", str((build_dir / archive).resolve()), member]
        relink = ["ld", "-r", "-arch", arch, member, "-o", str(wrapped)]
        for symbol in symbols:
            relink += ["-alias", symbol, WRAP_ALIAS_PREFIX + symbol, "-unexported_symbol", symbol]
        steps.append((extract, relink, wrapped))
    return steps


def build_command(
    sources: Sequence[Path],
    output: Path,
    repo_root: Path = DEFAULT_REPO_ROOT,
    build_dir: Optional[Path] = None,
    opt: str = "O1",
    sanitize: bool = False,
    asan: bool = False,
    extra_flags: Sequence[str] = (),
) -> List[str]:
    """Constructs the complete compiler + linker invocation for a zero-stub oracle binary."""
    if not sources:
        raise ValueError("At least one C++ source file is required")

    resolved_build_dir = build_dir if build_dir is not None else (repo_root / DEFAULT_BUILD_REL)
    compile_commands = resolved_build_dir / "compile_commands.json"
    if not compile_commands.is_file():
        raise FileNotFoundError(f"Missing compile_commands.json at {compile_commands}")
    if not (resolved_build_dir / "build.ninja").is_file():
        raise FileNotFoundError(f"Missing build.ninja at {resolved_build_dir / 'build.ninja'}")

    compiler, compile_flags = parse_compile_flags(compile_commands, repo_root)
    link_args = parse_link_args(resolved_build_dir, repo_root)

    # Ensure deterministic FP contraction stays disabled regardless of compile_commands.json.
    if "-ffp-contract=off" not in compile_flags:
        compile_flags.append("-ffp-contract=off")

    # Quiet legacy WWLib / DXVK header warnings that do not affect oracle correctness.
    for wflag in (
        "-Wno-macro-redefined",
        "-Wno-implicit-exception-spec-mismatch",
        "-Wno-new-returns-null",
    ):
        if wflag not in compile_flags:
            compile_flags.append(wflag)

    cmd: List[str] = [compiler, f"-{opt}"]
    if sanitize or asan:
        san = "address,undefined" if asan else "undefined"
        cmd.extend(["-g", "-fno-omit-frame-pointer", f"-fsanitize={san}"])

    cmd.extend(compile_flags)
    cmd.extend(extra_flags)
    cmd.extend(str(s) for s in sources)
    cmd.extend(["-o", str(output)])
    cmd.extend(link_args)
    if sys.platform == "darwin" and "-Wl,-w" not in cmd:
        cmd.append("-Wl,-w")
    return cmd


def ensure_libzh_ffi(repo_root: Path) -> None:
    """Builds `target/release/libzh_ffi.a` if not yet present on disk."""
    ffi_lib = repo_root / "target/release/libzh_ffi.a"
    if ffi_lib.is_file():
        return
    subprocess.check_call(
        ["cargo", "build", "--release", "-p", "zh_ffi"],
        cwd=str(repo_root),
    )


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Compile a C++ golden-vector oracle directly against vendor/GeneralsX libraries."
    )
    parser.add_argument("sources", nargs="+", type=Path, help="C++ driver source file(s).")
    parser.add_argument("-o", "--output", required=True, type=Path, help="Output executable path.")
    parser.add_argument(
        "--opt",
        choices=("O0", "O1", "O2"),
        default="O1",
        help="Optimization level (default: O1, always with -ffp-contract=off).",
    )
    parser.add_argument(
        "--sanitize",
        action="store_true",
        help="Enable UBSan checks (-fsanitize=undefined).",
    )
    parser.add_argument(
        "--asan",
        action="store_true",
        help="Also enable ASan (only for binaries without libz_gameengine global operator new + libprotobuf mismatch).",
    )
    parser.add_argument(
        "--extra-flag",
        action="append",
        default=[],
        help="Additional compiler/linker flag (repeatable).",
    )
    parser.add_argument(
        "--build-dir",
        type=Path,
        default=None,
        help="Override vendor build directory (default: vendor/GeneralsX/build/macos-vulkan).",
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=DEFAULT_REPO_ROOT,
        help="Override repository root path.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print command without executing compiler.",
    )
    parser.add_argument(
        "--print-cmd",
        action="store_true",
        help="Print resolved command before running.",
    )

    args = parser.parse_args(argv)
    repo_root = args.repo_root.resolve()
    work_dir = Path(f"{args.output.resolve()}.wrap")
    steps = wrap_commands(
        parse_wrap_directives(args.sources),
        args.build_dir if args.build_dir is not None else repo_root / DEFAULT_BUILD_REL,
        work_dir,
        platform.machine(),
    )
    cmd = build_command(
        sources=list(args.sources) + [wrapped for _, _, wrapped in steps],
        output=args.output,
        repo_root=args.repo_root.resolve(),
        build_dir=args.build_dir,
        opt=args.opt,
        sanitize=args.sanitize,
        asan=args.asan,
        extra_flags=args.extra_flag,
    )

    if args.dry_run or args.print_cmd:
        print(shlex.join(cmd))
    if args.dry_run:
        return 0

    ensure_libzh_ffi(args.repo_root.resolve())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    for extract, relink, _ in steps:
        work_dir.mkdir(parents=True, exist_ok=True)
        subprocess.check_call(extract, cwd=str(work_dir))
        subprocess.check_call(relink, cwd=str(work_dir))
    subprocess.check_call(cmd, cwd=str(args.repo_root.resolve()))
    return 0


if __name__ == "__main__":
    sys.exit(main())
