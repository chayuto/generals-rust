#!/usr/bin/env python3
"""Zero-token Wave Context Packer (`scripts/pack_wave.py`).

Generates a compact, deduplicated markdown context bundle (`/tmp/wave_<name>.md`)
containing only the exact Rust spine traits (`ObjectRecord`, `SimView`, `SimApi`,
`BehaviorModule`, selected `*Facet`, matching `BUILTIN_ZH_MODULES` entries,
`oracle_wp51_world::World` public signatures), any requested `--extra-refs`, and
the stripped C++ source sections of the requested `PKT-*` packets.

Usage:
  python3 scripts/pack_wave.py \
    --packets PKT-205,PKT-206,PKT-207 \
    --facet die \
    --out /tmp/wave_wp52_a.md
"""

from __future__ import annotations

import argparse
import glob
import re
import sys
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

DEFAULT_REPO_ROOT = Path(__file__).resolve().parent.parent

FACET_TRAIT_MAP: Dict[str, str] = {
    "update": "UpdateFacet",
    "body": "BodyFacet",
    "die": "DieFacet",
    "damage": "DamageFacet",
    "create": "CreateFacet",
    "collide": "CollideFacet",
    "contain": "ContainFacet",
    "upgrade": "UpgradeFacet",
    "special_power": "SpecialPowerFacet",
    "helper": "HelperFacet",
    "slow_death": "SlowDeathFacet",
    "destroy": "DestroyFacet",
    "ai": "AiUpdateFacet",
    "physics": "PhysicsFacet",
    "stealth": "StealthFacet",
}


def extract_braced_item(text: str, marker: str) -> str:
    """Extracts a top-level Rust struct/trait/impl block starting at `marker`."""
    idx = text.find(marker)
    if idx < 0:
        return ""
    line_start = text.rfind("\n", 0, idx) + 1
    brace_pos = text.find("{", idx)
    if brace_pos < 0:
        return ""

    depth = 0
    i = brace_pos
    n = len(text)
    in_str = False
    in_line_comment = False
    in_block_comment = False

    while i < n:
        ch = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
        elif in_block_comment:
            if ch == "*" and nxt == "/":
                in_block_comment = False
                i += 1
        elif in_str:
            if ch == "\\":
                i += 1
            elif ch == '"':
                in_str = False
        else:
            if ch == "/" and nxt == "/":
                in_line_comment = True
                i += 1
            elif ch == "/" and nxt == "*":
                in_block_comment = True
                i += 1
            elif ch == '"':
                in_str = True
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return text[line_start : i + 1].strip()
        i += 1
    return text[line_start:].strip()


def extract_oracle_world_signatures(oracle_path: Path) -> str:
    """Extracts public fields and method signatures of `oracle_wp51::World`."""
    if not oracle_path.is_file():
        return ""
    block = extract_braced_item(oracle_path.read_text(encoding="utf-8"), "struct World {")
    if not block:
        return ""

    out: List[str] = []
    brace_depth = 0
    for line in block.splitlines():
        opens = line.count("{")
        closes = line.count("}")
        next_depth = brace_depth + opens - closes
        if brace_depth <= 1:
            stripped = line.rstrip()
            if brace_depth == 1 and opens > closes and "{" in stripped:
                sig = stripped[: stripped.rfind("{")].rstrip() + ";"
                out.append(sig)
            else:
                out.append(stripped)
        brace_depth = next_depth
    return "\n".join(out)


def find_packet_files(repo_root: Path, packets: Sequence[str]) -> List[Tuple[str, Path]]:
    """Resolves `PKT-NNN` tokens (or numeric ids) to sorted `(label, Path)` pairs."""
    phase_dirs = sorted((repo_root / "packets").glob("phase*"))
    resolved: List[Tuple[str, Path]] = []
    for raw in packets:
        tok = raw.strip()
        if not tok:
            continue
        if not tok.upper().startswith("PKT-"):
            tok = f"PKT-{tok}"
        matches: List[Path] = []
        for pdir in phase_dirs:
            matches.extend(sorted(Path(p) for p in glob.glob(str(pdir / f"{tok}_*.md"))))
        if not matches:
            raise FileNotFoundError(f"Packet file not found for token '{tok}' under {repo_root / 'packets'}")
        path = matches[0]
        resolved.append((path.stem, path))
    return resolved


def extract_builtin_entries(module_factory_path: Path, packet_labels: Sequence[str]) -> List[str]:
    """Returns `BUILTIN_ZH_MODULES` lines matching any requested packet class name."""
    if not module_factory_path.is_file():
        return []
    text = module_factory_path.read_text(encoding="utf-8")
    tokens = []
    for label in packet_labels:
        cls_name = label.split("_", 1)[1] if "_" in label else label
        tokens.append(cls_name.lower())

    matched: List[str] = []
    for line in text.splitlines():
        if "BuiltinModuleEntry {" not in line:
            continue
        m = re.search(r'name:\s*"([^"]+)"', line)
        if not m:
            continue
        entry_lower = m.group(1).lower()
        if any(tok in entry_lower or entry_lower in tok for tok in tokens):
            matched.append(line.strip())
    return matched


def strip_cpp_packet_body(md_text: str) -> str:
    """Strips repeated packet preambles, GPL headers, and noisy divider banners."""
    marker = "## 3. Reference C++ Source"
    idx = md_text.find(marker)
    body = md_text[idx:] if idx >= 0 else md_text

    # Remove EA GPLv3 copyright header blocks repeated in every .h/.cpp slice.
    body = re.sub(r"/\*\s*\n\*\*\s*Command & Conquer Generals.*?\*/\s*\n?", "", body, flags=re.S)
    # Remove decorative separator banners and empty comment boxes.
    body = re.sub(r"(?m)^/{16,}\s*\n(?://[^\n]*\n){1,4}/{16,}\s*\n?", "", body)
    body = re.sub(r"(?m)^//[-=]{16,}[^\n]*\n?", "", body)
    # Collapse 3+ consecutive newlines to 2.
    body = re.sub(r"\n{3,}", "\n\n", body)
    return body.strip()


def read_extra_ref(repo_root: Path, spec: str) -> Tuple[str, str]:
    """Reads `path` or `path:start-end` (1-indexed inclusive) relative to `repo_root`."""
    if ":" in spec:
        rel_path, line_range = spec.rsplit(":", 1)
        m = re.match(r"^(\d+)-(\d+)$", line_range)
        if not m:
            raise ValueError(f"Invalid --extra-refs range '{spec}'; expected path or path:start-end")
        start, end = int(m.group(1)), int(m.group(2))
    else:
        rel_path, start, end = spec, 1, None

    p = Path(rel_path)
    full = p if p.is_absolute() else (repo_root / p)
    lines = full.read_text(encoding="utf-8").splitlines()
    slice_lines = lines[max(0, start - 1) : end]
    header = f"{rel_path}:{start}-{end if end is not None else len(lines)}"
    return header, "\n".join(slice_lines)


def build_wave_bundle(
    repo_root: Path,
    packets: Sequence[str],
    facet: str,
    extra_refs: Sequence[str] = (),
) -> str:
    """Builds the complete compact wave markdown bundle."""
    pkt_files = find_packet_files(repo_root, packets)
    sim_text = (repo_root / "crates/zh_logic/src/sim/mod.rs").read_text(encoding="utf-8")
    mod_text = (repo_root / "crates/zh_logic/src/modules/module.rs").read_text(encoding="utf-8")

    obj_record = extract_braced_item(sim_text, "pub struct ObjectRecord {")
    sim_view = extract_braced_item(sim_text, "pub trait SimView {")
    sim_api = extract_braced_item(sim_text, "pub trait SimApi: SimView {")
    snap_header = extract_braced_item(mod_text, "impl ModuleSnapshotHeader {")
    beh_trait = extract_braced_item(mod_text, "pub trait BehaviorModule: Send {")

    facet_key = facet.strip().lower()
    facet_blocks: List[str] = []
    if facet_key == "all":
        for trait_name in FACET_TRAIT_MAP.values():
            blk = extract_braced_item(mod_text, f"pub trait {trait_name} {{")
            if blk:
                facet_blocks.append(blk)
    elif facet_key != "none":
        trait_name = FACET_TRAIT_MAP.get(facet_key)
        if not trait_name:
            raise ValueError(f"Unknown --facet '{facet}'. Choices: {', '.join(sorted(FACET_TRAIT_MAP))}, none, all")
        blk = extract_braced_item(mod_text, f"pub trait {trait_name} {{")
        if blk:
            facet_blocks.append(blk)

    builtin_lines = extract_builtin_entries(
        repo_root / "crates/zh_logic/src/modules/module_factory.rs",
        [lbl for lbl, _ in pkt_files],
    )
    world_sigs = extract_oracle_world_signatures(repo_root / "tests/cpp_harness/oracle_wp51_world.h")

    sections: List[str] = [
        "# Wave Context Bundle\n",
        "## 1. Deduplicated Rust Determinism Rules",
        "- Types: `Bool`/`FALSE`/`TRUE`, `Int` (`i32`), `UnsignedInt` (`u32`), `Real` (`f32`), `NameKeyType`, `ObjectId`, `TeamId`, `XferVersion`.",
        "- Containers: ONLY `BTreeMap`/`BTreeSet` (ZERO `HashMap`/`HashSet`/`BinaryHeap`). Iterate keys deterministically.",
        "- Arithmetic & Lint (`scripts/lint_determinism.py`): NO `.round`/`.floor`/`.ceil`/`.trunc`/`.exp`/`.log`/`.sin`/`.cos`/`.sqrt` (even in names/comments without `DET` safety), NO `mul_add`, NO `usize` in `Xfer`, NO pointer-address ordering.",
        "- Governance: zero assistant/model names or attribution anywhere; never run `./scripts/bootstrap_repo.sh` or `git commit`; if using `lldb`, always pass `-k \"bt 15\" -k \"quit\" -o \"quit\"`.",
        "",
        "## 2. Spine Trait Signatures (`crates/zh_logic/src/sim/mod.rs` & `module.rs`)",
        "### `ObjectRecord`",
        "```rust\n" + obj_record + "\n```",
        "### `SimView` & `SimApi`",
        "```rust\n" + sim_view + "\n\n" + sim_api + "\n```",
        "### `ModuleSnapshotHeader` & `BehaviorModule`",
        "```rust\n" + snap_header + "\n\n" + beh_trait + "\n```",
    ]

    if facet_blocks:
        sections.append("### Requested Facet Trait(s)")
        sections.append("```rust\n" + "\n\n".join(facet_blocks) + "\n```")

    if builtin_lines:
        sections.append("### Matching `BUILTIN_ZH_MODULES` Entries (`module_factory.rs`)")
        sections.append("```rust\n" + "\n".join(builtin_lines) + "\n```")

    if world_sigs:
        sections.append("### `oracle_wp51::World` Public API (`tests/cpp_harness/oracle_wp51_world.h`)")
        sections.append("```cpp\n" + world_sigs + "\n```")

    if extra_refs:
        sections.append("## 3. Extra Scoped Reference Slices")
        for spec in extra_refs:
            hdr, content = read_extra_ref(repo_root, spec)
            sections.append(f"### `{hdr}`\n```\n{content}\n```")

    sections.append("## 4. Stripped C++ Reference Sources")
    for label, ppath in pkt_files:
        stripped_body = strip_cpp_packet_body(ppath.read_text(encoding="utf-8"))
        sections.append(f"### `{label}`\n{stripped_body}")

    return "\n\n".join(sections).rstrip() + "\n"


def parse_packets_arg(raw_list: Sequence[str]) -> List[str]:
    out: List[str] = []
    for item in raw_list:
        for tok in item.split(","):
            t = tok.strip()
            if t:
                out.append(t)
    return out


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Pack a compact deduplicated worker wave context bundle.")
    parser.add_argument(
        "--packets",
        nargs="+",
        required=True,
        help="Packet IDs (comma or space separated, e.g. PKT-205,PKT-206).",
    )
    parser.add_argument(
        "--facet",
        default="none",
        help=f"Primary facet trait to include ({', '.join(sorted(FACET_TRAIT_MAP))}, none, all).",
    )
    parser.add_argument("--out", required=True, type=Path, help="Output markdown path.")
    parser.add_argument(
        "--extra-refs",
        action="append",
        default=[],
        help="Optional extra file slice path[:start-end] relative to repo root.",
    )
    parser.add_argument("--repo-root", type=Path, default=DEFAULT_REPO_ROOT, help="Repository root.")

    args = parser.parse_args(argv)
    packets = parse_packets_arg(args.packets)
    bundle = build_wave_bundle(
        repo_root=args.repo_root.resolve(),
        packets=packets,
        facet=args.facet,
        extra_refs=args.extra_refs,
    )
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(bundle, encoding="utf-8")
    byte_len = len(bundle.encode("utf-8"))
    est_tokens = (byte_len + 3) // 4
    print(f"Packed {len(packets)} packet(s) -> {args.out} ({byte_len:,} bytes, ~{est_tokens:,} tokens)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
