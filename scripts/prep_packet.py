#!/usr/bin/env python3
"""
Tier 0 Deterministic Context Slicer (scripts/prep_packet.py).

Generates self-contained Translation Packets (`packets/phase<N>/<PKT-ID>_<Module>.md`)
so that `env:ephemeral-web` and `Tier 1A/1B/2` worker agents have 100% of the
required C++ source, extracted INI/Xfer/RNG/Facet invariants, and Rust conventions
without needing local access to `vendor/GeneralsX` or `~/Games/generalszh`.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
from typing import Any, Dict, List

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
MANIFEST_PATH = REPO_ROOT / "docs/MODULE_MANIFEST.json"
CONVENTIONS_PATH = REPO_ROOT / "docs/RUST_CONVENTIONS.md"
VENDOR_GX = REPO_ROOT / "vendor/GeneralsX"


def extract_cheatsheet(conventions_text: str) -> str:
    m = re.search(
        r"<!-- BEGIN WORKER CHEATSHEET -->(.*?)<!-- END WORKER CHEATSHEET -->",
        conventions_text,
        flags=re.DOTALL,
    )
    return m.group(1).strip() if m else conventions_text


def strip_cpp_boilerplate(src: str) -> str:
    """Strips the standard 25-line GPL comment header block to save tokens."""
    src = re.sub(
        r"\A/\*.*?\*/\s*(?://+\s*\n(?:.*?\n)*?//+\s*\n)?",
        "",
        src,
        count=1,
        flags=re.DOTALL,
    )
    return src.strip()


def analyze_cpp_invariants(sources: Dict[str, str]) -> Dict[str, List[str]]:
    combined = "\n".join(sources.values())

    includes = sorted(set(re.findall(r'#include\s+"([^"]+)"', combined)))
    xfer_calls = re.findall(r"xfer->(xfer[A-Za-z0-9_]+\s*\([^;]+\))", combined)
    rng_calls = re.findall(
        r"((?:Get)?Game(?:Logic|Client|Audio)RandomValue[A-Za-z0-9_]*\s*\([^;]+\))",
        combined,
    )
    retail_guards = sorted(
        set(re.findall(r"#\s*if(?:n?def)?\s+([^\n]*RETAIL_COMPATIBLE[^\n]*)", combined))
    )
    facets = sorted(
        set(
            re.findall(
                r"virtual\s+([A-Za-z0-9_]+Interface)\s*\*\s*(get[A-Za-z0-9_]+)\s*\(\s*\)",
                combined,
            )
        )
    )
    ini_fields = re.findall(
        r'\{\s*"([^"]+)"\s*,\s*([A-Za-z0-9_:]+)\s*,', combined
    )

    return {
        "includes": includes,
        "xfer_calls": [re.sub(r"\s+", " ", c) for c in xfer_calls],
        "rng_calls": [re.sub(r"\s+", " ", c) for c in rng_calls],
        "retail_guards": [g.strip() for g in retail_guards],
        "facets": [f"{ret}* {fn}()" for ret, fn in facets],
        "ini_fields": [f'"{name}" => {parser}' for name, parser in ini_fields],
    }


def fetch_cpp_file(rel_path: str, commit: str) -> str:
    local_candidate = VENDOR_GX / rel_path
    if local_candidate.is_file():
        return local_candidate.read_text(encoding="utf-8", errors="replace")

    out = subprocess.check_output(
        [
            "gh",
            "api",
            f"repos/fbraz3/GeneralsX/contents/{rel_path}?ref={commit}",
            "-H",
            "Accept: application/vnd.github.raw",
        ]
    )
    return out.decode("utf-8", errors="replace")


def render_packet_markdown(
    pkt: Dict[str, Any],
    sources: Dict[str, str],
    cheatsheet: str,
) -> str:
    inv = analyze_cpp_invariants(sources)
    stripped_sources = {k: strip_cpp_boilerplate(v) for k, v in sources.items()}

    lines: List[str] = [
        f"# Translation Packet `{pkt['packet_id']}`: `{pkt['module_name']}`",
        "",
        "## 1. Compartmentalization & Routing Metadata",
        f"- **Packet ID:** `{pkt['packet_id']}`",
        f"- **Work Package:** `{pkt['work_package']}` (Phase {pkt['phase']})",
        f"- **Complexity & Assigned Tier:** Class `{pkt['complexity_class']}` -> `{pkt['agent_tier']}`",
        f"- **Environment Access:** `{pkt['env_access']}`",
        f"- **FFI Seam Pattern:** {pkt['ffi_pattern']}",
        f"- **Allowed Rust Output File:** `{pkt['rust_target']}` *(Plus removing `#[ignore]` in the corresponding `crates/{pkt['rust_crate']}/tests/golden_*.rs` test file; do not modify other files)*",
        f"- **Mandatory V&V Gates:** `{', '.join(pkt['vv_gates'])}`",
        "- **Local / Ephemeral-Web Verification Commands:**",
        "  ```bash",
        "  python3 scripts/lint_determinism.py && cargo test --workspace",
        "  ```",
        "",
        "## 2. AST/Source-Extracted Determinism Invariants",
    ]

    lines.append(f"- **Direct Engine `#include` Dependencies ({len(inv['includes'])}):**")
    for inc in inv["includes"]:
        lines.append(f"  - `{inc}`")

    if inv["facets"]:
        lines.append(f"- **Implemented Module Facets ({len(inv['facets'])}):**")
        for fac in inv["facets"]:
            lines.append(f"  - `{fac}`")

    if inv["ini_fields"]:
        lines.append(f"- **INI `FieldParse` Table Entries ({len(inv['ini_fields'])}):**")
        for fld in inv["ini_fields"]:
            lines.append(f"  - `{fld}`")

    if inv["xfer_calls"]:
        lines.append(
            f"- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence ({len(inv['xfer_calls'])} calls — preserve order identically):**"
        )
        for idx, xc in enumerate(inv["xfer_calls"], start=1):
            lines.append(f"  {idx}. `{xc}`")
    else:
        lines.append("- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.")

    if inv["rng_calls"]:
        lines.append(
            f"- **PRNG Calls ({len(inv['rng_calls'])} — preserve call order & `lo >= hi` no-advance guard):**"
        )
        for rc in inv["rng_calls"]:
            lines.append(f"  - `{rc}`")

    if inv["retail_guards"]:
        lines.append(
            "- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = \"retail-crc\")]`):**"
        )
        for rg in inv["retail_guards"]:
            lines.append(f"  - `#if {rg}`")

    lines.extend([
        "",
        "---",
        "",
        cheatsheet,
        "",
        "---",
        "",
        "## 3. Reference C++ Source (`fbraz3/GeneralsX` at `84a7a20`)",
        "",
    ])

    for path, content in stripped_sources.items():
        lines.extend([
            f"### `{path}`",
            "```cpp",
            content,
            "```",
            "",
        ])

    return "\n".join(lines)


def main(argv: List[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Tier 0 Deterministic Context Slicer for generals-rust."
    )
    parser.add_argument(
        "targets",
        nargs="+",
        help="Packet IDs (e.g. PKT-001) or module names (e.g. RandomValue, ExperienceTracker).",
    )
    args = parser.parse_args(argv)

    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    commit = manifest["reference_commit"]
    cheatsheet = extract_cheatsheet(CONVENTIONS_PATH.read_text(encoding="utf-8"))

    lookup = {}
    for pkt in manifest["packets"]:
        lookup[pkt["packet_id"].upper()] = pkt
        lookup[pkt["module_name"].lower()] = pkt

    for target in args.targets:
        key = target.upper() if target.upper().startswith("PKT-") else target.lower()
        if key not in lookup:
            print(f"ERROR: Unknown packet or module '{target}'", file=sys.stderr)
            return 1

        pkt = lookup[key]
        sources = {}
        for cf in pkt["cpp_files"]:
            sources[cf["path"]] = fetch_cpp_file(cf["path"], commit)

        md = render_packet_markdown(pkt, sources, cheatsheet)
        out_dir = REPO_ROOT / f"packets/phase{pkt['phase']}"
        out_dir.mkdir(parents=True, exist_ok=True)
        out_path = out_dir / f"{pkt['packet_id']}_{pkt['module_name']}.md"
        out_path.write_text(md, encoding="utf-8")
        est_tokens = round(len(md) / 4)
        print(f"Generated {out_path.relative_to(REPO_ROOT)} ({len(md):,} bytes, ~{est_tokens:,} tokens)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
