#!/usr/bin/env python3
"""Self-test for `scripts/check_replays.py`, using a stand-in engine (no retail data)."""

from __future__ import annotations

import contextlib
import io
import os
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import check_replays  # noqa: E402

GOOD_LOG = [
    'Simulating Replay "a.rep"',
    "CRC Mismatch in Frame 100 (ignored) InGame:0000000A Replay:0000000B",
    "CRC Mismatch in Frame 200 (ignored) InGame:0000000C Replay:0000000D",
    "Elapsed Time: 00:00 Game Time: 00:07/00:07",
]


class CheckReplaysTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        root = pathlib.Path(self.tmp.name)
        self.replays = root / "replays"
        self.replays.mkdir()
        (self.replays / "a.rep").write_bytes(b"stand-in")
        self.log = root / "log.txt"
        self.exit_code = root / "exit.txt"
        self.binary = root / "engine"
        self.binary.write_text(
            f"#!{sys.executable}\n"
            "import os, pathlib, sys\n"
            "assert os.environ['SDL_VIDEODRIVER'] == 'dummy'\n"
            "assert '-headless' in sys.argv\n"
            f"sys.stdout.write(pathlib.Path({str(self.log)!r}).read_text())\n"
            f"sys.exit(int(pathlib.Path({str(self.exit_code)!r}).read_text()))\n"
        )
        self.binary.chmod(0o755)
        self.baseline = root / "out" / "baseline.json"
        self.set_run(GOOD_LOG)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def set_run(self, lines: list[str], code: int = 0) -> None:
        self.log.write_text("\n".join(lines) + "\n")
        self.exit_code.write_text(str(code))

    def run_main(self, mode: str, *extra: str) -> tuple[int, str]:
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
            code = check_replays.main(
                [
                    mode,
                    "--binary",
                    str(self.binary),
                    "--assets",
                    self.tmp.name,
                    "--replay-dir",
                    str(self.replays),
                    "--baseline",
                    str(self.baseline),
                    *extra,
                ]
            )
        return code, out.getvalue()

    def test_parse_log(self) -> None:
        parsed = check_replays.parse_log("\n".join(GOOD_LOG))
        self.assertEqual(parsed["crcs"][1], [200, "0000000C", "0000000D"])
        self.assertEqual(parsed["game_time"], ["00:07", "00:07"])

    def test_matching_run_passes(self) -> None:
        self.assertEqual(self.run_main("--write")[0], 0)
        code, out = self.run_main("--check")
        self.assertEqual(code, 0, out)

    def test_relative_binary_path_works(self) -> None:
        previous = os.getcwd()
        os.chdir(self.tmp.name)
        try:
            self.binary = pathlib.Path("engine")
            code, out = self.run_main("--write")
        finally:
            os.chdir(previous)
        self.assertEqual(code, 0, out)

    def test_one_changed_crc_fails(self) -> None:
        self.run_main("--write")
        self.set_run([line.replace("0000000C", "0000000E") for line in GOOD_LOG])
        code, out = self.run_main("--check")
        self.assertEqual(code, 1)
        self.assertIn("frame 200", out)

    def test_missing_entry_fails(self) -> None:
        self.run_main("--write")
        self.set_run(GOOD_LOG[:2] + GOOD_LOG[3:])
        self.assertEqual(self.run_main("--check")[0], 1)

    def test_early_stop_empty_trace_and_bad_exit_fail(self) -> None:
        self.run_main("--write")
        for lines, code in (
            (GOOD_LOG[:3] + ["Elapsed Time: 00:00 Game Time: 00:03/00:07"], 0),
            ([GOOD_LOG[0], GOOD_LOG[3]], 0),
            (GOOD_LOG, 3),
        ):
            self.set_run(lines, code)
            self.assertEqual(self.run_main("--check")[0], 1, (lines, code))

    def test_changed_replay_file_fails(self) -> None:
        self.run_main("--write")
        (self.replays / "a.rep").write_bytes(b"other")
        self.assertEqual(self.run_main("--check")[0], 1)

    def test_expect_rust_rejects_a_binary_without_rust(self) -> None:
        self.run_main("--write")
        code, out = self.run_main("--check", "--expect-rust")
        self.assertEqual(code, 1)
        self.assertIn("no zh_ symbols", out)


if __name__ == "__main__":
    unittest.main()
