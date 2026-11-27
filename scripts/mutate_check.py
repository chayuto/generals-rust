#!/usr/bin/env python3
"""Planted-bug check: change one line at a time and confirm the golden suite fails.

Run from the repository (or worktree) root:

    python3 scripts/mutate_check.py <test>[,<test>...] <file>... [--cap N] [--package NAME]

A file given as `diff:<path>` is mutated only on lines changed against HEAD.
Sources are restored after every trial. A suite that passes first time is not proof
until this has run. Read every survivor against the C++ before dismissing it.

For rechecks, import `run` from here: `subprocess.run(timeout=...)` kills cargo but
leaves a looping test binary running.
"""

import os
import pathlib
import random
import re
import signal
import subprocess
import sys

DEFAULT_CAP = 60
TRIAL_TIMEOUT = 300

OPERATORS = [
    (r" < ", " <= "), (r" <= ", " < "), (r" > ", " >= "), (r" >= ", " > "),
    (r" == ", " != "), (r" != ", " == "), (r" \+ ", " - "), (r" - ", " + "),
    (r" \* ", " + "), (r" && ", " || "), (r" \|\| ", " && "), (r"\bif !", "if "),
    (r"\btrue\b", "false"), (r"\bfalse\b", "true"),
    (r"\bTRUE\b(?=[;)])", "FALSE"), (r"\bFALSE\b(?=[;)])", "TRUE"),
    (r"wrapping_add\(", "wrapping_sub("), (r"\b1\b", "2"), (r"\b0\b", "1"),
]
STATEMENT = re.compile(r"^(self\.[\w.]+ [+\-]?= .*|sim\.\w+\(.*\)|self\.\w+\(.*\));$")


def run(cmd, timeout):
    """Run `cmd` in its own process group, killing the whole group on timeout."""
    process = subprocess.Popen(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        start_new_session=True,
    )
    try:
        out, err = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
        raise
    return subprocess.CompletedProcess(cmd, process.returncode, out, err)


def candidates(lines, wanted):
    """`(line index, mutated line)` pairs for the line indices in `wanted`."""
    found = []
    for index in wanted:
        line = lines[index]
        stripped = line.strip()
        if stripped.startswith(("//", "#", "use ")) or "xfer_version" in line:
            continue
        code = line.split("//")[0]
        for pattern, replacement in OPERATORS:
            if re.search(pattern, code):
                found.append((index, re.sub(pattern, replacement, code, count=1)))
        if STATEMENT.match(code.strip()):
            found.append((index, ""))
    return found


def changed_lines(path):
    """Zero-based indices of the lines changed against HEAD."""
    diff = subprocess.run(
        ["git", "diff", "-U0", "HEAD", "--", path], capture_output=True, text=True,
        check=True,
    ).stdout
    changed = []
    for match in re.finditer(r"^@@ -\S+ \+(\d+)(?:,(\d+))? @@", diff, re.M):
        start, count = int(match[1]), int(match[2] or 1)
        changed += range(start - 1, start - 1 + count)
    return changed


def main(argv):
    args = list(argv)
    cap, package = DEFAULT_CAP, "zh_logic"
    for flag in ("--cap", "--package"):
        if flag in args:
            at = args.index(flag)
            value = args[at + 1]
            del args[at:at + 2]
            if flag == "--cap":
                cap = int(value)
            else:
                package = value
    if len(args) < 2:
        print(__doc__)
        return 2

    tests, trials = args[0].split(","), []
    for spec in args[1:]:
        only_changed = spec.startswith("diff:")
        path = pathlib.Path(spec[5:] if only_changed else spec)
        lines = path.read_text().split("\n")
        wanted = changed_lines(str(path)) if only_changed else range(len(lines))
        trials += [(path, index, new) for index, new in candidates(lines, wanted)]

    random.seed(7)
    random.shuffle(trials)
    command = ["cargo", "test", "-q", "-p", package]
    command += [part for test in tests for part in ("--test", test)]

    caught = unbuildable = tested = 0
    survived = []
    for path, index, new in trials:
        if tested >= cap:
            break
        source = path.read_text()
        lines = source.split("\n")
        old, lines[index] = lines[index], new
        path.write_text("\n".join(lines))
        try:
            result = run(command, TRIAL_TIMEOUT)
            if result.returncode == 0:
                survived.append((path.name, index + 1, old.strip()[:80], new.strip()[:80]))
                tested += 1
            elif "error[" in result.stderr or "could not compile" in result.stderr:
                unbuildable += 1
            else:
                caught += 1
                tested += 1
        except subprocess.TimeoutExpired:
            caught += 1
            tested += 1
        finally:
            path.write_text(source)

    print("candidates", len(trials), "caught", caught, "nobuild", unbuildable,
          "survived", len(survived))
    for row in survived:
        print(*row, sep=" | ")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
