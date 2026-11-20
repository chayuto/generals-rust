#!/usr/bin/env python3
"""
Gate V1: Static Determinism Linter for generals-rust.

Scans Rust source files (.rs) in crates/zh_common, crates/zh_logic, crates/zh_ffi,
and crates/zh_sim for constructs that break cross-platform replay determinism:
  1. Randomized hash containers (std::collections::HashMap, HashSet, RandomState)
  2. Standard library BinaryHeap (violates GameLogic::m_sleepyUpdates tie-breaking)
  3. Non-deterministic libm float transcendentals (.sin(), .cos(), .tan(), .asin(),
     .acos(), .atan(), .atan2(), .sqrt(), .floor(), .ceil(), .powf(), .powi(),
     .exp(), .exp2(), .ln(), .log2(), .log10(), .cbrt(), .hypot())
  4. Fused multiply-add (.mul_add() / fmaf) which violates -ffp-contract=off
  5. Architecture-dependent pointer-width types (usize/isize) passed to Xfer
  6. BTreeMap / BTreeSet keyed by raw pointer (*const T / *mut T)
  7. Prohibited attribution trailers or session links

Lines explicitly annotated with `// determinism-ok: <reason>` are exempted.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys
from dataclasses import dataclass
from typing import Iterable, List, Pattern, Tuple


@dataclass(frozen=True)
class LintRule:
    code: str
    description: str
    pattern: Pattern[str]


RULES: Tuple[LintRule, ...] = (
    LintRule(
        code="DET001",
        description="Randomized HashMap/HashSet is forbidden in simulation logic; use BTreeMap/BTreeSet or slot arenas.",
        pattern=re.compile(r"\b(HashMap|HashSet|RandomState)\b"),
    ),
    LintRule(
        code="DET002",
        description="std::collections::BinaryHeap is forbidden; use SleepyUpdateHeap with strict '>' left-child tie-breaking.",
        pattern=re.compile(r"\bBinaryHeap\b"),
    ),
    LintRule(
        code="DET003",
        description="Non-deterministic std float transcendental or rounding method; use zh_common::gamemath::gm_*f instead.",
        pattern=re.compile(
            r"(?:\bf32::|\bf64::|\.)"
            r"(sin|cos|tan|asin|acos|atan|atan2|sinh|cosh|tanh|asinh|acosh|atanh|"
            r"sqrt|cbrt|hypot|floor|ceil|round|trunc|powf|powi|exp|exp2|ln|log|log2|log10|sin_cos)\s*\("
        ),
    ),
    LintRule(
        code="DET004",
        description="Fused multiply-add (.mul_add) violates -ffp-contract=off parity; use unfused `a * b + c`.",
        pattern=re.compile(r"(?:\bf32::|\bf64::|\.)mul_add\s*\("),
    ),
    LintRule(
        code="DET005",
        description="Pointer-keyed BTreeMap/BTreeSet is non-deterministic across ASLR runs; key by ObjectId/NameKeyType/Slot.",
        pattern=re.compile(r"\bBTree(?:Map|Set)\s*<\s*\*(?:const|mut)\b"),
    ),
    LintRule(
        code="DET006",
        description="Architecture-sized usize/isize serialized in Xfer/CRC; cast to fixed-width u32/i32/u16/u8.",
        pattern=re.compile(r"\bxfer[A-Za-z0-9_]*\s*\([^)]*\b(?:usize|isize)\b"),
    ),
    LintRule(
        code="DET007",
        description="Prohibited attribution trailer or session link.",
        pattern=re.compile(
            r"(?:Co-Authored-" + r"By:|Generated[- ]" + r"by:|Generated[- ]" + r"with:|conversation" + r"://)",
            re.IGNORECASE,
        ),
    ),
)

EXEMPT_MARKER = "determinism-ok:"


@dataclass(frozen=True)
class Violation:
    path: pathlib.Path
    line_no: int
    code: str
    description: str
    line_text: str


def strip_line_comment(line: str) -> Tuple[str, str]:
    """Splits a Rust source line into (code_part, comment_part)."""
    idx = line.find("//")
    if idx == -1:
        return line, ""
    return line[:idx], line[idx:]


def lint_source_text(text: str, path: pathlib.Path) -> List[Violation]:
    violations: List[Violation] = []
    for line_no, raw_line in enumerate(text.splitlines(), start=1):
        code_part, comment_part = strip_line_comment(raw_line)
        if EXEMPT_MARKER in comment_part:
            continue

        for rule in RULES:
            target = raw_line if rule.code == "DET007" else code_part
            if rule.pattern.search(target):
                violations.append(
                    Violation(
                        path=path,
                        line_no=line_no,
                        code=rule.code,
                        description=rule.description,
                        line_text=raw_line.strip(),
                    )
                )
    return violations


def collect_rust_files(paths: Iterable[pathlib.Path]) -> List[pathlib.Path]:
    files: List[pathlib.Path] = []
    for p in paths:
        if p.is_file() and p.suffix == ".rs":
            files.append(p)
        elif p.is_dir():
            files.extend(sorted(p.rglob("*.rs")))
    return files


def main(argv: List[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Gate V1: Static Determinism Linter for generals-rust."
    )
    parser.add_argument(
        "paths",
        nargs="*",
        default=["crates"],
        help="Files or directories to scan (defaults to 'crates').",
    )
    args = parser.parse_args(argv)

    root = pathlib.Path.cwd()
    target_paths = [root / p for p in args.paths if (root / p).exists()]
    rs_files = collect_rust_files(target_paths)

    all_violations: List[Violation] = []
    for rs_file in rs_files:
        text = rs_file.read_text(encoding="utf-8")
        all_violations.extend(lint_source_text(text, rs_file))

    if all_violations:
        for v in all_violations:
            print(
                f"{v.path}:{v.line_no}: [{v.code}] {v.description}\n  > {v.line_text}",
                file=sys.stderr,
            )
        print(
            f"\nGate V1 FAILED: {len(all_violations)} determinism violation(s) found across {len(rs_files)} file(s).",
            file=sys.stderr,
        )
        return 1

    print(f"Gate V1 PASSED: {len(rs_files)} Rust file(s) scanned, 0 violations.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
