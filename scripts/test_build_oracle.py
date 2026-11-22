#!/usr/bin/env python3
"""Automated self-tests for `scripts/build_oracle.py` (no vendor checkout required)."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import build_oracle


def create_fake_build_tree(repo_root: Path) -> Path:
    build_dir = repo_root / "vendor/GeneralsX/build/macos-vulkan"
    build_dir.mkdir(parents=True, exist_ok=True)
    stale_prefix = "/Users/stale/old-checkout"

    compile_commands = [
        {
            "directory": str(build_dir),
            "command": (
                "/usr/bin/c++ -DNDEBUG -DRTS_RELEASE -DRTS_ZEROHOUR=1 "
                f"-I{stale_prefix}/vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include "
                f"-I{stale_prefix}/crates/zh_ffi/include "
                f"-isystem {stale_prefix}/vendor/GeneralsX/build/macos-vulkan/_deps/googletest-src/include "
                "-O2 -g -std=c++20 -fPIC -Wsuggest-override -ffp-contract=off "
                "-include Precompiled/BasePrecomp.h -Winvalid-pch "
                f"-Xclang -include-pch -Xclang {stale_prefix}/vendor/GeneralsX/build/macos-vulkan/cmake_pch.hxx.pch "
                "-o CMakeFiles/z_gameengine.dir/Source/Common/GlobalData.cpp.o "
                f"-c {stale_prefix}/vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/GlobalData.cpp"
            ),
            "file": f"{stale_prefix}/vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/GlobalData.cpp",
        }
    ]
    (build_dir / "compile_commands.json").write_text(
        json.dumps(compile_commands, indent=2), encoding="utf-8"
    )

    build_ninja = f"""
build GeneralsMD/GeneralsXZH: CXX_EXECUTABLE_LINKER__z_generals_RelWithDebInfo Main.o | libz_gameengine.a
  CONFIG = RelWithDebInfo
  LINK_FLAGS = -Wl,-search_paths_first
  LINK_LIBRARIES = -Wl,-rpath,{stale_prefix}/vendor/GeneralsX/build/macos-vulkan/bin GeneralsMD/Code/GameEngine/libz_gameengine.a -lcomctl32 lib/libgtest.1.18.0.dylib {stale_prefix}/target/release/libzh_ffi.a _deps/sdl3-build/libSDL3.0.dylib -Wl,-framework,CoreFoundation
  TARGET_FILE = GeneralsMD/GeneralsXZH
"""
    (build_dir / "build.ninja").write_text(build_ninja, encoding="utf-8")
    return build_dir


class TestBuildOracle(unittest.TestCase):
    def test_rewrites_stale_paths_and_sanitizes_flags(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp).resolve()
            build_dir = create_fake_build_tree(repo_root)
            driver = repo_root / "driver.cpp"
            driver.write_text("int main() { return 0; }\n", encoding="utf-8")
            out_bin = repo_root / "out/oracle"

            cmd = build_oracle.build_command(
                sources=[driver],
                output=out_bin,
                repo_root=repo_root,
                build_dir=build_dir,
                opt="O2",
                sanitize=True,
                extra_flags=["-DTEST_FLAG=1"],
            )

            self.assertEqual(cmd[0], "/usr/bin/c++")
            self.assertIn("-O2", cmd)
            self.assertIn("-fsanitize=undefined", cmd)
            self.assertIn("-Wno-macro-redefined", cmd)
            self.assertIn("-Wno-implicit-exception-spec-mismatch", cmd)
            self.assertIn("-ffp-contract=off", cmd)
            self.assertIn("-DTEST_FLAG=1", cmd)
            self.assertIn("-include", cmd)
            self.assertIn("Precompiled/BasePrecomp.h", cmd)

            # Stale prefixes are replaced with active repo_root everywhere.
            root_posix = repo_root.as_posix()
            build_posix = build_dir.as_posix()
            joined = " ".join(cmd)
            self.assertNotIn("/Users/stale/old-checkout", joined)
            self.assertIn(f"-I{root_posix}/vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include", cmd)
            self.assertIn(f"{root_posix}/target/release/libzh_ffi.a", cmd)
            self.assertIn(f"{build_posix}/GeneralsMD/Code/GameEngine/libz_gameengine.a", cmd)
            self.assertIn(f"-Wl,-rpath,{build_posix}/bin", cmd)
            self.assertIn(f"-Wl,-rpath,{build_posix}/_deps/sdl3-build", cmd)

            # PCH, Windows-only libs, original source/output, and gtest are stripped.
            self.assertNotIn("-include-pch", cmd)
            self.assertNotIn("-Winvalid-pch", cmd)
            self.assertNotIn("-lcomctl32", cmd)
            self.assertFalse(any("libgtest" in tok for tok in cmd))
            self.assertFalse(any("googletest" in tok for tok in cmd))
            self.assertFalse(any("GlobalData.cpp" in tok for tok in cmd))

    def test_cli_dry_run_exits_zero(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp).resolve()
            build_dir = create_fake_build_tree(repo_root)
            driver = repo_root / "driver.cpp"
            driver.write_text("int main() { return 0; }\n", encoding="utf-8")
            rc = build_oracle.main(
                [
                    str(driver),
                    "-o",
                    str(repo_root / "oracle"),
                    "--repo-root",
                    str(repo_root),
                    "--build-dir",
                    str(build_dir),
                    "--dry-run",
                ]
            )
            self.assertEqual(rc, 0)


if __name__ == "__main__":
    unittest.main()
