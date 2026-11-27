#!/usr/bin/env python3
"""
Ranks the ported behavior modules by how hard they are to run inside the C++ engine
(OPEN_DECISIONS.md item 10) and writes `docs/SWAP_CANDIDATES.md`.

For each class the replays create (`docs/module_instances.json`, written by
`module_census.py`) it follows every function that is handed the sim, through shared
helpers, and records which `SimView`/`SimApi` methods are reached.

It reads source text, not types. A call it cannot pin to one function is followed
into every candidate, so the lists can only be too long, except for dispatch through
a trait object to another module, which is counted separately.

  python3 scripts/swap_candidates.py
"""

from __future__ import annotations

import json
import pathlib
import re
import sys
from dataclasses import dataclass, field

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_engine_sim  # noqa: E402

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE_ROOT = REPO_ROOT / "crates/zh_logic/src"
COUNTS = REPO_ROOT / "docs/module_instances.json"
OUTPUT = REPO_ROOT / "docs/SWAP_CANDIDATES.md"

# Reference-returning methods whose result is an object the adapter can own itself.
SERVICES = ("client", "paths", "terrain")
# A name with more definitions than this, called on an unknown receiver, is taken to
# be dispatch through a trait object and is not followed.
DISPATCH_LIMIT = 4
NOT_CALLS = frozenset("if while match for return fn loop Some Ok Err assert assert_eq".split())


@dataclass
class Function:
    name: str
    owner: str | None
    file: str
    signature: str
    body: str
    sims: tuple[str, ...] = field(default=())


def strip(source: str) -> str:
    """Blanks comments, strings and char literals, and drops the trailing test module."""
    pattern = re.compile(
        r'//[^\n]*|/\*.*?\*/|r(#*)".*?"\1|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'', re.DOTALL
    )
    code = pattern.sub(lambda m: '""' if m.group(0)[-1] in '"#' else " ", source)
    tests = re.search(r"^#\[cfg\(test\)\]\s*\nmod \w+", code, re.MULTILINE)
    return code[: tests.start()] if tests else code


def closing(code: str, start: int) -> int:
    """The index of the brace or bracket closing the one at `start`."""
    pairs = {"{": "}", "(": ")"}
    opener = code[start]
    depth = 0
    for index in range(start, len(code)):
        if code[index] == opener:
            depth += 1
        elif code[index] == pairs[opener]:
            depth -= 1
            if depth == 0:
                return index
    return len(code) - 1


def functions(source: str, file: str) -> list[Function]:
    """Every function with a body, with the type whose `impl` or macro encloses it."""
    code = strip(source)
    owners: list[tuple[int, int, str]] = []
    for match in re.finditer(r"^[ \t]*(?:pub(?:\([\w:]+\))? )?trait\s+(\w+)[^{;]*\{", code, re.MULTILINE):
        end = closing(code, match.end() - 1)
        owners.append((match.start(), end, f"trait:{match.group(1)}"))
    header = re.compile(r"^[ \t]*(?:unsafe )?impl\b([^{;]*)\{", re.MULTILINE)
    for match in header.finditer(code):
        target = match.group(1).split(" for ")[-1].split(" where ")[0]
        target = re.sub(r"^\s*<[^>]*>", "", target).strip()
        target = re.sub(r"<.*", "", target).split("::")[-1].strip(" $&")
        end = closing(code, match.end() - 1)
        owners.append((match.start(), end, target))
    for match in re.finditer(r"macro_rules!\s*(\w+)\s*\{", code):
        end = closing(code, match.end() - 1)
        owners.append((match.start(), end, f"macro:{match.group(1)}"))

    found = []
    for match in re.finditer(r"\bfn\s+(\w+)", code):
        position = match.end()
        while position < len(code) and code[position] not in "{;":
            if code[position] == "(":
                position = closing(code, position)
            position += 1
        if position >= len(code) or code[position] == ";":
            continue
        signature = " ".join(code[match.start() : position].split())
        body = code[position : closing(code, position) + 1]
        inside = [o for o in owners if o[0] < match.start() < o[1]]
        # A macro owns its functions even when they sit in an `impl` inside it.
        macro = [o for o in inside if o[2].startswith("macro:")]
        chosen = macro or sorted(inside, key=lambda o: o[1] - o[0])
        owner = chosen[0][2] if chosen else None
        sims = tuple(
            re.findall(r"\b(\w+)\s*:\s*&(?:'\w+\s+)?(?:mut\s+)?(?:dyn\s+)?(?:\w+::)*Sim(?:Api|View)\b", signature)
        )
        generic = re.findall(r"\b(\w+)\s*:\s*(?:\w+::)*Sim(?:Api|View)\b", signature)
        for name in generic:
            sims += tuple(re.findall(rf"\b(\w+)\s*:\s*&(?:mut\s+)?{name}\b", signature))
        found.append(Function(match.group(1), owner, file, signature, body, sims))
    return found


def struct_fields(source: str) -> dict[str, dict[str, str]]:
    """struct -> field -> the last type name in the field's declared type."""
    code = strip(source)
    found: dict[str, dict[str, str]] = {}
    for match in re.finditer(r"\bstruct\s+(\w+)[^{;(]*\{", code):
        body = code[match.end() : closing(code, match.end() - 1)]
        fields = {}
        for name, declared in re.findall(r"^\s*(?:pub(?:\([\w:]+\))? )?(\w+)\s*:\s*([^\n]+?),?$", body, re.MULTILINE):
            names = re.findall(r"[A-Z]\w*", declared)
            if names:
                fields[name] = names[-1]
        found[match.group(1)] = fields
    return found


class Program:
    def __init__(self, sources: dict[str, str], methods: frozenset[str]):
        self.methods = methods
        self.functions: list[Function] = []
        self.by_name: dict[str, list[Function]] = {}
        self.by_owner: dict[str, list[Function]] = {}
        self.macro_users: dict[str, list[str]] = {}
        self.fields: dict[str, dict[str, str]] = {}
        for file, source in sources.items():
            self.fields.update(struct_fields(source))
            for trait, target in re.findall(
                r"^[ \t]*impl(?:<[^>]*>)?\s+(?:\w+::)*(\w+)(?:<[^>]*>)?\s+for\s+(\w+)", strip(source), re.MULTILINE
            ):
                self.macro_users.setdefault(target, []).append(f"trait:{trait}")
            for function in functions(source, file):
                self.functions.append(function)
                self.by_name.setdefault(function.name, []).append(function)
                if function.owner:
                    self.by_owner.setdefault(function.owner, []).append(function)
            for macro, args in re.findall(r"^\s*(\w+)!\s*\(([^()]*)\)\s*;", strip(source), re.MULTILINE):
                first = args.split(",")[0].strip().split("::")[-1]
                self.macro_users.setdefault(first, []).append(f"macro:{macro}")

    def roots(self, module: str) -> list[Function]:
        found = list(self.by_owner.get(module, []))
        for macro in self.macro_users.get(module, []):
            found += self.by_owner.get(macro, [])
        return found

    def field_type(self, function: Function, module: str, chain: list[str]) -> str | None:
        """The type of `self.a.b`, or None for `self` itself and for unknown fields."""
        starts = [function.owner or "", module]
        for start in starts:
            current = start
            for name in chain:
                current = self.fields.get(current, {}).get(name)
                if current is None:
                    break
            else:
                return current if chain else None
        return None

    def callees(self, function: Function, module: str) -> tuple[list[Function], set[str]]:
        """Functions that `function` hands the sim to, and the names it could not follow."""
        found: list[Function] = []
        dispatched: set[str] = set()
        for match in re.finditer(r"\b(\w+)\s*(?:::<[^()]*>)?\(", function.body):
            name = match.group(1)
            if name in NOT_CALLS or name not in self.by_name:
                continue
            arguments = function.body[match.end() - 1 : closing(function.body, match.end() - 1) + 1]
            if not any(re.search(rf"[(,]\s*(?:&\s*(?:mut\s+)?\*?\s*)?{s}\s*[,)]", arguments) for s in function.sims):
                continue
            candidates = [c for c in self.by_name[name] if c.sims]
            before = function.body[: match.start()].rstrip()
            path = re.search(r"(\w+)\s*::$", before)
            if path:
                owner = module if path.group(1) == "Self" else path.group(1)
                if path.group(1) == "Self":
                    owner = function.owner or module
                narrowed = [c for c in candidates if c.owner == owner]
                narrowed = narrowed or [c for c in candidates if pathlib.Path(c.file).stem == owner]
                candidates = narrowed or candidates
            elif before.endswith("."):
                if re.search(r"\b(?:%s)\s*\.$" % "|".join(function.sims), before):
                    continue  # a sim method, recorded by `reached`
                receiver = re.search(r"\bself((?:\s*\.\s*\w+)*)\s*\.$", before)
                if receiver:
                    owner = self.field_type(function, module, re.findall(r"\w+", receiver.group(1)))
                    owners = (module, function.owner) if owner is None else (owner,)
                    own = [c for c in candidates if c.owner in owners]
                    inherited = [c for c in candidates if c.owner in self.macro_users.get(owner or module, [])]
                    candidates = own or inherited or candidates
                if len(candidates) > DISPATCH_LIMIT:
                    dispatched.add(name)
                    continue
            else:
                here = [c for c in candidates if c.file == function.file and c.owner is None]
                candidates = here or [c for c in candidates if c.owner is None]
            found += candidates
        return found, dispatched

    def reached(self, module: str) -> tuple[set[str], set[str]]:
        """(sim methods reached, calls dispatched to another module) for one module."""
        pending = [f for f in self.roots(module) if f.sims]
        seen: set[int] = set()
        methods: set[str] = set()
        dispatched: set[str] = set()
        while pending:
            function = pending.pop()
            if id(function) in seen:
                continue
            seen.add(id(function))
            for sim in function.sims:
                for method in re.findall(rf"\b{sim}\s*\.\s*(\w+)\s*\(", function.body):
                    if method in self.methods:
                        methods.add(method)
            callees, skipped = self.callees(function, module)
            pending += callees
            dispatched |= skipped
        return methods, dispatched


def trait_methods(source: str) -> tuple[frozenset[str], frozenset[str]]:
    """All method names, and those returning a reference."""
    every: set[str] = set()
    references: set[str] = set()
    for trait in gen_engine_sim.TRAITS:
        for name, signature, _ in gen_engine_sim.methods(gen_engine_sim.trait_body(source, trait)):
            every.add(name)
            if "->" in signature and "&" in signature.rsplit("->", 1)[1]:
                references.add(name)
    return frozenset(every), frozenset(references)


def render(rows: list[dict], missing: list[str]) -> str:
    """`rows` have name, instances, replays, data, services, methods, dispatched."""
    free = [r for r in rows if not r["data"]]
    total = sum(r["instances"] for r in rows)
    out = [
        "# Which ported modules can run in the C++ engine",
        "",
        "Written by `python3 scripts/swap_candidates.py`. Do not edit by hand.",
        "",
        "`OPEN_DECISIONS.md` item 10. **Data** methods return a reference into",
        "Rust-owned simulation state, which a callback cannot answer. A module with none",
        "can go in through the adapter as the traits stand. **Methods** is how many",
        "`SimView`/`SimApi` methods it can reach, each needing a C++ callback. The list",
        "comes from source text and errs long; it does not follow dispatch to other",
        "modules (**other**).",
        "",
        f"- Classes analysed: {len(rows)}; with no data method: {len(free)}.",
        f"- Instances analysed: {total}; in classes with no data method: "
        f"{sum(r['instances'] for r in free)}.",
        f"- Classes not found in the Rust source: {', '.join(missing) or 'none'}.",
        "",
        "## How many classes each data method blocks",
        "",
        "| Method | Classes | Instances | Only blocker for |",
        "|---|---|---|---|",
    ]
    blockers = sorted({m for r in rows for m in r["data"]})
    for method in sorted(blockers, key=lambda m: -sum(m in r["data"] for r in rows)):
        hit = [r for r in rows if method in r["data"]]
        only = sum(r["data"] == [method] for r in hit)
        out.append(f"| `{method}` | {len(hit)} | {sum(r['instances'] for r in hit)} | {only} |")
    for title, group in (("No data method", free), ("Blocked", [r for r in rows if r["data"]])):
        out += [
            "",
            f"## {title}",
            "",
            "| Module | Instances | Replays | Methods | Services | Other | Data methods |",
            "|---|---|---|---|---|---|---|",
        ]
        for r in sorted(group, key=lambda r: (-r["instances"], r["name"])):
            out.append(
                f"| `{r['name']}` | {r['instances']} | {r['replays']} | {len(r['methods'])} | "
                f"{', '.join(r['services']) or '-'} | {len(r['dispatched'])} | "
                f"{', '.join(r['data']) or '-'} |"
            )
    return "\n".join(out) + "\n"


def analyse(program: Program, references: frozenset[str], counts: dict) -> tuple[list[dict], list[str]]:
    rows, missing = [], []
    for name, info in counts.items():
        # A few Rust types differ from the C++ class name only in capitals.
        spelled = {n.lower(): n for n in program.by_owner}
        module = name if program.roots(name) else spelled.get(name.lower(), name)
        if not program.roots(module):
            missing.append(name)
            continue
        methods, dispatched = program.reached(module)
        refs = methods & references
        rows.append(
            {
                "name": name,
                "instances": info["instances"],
                "replays": info["replays"],
                "methods": sorted(methods),
                "services": sorted(refs & set(SERVICES)),
                "data": sorted(refs - set(SERVICES)),
                "dispatched": sorted(dispatched),
            }
        )
    return rows, sorted(missing)


def load() -> tuple[Program, frozenset[str]]:
    every, references = trait_methods(gen_engine_sim.TRAITS_SOURCE.read_text())
    sources = {
        str(p.relative_to(REPO_ROOT)): p.read_text()
        for p in sorted(SOURCE_ROOT.rglob("*.rs"))
        # The traits and their one in-Rust implementation are what we look through to.
        if p != gen_engine_sim.TRAITS_SOURCE
    }
    return Program(sources, every), references


def main() -> int:
    program, references = load()
    rows, missing = analyse(program, references, json.loads(COUNTS.read_text()))
    OUTPUT.write_text(render(rows, missing))
    print("\n".join(OUTPUT.read_text().splitlines()[11:40]))
    if "--detail" in sys.argv:
        wanted = sys.argv[sys.argv.index("--detail") + 1]
        for row in rows:
            if row["name"] == wanted:
                print(json.dumps(row, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
