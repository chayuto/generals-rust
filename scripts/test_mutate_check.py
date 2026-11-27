#!/usr/bin/env python3
"""Self-test for `scripts/mutate_check.py`."""

import os
import pathlib
import subprocess
import tempfile
import time
import unittest

import mutate_check


class TestMutateCheck(unittest.TestCase):
    def test_candidates_mutate_code_and_skip_comments(self) -> None:
        lines = [
            "// a < b",
            "use crate::a;",
            "if a < b {",
            "xfer.xfer_version(&mut v, 1)?;",
            "self.count += 3; // 0",
        ]
        found = mutate_check.candidates(lines, range(len(lines)))
        self.assertIn((2, "if a <= b {"), found)
        self.assertIn((4, ""), found)
        self.assertEqual({index for index, _ in found}, {2, 4})

    def test_only_wanted_lines_are_mutated(self) -> None:
        found = mutate_check.candidates(["a == b", "c == d"], [1])
        self.assertEqual(found, [(1, "c != d")])

    def test_timeout_kills_the_whole_process_group(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            pid_file = pathlib.Path(folder) / "child"
            with self.assertRaises(subprocess.TimeoutExpired):
                mutate_check.run(
                    ["sh", "-c", f"sleep 30 & echo $! > '{pid_file}'; wait"], 1
                )
            child = int(pid_file.read_text())
            for _ in range(50):
                try:
                    os.kill(child, 0)
                except ProcessLookupError:
                    return
                time.sleep(0.1)
            self.fail("the grandchild outlived the timeout")


if __name__ == "__main__":
    unittest.main()
