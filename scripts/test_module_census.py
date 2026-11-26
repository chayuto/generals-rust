#!/usr/bin/env python3
"""Self-test for `module_census.py` (no engine needed)."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import module_census as mc  # noqa: E402


class ModuleCensusTest(unittest.TestCase):
    def test_parse_and_rank(self) -> None:
        a = mc.parse_census("noise\nZH_CENSUS 0 A\nZH_CENSUS 0 A\nZH_CENSUS 1 Draw\nZH_CENSUS 0 B\n")
        b = mc.parse_census("ZH_CENSUS 0 B\n")
        self.assertEqual(mc.rank([a, b]), [("B", 2, 2), ("A", 1, 2)])

    def test_port_status_reads_the_real_port_map(self) -> None:
        status = mc.port_status((mc.REPO_ROOT / "docs/PORT_MAP.md").read_text())
        self.assertEqual(status["PKT-339"], "implemented")
        # `PORT_MAP.md` lists only ported packets.
        self.assertGreater(len(status), 200)

    def test_packet_comes_from_the_defining_file(self) -> None:
        sources = {
            "Core/x/Old.cpp": "Foo::Foo(int)\n{}\n",
            "GeneralsMD/y/New.cpp": "Foo::Foo( Thing *t )\n{}\n",
        }
        files = mc.defining_files({"Foo"}, sources)
        self.assertEqual(files, {"Foo": "GeneralsMD/y/New.cpp"})
        manifest = {
            "packets": [
                {"packet_id": "PKT-1", "module_name": "Foo", "cpp_files": []},
                {
                    "packet_id": "PKT-2",
                    "module_name": "New",
                    "cpp_files": [{"path": "GeneralsMD/y/New.cpp"}],
                },
            ]
        }
        self.assertEqual(mc.packet_of(files["Foo"], "Foo", manifest), "PKT-2")
        self.assertEqual(mc.packet_of(None, "Foo", manifest), "PKT-1")
        self.assertIsNone(mc.packet_of(None, "Bar", manifest))

    def test_render_lists_only_unported_modules(self) -> None:
        text = mc.render(
            [("Done", 2, 3, "PKT-1", "implemented", 10), ("Todo", 2, 1, "PKT-2", "unported", 20)], 2
        )
        self.assertIn("| `Todo` | `PKT-2` | 2 | 1 | 20 |", text)
        self.assertNotIn("| `Done`", text)
        self.assertIn("Instances with a ported packet: 3 of 4.", text)


if __name__ == "__main__":
    unittest.main()
