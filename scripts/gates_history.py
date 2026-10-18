#!/usr/bin/env python3
"""gates_history.py — the history check of a golden skip rule (engine/goldens/deps.toml): over main's first-parent
history since the goldens exist, how many golden-moving landings would the rule have skipped (must be 0), and what share
of the other commits it skips.

  scripts/gates_history.py [<deps.toml> ...]      # default: this tree's engine/goldens/deps.toml

A golden-moving landing = a commit that changes a golden (a png or walkabout_stats.ron) together with other files, or a
golden-only commit plus the code it belongs to: `Goldens: <ID> …` -> the run of commits before it whose subject starts
with <ID>; no ID (before land.py) -> the nearest of the 4 commits before it that touches render / bake / game / mungefs
source or scripts/ (read off the messages of the five such cases, 2026-10-01)."""
import os, re, subprocess, sys, tomllib

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
from gates import can_move_golden


def git(*a):
    return subprocess.run(["git", "-C", ROOT, *a], capture_output=True, text=True, check=True).stdout


def is_golden(f):
    return f.startswith("engine/goldens/") and (f.endswith(".png") or f.endswith("walkabout_stats.ron"))


log = git("log", "--first-parent", "--reverse", "--format=%x01%h%x02%s", "--name-only", "21c0f32^..main")
commits = []
for blk in log.split("\x01")[1:]:
    head, *files = [l for l in blk.split("\n") if l.strip()]
    h, s = head.split("\x02")
    commits.append((h, s, files))
movers, used = [], set()
for i, (h, s, files) in enumerate(commits):
    gold = [f for f in files if is_golden(f)]
    if not gold:
        continue
    rest = [f for f in files if not is_golden(f)]
    used.add(i)
    if rest:
        movers.append((h, s, rest)); continue
    m = re.match(r"Goldens: ([A-Z0-9]{3,8}) ", s)
    j, code, ids = i - 1, [], []
    if m:
        while j >= 0 and j not in used and commits[j][1].startswith(m.group(1)):
            code += commits[j][2]; ids.append(commits[j][0]); used.add(j); j -= 1
    else:  # before land.py: the nearest of the 4 commits before it that touches a drawing crate's or the bake's source
        for j in range(i - 1, max(i - 5, -1), -1):
            if j in used:
                break
            if any(re.match(r"engine/crates/(render|bake|game|mungefs)/src/|scripts/", f) for f in commits[j][2]):
                code, ids = commits[j][2], [commits[j][0]]; used.add(j)
                break
    movers.append((h + "<-" + ",".join(ids), s, code))
others = [c for i, c in enumerate(commits) if i not in used]
print(f"{len(commits)} commits since 21c0f32; {len(movers)} golden-moving landings; {len(others)} other commits")
for path in sys.argv[1:] or [os.path.join(ROOT, "engine/goldens/deps.toml")]:
    d = tomllib.load(open(path, "rb"))
    bad = [(h, s) for h, s, fs in movers if not any(can_move_golden(f, d) for f in fs)]
    sk = [c for c in others if not any(can_move_golden(f, d) for f in c[2])]
    code = [c for c in others if any(f.startswith("engine/crates/") for f in c[2])]
    skc = [c for c in code if not any(can_move_golden(f, d) for f in c[2])]
    print(f"\n{path}\n  golden-moving landings the rule would skip: {len(bad)}")
    for h, s in bad:
        print(f"    {h} {s[:110]}")
    print(f"  other commits skipped: {len(sk)} / {len(others)} = {100*len(sk)/len(others):.0f} %")
    print(f"  of those touching engine/crates: {len(skc)} / {len(code)} = {100*len(skc)/max(1,len(code)):.0f} %")
