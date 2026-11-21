#!/usr/bin/env python3
"""Deterministic port coverage & oracle progress map generator (`docs/PORT_MAP.md`).

Usage:
  python3 scripts/port_map.py --write
  python3 scripts/port_map.py --check
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List

DEFAULT_REPO_ROOT = Path(__file__).resolve().parent.parent
VECTOR_COUNT_RE = re.compile(r'"vector_count"\s*:\s*(\d+)')
IGNORED_TEST_RE = re.compile(r"(?m)^\s*#\[\s*ignore\b")


def classify_rust_target(path: Path) -> str:
    """Classifies a manifest rust_target as implemented, seeded, or unported."""
    if not path.is_file():
        return "unported"
    text = path.read_text(encoding="utf-8")
    if "todo!(" in text:
        return "seeded"
    return "implemented"


def find_packet_doc(repo_root: Path, pkt: Dict[str, Any]) -> str:
    candidate = repo_root / f"packets/phase{pkt['phase']}/{pkt['packet_id']}_{pkt['module_name']}.md"
    if candidate.is_file():
        return candidate.relative_to(repo_root).as_posix()
    return "-"


def inspect_golden_oracles(repo_root: Path) -> List[Dict[str, str]]:
    gv_dir = repo_root / "docs/golden_vectors"
    rows: List[Dict[str, str]] = []
    if not gv_dir.is_dir():
        return rows
    for p in sorted(gv_dir.glob("*.json"), key=lambda x: x.name):
        with p.open("r", encoding="utf-8", errors="replace") as f:
            header = f.read(4096)
            m = VECTOR_COUNT_RE.search(header)
            has_sha = '"source_sha256"' in header
            if m is None or not has_sha:
                full_text = header + f.read()
                if m is None:
                    m = VECTOR_COUNT_RE.search(full_text)
                if not has_sha:
                    has_sha = '"source_sha256"' in full_text
        vec_count = m.group(1) if m else "-"
        rows.append(
            {
                "path": p.relative_to(repo_root).as_posix(),
                "vector_count": vec_count,
                "source_sha256": "yes" if has_sha else "no",
            }
        )
    return rows


def inspect_golden_tests(repo_root: Path) -> List[Dict[str, str]]:
    rows: List[Dict[str, str]] = []
    crates_dir = repo_root / "crates"
    if not crates_dir.is_dir():
        return rows
    for crate_dir in sorted(crates_dir.iterdir(), key=lambda x: x.name):
        tests_dir = crate_dir / "tests"
        if not tests_dir.is_dir():
            continue
        for t in sorted(tests_dir.glob("golden_*.rs"), key=lambda x: x.name):
            text = t.read_text(encoding="utf-8")
            status = "seeded (ignored)" if IGNORED_TEST_RE.search(text) else "active"
            rows.append(
                {
                    "crate": crate_dir.name,
                    "path": t.relative_to(repo_root).as_posix(),
                    "status": status,
                }
            )
    return rows


def inspect_non_manifest_sources(
    repo_root: Path, manifest_targets: set[str]
) -> List[Dict[str, str]]:
    rows: List[Dict[str, str]] = []
    crates_dir = repo_root / "crates"
    if not crates_dir.is_dir():
        return rows
    for crate_dir in sorted(crates_dir.iterdir(), key=lambda x: x.name):
        src_dir = crate_dir / "src"
        if not src_dir.is_dir():
            continue
        for rs in sorted(src_dir.rglob("*.rs"), key=lambda x: x.as_posix()):
            if rs.name in ("lib.rs", "mod.rs", "main.rs"):
                continue
            rel = rs.relative_to(repo_root).as_posix()
            if rel in manifest_targets:
                continue
            rows.append(
                {
                    "crate": crate_dir.name,
                    "path": rel,
                    "status": classify_rust_target(rs),
                }
            )
    return rows


def render_port_map(repo_root: Path) -> str:
    manifest_path = repo_root / "docs/MODULE_MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    packets: List[Dict[str, Any]] = manifest["packets"]

    phase_stats: Dict[int, Dict[str, int]] = {}
    tracked_packets: List[Dict[str, str]] = []
    manifest_targets: set[str] = set()

    for pkt in sorted(packets, key=lambda x: (int(x["phase"]), str(x["packet_id"]))):
        phase = int(pkt["phase"])
        target_rel = Path(str(pkt["rust_target"])).as_posix()
        manifest_targets.add(target_rel)
        status = classify_rust_target(repo_root / target_rel)

        st = phase_stats.setdefault(
            phase, {"implemented": 0, "seeded": 0, "unported": 0, "total": 0}
        )
        st[status] += 1
        st["total"] += 1

        if status != "unported":
            tracked_packets.append(
                {
                    "packet_id": str(pkt["packet_id"]),
                    "module_name": str(pkt["module_name"]),
                    "phase": str(phase),
                    "work_package": str(pkt.get("work_package", "-")),
                    "status": status,
                    "rust_target": target_rel,
                    "packet_doc": find_packet_doc(repo_root, pkt),
                }
            )

    tot_impl = sum(v["implemented"] for v in phase_stats.values())
    tot_seed = sum(v["seeded"] for v in phase_stats.values())
    tot_unp = sum(v["unported"] for v in phase_stats.values())
    tot_all = sum(v["total"] for v in phase_stats.values())

    oracles = inspect_golden_oracles(repo_root)
    suites = inspect_golden_tests(repo_root)
    extras = inspect_non_manifest_sources(repo_root, manifest_targets)

    lines: List[str] = [
        "# Port Coverage & Oracle Progress Map (`generals-rust`)",
        "",
        "<!-- Auto-generated by `python3 scripts/port_map.py --write`. Do not edit by hand. -->",
        "",
        f"- **Reference Repo:** `{manifest['reference_repo']}` @ `{manifest['reference_commit']}`",
        f"- **Manifest Packets:** `{tot_all}` total (`{tot_impl}` implemented, `{tot_seed}` seeded skeletons, `{tot_unp}` unported)",
        f"- **Golden Vector Oracles:** `{len(oracles)}` JSON suites (`docs/golden_vectors/`)",
        f"- **Golden Test Suites:** `{len(suites)}` Rust suites (`crates/*/tests/golden_*.rs`)",
        "",
        "## Phase Summary",
        "",
        "| Phase | Implemented | Seeded | Unported | Total |",
        "|---:|---:|---:|---:|---:|",
    ]
    for phase in sorted(phase_stats):
        st = phase_stats[phase]
        lines.append(
            f"| Phase {phase} | {st['implemented']} | {st['seeded']} | {st['unported']} | {st['total']} |"
        )
    lines.append(
        f"| **Total** | **{tot_impl}** | **{tot_seed}** | **{tot_unp}** | **{tot_all}** |"
    )

    lines.extend(
        [
            "",
            "## Implemented & Seeded Manifest Packets",
            "",
            "| Packet | Module | Phase | WP | Status | Rust Target | Packet Doc |",
            "|---|---|---:|---|---|---|---|",
        ]
    )
    if tracked_packets:
        for row in tracked_packets:
            lines.append(
                f"| `{row['packet_id']}` | `{row['module_name']}` | {row['phase']} | "
                f"`{row['work_package']}` | `{row['status']}` | `{row['rust_target']}` | "
                f"`{row['packet_doc']}` |"
            )
    else:
        lines.append("| - | - | - | - | - | - | - |")

    lines.extend(
        [
            "",
            "## Supporting & Standalone Rust Modules (Non-1:1 Manifest Targets)",
            "",
            "| Crate | Source File | Status |",
            "|---|---|---|",
        ]
    )
    if extras:
        for row in extras:
            lines.append(
                f"| `{row['crate']}` | `{row['path']}` | `{row['status']}` |"
            )
    else:
        lines.append("| - | - | - |")

    lines.extend(
        [
            "",
            "## Golden Vector Oracles",
            "",
            "| Oracle JSON | Declared Vectors | `source_sha256` |",
            "|---|---:|---|",
        ]
    )
    if oracles:
        for row in oracles:
            lines.append(
                f"| `{row['path']}` | {row['vector_count']} | {row['source_sha256']} |"
            )
    else:
        lines.append("| - | - | - |")

    lines.extend(
        [
            "",
            "## Golden Verification Test Suites",
            "",
            "| Crate | Test Suite | Status |",
            "|---|---|---|",
        ]
    )
    if suites:
        for row in suites:
            lines.append(
                f"| `{row['crate']}` | `{row['path']}` | `{row['status']}` |"
            )
    else:
        lines.append("| - | - | - |")

    lines.append("")
    return "\n".join(lines)


def main(argv: List[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Generate or check docs/PORT_MAP.md from repository state."
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=DEFAULT_REPO_ROOT,
        help="Repository root directory (defaults to repo root).",
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="Write docs/PORT_MAP.md")
    mode.add_argument(
        "--check",
        action="store_true",
        help="Exit non-zero if docs/PORT_MAP.md is missing or out of date",
    )
    args = parser.parse_args(argv)

    repo_root = args.root.resolve()
    out_path = repo_root / "docs/PORT_MAP.md"
    rendered = render_port_map(repo_root)

    if args.write:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(rendered, encoding="utf-8", newline="\n")
        print(f"Wrote {out_path.relative_to(repo_root).as_posix()} ({len(rendered):,} bytes)")
        return 0

    if not out_path.is_file():
        print(
            "ERROR: docs/PORT_MAP.md does not exist; run `python3 scripts/port_map.py --write`.",
            file=sys.stderr,
        )
        return 1

    existing = out_path.read_text(encoding="utf-8").replace("\r\n", "\n")
    if existing != rendered:
        print(
            "ERROR: docs/PORT_MAP.md is out of date; run `python3 scripts/port_map.py --write`.",
            file=sys.stderr,
        )
        return 1

    print("OK: docs/PORT_MAP.md is up to date.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
