#!/usr/bin/env bash
# Listen to a map's baked sound bank.
#
# The discs store every sound property and sample id as an FNV hash and ship no
# strings, so the baked WAVs are named `<bank>/<hash>.wav`. `bake sound-names`
# recovers the readable names it can (see crates/bake/src/sound_names.rs) into
# `<mapdir>/sounds/names.ron`; this script joins the two so a human can browse.
#
#   scripts/audition.sh list   <map> [filter]        what is in the bank
#   scripts/audition.sh play   <map> [filter]        play them in order
#   scripts/audition.sh export <map> <outdir> [flt]  copy out under real names
#
#   map    = a directory name under engine/assets_baked (tat1, bf1_hot1, ...)
#   filter = case-insensitive substring of the property name, e.g. rifle
#            (properties with no recovered name are matched by their hex hash)
#
# Examples:
#   scripts/audition.sh list  bf1_hot1
#   scripts/audition.sh play  tat1 rifle_fire
#   scripts/audition.sh export bf2_kas2 /tmp/kashyyyk-sounds
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BAKED="$ROOT/engine/assets_baked"

usage() { sed -n '2,20p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 1; }
[ $# -ge 2 ] || usage
MODE="$1"; MAP="$2"; shift 2

DIR="$BAKED/$MAP/sounds"
if [ ! -f "$DIR/sounds.ron" ]; then
  echo "no baked sounds for '$MAP' ($DIR/sounds.ron missing)." >&2
  echo "maps with sounds:" >&2
  (cd "$BAKED" && ls -d */sounds 2>/dev/null | cut -d/ -f1 | tr '\n' ' ') >&2
  echo >&2
  echo "run: scripts/rebake.sh sounds" >&2
  exit 1
fi

OUTDIR=""
if [ "$MODE" = "export" ]; then
  [ $# -ge 1 ] || usage
  OUTDIR="$1"; shift
fi
FILTER="${1:-}"

# Pick a player. aplay is the most predictable for raw mono WAVs; ffplay is the
# fallback because it needs no ALSA default device.
PLAYER=""
if [ "$MODE" = "play" ]; then
  if command -v aplay >/dev/null; then PLAYER="aplay -q"
  elif command -v paplay >/dev/null; then PLAYER="paplay"
  elif command -v ffplay >/dev/null; then PLAYER="ffplay -nodisp -autoexit -loglevel quiet"
  else echo "no audio player found (tried aplay, paplay, ffplay)" >&2; exit 1; fi
fi

# The join lives in python because sounds.ron/names.ron are RON, and because
# one pass has to resolve property -> weighted sample list -> WAV path.
MODE="$MODE" DIR="$DIR" OUTDIR="$OUTDIR" FILTER="$FILTER" PLAYER="$PLAYER" MAP="$MAP" \
python3 - <<'PY'
import os, re, shlex, shutil, subprocess, sys

mode   = os.environ["MODE"]
d      = os.environ["DIR"]
outdir = os.environ["OUTDIR"]
filt   = os.environ["FILTER"].lower()
player = os.environ["PLAYER"]
mapname= os.environ["MAP"]

sounds = open(os.path.join(d, "sounds.ron")).read()

names = {}
npath = os.path.join(d, "names.ron")
if os.path.exists(npath):
    for h, n, src in re.findall(r'\((\d+),\s*"([^"]+)",\s*(Direct|Derived)\)', open(npath).read()):
        names[int(h)] = (n, src)
else:
    print("(no names.ron — run: scripts/rebake.sh sounds; names stay hex)", file=sys.stderr)

# sample id -> relative wav path
samples = dict((int(h), p) for h, p in
               re.findall(r'\((\d+),\s*"([0-9a-f]{8}/[0-9a-f]{8}\.wav)"\)', sounds))

# Property records: name_hash ... samples: [ (id, weight), ... ]. A planet's
# SOUND LVL defines the same property once per sublevel (tat1/tat2 x era), so
# collapse by hash and take the union of the samples — the runtime keeps one
# definition per hash and the point here is to hear everything a name can play.
by_hash = {}
for m in re.finditer(r'name_hash:\s*(\d+),(.*?)samples:\s*\[(.*?)\],\s*layers', sounds, re.S):
    h = int(m.group(1))
    ids = [int(x) for x in re.findall(r'\((\d+),\s*[-\d.]+\)', m.group(3))]
    for i in ids:
        if i in samples:
            by_hash.setdefault(h, []).append(i)
props = []
for h, ids in by_hash.items():
    seen, uniq = set(), []
    for i in ids:
        if i not in seen:
            seen.add(i); uniq.append(i)
    props.append((h, uniq))

def label(h):
    n, src = names.get(h, (None, None))
    return (n, src) if n else (f"{h:08x}", "hash")

# Named first (alphabetical), then the unnamed by hash — so browsing starts
# with the sounds you can actually identify.
props.sort(key=lambda p: (label(p[0])[1] == "hash", label(p[0])[0].lower()))
sel = [(h, ids) for h, ids in props if not filt or filt in label(h)[0].lower()]

if not sel:
    print(f"no properties in {mapname} match {filt!r} "
          f"({len(props)} playable of which {sum(1 for h,_ in props if label(h)[1]!='hash')} named)")
    sys.exit(0)

if mode == "list":
    tot = 0
    for h, ids in sel:
        n, src = label(h)
        secs = []
        for i in ids:
            p = os.path.join(d, samples[i])
            try:
                sz = os.path.getsize(p)
                # RIFF header is 44 bytes; rate is at offset 24, 16-bit mono.
                with open(p, "rb") as f:
                    hdr = f.read(44)
                rate = int.from_bytes(hdr[24:28], "little")
                secs.append((sz - 44) / 2 / max(rate, 1))
            except OSError:
                pass
        tot += len(ids)
        dur = f"{sum(secs):5.2f}s" if secs else "   ?  "
        print(f"{n:<52} {src:<7} {len(ids):>2} sample(s) {dur}")
    print(f"\n{len(sel)} properties, {tot} samples  [{mapname}]")
    print("Direct = string found verbatim on the discs; Derived = recombined "
          "candidate that matched the hash (may be a collision); hash = name "
          "not recovered.")

elif mode == "play":
    print(f"playing {len(sel)} properties from {mapname} — ctrl-C to stop")
    for h, ids in sel:
        n, src = label(h)
        for k, i in enumerate(ids):
            p = os.path.join(d, samples[i])
            print(f"  {n}  [{k+1}/{len(ids)}] {src}", flush=True)
            subprocess.run(shlex.split(player) + [p], check=False)

elif mode == "export":
    os.makedirs(outdir, exist_ok=True)
    written = 0
    for h, ids in sel:
        n, src = label(h)
        sub = os.path.join(outdir, n if src != "hash" else f"_unnamed/{n}")
        os.makedirs(sub, exist_ok=True)
        for k, i in enumerate(ids):
            shutil.copyfile(os.path.join(d, samples[i]),
                            os.path.join(sub, f"{k+1:02d}.wav"))
            written += 1
    print(f"exported {written} samples in {len(sel)} folders -> {outdir}")
    print("(_unnamed/ holds the properties whose name could not be recovered)")
else:
    print(f"unknown mode {mode!r}", file=sys.stderr); sys.exit(1)
PY
