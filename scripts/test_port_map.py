#!/usr/bin/env python3
"""Automated self-tests for scripts/port_map.py."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import port_map


def create_fixture_repo(root: Path) -> None:
    (root / "docs/golden_vectors").mkdir(parents=True, exist_ok=True)
    (root / "packets/phase1").mkdir(parents=True, exist_ok=True)
    (root / "crates/zh_common/src/core").mkdir(parents=True, exist_ok=True)
    (root / "crates/zh_common/tests").mkdir(parents=True, exist_ok=True)

    manifest = {
        "reference_repo": "fbraz3/GeneralsX",
        "reference_commit": "84a7a20769aa5749f388b1f49bd542c79b3c33e8",
        "packets": [
            {
                "packet_id": "PKT-001",
                "module_name": "Alpha",
                "phase": 1,
                "work_package": "WP-1.1",
                "rust_target": "crates/zh_common/src/core/alpha.rs",
            },
            {
                "packet_id": "PKT-002",
                "module_name": "Beta",
                "phase": 1,
                "work_package": "WP-1.2",
                "rust_target": "crates/zh_common/src/core/beta.rs",
            },
            {
                "packet_id": "PKT-003",
                "module_name": "Gamma",
                "phase": 2,
                "work_package": "WP-2.1",
                "rust_target": "crates/zh_common/src/core/gamma.rs",
            },
            {
                "packet_id": "PKT-004",
                "module_name": "Delta",
                "phase": 5,
                "work_package": "WP-5.4",
                "rust_target": "crates/zh_common/src/core/delta.rs",
            },
        ],
    }
    (root / "docs/MODULE_MANIFEST.json").write_text(
        json.dumps(manifest), encoding="utf-8"
    )
    (root / "crates/zh_common/src/core/alpha.rs").write_text(
        "pub fn ok() -> i32 { 42 }\n", encoding="utf-8"
    )
    (root / "crates/zh_common/src/core/beta.rs").write_text(
        'pub fn wip() -> i32 { todo!("PKT-002") }\n', encoding="utf-8"
    )
    (root / "crates/zh_common/src/core/delta.rs").write_text(
        "//! `PKT-004`: `Delta` module data. The behaviour is not ported.\n",
        encoding="utf-8",
    )
    (root / "crates/zh_common/src/extra.rs").write_text(
        "pub fn helper() {}\n", encoding="utf-8"
    )
    (root / "packets/phase1/PKT-002_Beta.md").write_text("# Beta\n", encoding="utf-8")
    (root / "docs/golden_vectors/PKT-001.json").write_text(
        '{\n  "source_sha256": {},\n  "vector_count": 10\n}\n', encoding="utf-8"
    )
    large_prefix = '"enum_table": "' + ("x" * 5000) + '"'
    (root / "docs/golden_vectors/PKT-999.json").write_text(
        '{\n  ' + large_prefix + ',\n  "source_sha256": {},\n  "vector_count": 777\n}\n',
        encoding="utf-8",
    )
    (root / "crates/zh_common/tests/golden_pkt001.rs").write_text(
        "//! When ready, remove the #[ignore] attribute below.\n#[test]\nfn pass() {}\n",
        encoding="utf-8",
    )
    (root / "crates/zh_common/tests/golden_pkt002.rs").write_text(
        '#[test]\n#[ignore = "pending"]\nfn wip() {}\n', encoding="utf-8"
    )


class PortMapTests(unittest.TestCase):
    def test_classification_determinism_and_check_roundtrip(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            create_fixture_repo(root)

            # Check fails before --write
            self.assertEqual(port_map.main(["--root", str(root), "--check"]), 1)

            # Deterministic rendering
            first = port_map.render_port_map(root)
            second = port_map.render_port_map(root)
            self.assertEqual(first, second)
            self.assertNotIn("\\", first)
            self.assertIn("| `PKT-001` | `Alpha` | 1 | `WP-1.1` | `implemented` |", first)
            self.assertIn("| `PKT-002` | `Beta` | 1 | `WP-1.2` | `seeded` |", first)
            self.assertIn("| Phase 2 | 0 | 0 | 0 | 1 | 1 |", first)
            self.assertIn("| `PKT-004` | `Delta` | 5 | `WP-5.4` | `data-only` |", first)
            self.assertIn("| Phase 5 | 0 | 1 | 0 | 0 | 1 |", first)
            self.assertIn("| `zh_common` | `crates/zh_common/src/extra.rs` | `implemented` |", first)
            self.assertIn("| `docs/golden_vectors/PKT-001.json` | 10 | yes |", first)
            self.assertIn("| `docs/golden_vectors/PKT-999.json` | 777 | yes |", first)
            self.assertIn("| `zh_common` | `crates/zh_common/tests/golden_pkt001.rs` | `active` |", first)
            self.assertIn("| `zh_common` | `crates/zh_common/tests/golden_pkt002.rs` | `seeded (ignored)` |", first)

            # Write and check pass
            self.assertEqual(port_map.main(["--root", str(root), "--write"]), 0)
            self.assertEqual(port_map.main(["--root", str(root), "--check"]), 0)

            # CRLF tolerance for Windows git checkouts
            out_md = root / "docs/PORT_MAP.md"
            out_md.write_bytes(first.replace("\n", "\r\n").encode("utf-8"))
            self.assertEqual(port_map.main(["--root", str(root), "--check"]), 0)

            # Stale content detection
            out_md.write_text(first + "\n<!-- stale -->\n", encoding="utf-8")
            self.assertEqual(port_map.main(["--root", str(root), "--check"]), 1)


if __name__ == "__main__":
    unittest.main()
