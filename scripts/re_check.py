#!/usr/bin/env python3
"""re_check.py — the RE self-checks as one-line tools (prompt §RE). Corpora: b1 = analysis/SWBFRONT.ELF-decomp.c,
b2 = analysis/SWBFRNT2.ELF-decomp.c; ELFs: extracted/bf1/SWBFRONT.ELF, extracted/bf2/SWBFRNT2.ELF (off = va - 0x160000 + 0x1000).

  re_check.py fn b2 708831 708404            check 1: the function each line lives in (+ the line itself)
  re_check.py absent b2 DAT_006615f9 0x5c8 [--card ID] [--src PATH]… [--elf]
                                             check 2: every hit of each term in BOTH decomp corpora, counts + enclosing functions.
                                             --src PATH (repeatable; file or dir, e.g. engine/assets_baked, extracted/) also greps
                                             the raw bytes; --elf also greps both ELFs (ASCII, and a 0x… term as a LE u32).
                                             Always writes a RECEIPT (engine/artifacts/loop/receipts/<card>_<id>.json) and prints
                                             `[receipt:<id>]` — paste that tag next to every "no class authors X / absent" claim;
                                             `loopctl.py check-absence analysis/re-x.md` refuses an absence claim without one.
  re_check.py writers b2 DAT_006615f9         check 5: every WRITE of a global (`X = ...`, incl. initialisers) with its function
  re_check.py word b2 0x60c1fc [n] [--f32|--u32|--str]   check 4: raw ELF words at a VA, offset formula stated
  re_check.py callers b2 FUN_005755b0         who calls a function (decomp text), with the caller's name
Check 3 (the twin) is judgement: open both functions with `fn` and walk their callers with `callers`.
Paste the commands and their output into the RE doc — that is the evidence the verifier and the auditor re-run.
"""
import datetime, hashlib, json, os, re, struct, sys

VERSION = 1  # receipt format / tool version
MAX_SRC_FILES = 200000

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
if not os.path.exists(os.path.join(ROOT, "analysis/SWBFRONT.ELF-decomp.c")):
    # A worktree: the corpora are gitignored, so only the main checkout (the git common dir's parent) has them.
    import subprocess
    _common = subprocess.run(["git", "-C", ROOT, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                             capture_output=True, text=True).stdout.strip()
    if _common:
        ROOT = os.path.dirname(_common)
CORP = {"b1": "analysis/SWBFRONT.ELF-decomp.c", "b2": "analysis/SWBFRNT2.ELF-decomp.c"}
ELF = {"b1": "extracted/bf1/SWBFRONT.ELF", "b2": "extracted/bf2/SWBFRNT2.ELF"}
HDR = re.compile(r"^/\* ==== ([0-9a-f]{8}) (\S+) ==== \*/")
_cache = {}


def lines(c):
    if c not in _cache:
        _cache[c] = open(os.path.join(ROOT, CORP[c]), errors="replace").read().split("\n")
    return _cache[c]


def header_index(c):
    key = c + "_h"
    if key not in _cache:
        _cache[key] = [(i + 1, m.group(2), m.group(1)) for i, l in enumerate(lines(c)) if (m := HDR.match(l))]
    return _cache[key]


def enclosing(c, n):
    import bisect
    h = header_index(c)
    i = bisect.bisect_right([x[0] for x in h], n) - 1
    return h[i] if i >= 0 else (0, "?", "?")


def fn(c, nums):
    for n in map(int, nums):
        hl, name, addr = enclosing(c, n)
        print(f"{c.upper()}:{n} is in {name} (0x{addr}, header at line {hl}): {lines(c)[n - 1].strip()[:160]}")


def loop_dir():
    return os.environ.get("OPENBF_LOOP_DIR") or os.path.join(ROOT, "engine/artifacts/loop")


def _scan_bytes(path, needles, hit_files, counts):
    try:
        data = open(path, "rb").read()
    except OSError:
        return 0
    for t, nb in needles:
        n = sum(data.count(b) for b in nb)
        if n:
            counts[t] = counts.get(t, 0) + n
            hit_files.setdefault(t, []).append(os.path.relpath(path, ROOT))
    return len(data)


def _needles(t, elf):
    nb = [t.encode("latin-1", "replace")]
    if elf and re.fullmatch(r"0x[0-9a-fA-F]{1,8}", t):
        nb.append(struct.pack("<I", int(t, 16)))
    return nb


def search(terms, srcs=(), elf=False, quiet=False):
    """The absence search; returns the receipt dict (not yet written). Decomp corpora b1+b2 always; `srcs` raw files/dirs and
    (elf) the two ELFs are grepped as bytes. Prints the usual per-term report unless quiet."""
    rec = dict(version=VERSION, tool="re_check.py absent", terms=list(terms), corpora=[], sources=[], total_hits=0)
    for t in terms:
        pat = re.compile(re.escape(t) + (r"\b" if re.match(r"\w", t[-1]) else ""))
        for c in ("b1", "b2"):
            hits = [i + 1 for i, l in enumerate(lines(c)) if pat.search(l)]
            fns = {}
            for n in hits:
                fns.setdefault(enclosing(c, n)[1], []).append(n)
            rec["corpora"].append(dict(term=t, corpus=c, path=CORP[c], lines=len(lines(c)), hits=len(hits), functions=len(fns)))
            rec["total_hits"] += len(hits)
            if not quiet:
                print(f"{t!r} in {c.upper()}: {len(hits)} hit(s) in {len(fns)} function(s)")
                for f, ns in list(fns.items())[:40]:
                    print(f"   {f}: lines {', '.join(map(str, ns[:12]))}{' …' if len(ns) > 12 else ''}")
    paths = [(p, False) for p in srcs] + ([(ELF[c], True) for c in ("b1", "b2")] if elf else [])
    for p, is_elf in paths:
        full = p if os.path.isabs(p) else os.path.join(ROOT, p)
        files = [full] if os.path.isfile(full) else [os.path.join(d, f) for d, _, fs in os.walk(full) for f in fs]
        if len(files) > MAX_SRC_FILES:
            files = files[:MAX_SRC_FILES]
        needles = [(t, _needles(t, is_elf)) for t in terms]
        counts, hit_files, nbytes = {}, {}, 0
        for f in files:
            nbytes += _scan_bytes(f, needles, hit_files, counts)
        hits = sum(counts.values())
        rec["sources"].append(dict(path=p, kind="elf" if is_elf else "raw", files=len(files), bytes=nbytes, hits=hits,
                                   per_term=counts, hit_files={t: v[:20] for t, v in hit_files.items()}))
        rec["total_hits"] += hits
        if not quiet:
            print(f"raw {p}: {len(files)} file(s), {nbytes} bytes scanned, {hits} hit(s) {counts or ''}")
    return rec


def write_receipt(rec, card=None):
    """Stamp + write the receipt; returns its id (8 hex)."""
    if not card:
        try:
            card = json.loads(os.environ.get("OPENBF_JOB_JSON", "") or "{}").get("card") or json.loads(os.environ.get("OPENBF_JOB_JSON", "{}")).get("id")
        except ValueError:
            card = None
    rec["card"] = card or "nocard"
    rec["timestamp"] = datetime.datetime.now().isoformat(timespec="seconds")
    rid = hashlib.sha1(json.dumps(rec, sort_keys=True).encode()).hexdigest()[:8]
    rec["id"] = rid
    d = os.path.join(loop_dir(), "receipts")
    os.makedirs(d, exist_ok=True)
    json.dump(rec, open(os.path.join(d, f"{re.sub(r'[^A-Za-z0-9_-]', '', rec['card'])}_{rid}.json"), "w"), indent=1)
    return rid


def absent(c0, terms):
    card, srcs, elf, ts, it = None, [], False, [], iter(terms)
    for x in it:
        if x == "--card":
            card = next(it, None)
        elif x == "--src":
            srcs.append(next(it, ""))
        elif x == "--elf":
            elf = True
        else:
            ts.append(x)
    rec = search(ts, srcs, elf)
    rid = write_receipt(rec, card)
    print(f"[receipt:{rid}]  {len(ts)} term(s), {rec['total_hits']} hit(s) in all sources — "
          f"{os.path.relpath(loop_dir(), ROOT)}/receipts/{rec['card']}_{rid}.json")


def writers(c, terms):
    for t in terms:
        pat = re.compile(r"(^|[^=!<>\w])" + re.escape(t) + r"(\s*\[[^\]]*\])?\s*(=[^=]|\+=|-=|\|=|&=|\^=)")
        hits = [(i + 1, l.strip()) for i, l in enumerate(lines(c)) if pat.search(l)]
        print(f"writes of {t} in {c.upper()}: {len(hits)}")
        for n, l in hits:
            print(f"   {c.upper()}:{n} in {enclosing(c, n)[1]}: {l[:150]}")
        print("   (pointer writes through aliases are not visible to a text search — say so if the claim needs them)")


def word(c, va, n=1, kind="u32"):
    va = int(va, 16) if isinstance(va, str) else va
    off = va - 0x160000 + 0x1000
    data = open(os.path.join(ROOT, ELF[c]), "rb").read()
    print(f"{ELF[c]} va 0x{va:x} -> file offset 0x{off:x} (off = va - 0x160000 + 0x1000)")
    if kind == "str":
        s = data[off:off + 256].split(b"\0")[0]
        print(f"   string: {s.decode('latin-1')!r}")
        return
    for k in range(int(n)):
        w = data[off + 4 * k: off + 4 * k + 4]
        u = struct.unpack("<I", w)[0]
        f = struct.unpack("<f", w)[0]
        print(f"   0x{va + 4 * k:x}: u32 0x{u:08x} ({u})   f32 {f!r}")


def callers(c, names):
    for nm in names:
        pat = re.compile(re.escape(nm) + r"\s*\(")
        hits = [i + 1 for i, l in enumerate(lines(c)) if pat.search(l) and not HDR.match(l)]
        own = [h for h in hits if enclosing(c, h)[1] != nm]
        print(f"calls of {nm} in {c.upper()}: {len(own)}")
        for n in own:
            print(f"   {c.upper()}:{n} in {enclosing(c, n)[1]}: {lines(c)[n - 1].strip()[:140]}")
        print("   (calls through function pointers / vtables are not visible here — search the address as a data word too)")


def main():
    a = sys.argv[1:]
    if len(a) < 3 or a[0] not in ("fn", "absent", "writers", "word", "callers") or a[1] not in CORP:
        print(__doc__); sys.exit(1)
    cmd, c, rest = a[0], a[1], a[2:]
    if cmd == "fn":
        fn(c, rest)
    elif cmd == "absent":
        absent(c, rest)
    elif cmd == "writers":
        writers(c, rest)
    elif cmd == "callers":
        callers(c, rest)
    elif cmd == "word":
        kind = "str" if "--str" in rest else "u32"
        nums = [x for x in rest if not x.startswith("--")]
        word(c, nums[0], nums[1] if len(nums) > 1 else 1, kind)


if __name__ == "__main__":
    main()
