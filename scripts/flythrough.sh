#!/usr/bin/env bash
# Tier-4 flythroughs: one orbit video per baked map.
#
#   scripts/flythrough.sh                  # every baked map
#   scripts/flythrough.sh tat1 bf1_kas1    # only these
#
# Output: engine/artifacts/flythrough/<map>.mp4 (local-only, gitignored).
# Framing comes from the engine's own `overview_camera`, so a map's orbit is
# the shot the golden gallery frames, swung around it — there are no
# hand-authored poses to drift out of step with the bake.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GAME="$ROOT/engine/target/release/game"
OUT="$ROOT/engine/artifacts/flythrough"
FRAMES="${FRAMES:-150}"
SIZE="${SIZE:-1280x720}"
FPS="${FPS:-30}"

[ -x "$GAME" ] || { echo "build first: cargo build --release -p game"; exit 1; }
command -v ffmpeg >/dev/null || { echo "ffmpeg not found"; exit 1; }
mkdir -p "$OUT"

maps=("$@")
if [ ${#maps[@]} -eq 0 ]; then
  for d in "$ROOT"/engine/assets_baked/*/; do
    [ -f "$d/manifest.ron" ] && maps+=("$(basename "$d")")
  done
fi

# The RTX's encoder when it works (a 30 s clip: 1.4 s of CPU against libx264's 15.9, measured 2026-10-11), else the CPU's.
# VIDEO_ENCODER=x264 forces the CPU encoder.
VENC=(-c:v libx264 -crf 20)
if [ "${VIDEO_ENCODER:-}" != x264 ] && ffmpeg -y -loglevel error -f lavfi -i color=size=640x448:rate=30 -frames:v 2 \
     -c:v h264_nvenc -preset p5 -rc vbr -cq 20 -b:v 0 -pix_fmt yuv420p -f null - >/dev/null 2>&1; then
  VENC=(-c:v h264_nvenc -preset p5 -rc vbr -cq 20 -b:v 0)
fi

for m in "${maps[@]}"; do
  dir="$ROOT/engine/assets_baked/$m"
  [ -f "$dir/manifest.ron" ] || { echo "skip $m (not a baked map)"; continue; }
  tmp="$(mktemp -d)"
  "$GAME" --map "$dir" --capture-orbit "$FRAMES,$tmp" --size "$SIZE" >/dev/null 2>&1
  ffmpeg -y -loglevel error -framerate "$FPS" -i "$tmp/%04d.png" \
    "${VENC[@]}" -pix_fmt yuv420p "$OUT/$m.mp4"
  echo "$m -> $OUT/$m.mp4 ($FRAMES frames @ ${FPS}fps, $SIZE)"
  rm -rf "$tmp"
done
echo "flythroughs done -> $OUT"
