#!/usr/bin/env python3
"""ucfb_tree.py -- walk a SWBF (PS2/PC) ucfb LVL container and print its chunk tree.

Format: file starts with a top-level chunk 'ucfb'. A chunk is
    [4-byte tag][u32 LE payload size][payload][pad to 4-byte alignment]
Chunks nest: a payload may itself be a sequence of chunks. There is no
explicit container flag, so we recurse into a payload iff the whole payload
parses cleanly as a chunk sequence (heuristic).

Tags are either 4 ASCII chars ('tex_', 'NAME') or a 32-bit FNV-1a hash of a
name (common in BF2 and in CORE.LVL of both games). Hashed tags print as
0xXXXXXXXX, or as #name# when the hash is resolved against a dictionary
(--dict FILE with one name per line; a small builtin list is always tried).

Usage:
    ucfb_tree.py FILE [--peek N] [--max-depth D] [--census] [--no-tree]
                      [--tag TAG] [--min-size N] [--max-lines N] [--dict FILE]
"""

import argparse
import string
import struct
import sys
from collections import Counter

PRINTABLE = set(string.ascii_letters + string.digits + "_ .-")

# Names worth trying against hashed tags (SWBF FNV-1a-32, lowercased input).
BUILTIN_NAMES = """
ucfb lvl_ scr_ mcfg name info body tex_ pal_ modl skel segm tern wrld locl
plan coll prim entc ordc wpnc expc comb font_ font texture model animation
sound streams stream config class script level shadow light fx common core
mission load shell world terrain sky path hint region barrier boundary
""".split()

FNV_OFFSET = 2166136261
FNV_PRIME = 16777619


def fnv1a(name: str) -> int:
    h = FNV_OFFSET
    for c in name.lower().encode("ascii", "ignore"):
        h ^= c
        h = (h * FNV_PRIME) & 0xFFFFFFFF
    return h


def ascii_tag(raw: bytes) -> bool:
    return all(chr(b) in PRINTABLE for b in raw)


def parses_as_chunk_seq(buf, off, end, depth=0):
    """True iff buf[off:end] tiles exactly as a sequence of chunks.

    Arbitrary 4-byte tags are allowed (hashes), so guard against false
    positives: a single-child tiling is only accepted if the child's tag is
    ASCII or the child's own payload recursively tiles.
    """
    pos = off
    children = []
    while pos < end:
        if end - pos < 8:
            return False
        (size,) = struct.unpack_from("<I", buf, pos + 4)
        body_end = pos + 8 + size
        if body_end > end:
            return False
        # Hashed (non-ASCII) tags are real, but a run of floats/zeros also
        # "tiles" (e.g. 3F800000 00000000 ...). Require hashed-tag chunks to
        # carry a payload.
        if not ascii_tag(buf[pos:pos + 4]) and size < 4:
            return False
        children.append(pos)
        pos = body_end + (-size % 4)
        if len(children) > 1000000:
            return False
    if not children:
        return False
    if len(children) == 1 and depth < 8:
        cpos = children[0]
        (csize,) = struct.unpack_from("<I", buf, cpos + 4)
        if not ascii_tag(buf[cpos:cpos + 4]):
            if csize < 8 or not parses_as_chunk_seq(
                    buf, cpos + 8, cpos + 8 + csize, depth + 1):
                return False
    return True


def peek_str(buf, off, size, n):
    data = buf[off:off + min(size, n)]
    hexs = data.hex(" ")
    txt = "".join(chr(b) if 32 <= b < 127 else "." for b in data)
    return f"{hexs}  |{txt}|"


def walk(buf, off, end, depth, ctx, path=""):
    pos = off
    while pos < end:
        if end - pos < 8:
            break
        raw = buf[pos:pos + 4]
        (size,) = struct.unpack_from("<I", buf, pos + 4)
        body_off = pos + 8
        body_end = body_off + size
        if body_end > end:
            break
        if ascii_tag(raw):
            tag_s = raw.decode("ascii")
        else:
            (h,) = struct.unpack_from("<I", buf, pos)
            tag_s = ctx["hashnames"].get(h) or f"0x{h:08X}"
        child_path = f"{path}/{tag_s.strip()}"
        recurse = (
            depth < ctx["max_depth"]
            and size >= 8
            and parses_as_chunk_seq(buf, body_off, body_end)
        )
        yield (depth, tag_s, size, pos, not recurse, child_path)
        if recurse:
            yield from walk(buf, body_off, body_end, depth + 1, ctx, child_path)
        pos = body_end + (-size % 4)


def main():
    ap = argparse.ArgumentParser(description="Print chunk tree of a ucfb LVL file")
    ap.add_argument("file")
    ap.add_argument("--peek", type=int, default=0, metavar="N",
                    help="show first N bytes of each leaf payload")
    ap.add_argument("--max-depth", type=int, default=32)
    ap.add_argument("--census", action="store_true",
                    help="print per-tag count/bytes summary at the end")
    ap.add_argument("--no-tree", action="store_true", help="suppress tree output")
    ap.add_argument("--tag", default=None,
                    help="only print subtrees containing this tag in their path")
    ap.add_argument("--min-size", type=int, default=0,
                    help="only print chunks with payload >= N bytes")
    ap.add_argument("--max-lines", type=int, default=0,
                    help="stop tree output after N lines (0 = unlimited)")
    ap.add_argument("--dict", dest="dictfile", default=None,
                    help="file of candidate names (one per line) to resolve hashed tags")
    args = ap.parse_args()

    names = list(BUILTIN_NAMES)
    if args.dictfile:
        with open(args.dictfile, "r", errors="ignore") as f:
            names += [ln.strip() for ln in f if ln.strip()]
    hashnames = {fnv1a(n): f"#{n}#" for n in names}

    with open(args.file, "rb") as f:
        buf = f.read()

    if buf[:4] != b"ucfb":
        print(f"warning: no ucfb magic (got {buf[:4]!r}); walking anyway",
              file=sys.stderr)

    ctx = {"max_depth": args.max_depth, "hashnames": hashnames}
    census = Counter()
    census_bytes = Counter()
    lines = 0
    truncated = False
    for depth, tag, size, off, leaf, path in walk(buf, 0, len(buf), 0, ctx):
        census[tag] += 1
        census_bytes[tag] += size
        if args.no_tree or size < args.min_size:
            continue
        if args.tag and args.tag not in path.split("/"):
            continue
        if args.max_lines and lines >= args.max_lines:
            truncated = True
            continue
        mark = "" if leaf else "+"
        line = f"{'  ' * depth}{tag}{mark}  size={size}  @0x{off:X}"
        if args.peek and leaf and size:
            line += "\n" + "  " * depth + "    " + peek_str(buf, off + 8, size, args.peek)
        print(line)
        lines += 1
    if truncated:
        print(f"... (tree truncated at {args.max_lines} lines)")

    if args.census:
        print("\n# census (tag, count, total payload bytes)")
        for tag, cnt in sorted(census.items(), key=lambda kv: -census_bytes[kv[0]]):
            print(f"{tag!r:14} {cnt:7d} {census_bytes[tag]:12d}")


if __name__ == "__main__":
    main()
