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
from typing import Any, Dict, List, Optional, Tuple

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
MANIFEST_PATH = REPO_ROOT / "docs/MODULE_MANIFEST.json"
CONVENTIONS_PATH = REPO_ROOT / "docs/RUST_CONVENTIONS.md"
VENDOR_GX = REPO_ROOT / "vendor/GeneralsX"

VENDOR_INCLUDE_SEARCH_DIRS: Tuple[str, ...] = (
    "GeneralsMD/Code/GameEngine/Include",
    "Core/GameEngine/Include",
    "GeneralsMD/Code/Libraries/Include",
    "Core/Libraries/Include",
    "GeneralsMD/Code/GameEngine/Source",
    "Core/GameEngine/Source",
)

PUB_ITEM_RE = re.compile(
    r"^\s*pub(?:\([^)]*\))?\s+(?:struct|enum|trait|type|const)\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)

CORE_INDEX_PREFIXES: Tuple[str, ...] = (
    "zh_common::types",
    "zh_common::crc",
    "zh_common::random_value",
    "zh_common::xfer",
    "zh_common::math",
    "zh_common::core",
    "zh_common::ini",
    "zh_logic::arena",
    "zh_logic::scheduler",
    "zh_logic::modules",
    "zh_logic::sim",
    "zh_logic::rules",
    "zh_logic::object",
)


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


def resolve_include_paths(
    includes: List[str],
    source_paths: List[str],
    vendor_root: pathlib.Path = VENDOR_GX,
) -> Dict[str, Optional[str]]:
    """Resolves C++ `#include` paths against local `vendor/GeneralsX` directories."""
    resolved: Dict[str, Optional[str]] = {}
    if not vendor_root.is_dir():
        for inc in includes:
            resolved[inc] = None
        return resolved

    source_dirs = [((vendor_root / sp).parent) for sp in source_paths]
    for inc in includes:
        found_rel: Optional[str] = None
        candidates = [sdir / inc for sdir in source_dirs] + [
            vendor_root / search_dir / inc for search_dir in VENDOR_INCLUDE_SEARCH_DIRS
        ]
        for cand in candidates:
            if cand.is_file():
                rel = cand.relative_to(vendor_root).as_posix()
                found_rel = f"vendor/GeneralsX/{rel}"
                break
        resolved[inc] = found_rel
    return resolved


def _rust_file_to_module_path(crate_name: str, src_dir: pathlib.Path, rs_path: pathlib.Path) -> str:
    rel = rs_path.relative_to(src_dir)
    if rs_path.name == "lib.rs":
        return crate_name
    if rs_path.name == "mod.rs":
        parts = rel.parent.parts
        return "::".join((crate_name, *parts)) if parts else crate_name
    parts = rel.with_suffix("").parts
    return "::".join((crate_name, *parts))


def build_rust_symbol_index(
    repo_root: pathlib.Path,
    target_crate: str,
    target_rel: str,
    max_symbols_per_mod: int = 14,
    max_lines: int = 55,
) -> List[str]:
    """Builds a compact deterministic index of existing Rust public symbols."""
    sibling_prefix = ""
    if target_rel:
        rel_path = pathlib.Path(target_rel)
        parts = rel_path.parts
        # crates/<crate>/src/<subpath...>
        if len(parts) >= 4 and parts[0] == "crates" and parts[2] == "src":
            mod_parts = parts[3:-1]
            sibling_prefix = "::".join((parts[1], *mod_parts)) if mod_parts else parts[1]

    entries: List[Tuple[int, str, List[str]]] = []
    for crate_name in ("zh_common", "zh_logic"):
        src_dir = repo_root / "crates" / crate_name / "src"
        if not src_dir.is_dir():
            continue
        for rs_path in sorted(src_dir.rglob("*.rs"), key=lambda p: p.as_posix()):
            if rs_path.name == "lib.rs":
                continue
            text = rs_path.read_text(encoding="utf-8", errors="replace")
            raw_syms = PUB_ITEM_RE.findall(text)
            if not raw_syms:
                continue
            seen = set()
            syms: List[str] = []
            for sym in raw_syms:
                if sym not in seen:
                    seen.add(sym)
                    syms.append(sym)
            mod_path = _rust_file_to_module_path(crate_name, src_dir, rs_path)
            is_sibling = bool(sibling_prefix) and (
                mod_path == sibling_prefix or mod_path.startswith(sibling_prefix + "::")
            )
            is_core = any(
                mod_path == prefix or mod_path.startswith(prefix + "::")
                for prefix in CORE_INDEX_PREFIXES
            )
            is_same_crate = crate_name == target_crate
            if not (is_sibling or is_core or is_same_crate):
                continue
            priority = 0 if is_sibling else (1 if is_core else 2)
            entries.append((priority, mod_path, syms))

    entries.sort(key=lambda item: (item[0], item[1]))
    lines: List[str] = []
    for _, mod_path, syms in entries[:max_lines]:
        shown = [f"`{s}`" for s in syms[:max_symbols_per_mod]]
        extra = len(syms) - len(shown)
        suffix = f", ... (+{extra})" if extra > 0 else ""
        lines.append(f"- `{mod_path}`: {', '.join(shown)}{suffix}")
    return lines


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
    vendor_root: pathlib.Path = VENDOR_GX,
    repo_root: pathlib.Path = REPO_ROOT,
) -> str:
    inv = analyze_cpp_invariants(sources)
    resolved_includes = resolve_include_paths(
        inv["includes"], list(sources.keys()), vendor_root=vendor_root
    )
    symbol_index_lines = build_rust_symbol_index(
        repo_root=repo_root,
        target_crate=str(pkt.get("rust_crate", "")),
        target_rel=str(pkt.get("rust_target", "")),
    )
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
        resolved = resolved_includes.get(inc)
        if resolved:
            lines.append(f"  - `{inc}` -> `{resolved}`")
        else:
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

    if symbol_index_lines:
        lines.extend([
            "",
            "## 2.5 Existing Rust Symbol Index (Use These Exact Paths)",
            *symbol_index_lines,
        ])

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
