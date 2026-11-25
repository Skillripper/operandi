#!/usr/bin/env python3
"""Who uses whom: the area-to-area dependency graph of operandi-core and
operandi-app.

Both crates file their source in area folders (docs/CODEMAP.md, "Area
folders"). This script reads every `.rs` file under each crate's `src/`,
finds the paths it names (`use crate::...`, `crate::x::y`, `super::...`,
`operandi_core::...`, grouped `use` trees) and works out which area each
path lands in, including names re-exported at the crate root
(`pub use bevel::*` in lib.rs, `use operandi_core::*` in main.rs). It then
reports:

- cycles: groups of areas that all reach each other (strongly connected
  components), and inside them the weakest links, the ones cheapest to cut;
- the biggest fan-in (areas many others use) and fan-out (areas that use
  many others);
- what app areas use from core.

Test code (`*_tests.rs`, `tests.rs`, `#[cfg(test)]` modules) is counted
separately and left out of the cycle check: it is allowed to reach
anywhere.

It needs only Python 3.9+ and reads the source as text (no build, no
rust-analyzer), so it runs in a few seconds. It is a text scan, not the
compiler. After a glob import of a crate root (`use super::*;` in a
top-level module, `use operandi_core::*;`) it resolves bare CamelCase names
(types, traits, enum names) against that root, but not bare lower-case
function names, and macros are not expanded. It sees the paths people
write, which is what coupling is about.

The report keeps whatever sits between the `<!-- findings -->` and
`<!-- /findings -->` lines when it is regenerated: that is where the
hand-written reading of the numbers goes.

    scripts/area-deps.py                 # write the report (docs/AREA_DEPENDENCIES.md + SVGs)
    scripts/area-deps.py --check         # CI: warn about cycle links not in the baseline
    scripts/area-deps.py --check --strict    # same, but fail (exit 1)
    scripts/area-deps.py --write-baseline    # accept today's cycle links as known
    scripts/area-deps.py --json out.json     # dump the graph for other tools

The baseline (scripts/area-deps-baseline.txt) lists the links that are
part of a cycle today. `--check` warns only about links that are new, so
the known debt stays visible in the report without failing every pull
request, and a branch that adds a new cycle hears about it. Remove a line
from the baseline when its link is gone (`--check` says which lines are
stale).
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CRATES = {
    # short name: (crate dir, root file, extern name)
    "core": ("crates/operandi-core", "lib.rs", "operandi_core"),
    "app": ("crates/operandi-app", "main.rs", "operandi_app"),
}
REPORT = REPO / "docs" / "AREA_DEPENDENCIES.md"
SVG_DIR = REPO / "docs" / "area-dependencies"
BASELINE = REPO / "scripts" / "area-deps-baseline.txt"
ROOT_AREA = "(root)"

IDENT = r"[A-Za-z_][A-Za-z0-9_]*"

# --------------------------------------------------------------------------
# Lexing: blank out comments and string/char literals, keep offsets.
# --------------------------------------------------------------------------


def strip_rust(text: str) -> str:
    """Replace comments and literals with spaces (newlines kept), so regexes
    only see code. Lifetimes (`'a`) are kept."""
    out = list(text)
    i, n = 0, len(text)

    def blank(a: int, b: int) -> None:
        for k in range(a, min(b, n)):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = text[i]
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            j = text.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif c == "/" and i + 1 < n and text[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c in "rb" and re.match(r"(?:br|r)#*\"", text[i : i + 300]) and (
            i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")
        ):
            m = re.match(r"(?:br|r)(#*)\"", text[i:])
            hashes = m.group(1)
            end = text.find('"' + hashes, i + m.end())
            end = n if end < 0 else end + 1 + len(hashes)
            blank(i, end)
            i = end
        elif c == '"' or (c == "b" and i + 1 < n and text[i + 1] == '"'):
            j = i + (2 if c == "b" else 1)
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            blank(i, j + 1)
            i = j + 1
        elif c == "'":
            # char literal: 'x', '\n', '\u{..}', or a lifetime 'a
            m = re.match(r"'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^\\'])'", text[i : i + 16])
            if m:
                blank(i, i + m.end())
                i += m.end()
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def match_brace(text: str, open_at: int) -> int:
    """Index just past the brace that closes the one at `open_at`."""
    depth = 0
    for k in range(open_at, len(text)):
        if text[k] == "{":
            depth += 1
        elif text[k] == "}":
            depth -= 1
            if depth == 0:
                return k + 1
    return len(text)


# --------------------------------------------------------------------------
# Module tree
# --------------------------------------------------------------------------

ATTR_RE = re.compile(r"#\s*\[([^\]]*)\]")
MOD_DECL_RE = re.compile(
    r"((?:#\s*\[[^\]]*\]\s*)*)(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+(" + IDENT + r")\s*([;{])"
)
ITEM_RE = re.compile(
    r"\bpub(?:\s*\([^)]*\))?\s+(?:unsafe\s+|async\s+|const\s+|extern\s+\"[^\"]*\"\s+)*"
    r"(?:struct|enum|fn|trait|type|const|static|union|mod)\s+(" + IDENT + r")"
)
ANY_ITEM_RE = re.compile(
    r"^(?:pub(?:\s*\([^)]*\))?\s+)?(?:unsafe\s+|async\s+|const\s+)*"
    r"(?:struct|enum|fn|trait|type|const|static|union)\s+(" + IDENT + r")",
    re.M,
)
MACRO_RE = re.compile(r"macro_rules!\s*(" + IDENT + r")")
USE_RE = re.compile(r"\b((?:pub(?:\s*\([^)]*\))?\s+)?)use\s+([^;]+);")


class SourceFile:
    def __init__(self, crate: str, path: Path, src_dir: Path, modpath: list, test: bool):
        self.crate = crate
        self.path = path
        self.rel = path.relative_to(src_dir).as_posix()
        self.modpath = modpath  # e.g. ["editor", "tool"]; [] for the root
        self.test = test or path.name.endswith("_tests.rs") or path.name == "tests.rs"
        raw = path.read_text(encoding="utf-8", errors="replace")
        self.lines = raw.count("\n") + 1
        self.code = strip_rust(raw)
        # Inline modules: (start, end, name, is_test) spans.
        self.inline = []
        for m in MOD_DECL_RE.finditer(self.code):
            if m.group(3) == "{":
                brace = m.end() - 1
                is_test = "cfg(test)" in m.group(1).replace(" ", "")
                self.inline.append((brace, match_brace(self.code, brace), m.group(2), is_test))

    @property
    def area(self) -> str:
        parts = self.rel.split("/")
        if len(parts) > 1:
            return parts[0]
        stem = parts[0][:-3]
        return ROOT_AREA if stem in ("lib", "main") else stem

    def scope_at(self, pos: int):
        """Module path and test flag of the code at `pos`."""
        path, test = list(self.modpath), self.test
        for start, end, name, is_test in self.inline:
            if start <= pos < end:
                path.append(name)
                test = test or is_test
        return path, test


def is_mod_rs(path: Path) -> bool:
    return path.name in ("mod.rs", "lib.rs", "main.rs")


def build_tree(crate: str, src_dir: Path, root_file: Path):
    """Follow `mod` declarations from the root, honouring `#[path]`."""
    files = {}
    stack = [(root_file, [], False)]
    while stack:
        path, modpath, test = stack.pop()
        if path in files or not path.exists() or not path.is_relative_to(src_dir):
            # Files outside src/ (a test borrowing core's tests/support) are
            # not part of any area.
            continue
        sf = SourceFile(crate, path, src_dir, modpath, test)
        files[path] = sf
        child_dir = path.parent if is_mod_rs(path) else path.parent / path.stem
        for m in MOD_DECL_RE.finditer(sf.code):
            if m.group(3) != ";":
                continue
            # Skip declarations inside inline modules: rare here, and their
            # files would resolve differently.
            if any(s <= m.start() < e for s, e, _, _ in sf.inline):
                continue
            attrs = m.group(1)
            name = m.group(2)
            pm = re.search(r'path\s*=\s*"([^"]+)"', strip_attr_strings(path, m))
            if pm:
                child = path.parent / pm.group(1)
            else:
                child = child_dir / (name + ".rs")
                if not child.exists():
                    child = child_dir / name / "mod.rs"
            child_test = test or "cfg(test)" in attrs.replace(" ", "")
            stack.append((child.resolve(), modpath + [name], child_test))
    return files


_raw_cache = {}


def strip_attr_strings(path: Path, m: re.Match) -> str:
    """The attribute text of a mod declaration, from the raw source (the
    stripped copy has its string literals blanked)."""
    raw = _raw_cache.get(path)
    if raw is None:
        raw = _raw_cache[path] = path.read_text(encoding="utf-8", errors="replace")
    return raw[m.start(1) : m.end(1)]


# --------------------------------------------------------------------------
# `use` trees
# --------------------------------------------------------------------------


def expand_use(tree: str) -> list:
    """`a::{b, c::{d, e as f}, *}` -> [[a,b],[a,c,d],[a,c,e],[a,*]]"""
    # Drop renames (`x as y`, `_ as _`) before squeezing out whitespace.
    tree = re.sub(r"\s+as\s+" + IDENT, "", tree)
    tree = re.sub(r"\s+", "", tree)

    def parse(s: str, prefix: list) -> list:
        out = []
        for part in split_top(s):
            if not part:
                continue
            if "{" in part:
                head, _, rest = part.partition("{")
                inner = rest[:-1] if rest.endswith("}") else rest
                segs = [x for x in head.split("::") if x]
                out.extend(parse(inner, prefix + segs))
            else:
                out.append(prefix + [x for x in part.split("::") if x])
        return out

    return parse(tree, [])


def split_top(s: str) -> list:
    parts, depth, cur = [], 0, []
    for ch in s:
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return parts


# --------------------------------------------------------------------------
# Crate model: top-level modules, root symbols, references
# --------------------------------------------------------------------------


class Crate:
    def __init__(self, name: str, rel_dir: str, root: str, extern: str):
        self.name = name
        self.extern = extern
        self.src = (REPO / rel_dir / "src").resolve()
        self.files = build_tree(name, self.src, (self.src / root).resolve())
        self.root = (self.src / root).resolve()
        # top-level module -> area
        self.mod_area = {}
        for sf in self.files.values():
            if len(sf.modpath) == 1:
                self.mod_area[sf.modpath[0]] = sf.area
        self.symbols = {}  # crate-root name -> area (filled by resolve_symbols)
        self.glob_crates = []  # extern crates glob-imported at the root

    def module_names(self, top: str) -> set:
        """Public names defined by a top-level module (its own file only)."""
        names = set()
        for sf in self.files.values():
            if sf.modpath == [top]:
                names.update(ITEM_RE.findall(sf.code))
                for m in USE_RE.finditer(sf.code):
                    if m.group(1).strip():
                        for p in expand_use(m.group(2)):
                            if p and p[-1] != "*" and p[-1] != "self":
                                names.add(p[-1])
        return names

    def resolve_symbols(self, crates: dict) -> None:
        root = self.files[self.root]
        # Private root items count too: child modules can see them.
        for name in ANY_ITEM_RE.findall(root.code) + MACRO_RE.findall(root.code):
            self.symbols.setdefault(name, ROOT_AREA)
        for m in USE_RE.finditer(root.code):
            for p in expand_use(m.group(2)):
                if not p:
                    continue
                head = p[0]
                if head == "crate":
                    p = p[1:]
                    head = p[0] if p else ""
                if head in self.mod_area:
                    area = self.mod_area[head]
                    if p[-1] == "*" and len(p) == 2:
                        for nm in self.module_names(head):
                            self.symbols.setdefault(nm, area)
                    else:
                        self.symbols.setdefault(p[-1], area)
                else:
                    for other in crates.values():
                        if other is not self and head == other.extern:
                            if p[-1] == "*" and len(p) == 2:
                                self.glob_crates.append(other.name)
                            elif len(p) >= 2:
                                self.symbols.setdefault(p[-1], other.lookup(p[1:]))
        # macro_rules! with #[macro_export] land at the root too
        for sf in self.files.values():
            for name in MACRO_RE.findall(sf.code):
                self.symbols.setdefault(name, sf.area)

    def lookup(self, segs: list):
        """Area for a path below this crate's root, as "crate/area"."""
        if not segs:
            return f"{self.name}/{ROOT_AREA}"
        head = segs[0]
        if head in self.mod_area:
            return f"{self.name}/{self.mod_area[head]}"
        sym = self.symbols.get(head)
        if sym is not None:
            return sym if "/" in sym else f"{self.name}/{sym}"
        return f"{self.name}/{ROOT_AREA}"


def collect_refs(crate: Crate, crates: dict):
    """Yield (source file, target "crate/area", target module, is_test)."""
    externs = {c.extern: c for c in crates.values() if c is not crate}
    other_ws = workspace_externs()
    path_re = re.compile(
        r"(?<![A-Za-z0-9_$])((?:crate|super|self|" + "|".join(re.escape(e) for e in externs) + r")(?:\s*::\s*(?:" + IDENT + r"|\*))+)"
    )
    for sf in crate.files.values():
        code = sf.code
        found = []  # (pos, segments)
        spans = []
        for m in USE_RE.finditer(code):
            spans.append((m.start(), m.end()))
            for p in expand_use(m.group(2)):
                found.append((m.start(), p))
        for m in path_re.finditer(code):
            if any(s <= m.start() < e for s, e in spans):
                continue
            found.append((m.start(), re.sub(r"\s+", "", m.group(1)).split("::")))
        if sf.path == crate.root:
            # Root file: bare `module::item` paths.
            for m in re.finditer(r"(?<![A-Za-z0-9_:$])(" + IDENT + r")\s*::", code):
                if m.group(1) in crate.mod_area and not any(s <= m.start() < e for s, e in spans):
                    found.append((m.start(), ["crate", m.group(1)]))
        globs = []  # (crate glob-imported, inline-module scope, is_test)
        for pos, segs in found:
            if not segs:
                continue
            scope, test = sf.scope_at(pos)
            target = resolve(crate, externs, other_ws, scope, segs)
            if target is None:
                continue
            area, module = target
            if area is GLOB:
                globs.append((module, scope, test))
                continue
            yield sf, area, module, test
        if globs and sf.path != crate.root:
            yield from bare_names(sf, crate, globs)


GLOB = object()  # marker: a glob import of a crate root
TYPE_NAME_RE = re.compile(r"(?<![A-Za-z0-9_:$.])([A-Z][A-Za-z0-9_]*)\b")


def bare_names(sf, crate, globs):
    """A file that glob-imports a crate root (`use super::*;` in a top-level
    module, `use operandi_core::*;`) then names its items bare. Resolve the
    type-like names (CamelCase) it uses but doesn't define itself against
    that root. Lower-case names are skipped: too many clash with locals."""
    local = set(ITEM_RE.findall(sf.code))
    local |= set(re.findall(r"\b(?:struct|enum|trait|type|use\s+[^;]*\bas)\s+(" + IDENT + r")", sf.code))
    for m in USE_RE.finditer(sf.code):
        for p in expand_use(m.group(2)):
            if p and p[-1] not in ("*", "self"):
                local.add(p[-1])
    # Generic parameters (`<T: Trait>`, `impl<S>`) are single capitals or
    # short; they rarely match a root name, so no special handling.
    seen = set()
    for m in TYPE_NAME_RE.finditer(sf.code):
        name = m.group(1)
        if name in local or name in seen or name in ("Self",):
            continue
        scope, test = sf.scope_at(m.start())
        for root, gscope, gtest in globs:
            # The glob reaches this name only from inside its own scope.
            if scope[: len(gscope)] != gscope:
                continue
            target = root_name(root, name)
            if target is None:
                continue
            seen.add(name)
            yield sf, target[0], target[1], test or gtest
            break


def root_name(c, name):
    """Area of a bare name as seen at crate `c`'s root (its own items and
    re-exports, then crates it glob-imports), or None."""
    sym = c.symbols.get(name)
    if sym is not None:
        return (sym if "/" in sym else f"{c.name}/{sym}"), name
    for other_name in c.glob_crates:
        other = c.all[other_name]
        if name in other.symbols:
            return other.lookup([name]), f"{other.name}::{name}"
    return None


def resolve(crate: Crate, externs: dict, other_ws: set, scope: list, segs: list):
    head = segs[0]
    if head == "crate":
        rest = segs[1:]
    elif head == "super" or head == "self":
        path = list(scope)
        k = 0
        while k < len(segs) and segs[k] in ("super", "self"):
            if segs[k] == "super":
                if not path:
                    return None
                path.pop()
            k += 1
        rest = path + segs[k:]
    elif head in externs:
        other = externs[head]
        rest = segs[1:]
        if rest == ["*"]:
            return GLOB, other
        module = rest[0] if rest else ROOT_AREA
        return other.lookup(rest), f"{other.name}::{module}"
    elif head in other_ws:
        return f"crate:{head}", head
    else:
        return None  # std, third-party, or a local name in scope
    if rest == ["*"]:
        return GLOB, crate  # `use super::*` in a top-level module, `use crate::*`
    if not rest or rest[0] == "self":
        return None
    top = rest[0]
    if top in crate.mod_area:
        return f"{crate.name}/{crate.mod_area[top]}", top
    sym = crate.symbols.get(top)
    if sym is not None:
        full = sym if "/" in sym else f"{crate.name}/{sym}"
        return full, top
    for name in crate.glob_crates:
        other = crate.all[name]
        if top in other.mod_area or top in other.symbols:
            return other.lookup([top]), f"{other.name}::{top}"
    return f"{crate.name}/{ROOT_AREA}", top


_ws_cache = None


def workspace_externs() -> set:
    """Other workspace crates' extern names (operandi_files, ...)."""
    global _ws_cache
    if _ws_cache is None:
        _ws_cache = {
            d.name.replace("-", "_")
            for d in (REPO / "crates").iterdir()
            if (d / "Cargo.toml").exists() and d.name not in ("operandi-core", "operandi-app")
        }
    return _ws_cache


# --------------------------------------------------------------------------
# Graph
# --------------------------------------------------------------------------


class Graph:
    def __init__(self):
        # (src, dst) -> {source file rel path: set of target modules}
        self.edges = defaultdict(lambda: defaultdict(set))
        self.test_edges = defaultdict(lambda: defaultdict(set))
        self.nodes = set()
        self.lines = defaultdict(int)
        self.files = defaultdict(int)

    def weight(self, e, test=False) -> int:
        return len((self.test_edges if test else self.edges)[e])

    def out(self, n):
        return sorted({d for (s, d) in self.edges if s == n})

    def inn(self, n):
        return sorted({s for (s, d) in self.edges if d == n})


def build_graph():
    crates = {k: Crate(k, *v) for k, v in CRATES.items()}
    for c in crates.values():
        c.all = crates
    for c in crates.values():
        c.resolve_symbols(crates)
    g = Graph()
    for c in crates.values():
        for sf in c.files.values():
            node = f"{c.name}/{sf.area}"
            g.nodes.add(node)
            g.lines[node] += sf.lines
            g.files[node] += 1
        for sf, dst, module, test in collect_refs(c, crates):
            src = f"{c.name}/{sf.area}"
            if src == dst:
                continue
            (g.test_edges if test else g.edges)[(src, dst)][sf.rel].add(module)
    # Test-only edges: drop those that also exist as production edges.
    for e in list(g.test_edges):
        if e in g.edges:
            del g.test_edges[e]
    return g, crates


def tarjan(nodes, succ):
    index, low, on, stack, out = {}, {}, set(), [], []
    counter = [0]

    def visit(v):
        index[v] = low[v] = counter[0]
        counter[0] += 1
        stack.append(v)
        on.add(v)
        for w in succ.get(v, ()):
            if w not in index:
                visit(w)
                low[v] = min(low[v], low[w])
            elif w in on:
                low[v] = min(low[v], index[w])
        if low[v] == index[v]:
            comp = []
            while True:
                w = stack.pop()
                on.discard(w)
                comp.append(w)
                if w == v:
                    break
            out.append(sorted(comp))

    sys.setrecursionlimit(10000)
    for v in sorted(nodes):
        if v not in index:
            visit(v)
    return out


def eades_order(nodes, weight):
    """Greedy feedback-arc-set ordering (Eades, Lin, Smyth 1993), weighted.
    Edges that point backwards in the order are the cheapest set to cut so
    the rest becomes a one-way stack."""
    nodes = set(nodes)
    left, right = [], []
    while nodes:
        changed = True
        while changed:
            changed = False
            for v in sorted(nodes):
                if not any((v, w) in weight for w in nodes if w != v):
                    right.insert(0, v)
                    nodes.discard(v)
                    changed = True
            for v in sorted(nodes):
                if not any((w, v) in weight for w in nodes if w != v):
                    left.append(v)
                    nodes.discard(v)
                    changed = True
        if nodes:

            def delta(v):
                o = sum(weight.get((v, w), 0) for w in nodes)
                i = sum(weight.get((w, v), 0) for w in nodes)
                return o - i

            v = max(sorted(nodes), key=delta)
            left.append(v)
            nodes.discard(v)
    return refine(left + right, weight)


def refine(order, weight):
    """Greedy cleanup of the Eades order: move one area at a time to the
    place that most lowers the weight of links pointing back, until no move
    helps. Eades alone strands hubs (an area whose users were all placed
    first ends up treated as a sink)."""
    out_w = defaultdict(dict)
    in_w = defaultdict(dict)
    for (s, d), w in weight.items():
        out_w[s][d] = w
        in_w[d][s] = w
    order = list(order)
    improved = True
    while improved:
        improved = False
        for v in list(order):
            rest = [x for x in order if x != v]
            # Cost of v's own links for each slot i (v inserted before rest[i]):
            # back = links v -> u with u before v, plus u -> v with u after v.
            best_i, best_cost = None, None
            before_out = 0  # weight v -> rest[:i]
            after_in = sum(in_w[v].values())  # weight rest[i:] -> v
            for i in range(len(rest) + 1):
                cost = before_out + after_in
                if best_cost is None or cost < best_cost:
                    best_i, best_cost = i, cost
                if i < len(rest):
                    u = rest[i]
                    before_out += out_w[v].get(u, 0)
                    after_in -= in_w[v].get(u, 0)
            cur = order.index(v)
            cur_cost = sum(out_w[v].get(u, 0) for u in order[:cur]) + sum(
                in_w[v].get(u, 0) for u in order[cur + 1 :]
            )
            if best_cost < cur_cost:
                rest.insert(best_i, v)
                order = rest
                improved = True
    return order


def analyse(g: Graph):
    weight = {e: g.weight(e) for e in g.edges}
    succ = defaultdict(list)
    for s, d in g.edges:
        succ[s].append(d)
    comps = [c for c in tarjan(g.nodes, succ) if len(c) > 1]
    in_comp = {n: i for i, c in enumerate(comps) for n in c}
    cycle_edges = {e for e in g.edges if e[0] in in_comp and in_comp.get(e[1]) == in_comp[e[0]]}
    # Per crate, an order with users first and what they use last, that
    # leaves the fewest (weighted) links pointing back up.
    order = {}
    for crate in CRATES:
        ns = [n for n in g.nodes if n.startswith(crate + "/")]
        w = {e: v for e, v in weight.items() if e[0] in ns and e[1] in ns}
        order[crate] = eades_order(ns, w)
    back = set()
    for crate, o in order.items():
        pos = {n: i for i, n in enumerate(o)}
        for e in weight:
            if e[0] in pos and e[1] in pos and pos[e[0]] > pos[e[1]]:
                back.add(e)
    return comps, cycle_edges, order, back


# --------------------------------------------------------------------------
# Output
# --------------------------------------------------------------------------


def short(n: str) -> str:
    return n.split("/", 1)[1] if "/" in n else n


def git_head() -> str:
    try:
        return subprocess.check_output(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], text=True).strip()
    except Exception:
        return "?"


def example_files(g: Graph, e, k=3) -> str:
    files = sorted(g.edges[e].items(), key=lambda kv: (-len(kv[1]), kv[0]))
    shown = ", ".join(f"`{f}`" for f, _ in files[:k])
    more = len(files) - k
    return shown + (f" +{more}" if more > 0 else "")


def write_svg_matrix(g: Graph, crate: str, order: list, back: set, comps: list, path: Path) -> None:
    """Dependency structure matrix: row uses column. Rows and columns are in
    the same order (users above what they use), so links with the flow sit
    above the diagonal and the ones against it, drawn in red, below."""
    n = len(order)
    cell = 13
    label_w = 150
    top = 150
    size_w = label_w + n * cell + 20
    size_h = top + n * cell + 40
    pos = {v: i for i, v in enumerate(order)}
    comp_of = {v: i for i, c in enumerate(comps) for v in c}
    maxw = max([g.weight(e) for e in g.edges if e[0] in pos and e[1] in pos] or [1])
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{size_w}" height="{size_h}" '
        f'viewBox="0 0 {size_w} {size_h}" font-family="Helvetica, Arial, sans-serif" font-size="10">',
        "<style>.l{fill:#222}.g{stroke:#ddd;stroke-width:0.5}@media (prefers-color-scheme: dark)"
        "{.l{fill:#ddd}.bg{fill:#1e1e1e}.g{stroke:#444}}</style>",
        f'<rect class="bg" width="{size_w}" height="{size_h}" fill="#fff"/>',
        f'<text class="l" x="8" y="16" font-size="12" font-weight="bold">operandi-{crate}: row uses column. '
        "Grey = with the flow, red = against it (part of a cycle).</text>",
    ]
    for v, i in pos.items():
        y = top + i * cell + cell - 3
        mark = " ●" if v in comp_of else ""
        out.append(f'<text class="l" x="{label_w - 4}" y="{y}" text-anchor="end">{short(v)}{mark}</text>')
        x = label_w + i * cell + cell - 3
        out.append(f'<text class="l" x="{x}" y="{top - 4}" transform="rotate(-60 {x} {top - 4})">{short(v)}</text>')
    for k in range(n + 1):
        out.append(f'<line class="g" x1="{label_w}" y1="{top + k * cell}" x2="{label_w + n * cell}" y2="{top + k * cell}"/>')
        out.append(f'<line class="g" x1="{label_w + k * cell}" y1="{top}" x2="{label_w + k * cell}" y2="{top + n * cell}"/>')
    for e in g.edges:
        if e[0] not in pos or e[1] not in pos:
            continue
        w = g.weight(e)
        alpha = 0.25 + 0.75 * (w / maxw) ** 0.5
        colour = "#d62728" if e in back else "#555"
        x = label_w + pos[e[1]] * cell
        y = top + pos[e[0]] * cell
        out.append(
            f'<rect x="{x + 1}" y="{y + 1}" width="{cell - 2}" height="{cell - 2}" fill="{colour}" '
            f'fill-opacity="{alpha:.2f}"><title>{short(e[0])} uses {short(e[1])}: {w} files</title></rect>'
        )
    out.append(
        f'<text class="l" x="8" y="{size_h - 14}">● = in a cycle. Darker = more files. '
        "Hover a cell for the count. Generated by scripts/area-deps.py.</text>"
    )
    out.append("</svg>")
    path.write_text("\n".join(out) + "\n", encoding="utf-8")


def write_report(g: Graph, crates, comps, cycle_edges, order, back) -> None:
    head = git_head()
    today = datetime.date.today().isoformat()
    L = []
    w = L.append
    w("# Area dependencies")
    w("")
    w(f"Generated by `scripts/area-deps.py` on {today} at `{head}`; rerun it to refresh. "
      "Areas are the folders in [CODEMAP.md](CODEMAP.md). A link \"A uses B\" means at "
      "least one file in A names a path in B (`use`, `crate::`, `super::`, `operandi_core::`); "
      "its weight is the number of A's files that do. Test code is left out (counted at the end).")
    w("")
    for crate in CRATES:
        ns = [n for n in g.nodes if n.startswith(crate + "/")]
        links = [e for e in g.edges if e[0] in ns and e[1] in ns]
        w(f"- **operandi-{crate}**: {len(ns)} areas, {sum(g.files[n] for n in ns)} files, "
          f"{len(links)} area links, {sum(1 for e in links if e in back)} against the flow.")
    w("")
    w("Pictures: [core matrix](area-dependencies/core.svg), [app matrix](area-dependencies/app.svg) "
      "(row uses column; red cells are links against the flow).")
    w("")
    w("**The flow.** The script puts each crate's areas in the order that leaves the fewest "
      "links pointing the wrong way: areas that mostly *use* others first, areas that are mostly "
      "*used* last (the base). A link from a lower area back up to a higher one is \"against the "
      "flow\"; those are the links that make the cycles.")
    w("")
    for crate in CRATES:
        w(f"- operandi-{crate}, top to base: " + " → ".join(f"`{short(n)}`" for n in order[crate]))
    w("")
    findings = ""
    if REPORT.exists():
        m = re.search(r"<!-- findings -->.*?<!-- /findings -->", REPORT.read_text(encoding="utf-8"), re.S)
        if m:
            findings = m.group(0)
    w(findings or "<!-- findings -->\n<!-- /findings -->")
    w("")

    w("## Cycles")
    w("")
    if not comps:
        w("None.")
    for comp in sorted(comps, key=len, reverse=True):
        crate = comp[0].split("/")[0]
        inner = [e for e in g.edges if e in cycle_edges and e[0] in comp]
        w(f"### operandi-{crate}: {len(comp)} areas reach each other")
        w("")
        w(", ".join(f"`{short(n)}`" for n in comp))
        w("")
        cut = sorted((e for e in inner if e in back), key=lambda e: (g.weight(e), e))
        w(f"{len(inner)} links run inside this group. The {len(cut)} below point against the "
          "flow (most-used areas at the bottom); cutting them would break every cycle. "
          "Lightest first, since those are the cheapest to cut:")
        w("")
        w("| Link | Files | Reverse link | Where |")
        w("|---|---:|---:|---|")
        for e in cut:
            rev = g.weight((e[1], e[0])) if (e[1], e[0]) in g.edges else 0
            w(f"| {short(e[0])} → {short(e[1])} | {g.weight(e)} | {rev} | {example_files(g, e)} |")
        w("")
        pairs = sorted(
            {tuple(sorted(e)) for e in inner if (e[1], e[0]) in g.edges},
            key=lambda p: -min(g.weight(p), g.weight((p[1], p[0]))),
        )
        if pairs:
            w("Two-way pairs (each uses the other), most tangled first:")
            w("")
            w("| Pair | A → B | B → A |")
            w("|---|---:|---:|")
            for a, b in pairs[:25]:
                w(f"| {short(a)} ↔ {short(b)} | {g.weight((a, b))} | {g.weight((b, a))} |")
            if len(pairs) > 25:
                w(f"| … {len(pairs) - 25} more | | |")
            w("")

    w("## Fan-in and fan-out")
    w("")
    w("Fan-in: how many other areas use this one. Fan-out: how many it uses. "
      "Files: the weighted sum (files on the other side). High fan-in should mean a stable base; "
      "high fan-in *and* fan-out in one area is a hub that changes ripple through.")
    w("")
    for crate in CRATES:
        ns = [n for n in g.nodes if n.startswith(crate + "/")]
        rows = []
        for n in ns:
            fi = [s for s in g.inn(n)]
            fo = [d for d in g.out(n)]
            rows.append((n, len(fi), sum(g.weight((s, n)) for s in fi), len(fo), sum(g.weight((n, d)) for d in fo)))
        w(f"### operandi-{crate}")
        w("")
        w("| Area | Files | Lines | Fan-in | (files) | Fan-out | (files) | In a cycle |")
        w("|---|---:|---:|---:|---:|---:|---:|:-:|")
        comp_nodes = {x for c in comps for x in c}
        for n, fi, fiw, fo, fow in sorted(rows, key=lambda r: -(r[1] + r[3])):
            w(f"| {short(n)} | {g.files[n]} | {g.lines[n]:,} | {fi} | {fiw} | {fo} | {fow} | "
              f"{'●' if n in comp_nodes else ''} |")
        w("")

    w("## What the app uses from core")
    w("")
    w("App area → core area, by files. Only the 30 heaviest links.")
    w("")
    w("| App area | Core area | Files |")
    w("|---|---|---:|")
    cross = sorted(
        (e for e in g.edges if e[0].startswith("app/") and e[1].startswith("core/")),
        key=lambda e: -g.weight(e),
    )
    for e in cross[:30]:
        w(f"| {short(e[0])} | {short(e[1])} | {g.weight(e)} |")
    w("")
    by_core = defaultdict(int)
    for e in cross:
        by_core[e[1]] += 1
    w("Core areas used by the most app areas: " + ", ".join(
        f"`{short(k)}` ({v})" for k, v in sorted(by_core.items(), key=lambda kv: -kv[1])[:12]) + ".")
    w("")
    ext = defaultdict(int)
    for e in g.edges:
        if e[1].startswith("crate:"):
            ext[(e[0].split("/")[0], e[1][6:])] += g.weight(e)
    if ext:
        w("Other workspace crates named directly: " + ", ".join(
            f"{c} → `{x}` ({v} files)" for (c, x), v in sorted(ext.items(), key=lambda kv: -kv[1])) + ".")
        w("")

    w("## Test-only links")
    w("")
    t = sorted(g.test_edges, key=lambda e: -g.weight(e, test=True))
    w(f"{len(t)} links appear only in test code and are left out above. Heaviest: " + ", ".join(
        f"{e[0]} → {e[1]} ({g.weight(e, test=True)})" for e in t[:10]) + ".")
    w("")
    w("## How it works")
    w("")
    w("See the comment at the top of `scripts/area-deps.py`. CI runs `scripts/area-deps.py --check`, "
      "which warns (and does not fail yet) when a branch adds a link inside a cycle that is not in "
      "`scripts/area-deps-baseline.txt`.")
    REPORT.write_text("\n".join(L) + "\n", encoding="utf-8")


def self_test() -> int:
    """Small checks of the pieces most likely to break: the lexer, `use`
    trees, the cycle finder and the flow order."""
    code = strip_rust('let s = "crate::a"; // crate::b\nlet c = \'x\'; /* crate::c /* nested */ */ crate::d::e;')
    assert "crate::a" not in code and "crate::b" not in code and "crate::c" not in code, code
    assert "crate::d::e" in code
    code = strip_rust("fn f<'a>(x: &'a str) -> &'a str { r#\"crate::q\"# ; crate::z }")
    assert "crate::q" not in code and "crate::z" in code and "'a" in code, code
    got = expand_use("crate::{a::{b, c as d}, e, f::*}")
    assert got == [["crate", "a", "b"], ["crate", "a", "c"], ["crate", "e"], ["crate", "f", "*"]], got
    assert expand_use("super :: x as y") == [["super", "x"]]
    comps = tarjan({"a", "b", "c", "d"}, {"a": ["b"], "b": ["c"], "c": ["a"], "d": ["a"]})
    assert [c for c in comps if len(c) > 1] == [["a", "b", "c"]], comps
    order = eades_order(["top", "mid", "base"], {("top", "mid"): 5, ("mid", "base"): 5, ("base", "top"): 1})
    assert order == ["top", "mid", "base"], order
    print("area-deps self-test: ok")
    return 0


def cycle_lines(g, cycle_edges):
    return sorted(f"{s} -> {d}" for s, d in cycle_edges)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--check", action="store_true", help="compare cycle links with the baseline")
    ap.add_argument("--strict", action="store_true", help="with --check: exit 1 on new cycle links")
    ap.add_argument("--write-baseline", action="store_true", help="accept today's cycle links")
    ap.add_argument("--json", metavar="FILE", help="dump the graph as JSON")
    ap.add_argument("--no-report", action="store_true", help="don't write the report")
    ap.add_argument("--self-test", action="store_true", help="check the parser on small samples")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    g, crates = build_graph()
    comps, cycle_edges, order, back = analyse(g)

    if args.json:
        Path(args.json).write_text(json.dumps({
            "nodes": {n: {"files": g.files[n], "lines": g.lines[n]} for n in sorted(g.nodes)},
            "edges": [{"from": s, "to": d, "files": sorted(g.edges[(s, d)])} for s, d in sorted(g.edges)],
            "test_edges": [{"from": s, "to": d, "files": sorted(g.test_edges[(s, d)])} for s, d in sorted(g.test_edges)],
            "cycles": comps,
            "order": order,
        }, indent=1), encoding="utf-8")

    # The debt the check tracks: links inside a cycle that point against the
    # flow. New links *with* the flow (a tool area using geometry) are fine
    # even inside the tangle.
    debt = cycle_edges & back

    if args.write_baseline:
        BASELINE.write_text(
            "# Area links inside a dependency cycle, accepted as known debt\n"
            "# (docs/AREA_DEPENDENCIES.md). Written by `scripts/area-deps.py\n"
            "# --write-baseline`; `--check` lists the lines that are no longer\n"
            "# needed. Every link in a cycle is listed, not only today's links\n"
            "# against the flow, because the flow order shifts as the code changes.\n"
            + "\n".join(cycle_lines(g, cycle_edges)) + "\n",
            encoding="utf-8",
        )
        print(f"wrote {BASELINE.relative_to(REPO)} ({len(cycle_edges)} links)")
        return 0

    if args.check:
        known = set()
        if BASELINE.exists():
            known = {
                ln.strip() for ln in BASELINE.read_text(encoding="utf-8").splitlines()
                if ln.strip() and not ln.startswith("#")
            }
        new = sorted(set(cycle_lines(g, debt)) - known)
        # Warn only about links that are both new to the cycle and against
        # the flow; stale lines are links gone or no longer in a cycle.
        gone = sorted(known - set(cycle_lines(g, cycle_edges)))
        gh = os.environ.get("GITHUB_ACTIONS") == "true"
        level = "error" if args.strict else "warning"
        for ln in new:
            s, d = ln.split(" -> ")
            msg = (f"new area link against the flow, inside a dependency cycle: {ln} "
                   f"({example_files(g, (s, d))}). See docs/AREA_DEPENDENCIES.md; if it is "
                   "intended, add the line to scripts/area-deps-baseline.txt.")
            print(f"::{level}::{msg}" if gh else f"{level}: {msg}")
        for ln in gone:
            print(f"note: baseline line no longer needed (link gone or no longer in a cycle): {ln}")
        print(f"area-deps: {len(comps)} cycle group(s), {len(debt)} links against the flow "
              f"inside them, {len(new)} new, {len(gone)} stale baseline lines.")
        return 1 if (args.strict and new) else 0

    if not args.no_report:
        SVG_DIR.mkdir(parents=True, exist_ok=True)
        for crate in CRATES:
            cc = [c for c in comps if c[0].startswith(crate + "/")]
            write_svg_matrix(g, crate, order[crate], back, cc, SVG_DIR / f"{crate}.svg")
        write_report(g, crates, comps, cycle_edges, order, back)
        print(f"wrote {REPORT.relative_to(REPO)} and {SVG_DIR.relative_to(REPO)}/*.svg")
    return 0


if __name__ == "__main__":
    sys.exit(main())
